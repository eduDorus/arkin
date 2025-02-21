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
    pub fn deposit(
        &self,
        debit_venue: &Arc<Venue>,
        credit_venue: &Arc<Venue>,
        asset: &Tradable,
        amount: Decimal,
    ) -> Result<(), AccountingError> {
        let debit_account_type = AccountType::VenueSpot;
        let credit_account_type = AccountType::ClientSpot;

        let debit_account = self
            .ledger
            .find_or_create_account(debit_venue, &asset, None, &debit_account_type)?;
        let credit_account = self
            .ledger
            .find_or_create_account(credit_venue, &asset, None, &credit_account_type)?;

        self.ledger.transfer(&debit_account, &credit_account, amount)
    }

    pub fn withdraw(
        &self,
        debit_venue: &Arc<Venue>,
        credit_venue: &Arc<Venue>,
        asset: &Tradable,
        amount: Decimal,
    ) -> Result<(), AccountingError> {
        let debit_account_type = AccountType::ClientSpot;
        let credit_account_type = AccountType::VenueSpot;

        let debit_account = self
            .ledger
            .find_or_create_account(debit_venue, asset, None, &debit_account_type)?;
        let credit_account = self
            .ledger
            .find_or_create_account(credit_venue, asset, None, &credit_account_type)?;

        self.ledger.transfer(&debit_account, &credit_account, amount)
    }

    pub fn exchange(
        &self,
        venue: Arc<Venue>,
        debit_asset: Tradable,
        credit_asset: Tradable,
        debit_amount: Decimal,
        credit_amount: Decimal,
    ) -> Result<(), AccountingError> {
        let user_account_type = AccountType::ClientSpot;
        let venue_account_type = AccountType::VenueSpot;

        let debit_account = self
            .ledger
            .find_or_create_account(&venue, &debit_asset, None, &user_account_type)?;
        let credit_account = self
            .ledger
            .find_or_create_account(&venue, &credit_asset, None, &user_account_type)?;
        let venue_debit_account =
            self.ledger
                .find_or_create_account(&venue, &debit_asset, None, &venue_account_type)?;
        let venue_credit_account =
            self.ledger
                .find_or_create_account(&venue, &credit_asset, None, &venue_account_type)?;

        self.ledger.exchange(
            &debit_account,
            &credit_account,
            &venue_debit_account,
            &venue_credit_account,
            debit_amount,
            credit_amount,
        )
    }

    pub fn margin_trade(
        &self,
        side: MarketSide,
        strategy: Arc<Strategy>,
        instrument: Arc<Instrument>,
        commission_asset: Option<Arc<Asset>>,
        instrument_amount: Decimal,
        instrument_unit_price: Decimal,
        margin_amount: Decimal,
        commission_amount: Decimal,
    ) -> Result<(), AccountingError> {
        let venue = instrument.venue.clone();
        let instrument_asset = Tradable::Instrument(instrument.clone());
        let margin_asset = Tradable::Asset(instrument.margin_asset.clone());
        let commission_asset = Tradable::Asset(commission_asset.unwrap_or_else(|| instrument.margin_asset.clone()));

        // Get accounts
        let user_margin_account =
            self.ledger
                .find_or_create_account(&venue, &margin_asset, None, &AccountType::ClientMargin)?;
        let venue_margin_account =
            self.ledger
                .find_or_create_account(&venue, &margin_asset, None, &AccountType::VenueMargin)?;

        let user_instrument_account =
            self.ledger
                .find_or_create_account(&venue, &instrument_asset, Some(&strategy), &AccountType::Strategy)?;
        let venue_instrument_account =
            self.ledger
                .find_or_create_account(&venue, &instrument_asset, None, &AccountType::VenueSpot)?;

        // Commission accounts
        let commission_debit_account =
            self.ledger
                .find_or_create_account(&venue, &commission_asset, None, &AccountType::ClientSpot)?;
        let commission_credit_account =
            self.ledger
                .find_or_create_account(&venue, &commission_asset, None, &AccountType::VenueSpot)?;

        // Todo calculate margin amount

        // Not we figure out who to debit and who to credit
        let (margin_debit_account, margin_credit_account) = match side {
            MarketSide::Buy => (user_margin_account, venue_margin_account),
            MarketSide::Sell => (venue_margin_account, user_margin_account),
        };

        let (instrument_debit_account, instrument_credit_account) = match side {
            MarketSide::Buy => (venue_instrument_account, user_instrument_account),
            MarketSide::Sell => (user_instrument_account, venue_instrument_account),
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
            instrument_unit_price,
            commission_amount,
            dec!(0),
        )
    }
}

