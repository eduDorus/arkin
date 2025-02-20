use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::{collections::HashMap, sync::Arc};
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{ledger::Ledger, Accounting, AccountingError, AccountingService};

#[derive(Debug, TypedBuilder)]
pub struct LedgerAccounting {
    pubsub: Arc<PubSub>,
    #[builder(default = Ledger::builder().build())]
    ledger: Ledger, // Changed to Arc for sharing
}

impl LedgerAccounting {
    pub fn order_update(&self, order: Arc<VenueOrder>) -> Result<(), AccountingError> {
        match order.instrument.instrument_type {
            InstrumentType::Spot => {
                info!("Portfolio processing spot order: {}", order);
            }
            InstrumentType::Perpetual => {
                info!("Portfolio processing perpetual order: {}", order);
                let margin_amount = order.last_fill_price * order.last_fill_quantity * order.instrument.contract_size;
                self.margin_trade(
                    order.side,
                    order.strategy.clone(),
                    order.instrument.clone(),
                    order.commission_asset.clone(),
                    order.last_fill_quantity,
                    margin_amount,
                    order.last_fill_commission,
                )?;
            }
            InstrumentType::Future => {
                // Handle future order
                info!("Portfolio processing future order: {}", order);
            }
            _ => {
                error!("Unsupported instrument type: {}", order.instrument.instrument_type);
                return Err(AccountingError::UnsupportedInstrumentType(
                    order.instrument.instrument_type.clone(),
                ));
            }
        }

        Ok(())
    }

    pub fn deposit(
        &self,
        from_venue: &Arc<Venue>,
        to_venue: &Arc<Venue>,
        asset: &Tradable,
        amount: Decimal,
    ) -> Result<(), AccountingError> {
        let from_account_type = AccountType::Venue(from_venue.clone());
        let to_account_type = AccountType::VenueAccount(to_venue.clone());

        let from_account = self.ledger.find_or_create_account(&from_account_type, &asset);
        let to_account = self.ledger.find_or_create_account(&to_account_type, &asset);

        self.ledger.transfer(from_account, to_account, amount)
    }

    pub fn withdraw(
        &self,
        from_venue: &Arc<Venue>,
        to_venue: &Arc<Venue>,
        asset: &Tradable,
        amount: Decimal,
    ) -> Result<(), AccountingError> {
        let from_account_type = AccountType::VenueAccount(from_venue.clone());
        let to_account_type = AccountType::Venue(to_venue.clone());

        let from_account = self.ledger.find_or_create_account(&from_account_type, &asset);
        let to_account = self.ledger.find_or_create_account(&to_account_type, &asset);

        self.ledger.transfer(from_account, to_account, amount)
    }

    pub fn exchange(
        &self,
        venue: Arc<Venue>,
        from_asset: Tradable,
        to_asset: Tradable,
        from_amount: Decimal,
        to_amount: Decimal,
    ) -> Result<(), AccountingError> {
        let user_account_type = AccountType::VenueAccount(venue.clone());
        let venue_account_type = AccountType::Venue(venue.clone());

        let from_account = self.ledger.find_or_create_account(&user_account_type, &from_asset);
        let to_account = self.ledger.find_or_create_account(&user_account_type, &to_asset);
        let venue_from_account = self.ledger.find_or_create_account(&venue_account_type, &from_asset);
        let venue_to_account = self.ledger.find_or_create_account(&venue_account_type, &to_asset);

        self.ledger.exchange(
            from_account,
            to_account,
            venue_from_account,
            venue_to_account,
            from_amount,
            to_amount,
        )
    }

