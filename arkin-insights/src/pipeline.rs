use std::collections::HashMap;
use std::sync::Arc;

use arkin_core::prelude::*;
use parking_lot::Mutex;
use petgraph::graph::NodeIndex;
use petgraph::{
    algo::toposort,
    dot::{Config, Dot},
    graph::DiGraph,
};
use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use time::OffsetDateTime;
use tracing::debug;

use crate::config::PipelineConfig;
use crate::feature_factory::FeatureFactory;
use crate::state::InsightsState;
use crate::Feature;

#[derive(Debug)]
pub struct PipelineGraph {
    state: Arc<InsightsState>,
    graph: Arc<DiGraph<Arc<dyn Feature>, ()>>,
    order: Vec<NodeIndex>,
    pool: ThreadPool,
}

impl PipelineGraph {
    pub fn from_config(pipeline: Arc<Pipeline>, state: Arc<InsightsState>, features: &PipelineConfig) -> Self {
        let features = FeatureFactory::from_config(pipeline.clone(), state.clone(), &features.features);
        let mut graph = DiGraph::new();

        // Create a mapping from Node IDs to Node Indices
        let mut id_to_index = HashMap::new();

        // Add features as nodes and populate the mapping
        for feature in features {
            let output_ids = feature.outputs().to_vec();
            let node_index = graph.add_node(feature);
            for id in output_ids {
                id_to_index.insert(id.clone(), node_index);
            }
        }

        for (id, index) in &id_to_index {
            debug!("Node ID: {:?}, Node Index: {:?}", id, index);
        }

        // Add edges automatically
        let mut edges_to_add = Vec::new();
        for target_node in graph.node_indices() {
            for source_id in graph[target_node].inputs() {
                // We don't need to have a dependency on raw inputs
                if RAW_FEATURE_IDS.contains(&source_id) {
                    continue;
                }
                let source_node = match id_to_index.get(&source_id) {
                    Some(node) => *node,
                    None => panic!("Node not found: {:?}", source_id),
                };
                edges_to_add.push((source_node, target_node));
            }
        }
        for (source, target) in &edges_to_add {
            debug!("Edge: {:?} -> {:?}", source, target);
        }

        // Add edges to the graph
        for (source, target) in edges_to_add {
            graph.add_edge(source, target, ());
        }

        // Save the topological order for parallel processing
        let order = toposort(&graph, None).expect("Cycle detected in graph");

        debug!("{:?}", Dot::with_config(&graph, &[Config::EdgeIndexLabel]));

        PipelineGraph {
            state,
            graph: graph.into(),
            order,
            pool: ThreadPoolBuilder::default().build().expect("Failed to create thread pool"),
        }
    }

