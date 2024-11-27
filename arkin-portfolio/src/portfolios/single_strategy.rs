use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use arkin_core::prelude::*;
use async_trait::async_trait;
use dashmap::DashMap;
use derive_builder::Builder;
use rust_decimal::Decimal;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use crate::{Portfolio, PortfolioError};

#[derive(Debug, Clone, Builder)]
pub struct SingleStrategyPortfolio {
    pubsub: Arc<PubSub>,
    #[builder(default = "DashMap::new()")]
    positions: DashMap<Arc<Instrument>, BTreeSet<Position>>,
    #[builder(default = "DashMap::new()")]
    holdings: DashMap<AssetId, Holding>,
}

#[async_trait]
impl Portfolio for SingleStrategyPortfolio {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), PortfolioError> {
        info!("Starting portfolio...");
        let mut price_updates = self.pubsub.subscribe::<Tick>();
        let mut balance_updates = self.pubsub.subscribe::<Holding>();
        let mut position_updates = self.pubsub.subscribe::<Position>();
        let mut fill_updates = self.pubsub.subscribe::<ExecutionOrderFill>();
        loop {
            tokio::select! {
                Ok(tick) = price_updates.recv() => {
                    if let Err(e) = self.price_update(tick).await {
                        error!("Failed to process price update: {}", e);
                    }
                }
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
                Ok(fill) = fill_updates.recv() => {
                    if let Err(e) = self.fill_update(fill).await {
                        error!("Failed to process fill update: {}", e);
                    }
                }
                _ = shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }

    async fn price_update(&self, tick: Tick) -> Result<(), PortfolioError> {
        debug!("Portfolio processing price update: {}", tick);

        // Update the price of the instrument in the position
        if let Some(mut e) = self.positions.get_mut(&tick.instrument) {
            if let Some(last_position) = e.value().last() {
                let mut position = last_position.clone();
                position.update_price(tick.mid_price());
                e.value_mut().replace(position);
            }
        }
        Ok(())
    }

    async fn balance_update(&self, holding: Holding) -> Result<(), PortfolioError> {
        info!("Portfolio processing balance update: {}", holding);
        // Check if we have the asset in the holdings else create
        if self.holdings.contains_key(&holding.asset) {
            self.holdings.alter(&holding.asset, |_, mut h| {
                h.quantity = holding.quantity;
                h
            });
        } else {
            self.holdings.insert(holding.asset.clone(), holding);
        }
        Ok(())
    }

    async fn position_update(&self, _position: Position) -> Result<(), PortfolioError> {
        unimplemented!("Position update not implemented");
    }

    async fn fill_update(&self, fill: ExecutionOrderFill) -> Result<(), PortfolioError> {
        info!("Portfolio processing fill update: {}", fill);
        // Reduce the balance of the quote asset
        let cost = fill.total_cost();
        let quote_asset = fill.instrument.quote_asset.clone();
        if let Some(mut holding) = self.holdings.get_mut(&quote_asset) {
            holding.quantity += cost;
        } else {
            return Err(PortfolioError::AssetNotFound(quote_asset.to_string()));
        }

        // Update the position
        let instrument = fill.instrument.clone();
        let mut entry = self.positions.entry(instrument).or_insert_with(|| BTreeSet::new());
        if let Some(last_position) = entry.value().last() {
            if last_position.is_open() {
                let mut position = last_position.clone();
                let remaining_fill = position.update_fill(fill);
                entry.value_mut().replace(position);
                if let Some(remaining_fill) = remaining_fill {
                    let new_position = Position::from(remaining_fill);
                    entry.value_mut().insert(new_position);
                }
            } else {
                let new_position = Position::from(fill);
                entry.value_mut().insert(new_position);
            }
        } else {
            let new_position = Position::from(fill);
            entry.value_mut().insert(new_position);
        }
        Ok(())
    }

    async fn balance(&self, asset: &AssetId) -> Option<Holding> {
        self.holdings.get(asset).map(|v| v.value().clone())
    }

    async fn total_balance(&self) -> HashMap<AssetId, Holding> {
        self.holdings.iter().map(|v| (v.key().clone(), v.value().clone())).collect()
    }

    async fn list_positions_with_quote_asset(
        &self,
        quote_asset: &AssetId,
    ) -> HashMap<Arc<Instrument>, BTreeSet<Position>> {
        self.positions
            .iter()
            .filter(|e| &e.key().quote_asset == quote_asset)
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    async fn list_positions_with_instrument(
        &self,
        instrument: &Arc<Instrument>,
    ) -> HashMap<Arc<Instrument>, BTreeSet<Position>> {
        self.positions
            .iter()
            .filter(|e| e.key() == instrument)
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    async fn get_open_position(&self, instrument: &Arc<Instrument>) -> Option<Position> {
        self.positions
            .get(instrument)
            .and_then(|v| v.iter().find(|p| p.is_open()).cloned())
    }

    async fn list_open_positions(&self) -> HashMap<Arc<Instrument>, Position> {
        self.positions
            .iter()
            .filter_map(|e| {
                if let Some(p) = e.value().last() {
                    if p.is_open() {
                        Some((e.key().clone(), p.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    async fn list_open_positions_with_quote_asset(&self, quote_asset: &AssetId) -> HashMap<Arc<Instrument>, Position> {
        self.positions
            .iter()
            .filter_map(|e| {
                if let Some(p) = e.value().last() {
                    if p.is_open() && p.instrument.quote_asset == *quote_asset {
                        Some((e.key().clone(), p.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    async fn list_closed_positions(&self) -> HashMap<Arc<Instrument>, BTreeSet<Position>> {
        self.positions
            .iter()
            .map(|e| (e.key().clone(), e.value().iter().filter(|p| p.is_closed()).cloned().collect()))
            .collect()
    }

    async fn capital(&self, asset: &AssetId) -> Notional {
        let current_balance = self.balance(asset).await;
        let current_positions = self.list_open_positions_with_quote_asset(asset).await;
        let positions_value = current_positions
            .iter()
            .fold(Notional::ZERO, |acc, (_, s)| acc + s.market_value());

        match current_balance {
            Some(b) => b.quantity + positions_value,
            None => positions_value,
        }
    }

    async fn total_capital(&self) -> HashMap<AssetId, Notional> {
        let mut capital = HashMap::new();
        for entry in self.holdings.iter() {
            capital.insert(entry.key().clone(), self.capital(entry.key()).await);
        }
        capital
    }

    async fn buying_power(&self, asset: &AssetId) -> Notional {
        self.holdings.get(asset).map(|v| v.value().quantity).unwrap_or(Notional::ZERO)
    }

    async fn total_buying_power(&self) -> HashMap<AssetId, Notional> {
        self.holdings.iter().map(|v| (v.key().clone(), v.value().quantity)).collect()
    }

    async fn pnl_asset(&self, asset: &AssetId) -> Notional {
        self.list_positions_with_quote_asset(asset)
            .await
            .iter()
            .fold(Notional::ZERO, |acc, (_, p)| {
                acc + p.iter().fold(Decimal::ZERO, |acc, p| acc + p.realized_pnl)
            })
    }

    async fn pnl_instrument(&self, instrument: &Arc<Instrument>) -> Notional {
        self.list_positions_with_instrument(instrument)
            .await
            .iter()
            .fold(Notional::ZERO, |acc, (_, p)| {
                acc + p.iter().fold(Decimal::ZERO, |acc, p| acc + p.realized_pnl)
            })
    }

    async fn total_pnl(&self) -> HashMap<AssetId, Notional> {
        let mut pnl = HashMap::new();
        for entry in self.holdings.iter() {
            pnl.insert(entry.key().clone(), self.pnl_asset(entry.key()).await);
        }
        pnl
    }

    async fn commission_asset(&self, asset: &AssetId) -> Notional {
        self.list_positions_with_quote_asset(asset)
            .await
            .iter()
            .fold(Notional::ZERO, |acc, (_, p)| {
                acc + p.iter().fold(Decimal::ZERO, |acc, p| acc + p.total_commission)
            })
    }

    async fn commission_instrument(&self, instrument: &Arc<Instrument>) -> Notional {
        self.list_positions_with_instrument(instrument)
            .await
            .iter()
            .fold(Notional::ZERO, |acc, (_, p)| {
                acc + p.iter().fold(Decimal::ZERO, |acc, p| acc + p.total_commission)
            })
    }

    async fn total_commission(&self) -> HashMap<AssetId, Notional> {
        let mut commission = HashMap::new();
        for entry in self.holdings.iter() {
            commission.insert(entry.key().clone(), self.commission_asset(entry.key()).await);
        }
        commission
    }
}