#[async_trait]
impl Accounting for LedgerAccounting {
    // --- Update Methods ---

    /// Updates the ledger with a balance update from an exchange.
    /// This is a placeholder; a full implementation would reconcile the ledger with the update.
    async fn balance_update(&self, _update: Arc<BalanceUpdate>) -> Result<(), AccountingError> {
        // Placeholder: In a real scenario, update the balance of the corresponding account
        // For example, adjust the VenueAccount balance based on update.amount
        Ok(())
    }

    /// Updates the ledger with a position update from an exchange.
    /// This is a placeholder; a full implementation would reconcile strategy positions.
    async fn position_update(&self, _update: Arc<PositionUpdate>) -> Result<(), AccountingError> {
        // Placeholder: Update the position in the strategy's account for the instrument
        Ok(())
    }

    /// Processes an order fill and updates the ledger accordingly.
    async fn order_update(&self, order: Arc<VenueOrder>) -> Result<(), AccountingError> {
        match order.instrument.instrument_type {
            InstrumentType::Spot => {
                info!("Portfolio processing spot order: {}", order);
                // Placeholder: Implement spot order logic (e.g., transfer assets)
                // For a buy: debit quote asset, credit base asset
                // For a sell: debit base asset, credit quote asset
            }
            InstrumentType::Perpetual => {
                info!("Portfolio processing perpetual order: {}", order);
                let margin_amount = order.last_fill_price * order.last_fill_quantity;
                self.margin_trade(
                    order.side,
                    order.strategy.clone(),
                    order.instrument.clone(),
                    order.commission_asset.clone(),
                    order.last_fill_quantity,
                    order.last_fill_price,
                    margin_amount,
                    order.last_fill_commission,
                )?;
            }
            InstrumentType::Future => {
                info!("Portfolio processing future order: {}", order);
                // Placeholder: Handle future order (similar to perpetual but with expiry)
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

    // --- Balance Queries ---

    /// Returns the total balance of an asset on a specific venue.
    async fn balance(&self, venue: &Arc<Venue>, asset: &Arc<Asset>) -> Decimal {
        let mut balance = Decimal::ZERO;

        if let Some(account) = self
            .ledger
            .find_account(&venue, &asset.clone().into(), None, &AccountType::ClientSpot)
        {
            balance += self.ledger.balance(account.id);
        }

        info!("Balance for {} on {}: {}", asset, venue, balance);
        balance
    }

    async fn margin_balance(&self, venue: &Arc<Venue>, asset: &Arc<Asset>) -> Decimal {
        let mut balance = Decimal::ZERO;

        if let Some(account) = self
            .ledger
            .find_account(&venue, &asset.clone().into(), None, &AccountType::ClientMargin)
        {
            balance += self.ledger.balance(account.id);
        }

        if let Some(account) = self
            .ledger
            .find_account(&venue, &asset.clone().into(), None, &AccountType::VenueMargin)
        {
            balance += self.ledger.balance(account.id);
        }

        info!("Margin Balance for {} on {}: {}", asset, venue, balance);
        balance
    }

    async fn available_margin_balance(&self, venue: &Arc<Venue>, asset: &Arc<Asset>) -> Decimal {
        let mut balance = Decimal::ZERO;

        if let Some(account) = self
            .ledger
            .find_account(&venue, &asset.clone().into(), None, &AccountType::ClientMargin)
        {
            balance += self.ledger.balance(account.id);
        }

        info!("Available Margin Balance for {} on {}: {}", asset, venue, balance);
        balance
    }

    // --- Position Queries (Global) ---

    /// Returns the total position size for an instrument across all strategies.
    async fn position(&self, instrument: &Arc<Instrument>) -> Decimal {
        let accounts = self.ledger.accounts();
        let mut total_position = Decimal::ZERO;
        for account in accounts {
            if AccountType::Strategy == account.account_type
                && account.asset == Tradable::Instrument(instrument.clone())
            {
                total_position += self.ledger.balance(account.id);
            }
        }
        total_position
    }

    /// Returns the total position notional value for an instrument
    async fn position_notional(&self, instrument: &Arc<Instrument>) -> Decimal {
        let accounts = self.ledger.accounts();
        let mut total_position = Decimal::ZERO;
        for account in accounts {
            if AccountType::Strategy == account.account_type
                && account.asset == Tradable::Instrument(instrument.clone())
            {
                total_position += self.ledger.position(account.id);
            }
        }
        total_position
    }

    /// Returns all open positions across all instruments globally.
    async fn positions(&self) -> HashMap<Arc<Instrument>, Decimal> {
        let accounts = self.ledger.accounts();
        let mut positions = HashMap::new();
        for account in accounts {
            if account.account_type == AccountType::Strategy {
                if let Tradable::Instrument(instrument) = &account.asset {
                    let balance = self.ledger.balance(account.id);
                    let entry = positions.entry(instrument.clone()).or_insert(Decimal::ZERO);
                    *entry += balance;
                }
            }
        }
        positions
    }

    async fn positions_notional(&self) -> HashMap<Arc<Instrument>, Decimal> {
        let accounts = self.ledger.accounts();
        let mut positions = HashMap::new();
        for account in accounts {
            if account.account_type == AccountType::Strategy {
                if let Tradable::Instrument(instrument) = &account.asset {
                    let balance = self.ledger.position(account.id);
                    let entry = positions.entry(instrument.clone()).or_insert(Decimal::ZERO);
                    *entry += balance;
                }
            }
        }
        positions
    }

    // --- Strategy-Specific Queries ---

    /// Returns the position size for an instrument under a specific strategy.
    async fn strategy_position(&self, strategy: &Arc<Strategy>, instrument: &Arc<Instrument>) -> Decimal {
        if let Some(account) = self.ledger.find_account(
            &instrument.venue,
            &instrument.clone().into(),
            Some(strategy),
            &AccountType::Strategy,
        ) {
            self.ledger.balance(account.id)
        } else {
            Decimal::ZERO
        }
    }

    async fn strategy_position_notional(&self, strategy: &Arc<Strategy>, instrument: &Arc<Instrument>) -> Decimal {
        if let Some(account) = self.ledger.find_account(
            &instrument.venue,
            &instrument.clone().into(),
            Some(strategy),
            &AccountType::Strategy,
        ) {
            self.ledger.position(account.id)
        } else {
            Decimal::ZERO
        }
    }

    async fn strategy_positions(&self, strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal> {
        let strategy = Some(strategy.clone());
        let accounts = self.ledger.accounts();
        let mut positions = HashMap::new();
        for account in accounts {
            if account.account_type == AccountType::Strategy && account.strategy == strategy {
                if let Tradable::Instrument(instrument) = &account.asset {
                    let balance = self.ledger.balance(account.id);
                    let entry = positions.entry(instrument.clone()).or_insert(Decimal::ZERO);
                    *entry += balance;
                }
            }
        }
        positions
    }

    async fn strategy_positions_notional(&self, strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal> {
        let strategy = Some(strategy.clone());
        let accounts = self.ledger.accounts();
        let mut positions = HashMap::new();
        for account in accounts {
            if account.account_type == AccountType::Strategy && account.strategy == strategy {
                if let Tradable::Instrument(instrument) = &account.asset {
                    let balance = self.ledger.position(account.id);
                    let entry = positions.entry(instrument.clone()).or_insert(Decimal::ZERO);
                    *entry += balance;
                }
            }
        }
        positions
    }

    /// Placeholder: Returns the realized PnL for a strategy and instrument.
    async fn strategy_realized_pnl(&self, _strategy: &Arc<Strategy>, _instrument: &Arc<Instrument>) -> Decimal {
        // Placeholder: Requires tracking entry prices and realized gains/losses
        Decimal::ZERO
    }

    /// Placeholder: Returns the unrealized PnL for a strategy and instrument.
    async fn strategy_unrealized_pnl(&self, _strategy: &Arc<Strategy>, _instrument: &Arc<Instrument>) -> Decimal {
        // Placeholder: Requires current market price and position size
        Decimal::ZERO
    }

    /// Placeholder: Returns the total PnL (realized + unrealized) for a strategy by asset.
    async fn strategy_total_pnl(&self, _strategy: &Arc<Strategy>) -> HashMap<Arc<Asset>, Decimal> {
        // Placeholder: Sum realized and unrealized PnL per asset
        HashMap::new()
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
                          self.order_update(order).await.unwrap_or_else(|e| {
                            error!("Failed to process order update: {}", e);
                          });
                        }
                        _ => {}
                    }
                }
                _ = shutdown.cancelled() => {
                    info!("Accounting shutting down...");
                    let transfers = self.ledger.get_transfers();
                    for t in transfers {
                        info!(" - {}", t);
                    }

                    let accounts = self.ledger.accounts();
                    for account in accounts {
                        info!("BALANCE {}: {}", account, self.ledger.balance(account.id));
                        info!("POSITION {}: {}", account, self.ledger.position(account.id));
                    }

                    break;
                }
            }
        }
        Ok(())
    }
}
