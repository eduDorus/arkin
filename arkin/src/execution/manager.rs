use tracing::{debug, warn};

use super::{Execution, ExecutionEndpoint, ExecutionEndpointFactory};
use crate::{
    config::ExecutionManagerConfig,
    models::{Allocation, Event, Notional, Order, Position, Price, Tick, Venue},
    portfolio::Portfolio,
    state::StateManager,
};
use core::fmt;
use std::{collections::HashMap, sync::Arc};

pub struct ExecutionManager {
    state: Arc<StateManager>,
    portfolio: Arc<Portfolio>,
    endpoints: HashMap<Venue, Box<dyn ExecutionEndpoint>>,
    default_endpoint: Venue,
    rebalance_threshold: Notional,
}

impl ExecutionManager {
    pub fn from_config(state: Arc<StateManager>, portfolio: Arc<Portfolio>, config: &ExecutionManagerConfig) -> Self {
        let endpoints = ExecutionEndpointFactory::from_config(state.clone(), &config.endpoints)
            .into_iter()
            .map(|endpoint| (endpoint.venue().clone(), endpoint))
            .collect();
        Self {
            state,
            endpoints,
            portfolio,
            default_endpoint: config.default_endpoint.clone(),
            rebalance_threshold: config.rebalance_threshold.into(),
        }
    }
}

impl Execution for ExecutionManager {
    fn allocate(&self, allocations: &[Allocation]) {
        if allocations.is_empty() {
            warn!("No allocations to execute");
            return;
        }

        let positions = self.portfolio.positions(&allocations[0].event_time);

        // Difference between current position and allocation
        let new_allocations = allocations.iter().filter_map(|a| {
            let pos = positions
                .get(&(a.strategy_id.clone(), a.instrument.clone()))
                .expect("There should be a position");
            if let Some(tick) = self.state.latest_event_by_instrument::<Tick>(&a.instrument, &a.event_time) {
                Some(EnrichedAllocation::new(tick.mid_price(), a.clone(), pos.clone()))
            } else {
                warn!("No price found for instrument: {}", a.instrument);
                None
            }
        });

        // Filter out allocations that are below the rebalance threshold of the portfolio
        let filtered_allocations = new_allocations
            .into_iter()
            .filter(|a| a.difference().abs() > self.rebalance_threshold)
            .collect::<Vec<_>>();

        for a in &filtered_allocations {
            debug!("Final allocation: {}", a);
        }

        // Create orders
        let orders = filtered_allocations
            .into_iter()
            .map(|a| {
                let quantity = a.difference() / a.current_price;
                Order::new_market(
                    a.allocation.event_time,
                    a.allocation.instrument,
                    a.allocation.strategy_id,
                    quantity,
                )
            })
            .collect();

        // Mimick execution by filling all orders and update the state with fills
        if let Some(endpoint) = self.endpoints.get(&self.default_endpoint) {
            let fills = endpoint.place_orders(orders);
            for fill in fills {
                self.state.add_event(Event::Fill(fill));
            }
        }
    }
}

struct EnrichedAllocation {
    current_price: Price,
    allocation: Allocation,
    position: Position,
}

impl EnrichedAllocation {
    fn new(current_price: Price, allocation: Allocation, position: Position) -> Self {
        Self {
            current_price,
            allocation,
            position,
        }
    }

    fn difference(&self) -> Notional {
        self.allocation.notional - self.current_price * self.position.quantity
    }

    fn exposure(&self) -> Notional {
        self.current_price * self.position.quantity
    }
}

impl fmt::Display for EnrichedAllocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ENRICHED ALLOCATION {} {}: strategy: {} current price: {} notional: {} diff notional: {} current exposure: {}",
            self.allocation.event_time,
            self.allocation.instrument,
            self.allocation.strategy_id,
            self.current_price,
            self.allocation.notional,
            self.difference(),
            self.exposure()

        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{ExecutionEndpointConfig, SimulationConfig},
        logging,
        models::Notional,
        portfolio::Portfolio,
        test_utils,
    };
    use rust_decimal::prelude::*;

    #[test]
    fn test_execution_manager() {
        logging::init_test_tracing();

        let instrument = test_utils::test_perp_instrument();
        let allocations = test_utils::allocations(&instrument);

        let state = test_utils::TestStateBuilder::default().add_ticks(&instrument).build();
        let portfolio = Arc::new(Portfolio::new(state.clone(), Notional::from(1000.)));
        let manager = ExecutionManager::from_config(
            state,
            portfolio,
            &ExecutionManagerConfig {
                endpoints: vec![ExecutionEndpointConfig::Simulation(SimulationConfig {
                    latency: 200,
                    commission_maker: Decimal::from_f64(0.00015).unwrap(),
                    commission_taker: Decimal::from_f64(0.0003).unwrap(),
                    max_orders_per_minute: 60,
                    max_order_size_notional: Decimal::from_f64(2000.).unwrap(),
                    min_order_size_notional: Decimal::from_f64(10.).unwrap(),
                })],
                default_endpoint: Venue::Simulation,
                rebalance_threshold: Decimal::from_f64(50.).unwrap(),
            },
        );

        manager.allocate(&allocations);
    }
}
