use tracing::info;

use crate::models::Order;

pub trait OrderManager {
    fn handle_order_update(&self, update: &Order);
}

pub enum OrderManagerType {
    SingleVenue(SingleOrderManager),
}

impl OrderManager for OrderManagerType {
    fn handle_order_update(&self, update: &Order) {
        match self {
            OrderManagerType::SingleVenue(manager) => manager.handle_order_update(update),
        }
    }
}

#[derive(Default)]
pub struct SingleOrderManager {}

impl SingleOrderManager {
    pub fn new() -> Self {
        SingleOrderManager {}
    }
}

impl OrderManager for SingleOrderManager {
    fn handle_order_update(&self, update: &Order) {
        info!("OrderManager received order update: {}", update)
    }
}
