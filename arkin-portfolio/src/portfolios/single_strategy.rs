use std::{collections::HashMap, sync::Arc};

use arkin_core::prelude::*;
use async_trait::async_trait;
use dashmap::DashMap;
use rust_decimal::Decimal;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use typed_builder::TypedBuilder;

use crate::{Accounting, PortfolioError};

#[derive(Debug, Clone, TypedBuilder)]
pub struct SingleStrategyPortfolio {
    pubsub: Arc<PubSub>,
    #[builder(default = DashMap::new())]
    positions: DashMap<Arc<Instrument>, Arc<PositionUpdate>>,
    #[builder(default = DashMap::new())]
    balances: DashMap<Arc<Asset>, Arc<BalanceUpdate>>,
}

#[async_trait]
impl Accounting for SingleStrategyPortfolio {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), PortfolioError> {
        info!("Starting portfolio...");
        let mut balance_updates = self.pubsub.subscribe::<BalanceUpdate>();
        let mut position_updates = self.pubsub.subscribe::<PositionUpdate>();
        loop {
            tokio::select! {
                Ok(balance) = balance_updates.recv() => {
                    if let Err(e) = self.balance_update(balance).await {
                        error!("Failed to process balance update: {}", e);
                    }
                }
                Ok(position) = position_updates.recv() => {
                    if let Err(e) = self.position_update(position).await {
                        error!("Failed to process position update: {}", e);
                    }
                }
                // Ok(fill) = fill_updates.recv() => {
                //     if let Err(e) = self.fill_update(fill).await {
                //         error!("Failed to process fill update: {}", e);
                //     }
                // }
                _ = shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }

    async fn balance_update(&self, update: Arc<BalanceUpdate>) -> Result<(), PortfolioError> {
        info!("Portfolio processing balance update: {}", update);
        self.balances.insert(update.asset.clone(), update);
        Ok(())
    }

    async fn position_update(&self, update: Arc<PositionUpdate>) -> Result<(), PortfolioError> {
        info!("Portfolio processing position update: {}", update);
        self.positions.insert(update.instrument.clone(), update);
        Ok(())
    }

    async fn balance(&self, asset: &Arc<Asset>) -> Option<Arc<BalanceUpdate>> {
        self.balances.get(asset).map(|v| v.value().clone())
    }

    async fn available_balance(&self, asset: &Arc<Asset>) -> Decimal {
        let current_balance = self.balance(asset).await;
        let current_positions = self.get_positions_by_quote_asset(asset).await;
        let positions_value = current_positions
            .iter()
            .fold(Decimal::ZERO, |acc, (_, s)| acc + s.notional_value());

        match current_balance {
            Some(b) => b.quantity + positions_value,
            None => positions_value,
        }
    }

    async fn get_position_by_instrument(&self, instrument: &Arc<Instrument>) -> Option<Arc<PositionUpdate>> {
        self.positions.get(instrument).map(|v| v.value().clone())
    }

    async fn get_positions(&self) -> HashMap<Arc<Instrument>, Arc<PositionUpdate>> {
        self.positions.iter().map(|e| (e.key().clone(), e.value().clone())).collect()
    }

    async fn get_positions_by_quote_asset(
        &self,
        quote_asset: &Arc<Asset>,
    ) -> HashMap<Arc<Instrument>, Arc<PositionUpdate>> {
        self.positions
            .iter()
            .filter_map(|e| {
                if e.key().quote_asset == *quote_asset {
                    Some((e.key().clone(), e.value().clone()))
                } else {
                    None
                }
            })
            .collect()
    }
}
