use std::sync::Arc;

use crate::{config::ExecutionEndpointConfig, state::StateManager};

use super::{binance::BinanceEndpoint, ExecutionEndpoint, SimulationEndpoint};

pub struct ExecutionEndpointFactory {}

impl ExecutionEndpointFactory {
    pub fn from_config(
        state: Arc<StateManager>,
        configs: &[ExecutionEndpointConfig],
    ) -> Vec<Box<dyn ExecutionEndpoint>> {
        configs
            .iter()
            .map(|config| {
                let endpoint: Box<dyn ExecutionEndpoint> = match config {
                    ExecutionEndpointConfig::Simulation(c) => {
                        Box::new(SimulationEndpoint::from_config(state.clone(), c))
                    }
                    ExecutionEndpointConfig::Binance(c) => Box::new(BinanceEndpoint::from_config(c)),
                };
                endpoint
            })
            .collect()
    }
}
