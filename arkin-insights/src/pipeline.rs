use std::collections::HashMap;
use std::sync::Arc;

use arkin_core::prelude::*;
use parking_lot::Mutex;
use parking_lot::RwLock;
use petgraph::graph::NodeIndex;
use petgraph::{
    algo::toposort,
    dot::{Config, Dot},
    graph::DiGraph,
};
use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use time::UtcDateTime;
use tracing::debug;
use tracing::info;

use crate::state::InsightsState;
use crate::Feature;

#[derive(Debug)]
pub struct PipelineGraph {
    graph: Arc<DiGraph<Arc<dyn Feature>, ()>>,
    order: Vec<NodeIndex>,
    indegrees: Vec<i32>,
    pool: ThreadPool,
}

impl PipelineGraph {
    // CRITICAL: Make async, take Arc<P> for ownership
    pub fn new(features: Vec<Arc<dyn Feature>>) -> Self {
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
                // Check if this is a raw input (no feature produces it)
                let source_node = match id_to_index.get(&source_id) {
                    Some(node) => *node,
                    None => {
                        // This is a raw input, skip adding edge
                        debug!("Raw input detected: {:?}", source_id);
                        continue;
                    }
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

        let mut indegrees = vec![0; graph.node_count()];
        for edge in graph.edge_indices() {
            let target = graph.edge_endpoints(edge).unwrap().1;
            indegrees[target.index()] += 1;
        }

        PipelineGraph {
            graph: graph.into(),
            order,
            indegrees,
            pool: ThreadPoolBuilder::default().build().expect("Failed to create thread pool"),
        }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn print_dot(&self) {
        info!("{:?}", Dot::with_config(&*self.graph, &[Config::EdgeIndexLabel]));
    }

    /// Export graph to DOT format for visualization with Graphviz
    pub fn to_dot_string(&self) -> String {
        format!("{:?}", Dot::with_config(&*self.graph, &[Config::EdgeNoLabel]))
    }

    /// Export graph to DOT format with custom node labels (simplified output names only)
    pub fn to_dot_string_simple(&self) -> String {
        let mut dot = String::from("digraph {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=rounded];\n\n");

        // Get raw inputs
        let raw_inputs = self.get_raw_inputs();

        // Add raw input nodes with special styling
        dot.push_str("  // Raw inputs\n");
        dot.push_str("  node [shape=ellipse, style=filled, fillcolor=lightblue];\n");
        for (i, input) in raw_inputs.iter().enumerate() {
            dot.push_str(&format!("  raw_{} [label=\"{}\"];\n", i, input));
        }
        dot.push('\n');

        // Add feature nodes
        dot.push_str("  // Features\n");
        dot.push_str("  node [shape=box, style=rounded, fillcolor=white];\n");
        for node in self.graph.node_indices() {
            let outputs = self.graph[node].outputs();
            let label = outputs.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
            dot.push_str(&format!("  {} [label=\"{}\"];\n", node.index(), label));
        }
        dot.push('\n');

        // Create a mapping from raw input names to their pseudo-node IDs
        let mut raw_input_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for (i, input) in raw_inputs.iter().enumerate() {
            raw_input_map.insert(input.clone(), format!("raw_{}", i));
        }

        // Add edges from raw inputs to features
        dot.push_str("  // Edges from raw inputs\n");
        for node in self.graph.node_indices() {
            for input in self.graph[node].inputs() {
                if let Some(raw_id) = raw_input_map.get(input.as_str()) {
                    dot.push_str(&format!("  {} -> {};\n", raw_id, node.index()));
                }
            }
        }
        dot.push('\n');

        // Add edges between features
        dot.push_str("  // Edges between features\n");
        for edge in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge) {
                dot.push_str(&format!("  {} -> {};\n", source.index(), target.index()));
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Export pipeline graph to SVG file using graphviz
    ///
    /// # Arguments
    /// * `path` - Output path for the SVG file (e.g., "/tmp/pipeline_graph.svg")
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(String)` with error message if graphviz is not available or export fails
    ///
    pub fn export_svg(&self, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        use graphviz_rust::{cmd::Format, exec, parse, printer::PrinterContext};

        let dot_string = self.to_dot_string_simple();

        // Parse DOT string
        let graph = parse(&dot_string).map_err(|e| format!("Failed to parse DOT: {:?}", e))?;

        // Execute graphviz to generate SVG
        let svg = exec(graph, &mut PrinterContext::default(), vec![Format::Svg.into()])
            .map_err(|e| format!("Failed to execute graphviz: {:?}", e))?;

        // Write to file
        std::fs::write(&path, svg).map_err(|e| format!("Failed to write SVG file: {}", e))?;

        Ok(())
    }

    /// Print a tree-like ASCII view of the pipeline showing data flow
    pub fn print_tree(&self) {
        info!("Pipeline Flow (Layer by Layer):");
        info!("================================");

        let raw_inputs = self.get_raw_inputs();
        info!("RAW INPUTS: {}", raw_inputs.join(", "));
        info!("");

        let layers = self.get_layers();
        for (i, layer) in layers.iter().enumerate() {
            info!("LAYER {} ({} features):", i + 1, layer.len());
            for feature in layer {
                let outputs = feature.outputs();
                let inputs = feature.inputs();
                info!(
                    "  {} <- [{}]",
                    outputs.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    inputs.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                );
            }
            info!("");
        }
    }

    /// Get all raw input IDs (inputs that no feature produces)
    pub fn get_raw_inputs(&self) -> Vec<String> {
        let mut all_outputs: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut all_inputs: std::collections::HashSet<String> = std::collections::HashSet::new();

        for node in self.graph.node_indices() {
            for output in self.graph[node].outputs() {
                all_outputs.insert(output.to_string());
            }
            for input in self.graph[node].inputs() {
                all_inputs.insert(input.to_string());
            }
        }

        // Raw inputs are those that are used as inputs but never produced as outputs
        all_inputs.difference(&all_outputs).cloned().collect::<Vec<_>>()
    }

    /// Validate the DAG structure - checks for cycles and unreachable nodes
    pub fn validate(&self) -> Result<(), String> {
        // Check for cycles using petgraph's is_cyclic_directed
        if petgraph::algo::is_cyclic_directed(&*self.graph) {
            return Err("Cycle detected in graph".to_string());
        }

        // Get raw inputs
        let raw_inputs = self.get_raw_inputs();

        // Check for nodes with no incoming edges from other features (sources)
        let sources: Vec<_> = self
            .graph
            .node_indices()
            .filter(|&node| self.graph.neighbors_directed(node, petgraph::Incoming).count() == 0)
            .collect();

        // Check for nodes with no outgoing edges (sinks/leaf nodes)
        let sinks: Vec<_> = self
            .graph
            .node_indices()
            .filter(|&node| self.graph.neighbors_directed(node, petgraph::Outgoing).count() == 0)
            .collect();

        info!("DAG Validation:");
        info!("  Raw inputs (not produced by any feature): {}", raw_inputs.len());
        for input in &raw_inputs {
            info!("    - {}", input);
        }
        info!("  Source features (depend only on raw inputs): {}", sources.len());
        info!("  Sink features (terminal outputs): {}", sinks.len());
        info!("  Total nodes: {}", self.graph.node_count());
        info!("  Total edges: {}", self.graph.edge_count());

        Ok(())
    }

    /// Get all source nodes (features that only depend on raw inputs)
    pub fn get_sources(&self) -> Vec<&Arc<dyn Feature>> {
        self.graph
            .node_indices()
            .filter(|&node| self.graph.neighbors_directed(node, petgraph::Incoming).count() == 0)
            .map(|node| &self.graph[node])
            .collect()
    }

    /// Get all sink nodes (terminal features with no dependents)
    pub fn get_sinks(&self) -> Vec<&Arc<dyn Feature>> {
        self.graph
            .node_indices()
            .filter(|&node| self.graph.neighbors_directed(node, petgraph::Outgoing).count() == 0)
            .map(|node| &self.graph[node])
            .collect()
    }

    /// Get dependencies for a specific output
    pub fn get_dependencies(&self, output: &str) -> Option<Vec<Arc<dyn Feature>>> {
        // Find the node that produces this output
        let node = self
            .graph
            .node_indices()
            .find(|&node| self.graph[node].outputs().iter().any(|o| o.as_str() == output))?;

        // Get all upstream dependencies using DFS
        let mut visited = std::collections::HashSet::new();
        let mut deps = Vec::new();
        self.collect_dependencies(node, &mut visited, &mut deps);

        Some(deps)
    }

    fn collect_dependencies(
        &self,
        node: NodeIndex,
        visited: &mut std::collections::HashSet<NodeIndex>,
        deps: &mut Vec<Arc<dyn Feature>>,
    ) {
        if visited.contains(&node) {
            return;
        }
        visited.insert(node);

        for neighbor in self.graph.neighbors_directed(node, petgraph::Incoming) {
            deps.push(Arc::clone(&self.graph[neighbor]));
            self.collect_dependencies(neighbor, visited, deps);
        }
    }

    /// Get the topological layers (features at each depth level)
    pub fn get_layers(&self) -> Vec<Vec<&Arc<dyn Feature>>> {
        let mut layers: Vec<Vec<&Arc<dyn Feature>>> = Vec::new();
        let mut depth_map = HashMap::new();

        // Calculate depth for each node using topological order
        for &node in &self.order {
            let max_dep_depth = self
                .graph
                .neighbors_directed(node, petgraph::Incoming)
                .filter_map(|dep| depth_map.get(&dep))
                .max()
                .unwrap_or(&0);

            let depth = max_dep_depth + 1;
            depth_map.insert(node, depth);

            // Ensure we have enough layers
            while layers.len() < depth {
                layers.push(Vec::new());
            }

            layers[depth - 1].push(&self.graph[node]);
        }

        layers
    }

    /// Print a summary of the pipeline structure
    pub fn print_summary(&self) {
        info!("Pipeline Summary:");
        info!("  Total Features: {}", self.graph.node_count());
        info!("  Total Dependencies: {}", self.graph.edge_count());

        let raw_inputs = self.get_raw_inputs();
        info!("  Raw Inputs: {}", raw_inputs.len());
        for input in &raw_inputs {
            info!("    - {}", input);
        }

        let sources = self.get_sources();
        info!("  Source Features (depend only on raw inputs): {}", sources.len());
        for source in &sources {
            info!("    - outputs: {:?}", source.outputs());
        }

        let sinks = self.get_sinks();
        info!("  Terminal Features: {}", sinks.len());
        for sink in &sinks {
            info!("    - outputs: {:?}", sink.outputs());
        }

        // Find bottleneck features (those with many dependents)
        let mut dependents_count: Vec<(NodeIndex, usize)> = self
            .graph
            .node_indices()
            .map(|node| {
                let count = self.graph.neighbors_directed(node, petgraph::Outgoing).count();
                (node, count)
            })
            .collect();
        dependents_count.sort_by(|a, b| b.1.cmp(&a.1));

        info!("  Bottleneck Features (top 5 by number of dependents):");
        for (node, count) in dependents_count.iter().take(5) {
            if *count > 0 {
                info!("    - {:?} ({} dependents)", self.graph[*node].outputs(), count);
            }
        }

        let layers = self.get_layers();
        info!("  Pipeline Depth: {} layers", layers.len());
        for (i, layer) in layers.iter().enumerate() {
            info!("    Layer {}: {} features", i + 1, layer.len());
        }
    }

    // Topological Sorting in parallel, which can be efficiently implemented using Kahn's algorithm
    pub fn calculate(
        &self,
        state: &Arc<InsightsState>,
        pipeline: &Arc<Pipeline>,
        event_time: UtcDateTime,
        instruments: &[Arc<Instrument>],
    ) -> Vec<Arc<Insight>> {
        // Step 1: Calculate in-degrees
        let in_degrees = Arc::new(RwLock::new(self.indegrees.clone()));

        // Step 2: Enqueue nodes with zero in-degree
        let (queue_tx, queue_rx) = kanal::unbounded();
        for node in &self.order {
            if in_degrees.read()[node.index()] == 0 {
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
                // let state_clone = Arc::clone(&state);

                s.spawn(move |_| {
                    // Process the node
                    let feature = &graph[node];

                    // Calculate the feature
                    let insights = instruments
                        .par_iter()
                        .filter_map(|instrument| feature.calculate(state, pipeline, instrument, event_time))
                        .flatten()
                        .collect::<Vec<_>>();
                    pipeline_result.lock().extend(insights);

                    // Update in-degrees of neighbors and enqueue new zero in-degree nodes
                    for neighbor in graph.neighbors_directed(node, petgraph::Outgoing) {
                        let mut in_degrees = in_degrees.write();
                        in_degrees[neighbor.index()] -= 1;
                        if in_degrees[neighbor.index()] == 0 {
                            debug!("Ready node: {}", neighbor.index());
                            queue_tx.send(Some(neighbor)).expect("Failed to send ready node");
                        }
                    }
                    debug!("Dependency count: {:?}", in_degrees);
                    if in_degrees.read().iter().all(|&x| x == 0) {
                        debug!("All nodes processed");
                        queue_tx.send(None).expect("Failed to send exit message");
                    }
                });
            }
        });
        debug!("Finished graph calculation");
        let mut lock = pipeline_result.lock();
        
        std::mem::take(&mut *lock)
    }

    // pub async fn calculate_async(&self, tick: &InsightsTick) -> Vec<Arc<Insight>> {
    //     // Step 1: Calculate in-degrees
    //     let in_degrees = Arc::new(Mutex::new(vec![0; self.graph.node_count()]));
    //     for edge in self.graph.edge_indices() {
    //         let target = self.graph.edge_endpoints(edge).unwrap().1;
    //         in_degrees.lock()[target.index()] += 1;
    //     }
    //     debug!("In-Degree count: {:?}", in_degrees);

    //     // Step 2: Enqueue nodes with zero in-degree
    //     let (queue_tx, queue_rx) = kanal::unbounded_async();
    //     for node in &self.order {
    //         if in_degrees.lock()[node.index()] == 0 {
    //             debug!("Ready node: {:?}", self.graph[*node]);
    //             queue_tx.send(Some(*node)).await.expect("Failed to send ready node");
    //         }
    //     }

    //     // Step 3: Parallel processing
    //     let mut tasks = Vec::with_capacity(self.graph.node_count());
    //     while let Some(node_index) = queue_rx.recv().await.expect("Failed to receive ready node") {
    //         let instruments = instruments.clone();
    //         let graph = Arc::clone(&self.graph);
    //         let in_degrees = Arc::clone(&in_degrees);
    //         let queue_tx = queue_tx.clone();

    //         let task = tokio::spawn(async move {
    //             // Calculate the feature
    //             let feature = &graph[node_index];
    //             feature.async_calculate().await;

    //             // Update dependencies and push ready nodes to the queue
    //             for neighbor in graph.neighbors_directed(node_index, petgraph::Outgoing) {
    //                 let mut count = in_degrees.lock()[neighbor.index()];
    //                 count -= 1;
    //                 in_degrees.lock()[neighbor.index()] = count;

    //                 if count == 0 {
    //                     debug!("Ready node: {:?}", graph[neighbor]);
    //                     queue_tx.send(Some(neighbor)).await.expect("Failed to send ready node");

    //                     debug!("Dependency count: {:?}", in_degrees);
    //                     if in_degrees.lock().iter().all(|&x| x == 0) {
    //                         queue_tx.send(None).await.expect("Failed to send ready node");
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
}

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
