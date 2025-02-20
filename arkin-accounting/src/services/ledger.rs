use async_trait::async_trait;
use rust_decimal::Decimal;
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
        instrument_unit_price: Decimal,
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
        // let user_instrument_account_balance = self.ledger.get_balance(user_instrument_account.id);
        // let position_side = match user_instrument_account_balance {
        //     balance if balance.is_zero() => None,
        //     balance if balance > Decimal::ZERO => Some(PositionSide::Long),
        //     balance if balance < Decimal::ZERO => Some(PositionSide::Short),
        //     balance => return Err(AccountingError::InvalidBalance(balance)),
        // };

        // Now here comes the tricky part. If our order fills more then the current position we have two transfers
        // The first is we sell back to the ex

        // // Figure out if we need to post margin or we get margin back
        // let margin_amount = match (position_side, side) {
        //     (None, MarketSide::Buy) => margin_amount,
        //     (None, MarketSide::Sell) => margin_amount,
        //     (Some(PositionSide::Long), MarketSide::Buy) => margin_amount,
        //     // If we sell more then
        //     (Some(PositionSide::Long), MarketSide::Sell) => {

        //     },
        //     (Some(PositionSide::Short), MarketSide::Buy) => margin_amount,
        //     (Some(PositionSide::Short), MarketSide::Sell) => -margin_amount,
        // };

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

        // Check how much we have on our account
        let account_type = AccountType::VenueAccount(venue.clone());
        if let Some(account) = self.ledger.find_account(&account_type, &asset.clone().into()) {
            balance += self.ledger.get_balance(account.id);
        }

        // Check how much we have locked in positions
        let accounts = self.ledger.get_accounts();
        for account in accounts {
            // Check if the account is a strategy account
            if let AccountType::Strategy(_) = account.account_type {
                // Check if the asset is the same as the one we are looking for
                if let Tradable::Instrument(instrument) = &account.asset {
                    // Check if the instrument is on the same venue
                    if instrument.venue == *venue {
                        // Check if the asset is the same as the one we are looking for
                        if instrument.quote_asset == *asset {
                            // Add the balance to the total balance
                            balance += self.ledger.get_position(account.id);
                        }
                    }
                }
            }
        }
        info!("Balance for {} on {}: {}", asset, venue, balance);
        balance
    }

    /// Returns the available balance (free to use) of an asset on a specific venue.
    /// Currently assumes no locked funds; extend this for margin/orders if needed.
    async fn available_balance(&self, venue: &Arc<Venue>, asset: &Arc<Asset>) -> Decimal {
        let account_type = AccountType::VenueAccount(venue.clone());
        if let Some(account) = self.ledger.find_account(&account_type, &asset.clone().into()) {
            self.ledger.get_balance(account.id)
        } else {
            Decimal::ZERO
        }
    }

    // --- Position Queries (Global) ---

    /// Returns the total position size for an instrument across all strategies.
    async fn get_position(&self, instrument: &Arc<Instrument>) -> Decimal {
        let accounts = self.ledger.get_accounts();
        let mut total_position = Decimal::ZERO;
        for account in accounts {
            if let AccountType::Strategy(_) = account.account_type {
                if account.asset == Tradable::Instrument(instrument.clone()) {
                    total_position += self.ledger.get_balance(account.id);
                }
            }
        }
        total_position
    }

    /// Returns all open positions across all instruments globally.
    async fn get_positions(&self) -> HashMap<Arc<Instrument>, Decimal> {
        let accounts = self.ledger.get_accounts();
        let mut positions = HashMap::new();
        for account in accounts {
            if let AccountType::Strategy(_) = account.account_type {
                if let Tradable::Instrument(instrument) = &account.asset {
                    let balance = self.ledger.get_balance(account.id);
                    let entry = positions.entry(instrument.clone()).or_insert(Decimal::ZERO);
                    *entry += balance;
                }
            }
        }
        positions
    }

    // --- Strategy-Specific Queries ---

    /// Returns the position size for an instrument under a specific strategy.
    async fn get_strategy_position(&self, strategy: &Arc<Strategy>, instrument: &Arc<Instrument>) -> Decimal {
        let account_type = AccountType::Strategy(strategy.clone());
        if let Some(account) = self
            .ledger
            .find_account(&account_type, &Tradable::Instrument(instrument.clone()))
        {
            self.ledger.get_balance(account.id)
        } else {
            Decimal::ZERO
        }
    }

    /// Returns all open positions for a specific strategy.
    async fn get_strategy_positions(&self, strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal> {
        let accounts = self.ledger.get_accounts();
        let mut positions = HashMap::new();
        for account in accounts {
            if account.account_type == AccountType::Strategy(strategy.clone()) {
                if let Tradable::Instrument(instrument) = &account.asset {
                    let balance = self.ledger.get_balance(account.id);
                    positions.insert(instrument.clone(), balance);
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

    // --- Capital and Buying Power ---

    /// Returns the total capital (net worth) across all assets.
    async fn total_capital(&self) -> HashMap<Arc<Asset>, Decimal> {
        let accounts = self.ledger.get_accounts();
        let mut capital = HashMap::new();
        for account in accounts {
            if let AccountType::VenueAccount(_) = account.account_type {
                if let Tradable::Asset(asset) = &account.asset {
                    let balance = self.ledger.get_balance(account.id);
                    let entry = capital.entry(asset.clone()).or_insert(Decimal::ZERO);
                    *entry += balance;
                }
            }
        }
        capital
    }

    /// Returns the buying power for an asset across all venues.
    /// Placeholder: Assumes buying power equals available balance; extend for leverage/margin.
    async fn buying_power(&self, asset: &Arc<Asset>) -> Decimal {
        let accounts = self.ledger.get_accounts();
        let mut total = Decimal::ZERO;
        for account in accounts {
            if let AccountType::VenueAccount(_) = account.account_type {
                if account.asset == Tradable::Asset(asset.clone()) {
                    total += self.ledger.get_balance(account.id);
                }
            }
        }
        total
    }

    /// Placeholder: Returns the buying power for an asset under a specific strategy.
    async fn strategy_buying_power(&self, _strategy: &Arc<Strategy>, _asset: &Arc<Asset>) -> Decimal {
        // Placeholder: Define based on margin model (e.g., available margin)
        Decimal::ZERO
    }

    // --- PnL Queries (Global) ---

    /// Placeholder: Returns the realized PnL for an instrument globally.
    async fn realized_pnl(&self, _instrument: &Arc<Instrument>) -> Decimal {
        // Placeholder: Requires tracking realized gains/losses
        Decimal::ZERO
    }

    /// Placeholder: Returns the unrealized PnL for an instrument globally.
    async fn unrealized_pnl(&self, _instrument: &Arc<Instrument>) -> Decimal {
        // Placeholder: Requires current market price
        Decimal::ZERO
    }

    /// Placeholder: Returns the total PnL (realized + unrealized) globally by asset.
    async fn total_pnl(&self) -> HashMap<Arc<Asset>, Decimal> {
        // Placeholder: Sum realized and unrealized PnL
        HashMap::new()
    }

    // --- Commission Queries ---

    /// Returns the total commission paid for a specific instrument.
    async fn commission(&self, instrument: &Arc<Instrument>) -> Decimal {
        let transfers = self.ledger.get_transfers();
        let mut total_commission = Decimal::ZERO;
        for t in transfers.iter() {
            if t.transfer_type == TransferType::Commission {
                if let AccountType::Strategy(_) = t.debit_account.account_type {
                    if t.debit_account.asset == Tradable::Instrument(instrument.clone()) {
                        total_commission += t.amount;
                    }
                }
            }
        }
        total_commission
    }

    /// Returns the total commission paid across all instruments, grouped by asset.
    async fn total_commission(&self) -> HashMap<Arc<Asset>, Decimal> {
        let transfers = self.ledger.get_transfers();
        let mut commissions = HashMap::new();
        for t in transfers.iter() {
            if t.transfer_type == TransferType::Commission {
                if let Tradable::Asset(asset) = &t.credit_account.asset {
                    let entry = commissions.entry(asset.clone()).or_insert(Decimal::ZERO);
                    *entry += t.amount;
                }
            }
        }
        commissions
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

                    let accounts = self.ledger.get_accounts();
                    for account in accounts {
                        info!("BALANCE {}: {}", account, self.ledger.get_balance(account.id));
                        info!("POSITION {}: {}", account, self.ledger.get_position(account.id));
                    }

                    info!("Final Balance: {}", self.get_position(&test_inst_binance_btc_usdt_perp()).await);


                    break;
                }
            }
        }
        Ok(())
    }
}