    pub fn margin_trade(
        &self,
        side: MarketSide,
        strategy: Arc<Strategy>,
        instrument: Arc<Instrument>,
        commission_asset: Option<Arc<Asset>>,
        instrument_amount: Decimal,
        margin_amount: Decimal,
        commission_amount: Decimal,
    ) -> Result<(), AccountingError> {
        let venue = instrument.venue.clone();
        let instrument_asset = Tradable::Instrument(instrument.clone());
        let margin_asset = Tradable::Asset(instrument.margin_asset.clone());
        let commission_asset = Tradable::Asset(commission_asset.unwrap_or_else(|| instrument.margin_asset.clone()));

        // Get account types
        let asset_account_type = AccountType::VenueAccount(venue.clone());
        let instrument_account_type = AccountType::Strategy(strategy.clone());
        let venue_account_type = AccountType::Venue(venue.clone());

        // Get accounts
        let user_margin_account = self.ledger.find_or_create_account(&asset_account_type, &margin_asset);
        let venue_margin_account = self.ledger.find_or_create_account(&venue_account_type, &margin_asset);

        let user_instrument_account = self.ledger.find_or_create_account(&instrument_account_type, &instrument_asset);
        let venue_instrument_account = self.ledger.find_or_create_account(&venue_account_type, &instrument_asset);

        // Commission accounts
        let commission_debit_account = self.ledger.find_or_create_account(&asset_account_type, &commission_asset);
        let commission_credit_account = self.ledger.find_or_create_account(&venue_account_type, &commission_asset);

        // Figure out if there is already a position and which side
        let user_instrument_account_balance = self.ledger.get_balance(user_instrument_account.id)?;
        let position_side = match user_instrument_account_balance {
            balance if balance.is_zero() => None,
            balance if balance > Decimal::ZERO => Some(PositionSide::Long),
            balance if balance < Decimal::ZERO => Some(PositionSide::Short),
            balance => return Err(AccountingError::InvalidBalance(balance)),
        };

        // Not we figure out who to debit and who to credit
        let (margin_debit_account, margin_credit_account) = match (position_side, side) {
            (None, MarketSide::Buy) => (user_margin_account, venue_margin_account),
            (None, MarketSide::Sell) => (user_margin_account, venue_margin_account),
            (Some(PositionSide::Long), MarketSide::Buy) => (user_margin_account, venue_margin_account),
            (Some(PositionSide::Long), MarketSide::Sell) => (venue_margin_account, user_margin_account),
            (Some(PositionSide::Short), MarketSide::Buy) => (venue_margin_account, user_margin_account),
            (Some(PositionSide::Short), MarketSide::Sell) => (user_margin_account, venue_margin_account),
        };

        let (instrument_debit_account, instrument_credit_account) = match (position_side, side) {
            (None, MarketSide::Buy) => (venue_instrument_account, user_instrument_account),
            (None, MarketSide::Sell) => (venue_instrument_account, user_instrument_account),
            (Some(PositionSide::Long), MarketSide::Buy) => (venue_instrument_account, user_instrument_account),
            (Some(PositionSide::Long), MarketSide::Sell) => (user_instrument_account, venue_instrument_account),
            (Some(PositionSide::Short), MarketSide::Buy) => (user_instrument_account, venue_instrument_account),
            (Some(PositionSide::Short), MarketSide::Sell) => (venue_instrument_account, user_instrument_account),
        };

        // Now we can do the transfer
        self.ledger.margin_trade(
            margin_debit_account,
            margin_credit_account,
            instrument_debit_account,
            instrument_credit_account,
            commission_debit_account,
            commission_credit_account,
            margin_amount,
            instrument_amount,
            commission_amount,
        )
    }
}

#[async_trait]
impl AccountingService for LedgerAccounting {}

#[async_trait]
impl RunnableService for LedgerAccounting {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        let mut rx = self.pubsub.subscribe();

        while !shutdown.is_cancelled() {
            select! {
                Ok(event) = rx.recv() => {
                    match event {
                        Event::VenueOrderFillUpdate(order) => {
                          self.order_update(order).unwrap_or_else(|e| error!("Failed to process order update: {}", e));
                        }
                        _ => {}
                    }
                }
                _ = shutdown.cancelled() => {
                    info!("Portfolio shutting down");
                    break;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Accounting for LedgerAccounting {
    /// Update the current balance of a given asset
    /// This comes from the exchange and should be reconciled with the portfolio
    async fn balance_update(&self, _update: Arc<BalanceUpdate>) -> Result<(), AccountingError> {
        Ok(())
    }

    /// Update the current position of a given instrument
    /// This comes from the exchange and should be reconciled with the portfolio
    async fn position_update(&self, _update: Arc<PositionUpdate>) -> Result<(), AccountingError> {
        Ok(())
    }

    /// Provides the current price of a specific assets in the portfolio
    async fn balance(&self, _asset: &Arc<Asset>) -> Option<Arc<BalanceUpdate>> {
        None
    }

    /// Provides the total value of a given asset minus the liabilities. It's the the total net worth in this asset.
    async fn available_balance(&self, _asset: &Arc<Asset>) -> Decimal {
        dec!(10_000)
    }

    /// Provide the current open position
    async fn get_position_by_instrument(&self, _instrument: &Arc<Instrument>) -> Option<Arc<PositionUpdate>> {
        None
    }

    /// Provies a list of all open positions
    async fn get_positions(&self) -> HashMap<Arc<Instrument>, Arc<PositionUpdate>> {
        HashMap::new()
    }

    /// Provides a list of all open positions with a given quote asset
    async fn get_positions_by_quote_asset(&self, _asset: &Arc<Asset>) -> HashMap<Arc<Instrument>, Arc<PositionUpdate>> {
        HashMap::new()
    }
}
