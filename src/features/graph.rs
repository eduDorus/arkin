use petgraph::{algo::toposort, graph::DiGraph};
use tracing::info;

use super::{Feature, FeatureID};

pub struct FeatureGraph {
    graph: DiGraph<Box<dyn Feature>, ()>,
}

impl Default for FeatureGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureGraph {
    pub fn new() -> Self {
        FeatureGraph {
            graph: DiGraph::new(),
        }
    }

    pub fn add_node<F: Feature + 'static>(&mut self, feature: F) {
        self.graph.add_node(Box::new(feature));
    }

    pub fn add_edge(&mut self, source: &FeatureID, target: &FeatureID) {
        let source_node = self.graph.node_indices().find(|i| self.graph[*i].id() == source).unwrap();
        let target_node = self.graph.node_indices().find(|i| self.graph[*i].id() == target).unwrap();
        self.graph.add_edge(source_node, target_node, ());
    }

    pub fn connect_nodes(&mut self) {
        let mut edges_to_add = vec![];
        for node in self.graph.node_indices() {
            let node_id = self.graph[node].id();
            for source in self.graph[node].sources() {
                edges_to_add.push((source.clone(), node_id.clone()));
            }
        }

        for (source, target) in edges_to_add {
            self.add_edge(&source, &target);
        }
    }

    pub async fn calculate(&self) {
        let res = toposort(&self.graph, None);
        match res {
            Ok(order) => {
                for node in order {
                    let feature = &self.graph[node];
                    feature.calculate().await;
                }
            }
            Err(e) => {
                info!("Error: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        features::{EMAGen, SMAGen, SpreadGen, VWAPGen, VolumeGen},
        logging,
    };
    use std::time::Duration;

    #[test]
    fn test_pipeline_add_node() {
        let mut graph = FeatureGraph::new();
        let vwap = VWAPGen::new("vwap".into(), Duration::from_secs(60));
        graph.add_node(vwap);
        assert_eq!(graph.graph.node_count(), 1);
    }

    #[test]
    fn test_pipeline_add_edge() {
        let mut graph = FeatureGraph::new();
        let vwap = VWAPGen::new("vwap".into(), Duration::from_secs(60));
        let ema = EMAGen::new("ema_vwap_50".into(), "vwap".into(), Duration::from_secs(300));

        graph.add_node(vwap);
        graph.add_node(ema);

        graph.connect_nodes();
        assert_eq!(graph.graph.edge_count(), 1);
    }

    #[tokio::test]
    async fn test_pipeline_calculate() {
        logging::init_test_tracing();

        // Create graph
        let mut graph = FeatureGraph::new();

        // Create features
        let vwap = VWAPGen::new("vwap".into(), Duration::from_secs(60));
        let ema = EMAGen::new("ema_vwap_50".into(), "vwap".into(), Duration::from_secs(50));
        let sma = SMAGen::new("sma_vwap_50".into(), "vwap".into(), Duration::from_secs(50));
        let spread = SpreadGen::new("spread".into(), "sma_vwap_50".into(), "ema_vwap_50".into());
        let volume = VolumeGen::new("volume".into(), Duration::from_secs(60));

        // Create nodes
        graph.add_node(vwap);
        graph.add_node(ema);
        graph.add_node(sma);
        graph.add_node(spread);
        graph.add_node(volume);

        // Connect nodes
        graph.connect_nodes();

        // Calculate
        graph.calculate().await;
        assert_eq!(graph.graph.node_count(), 5);
        assert_eq!(graph.graph.edge_count(), 4);
    }
}
