use tracing::info;

use crate::models::OrderUpdate;

pub mod errors;

pub trait OrderManager {
    fn handle_order_update(&self, update: &OrderUpdate);
}

pub enum OrderManagerType {
    SingleVenue(SingleOrderManager),
}

impl OrderManager for OrderManagerType {
    fn handle_order_update(&self, update: &OrderUpdate) {
        match self {
            OrderManagerType::SingleVenue(manager) => manager.handle_order_update(update),
        }
    }
}

pub struct SingleOrderManager {}

impl SingleOrderManager {
    pub fn new() -> Self {
        SingleOrderManager {}
    }
}

impl OrderManager for SingleOrderManager {
    fn handle_order_update(&self, update: &OrderUpdate) {
        info!("OrderManager received order update: {}", update)
    }
}