    // Topological Sorting in parallel, which can be efficiently implemented using Kahn's algorithm
    pub fn calculate(&self, instruments: &[Arc<Instrument>], timestamp: OffsetDateTime) -> Vec<Arc<Insight>> {
        // Step 1: Calculate in-degrees
        let in_degrees = Arc::new(Mutex::new(vec![0; self.graph.node_count()]));
        for edge in self.graph.edge_indices() {
            let target = self.graph.edge_endpoints(edge).unwrap().1;
            in_degrees.lock()[target.index()] += 1;
        }
        debug!("In-Degree count: {}", in_degrees.lock().iter().fold(0, |acc, x| acc + x));

        // Step 2: Enqueue nodes with zero in-degree
        let (queue_tx, queue_rx) = kanal::unbounded();
        for node in &self.order {
            if in_degrees.lock()[node.index()] == 0 {
                debug!("Ready node: {}", node.index());
                queue_tx.send(Some(*node)).expect("Failed to send ready node");
            }
        }

        // Step 3: Parallel processing
        let pipeline_result = Arc::new(Mutex::new(Vec::new()));
        self.pool.scope(|s| {
            while let Some(node) = queue_rx.recv().expect("Failed to receive data") {
                let graph = Arc::clone(&self.graph);
                let in_degrees = Arc::clone(&in_degrees);
                let queue_tx = queue_tx.clone();
                let pipeline_result = Arc::clone(&pipeline_result);
                let state = Arc::clone(&self.state);

                s.spawn(move |_| {
                    // Process the node
                    let feature = &graph[node];

                    // Calculate the feature
                    let insights = instruments
                        .par_iter()
                        .filter_map(|instrument| feature.calculate(instrument, timestamp))
                        .flatten()
                        .map(|i| {
                            let value = (i.value * 1_000_000.0).round() / 1_000_000.0;
                            // Clone the insight and set the value
                            let mut insight = i.as_ref().clone();
                            insight.value = value;
                            insight.into()
                        })
                        .collect::<Vec<_>>();
                    state.insert_batch(&insights);
                    pipeline_result.lock().extend(insights);

                    // Update in-degrees of neighbors and enqueue new zero in-degree nodes
                    for neighbor in graph.neighbors_directed(node, petgraph::Outgoing) {
                        let mut in_degrees = in_degrees.lock();
                        in_degrees[neighbor.index()] -= 1;
                        if in_degrees[neighbor.index()] == 0 {
                            debug!("Ready node: {}", neighbor.index());
                            queue_tx.send(Some(neighbor)).expect("Failed to send ready node");
                        }
                    }
                    debug!("Dependency count: {:?}", in_degrees);
                    if in_degrees.lock().iter().all(|&x| x == 0) {
                        debug!("All nodes processed");
                        queue_tx.send(None).expect("Failed to send exit message");
                    }
                });
            }
        });
        debug!("Finished graph calculation");
        let mut lock = pipeline_result.lock();
        let res = std::mem::take(&mut *lock);
        res
    }
}

// COULD BE USED IN THE FUTURE IF WE HAVE ASYNC FEATURES
// pub async fn calculate_async(&self) {
//     // Step 1: Calculate in-degrees
//     let in_degrees = Arc::new(Mutex::new(vec![0; self.graph.node_count()]));
//     for edge in self.graph.edge_indices() {
//         let target = self.graph.edge_endpoints(edge).unwrap().1;
//         in_degrees.lock()[target.index()] += 1;
//     }
//     debug!("In-Degree count: {:?}", in_degrees);

//     // Step 2: Enqueue nodes with zero in-degree
//     let (queue_tx, queue_rx) = flume::unbounded();
//     for node in &self.order {
//         if in_degrees.lock()[node.index()] == 0 {
//             debug!("Ready node: {:?}", self.graph[*node]);
//             queue_tx.send(Some(*node)).expect("Failed to send ready node");
//         }
//     }

//     // Step 3: Parallel processing
//     let mut tasks = Vec::with_capacity(self.graph.node_count());
//     while let Some(node_index) = queue_rx.recv_async().await.expect("Failed to receive ready node") {
//         let graph = Arc::clone(&self.graph);
//         let in_degrees = Arc::clone(&in_degrees);
//         let queue_tx = queue_tx.clone();

//         let task = tokio::spawn(async move {
//             // Calculate the feature
//             let feature = &graph[node_index];
//             feature.calculate_async().await;

//             // Update dependencies and push ready nodes to the queue
//             for neighbor in graph.neighbors_directed(node_index, petgraph::Outgoing) {
//                 let mut count = in_degrees.lock()[neighbor.index()];
//                 count -= 1;
//                 in_degrees.lock()[neighbor.index()] = count;

//                 if count == 0 {
//                     debug!("Ready node: {:?}", graph[neighbor]);
//                     queue_tx.send_async(Some(neighbor)).await.expect("Failed to send ready node");

//                     debug!("Dependency count: {:?}", in_degrees);
//                     if in_degrees.lock().iter().all(|&x| x == 0) {
//                         queue_tx.send_async(None).await.expect("Failed to send ready node");
//                     }
//                 }
//             }
//         });
//         tasks.push(task);
//     }

//     // Wait on all tasks to finish
//     for task in tasks {
//         let _ = task.await;
//     }

