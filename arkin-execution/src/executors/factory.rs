use crate::{config::ExecutionEndpointConfig, simulation::SimulationEndpoint};

use super::{binance::BinanceEndpoint, ExecutionEndpoint};

pub struct ExecutionEndpointFactory {}

impl ExecutionEndpointFactory {
    pub fn from_config(configs: &[ExecutionEndpointConfig]) -> Vec<Box<dyn ExecutionEndpoint>> {
        configs
            .iter()
            .map(|config| {
                let endpoint: Box<dyn ExecutionEndpoint> = match config {
                    ExecutionEndpointConfig::Simulation(c) => Box::new(SimulationEndpoint::from_config(c)),
                    ExecutionEndpointConfig::Binance(c) => Box::new(BinanceEndpoint::from_config(c)),
                };
                endpoint
            })
            .collect()
    }
}
