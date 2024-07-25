use std::time::Duration;

use petgraph::{
    algo::toposort,
    dot::{Config, Dot},
    graph::DiGraph,
};
use tracing::info;

use crate::config::{FeatureConfig, PipelineConfig};

use super::{EMAGen, Feature, FeatureID, SMAGen, SpreadGen, VWAPGen, VolumeGen};

#[derive(Default)]
pub struct Pipeline {
    graph: DiGraph<Box<dyn Feature>, ()>,
}

impl Pipeline {
    pub fn from_config(config: &PipelineConfig) -> Self {
        let mut graph = DiGraph::new();

        // Create nodes
        config.features.iter().for_each(|c| {
            let f: Box<dyn Feature> = match &c {
                FeatureConfig::Volume(c) => Box::new(VolumeGen::new(c.id.clone(), Duration::from_secs(c.window))),
                FeatureConfig::VWAP(c) => Box::new(VWAPGen::new(c.id.clone(), Duration::from_secs(c.window))),
                FeatureConfig::SMA(c) => {
                    Box::new(SMAGen::new(c.id.clone(), c.source.clone(), Duration::from_secs(c.window)))
                }
                FeatureConfig::EMA(c) => {
                    Box::new(EMAGen::new(c.id.clone(), c.source.clone(), Duration::from_secs(c.window)))
                }
                FeatureConfig::Spread(c) => Box::new(SpreadGen::new(
                    c.id.clone(),
                    c.front_component.clone(),
                    c.back_component.clone(),
                )),
            };
            graph.add_node(f);
        });

        // Add edges automatically
        let mut edges_to_add = vec![];
        for target_node in graph.node_indices() {
            for source in graph[target_node].sources() {
                let source_node = graph.node_indices().find(|i| graph[*i].id() == source).unwrap();
                edges_to_add.push((source_node, target_node));
            }
        }
        for (source, target) in edges_to_add {
            graph.add_edge(source, target, ());
        }

        info!("{:?}", Dot::with_config(&graph, &[Config::EdgeIndexLabel]));

        Pipeline { graph }
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
        config::{EMAConfig, SMAConfig, SpreadConfig, VWAPConfig, VolumeConfig},
        features::{EMAGen, SMAGen, SpreadGen, VWAPGen, VolumeGen},
        logging,
    };
    use std::time::Duration;

    #[test]
    fn test_pipeline_add_node() {
        let mut graph = Pipeline::default();
        let vwap = VWAPGen::new("vwap".into(), Duration::from_secs(60));
        graph.add_node(vwap);
        assert_eq!(graph.graph.node_count(), 1);
    }

    #[test]
    fn test_pipeline_add_edge() {
        let mut graph = Pipeline::default();
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
        let mut graph = Pipeline::default();

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

    #[tokio::test]
    async fn test_pipeline_from_config() {
        logging::init_test_tracing();

        let config = PipelineConfig {
            name: "test_pipeline".to_string(),
            frequency: 1,
            features: vec![
                FeatureConfig::Volume(VolumeConfig {
                    id: "volume_1_min".into(),
                    window: 60,
                }),
                FeatureConfig::VWAP(VWAPConfig {
                    id: "vwap_1_min".into(),
                    window: 60,
                }),
                FeatureConfig::SMA(SMAConfig {
                    id: "sma_10_min".into(),
                    source: "vwap_1_min".into(),
                    window: 600,
                }),
                FeatureConfig::EMA(EMAConfig {
                    id: "ema_10_min".into(),
                    source: "vwap_1_min".into(),
                    window: 600,
                }),
                FeatureConfig::Spread(SpreadConfig {
                    id: "spread".into(),
                    front_component: "sma_10_min".into(),
                    back_component: "ema_10_min".into(),
                }),
            ],
        };

        let pipeline = Pipeline::from_config(&config);
        pipeline.calculate().await;
    }
}