//     info!("Finished graph calculation");
// }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         config::{
//             EMAFeatureConfig, FeatureConfig, SMAFeatureConfig, SpreadFeatureConfig, VWAPFeatureConfig,
//             VolumeFeatureConfig,
//         },
//         logging,
//     };

// #[test]
// fn test_pipeline_add_node() {
//     let mut graph = Pipeline::default();
//     let vwap = VWAPGen::new("vwap".into(), Duration::from_secs(60));
//     graph.add_node(vwap);
//     assert_eq!(graph.graph.node_count(), 1);
// }

// #[test]
// fn test_pipeline_add_edge() {
//     let mut graph = Pipeline::default();
//     let vwap = VWAPGen::new("vwap".into(), Duration::from_secs(60));
//     let ema = EMAGen::new("ema_vwap_50".into(), "vwap".into(), Duration::from_secs(300));

//     graph.add_node(vwap);
//     graph.add_node(ema);

//     graph.connect_nodes();
//     assert_eq!(graph.graph.edge_count(), 1);
// }

// #[tokio::test]
// async fn test_pipeline_calculate() {
//     logging::init_test_tracing();

//     // Create graph
//     let mut graph = Pipeline::default();

//     // Create features
//     let vwap = VWAPGen::new("vwap".into(), Duration::from_secs(60));
//     let ema = EMAGen::new("ema_vwap_50".into(), "vwap".into(), Duration::from_secs(50));
//     let sma = SMAGen::new("sma_vwap_50".into(), "vwap".into(), Duration::from_secs(50));
//     let spread = SpreadGen::new("spread".into(), "sma_vwap_50".into(), "ema_vwap_50".into());
//     let volume = VolumeGen::new("volume".into(), Duration::from_secs(60));

//     // Create nodes
//     graph.add_node(vwap);
//     graph.add_node(ema);
//     graph.add_node(sma);
//     graph.add_node(spread);
//     graph.add_node(volume);

//     // Connect nodes
//     graph.connect_nodes();

//     // Calculate
//     graph.calculate().await;
//     assert_eq!(graph.graph.node_count(), 5);
//     assert_eq!(graph.graph.edge_count(), 4);
// }

// #[tokio::test(flavor = "multi_thread")]
// async fn test_pipeline_from_config() {
//     logging::init_test_tracing();

//     let config = PipelineConfig {
//         name: "test_pipeline".to_string(),
//         frequency: 1,
//         features: vec![
//             FeatureConfig::Volume(VolumeFeatureConfig {
//                 id: "volume_1_min".(),
//                 window: 60,
//             }),
//             FeatureConfig::VWAP(VWAPFeatureConfig {
//                 id: "vwap_1_min".into(),
//                 window: 60,
//             }),
//             FeatureConfig::SMA(SMAFeatureConfig {
//                 id: "sma_10_min".into(),
//                 source: "vwap_1_min".into(),
//                 period: 10,
//             }),
//             FeatureConfig::EMA(EMAFeatureConfig {
//                 id: "ema_10_min".into(),
//                 source: "vwap_1_min".into(),
//                 period: 10,
//             }),
//             FeatureConfig::Spread(SpreadFeatureConfig {
//                 id: "spread_vwap".into(),
//                 front_component: "sma_10_min".into(),
//                 back_component: "ema_10_min".into(),
//             }),
//             FeatureConfig::SMA(SMAFeatureConfig {
//                 id: "sma_volume_10_min".into(),
//                 source: "volume_1_min".into(),
//                 period: 10,
//             }),
//             FeatureConfig::Spread(SpreadFeatureConfig {
//                 id: "spread_volume".into(),
//                 front_component: "sma_volume_10_min".into(),
//                 back_component: "volume_1_min".into(),
//             }),
//             FeatureConfig::Spread(SpreadFeatureConfig {
//                 id: "spread_vwap_volume".into(),
//                 front_component: "spread_vwap".into(),
//                 back_component: "spread_volume".into(),
//             }),
//         ],
//     };

//     let pipeline = Pipeline::from_config(&config);
//     pipeline.calculate();
// }
// }
