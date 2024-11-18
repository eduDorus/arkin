use std::{str::FromStr, sync::Arc};

use arkin_core::prelude::*;
use async_trait::async_trait;
use rust_decimal::prelude::*;
use tracing::info;
use uuid::Uuid;

use crate::{errors::ExecutionError, protocol::Order};



pub struct OrderManager<T: Executor> {
    executor: T,
}

impl<T: Executor> OrderManager<T> {
    pub fn new(executor: T) -> Self {
        info!("Initializing order manager");
        Self { executor }
    }

    async fn place_order(&self) {
        info!("Running order manager");

        let order = sim_order();
        let res = T::place_order(order).await;
        match res {
            Ok(_) => info!("Order placed successfully"),
            Err(e) => info!("Error placing order: {:?}", e),
        }
    }
}

pub fn sim_order() -> Order {
    let venue = Venue {
        id: Uuid::parse_str("48adfe42-29fb-4402-888a-0204bf417e32").expect("Invalid UUID"),
        name: "Binance".into(),
        venue_type: "exchange".into(),
    };
    let instrument = Instrument {
        id: Uuid::from_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6").expect("Invalid UUID"),
        venue,
        symbol: "perp-btc-usdt@binance".into(),
        venue_symbol: "BTCUSDT".into(),
        instrument_type: InstrumentType::Perpetual,
        base_asset: "btc".into(),
        quote_asset: "usdt".into(),
        maturity: None,
        strike: None,
        option_type: None,
        contract_size: Decimal::from_f64(1.0).expect("Invalid decimal"),
        price_precision: 2,
        quantity_precision: 3,
        base_precision: 8,
        quote_precision: 8,
        tick_size: Decimal::from_f64(0.10).expect("Invalid decimal"),
        lot_size: Decimal::from_f64(0.001).expect("Invalid decimal"),
        status: InstrumentStatus::Trading,
    };

    // Wrap in arc
    let instrument = Arc::new(instrument);

    // Simulate sending orders
    let order = Order::new(1, instrument, MarketSide::Buy, 10000.0, 1.0);

    order
}

#[cfg(test)]
mod tests {

    use arkin_core::test_setup;

    use super::*;

    #[tokio::test]
    async fn test_execution_manager() {
        test_setup();

        // Create Order Manager
        // let mut manager = OrderManager::new();

        // Creating executors
        // let sim_executor = sim::SimulationExecutor::new();

        info!("Done");
    }
}

// impl OrderManager {
//     pub fn new() -> Self {
//         Self {
//             executors: HashMap::new(),
//             message_rx,
//         }
//     }

//     pub fn add_executor(&mut self, executor_handle: ExecutorHandle) {
//         self.executors.insert(executor_handle.name.clone(), executor_handle);
//     }

//     pub async fn run(&mut self) {
//         let venue = Venue {
//             id: Uuid::parse_str("48adfe42-29fb-4402-888a-0204bf417e32").expect("Invalid UUID"),
//             name: "Binance".into(),
//             venue_type: "exchange".into(),
//         };
//         let instrument = Instrument {
//             id: Uuid::from_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6").expect("Invalid UUID"),
//             venue,
//             symbol: "perp-btc-usdt@binance".into(),
//             venue_symbol: "BTCUSDT".into(),
//             instrument_type: InstrumentType::Perpetual,
//             base_asset: "btc".into(),
//             quote_asset: "usdt".into(),
//             maturity: None,
//             strike: None,
//             option_type: None,
//             contract_size: Decimal::from_f64(1.0).expect("Invalid decimal"),
//             price_precision: 2,
//             quantity_precision: 3,
//             base_precision: 8,
//             quote_precision: 8,
//             tick_size: Decimal::from_f64(0.10).expect("Invalid decimal"),
//             lot_size: Decimal::from_f64(0.001).expect("Invalid decimal"),
//             status: InstrumentStatus::Trading,
//         };

//         // Wrap in arc
//         let instrument = Arc::new(instrument);

//         // Simulate sending orders
//         let order = Order::new(1, instrument, MarketSide::Buy, 10000.0, 1.0);

//         // Send order to all executors
//         for executor in self.executors.values() {
//             executor
//                 .command_tx
//                 .send_async(ExecutorRequest::PlaceOrder(order.clone()))
//                 .await
//                 .unwrap();
//         }
//     }
// }
