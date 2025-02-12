use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{
    traits::{Executor, ExecutorService},
    ExecutorError,
};

#[derive(Debug, TypedBuilder)]

pub struct SimulationExecutor {
    pubsub: Arc<PubSub>,
    #[builder(default)]
    orders: DashMap<VenueOrderId, Arc<VenueOrder>>,
    #[builder(default = dec!(0.0005))]
    taker_commission: Decimal,
    #[builder(default = dec!(0.0002))]
    maker_commission: Decimal,
    #[builder(default = DashMap::new())]
    balances: DashMap<Arc<Asset>, Balance>,
}

impl SimulationExecutor {
    pub fn list_open_orders(&self) -> Vec<Arc<VenueOrder>> {
        self.orders
            .iter()
            .filter(|order| !order.value().is_active())
            .map(|order| order.value().clone())
            .collect()
    }

    pub fn fill_order(&self, _id: VenueOrderId, _fill: Arc<VenueOrderFill>) {
        unimplemented!("SimulationExecutor::fill_order")
        // if let Some(mut order) = self.orders.get_mut(&id) {
        //     order.add_fill(fill.clone());
        //     info!("SimulationExecutor filled order: {}", fill);
        // }

        // // Remove the order if it is filled
        // let is_finalized = self.orders.get(&id).map(|order| order.is_finalized()).unwrap_or(false);
        // if is_finalized {
        //     self.orders.remove(&id);
        // }
    }

    pub fn update_balance(&self, _asset: &Arc<Asset>, _quantity: Decimal) {
        unimplemented!("SimulationExecutor::update_balance")
        // let mut entry = self
        //     .balances
        //     .entry(asset.clone())
        //     .or_insert(Balance::builder().asset(asset.clone()).quantity(dec!(0)).build());
        // entry.quantity += quantity;
    }

    pub fn get_balance(&self, asset: &Arc<Asset>) -> Option<Balance> {
        self.balances.get(asset).map(|holding| holding.value().clone())
    }

    pub async fn tick_update(&self, tick: Arc<Tick>) {
        // Fill the order
        let open_orders = self.list_open_orders();

        // check if we got a price for the instrument
        for order in open_orders.iter() {
            if order.instrument == tick.instrument {
                let price = if order.side == MarketSide::Buy {
                    tick.ask_price()
                } else {
                    tick.bid_price()
                };
                let quantity = order.quantity;

                // Calculate commission
                let mut commission = match order.order_type {
                    VenueOrderType::Market => (price * quantity) * self.taker_commission,
                    VenueOrderType::Limit => (price * quantity) * self.maker_commission,
                    _ => unimplemented!("Unsupported order type"),
                };
                commission = commission.round_dp(order.instrument.price_precision);

                // Create the fill
                let fill = VenueOrderFill::builder()
                    .venue_order(order.clone())
                    .instrument(order.instrument.clone())
                    .side(order.side.clone())
                    .price(price)
                    .quantity(order.quantity)
                    .commission(commission)
                    .build();
                let fill = Arc::new(fill);

                // Subtract the value from the balance
                // self.update_balance(&order.instrument.base_asset, fill.market_value() + fill.commission);
                self.fill_order(order.id.clone(), fill.clone());

                // Publish
                // info!("Publishing new balance: {}", holding);
                // self.pubsub.publish::<Holding>(holding);
                info!("SimulationExecutor publishing order filled: {}", order);
                self.pubsub.publish(fill).await;
            }
        }
    }
}

#[async_trait]
impl Executor for SimulationExecutor {
    async fn get_account(&self) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::get_account")
    }

    async fn get_balances(&self) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::get_balances")
    }

    async fn get_positions(&self) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::get_positions")
    }

    async fn place_order(&self, order: Arc<VenueOrder>) -> Result<(), ExecutorError> {
        self.orders.insert(order.id, order.clone());
        info!("SimulationExecution placed order: {}", order);
        Ok(())
    }

    async fn place_orders(&self, orders: Vec<Arc<VenueOrder>>) -> Result<(), ExecutorError> {
        for order in orders {
            self.orders.insert(order.id, order);
        }
        Ok(())
    }

    async fn modify_order(&self, _order: Arc<VenueOrder>) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::modify_order")
    }

    async fn modify_orders(&self, _orders: Vec<Arc<VenueOrder>>) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::modify_orders")
    }

    async fn cancel_order(&self, _id: VenueOrderId) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::cancel_order")
        // if let Some(mut order) = self.orders.get_mut(&id) {
        //     order.cancel();
        //     info!("SimulationExecution cancelled order: {}", *order);
        //     Ok(())
        // } else {
        //     return Err(ExecutorError::InvalidOrder(id.to_string()));
        // }
    }

    async fn cancel_orders(&self, ids: Vec<VenueOrderId>) -> Result<(), ExecutorError> {
        for id in ids {
            self.cancel_order(id).await?;
        }
        Ok(())
    }

    async fn cancel_orders_by_instrument(&self, _instrument: Arc<Instrument>) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::cancel_orders_by_instrument")
    }

    async fn cancel_all_orders(&self) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::cancel_all_orders")
        // for mut order in self.orders.iter_mut() {
        //     order.cancel();
        //     info!("SimulationExecution cancelled order: {}", *order);
        // }
        // Ok(())
    }
}

#[async_trait]
impl RunnableService for SimulationExecutor {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting simulation executor...");
        // TODO: Send current balance
        let holding = Arc::new(
            Balance::builder()
                .asset(test_usdt_asset())
                .portfolio(test_portfolio())
                .quantity(dec!(10000))
                .build(),
        );
        self.update_balance(&holding.asset, holding.quantity);
        info!("Sending initial balance: {}", holding);
        self.pubsub.publish(holding).await;

        let mut rx = self.pubsub.subscribe();
        loop {
            select! {
                Ok(event) = rx.recv() => {
                    match event {
                       Event::VenueOrder(order) => {
                           info!("SimulationExecutor received order: {}", order);
                           self.place_order(order).await?;
                       }
                       Event::Tick(tick) => {
                           debug!("SimulationExecutor received tick: {}", tick.instrument);
                           self.tick_update(tick).await;
                       }
                       _ => {}

                    }
                }
                _ = shutdown.cancelled() => {
                    info!("SimulationExecutor shutdown...");
                    break;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ExecutorService for SimulationExecutor {}
