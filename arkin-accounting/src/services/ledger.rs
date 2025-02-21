use async_trait::async_trait;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::{collections::HashMap, sync::Arc};
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{ledger::Ledger, Accounting, AccountingError, AccountingService};

#[derive(Debug, TypedBuilder)]
pub struct LedgerAccounting {
    pubsub: Arc<PubSub>,
    #[builder(default = Ledger::builder().build())]
    ledger: Ledger,
}

impl LedgerAccounting {
    pub fn deposit(
        &self,
        debit_venue: &Arc<Venue>,
        credit_venue: &Arc<Venue>,
        asset: &Tradable,
        amount: Decimal,
        account_type: &AccountType,
    ) -> Result<(), AccountingError> {
        let debit_account = self.ledger.find_or_create(debit_venue, &asset, &AccountType::VenueSpot);
        let credit_account = self.ledger.find_or_create(credit_venue, &asset, account_type);

        self.ledger.transfer(&debit_account, &credit_account, amount)
    }

    pub fn withdraw(
        &self,
        debit_venue: &Arc<Venue>,
        credit_venue: &Arc<Venue>,
        asset: &Tradable,
        amount: Decimal,
        account_type: &AccountType,
    ) -> Result<(), AccountingError> {
        let debit_account = self.ledger.find_or_create(debit_venue, asset, account_type);
        let credit_account = self.ledger.find_or_create(credit_venue, asset, &AccountType::VenueSpot);

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
        let transfer_group_id = Uuid::new_v4();

        let debit_account = self.ledger.find_or_create(&venue, &debit_asset, &AccountType::ClientSpot);
        let venue_credit_account = self.ledger.find_or_create(&venue, &debit_asset, &AccountType::VenueSpot);

        let t1 = Transfer::builder()
            .transfer_group_id(transfer_group_id)
            .strategy(None)
            .debit_account(debit_account)
            .credit_account(venue_credit_account)
            .amount(debit_amount)
            .unit_price(Decimal::ONE)
            .transfer_type(TransferType::Exchange)
            .build()
            .into();

        let venue_debit_account = self.ledger.find_or_create(&venue, &credit_asset, &AccountType::VenueSpot);
        let credit_account = self.ledger.find_or_create(&venue, &credit_asset, &AccountType::ClientSpot);

        let t2 = Transfer::builder()
            .transfer_group_id(transfer_group_id)
            .strategy(None)
            .debit_account(venue_debit_account)
            .credit_account(credit_account)
            .amount(credit_amount)
            .unit_price(Decimal::ONE)
            .transfer_type(TransferType::Exchange)
            .build()
            .into();

        self.ledger.apply_transfers(&[t1, t2])
    }

    pub fn margin_trade(
        &self,
        side: MarketSide,
        strategy: Arc<Strategy>,
        instrument: Arc<Instrument>,
        commission_asset: Option<Arc<Asset>>,
        amount: Decimal,
        price: Decimal,
        margin_rate: Decimal,
        commission_rate: Decimal,
    ) -> Result<(), AccountingError> {
        info!("Starting Margin Trade...");
        info!("Side: {}", side);
        info!("Price: {}", price);
        info!("Amount: {}", amount);
        info!("Margin Rate: {}", margin_rate);
        info!("Commission Rate: {}", commission_rate);
        let venue = instrument.venue.clone();
        let inst_asset = Tradable::Instrument(instrument.clone());
        let margin_asset = Tradable::Asset(instrument.margin_asset.clone());
        let commission_asset = Tradable::Asset(commission_asset.unwrap_or_else(|| instrument.margin_asset.clone()));

        //  Find or create necessary accounts
        let user_margin = self.ledger.find_or_create(&venue, &margin_asset, &AccountType::ClientMargin);
        let venue_margin = self.ledger.find_or_create(&venue, &margin_asset, &AccountType::VenueMargin);
        let user_inst = self.ledger.find_or_create(&venue, &inst_asset, &AccountType::ClientInstrument);
        let venue_inst = self.ledger.find_or_create(&venue, &inst_asset, &AccountType::VenueInstrument);
        let venue_spot = self.ledger.find_or_create(&venue, &commission_asset, &AccountType::VenueSpot);

        let current_position = self.ledger.position_amount(user_inst.id, &strategy);
        info!("Current position from ledger: {}", current_position);
        let new_position = match side {
            MarketSide::Buy => current_position + amount,
            MarketSide::Sell => current_position - amount,
        };
        info!("New Position after {} will be: {}", side, new_position);

        // Calculate amount closed and PnL
        let amount_closed = if (current_position > Decimal::ZERO && new_position <= Decimal::ZERO)
            || (current_position < Decimal::ZERO && new_position >= Decimal::ZERO)
        {
            info!("Position will fully close: {} -> {}", current_position, new_position);
            current_position.abs() // Full close before flip
        } else {
            info!("Position will not close fully: {} -> {}", current_position, new_position);
            amount.min(current_position.abs()) // Partial close
        };
        info!("Amount closed: {}", amount_closed);

        let entry_price = self.ledger.cost_basis(user_inst.id, &strategy); // Assume stored
        info!("Entry price from ledger: {}", entry_price);
        let pnl = if current_position > Decimal::ZERO {
            info!("Calculating PnL for long position");
            (price - entry_price) * amount_closed
        } else if current_position < Decimal::ZERO {
            info!("Calculating PnL for short position");
            (entry_price - price) * amount_closed
        } else {
            info!("No PnL for flat position");
            dec!(0)
        };

        // Margin adjustments
        let new_margin = new_position.abs() * price * margin_rate;
        info!("New margin required: {}", new_margin);
        let current_margin = self.ledger.margin_posted(venue_margin.id, &strategy);
        info!("Current margin posted: {}", current_margin);
        let margin_delta = new_margin - current_margin;
        info!("Margin delta: {}", margin_delta);

        //  Calculate commission
        let commission = amount * price * commission_rate;
        info!("Commission: {}", commission);

        // Step 7: Create transfers
        let transfer_group_id = Uuid::new_v4();
        let mut transfers = Vec::new();

        // Margin adjustment
        if margin_delta > dec!(0) {
            // Post additional margin
            transfers.push(Arc::new(
                Transfer::builder()
                    .transfer_group_id(transfer_group_id)
                    .strategy(Some(strategy.clone()))
                    .debit_account(user_margin.clone())
                    .credit_account(venue_margin.clone())
                    .amount(margin_delta)
                    .unit_price(Decimal::ONE)
                    .transfer_type(TransferType::Margin)
                    .build(),
            ));
        } else if margin_delta < dec!(0) {
            // Free margin
            transfers.push(Arc::new(
                Transfer::builder()
                    .transfer_group_id(transfer_group_id)
                    .strategy(Some(strategy.clone()))
                    .debit_account(venue_margin.clone())
                    .credit_account(user_margin.clone())
                    .amount(margin_delta.abs())
                    .unit_price(Decimal::ONE)
                    .transfer_type(TransferType::Margin)
                    .build(),
            ));
        }

        // Commission payment
        transfers.push(Arc::new(
            Transfer::builder()
                .transfer_group_id(transfer_group_id)
                .strategy(Some(strategy.clone()))
                .debit_account(user_margin.clone())
                .credit_account(venue_spot.clone())
                .amount(commission)
                .unit_price(Decimal::ONE)
                .transfer_type(TransferType::Commission)
                .build(),
        ));

        // Instrument trade
        let (debit_inst, credit_inst) = if side == MarketSide::Buy {
            (venue_inst.clone(), user_inst.clone())
        } else {
            (user_inst.clone(), venue_inst.clone())
        };
        transfers.push(Arc::new(
            Transfer::builder()
                .transfer_group_id(transfer_group_id)
                .strategy(Some(strategy.clone()))
                .debit_account(debit_inst)
                .credit_account(credit_inst)
                .amount(amount)
                .unit_price(price)
                .transfer_type(TransferType::Trade)
                .build(),
        ));

        // PnL transfer
        if amount_closed > dec!(0) {
            if pnl > Decimal::ZERO {
                // Profit: venue_spot -> user_margin
                transfers.push(Arc::new(
                    Transfer::builder()
                        .transfer_group_id(transfer_group_id)
                        .strategy(Some(strategy.clone()))
                        .debit_account(venue_spot.clone())
                        .credit_account(user_margin.clone())
                        .amount(pnl)
                        .unit_price(Decimal::ONE)
                        .transfer_type(TransferType::PnL)
                        .build(),
                ));
            } else if pnl < dec!(0) {
                // Loss: user_margin -> venue_spot
                transfers.push(Arc::new(
                    Transfer::builder()
                        .transfer_group_id(transfer_group_id)
                        .strategy(Some(strategy.clone()))
                        .debit_account(user_margin.clone())
                        .credit_account(venue_spot.clone())
                        .amount(pnl.abs())
                        .unit_price(Decimal::ONE)
                        .transfer_type(TransferType::PnL)
                        .build(),
                ));
            }
        }

        for t in &transfers {
            info!("Transfers:");
            info!(" - {}", t);
        }

        // Apply transfers atomically
        self.ledger.apply_transfers(&transfers)
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
        let tradable = Tradable::Asset(asset.clone());
        let account = self.ledger.find_account(&venue, &tradable, &AccountType::ClientSpot);

        if let Some(account) = account {
            self.ledger.balance(account.id)
        } else {
            Decimal::ZERO
        }
    }

    async fn margin_balance(&self, venue: &Arc<Venue>, asset: &Arc<Asset>) -> Decimal {
        let tradable = Tradable::Asset(asset.clone());
        let client_margin = self.ledger.find_account(&venue, &tradable, &AccountType::ClientMargin);
        let venue_margin = self.ledger.find_account(&venue, &tradable, &AccountType::VenueMargin);

        let mut balance = Decimal::ZERO;
        if let Some(account) = client_margin {
            balance += self.ledger.balance(account.id);
        }

        if let Some(account) = venue_margin {
            balance += self.ledger.balance(account.id);
        }
        balance
    }

    async fn available_margin_balance(&self, venue: &Arc<Venue>, asset: &Arc<Asset>) -> Decimal {
        let tradable = Tradable::Asset(asset.clone());
        let account = self.ledger.find_account(&venue, &tradable, &AccountType::ClientMargin);

        if let Some(account) = account {
            self.ledger.balance(account.id)
        } else {
            Decimal::zero()
        }
    }

    // --- Position Queries (Global) ---

    /// Returns the total position size for an instrument across all strategies.
    async fn position(&self, _instrument: &Arc<Instrument>) -> Decimal {
        todo!()
    }

    /// Returns the total position notional value for an instrument
    async fn position_notional(&self, instrument: &Arc<Instrument>) -> Decimal {
        let accounts = self.ledger.accounts();
        let total_position = Decimal::ZERO;
        for account in accounts {
            if AccountType::ClientInstrument == account.account_type
                && account.asset == Tradable::Instrument(instrument.clone())
            {
                // total_position += self.ledger.position(account.id);
            }
        }
        total_position
    }

    /// Returns all open positions across all instruments globally.
    async fn positions(&self) -> HashMap<Arc<Instrument>, Decimal> {
        let accounts = self.ledger.accounts();
        let mut positions = HashMap::new();
        for account in accounts {
            if account.account_type == AccountType::ClientInstrument {
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
        let positions = HashMap::new();
        for account in accounts {
            if account.account_type == AccountType::ClientInstrument {
                if let Tradable::Instrument(_instrument) = &account.asset {
                    // let balance = self.ledger.position(account.id);
                    // let entry = positions.entry(instrument.clone()).or_insert(Decimal::ZERO);
                    // *entry += balance;
                }
            }
        }
        positions
    }

    // --- Strategy-Specific Queries ---

    /// Returns the position size for an instrument under a specific strategy.
    async fn strategy_position(&self, _strategy: &Arc<Strategy>, instrument: &Arc<Instrument>) -> Decimal {
        if let Some(account) =
            self.ledger
                .find_account(&instrument.venue, &instrument.clone().into(), &AccountType::ClientInstrument)
        {
            self.ledger.balance(account.id)
        } else {
            Decimal::ZERO
        }
    }

    async fn strategy_position_notional(&self, strategy: &Arc<Strategy>, instrument: &Arc<Instrument>) -> Decimal {
        if let Some(account) =
            self.ledger
                .find_account(&instrument.venue, &instrument.clone().into(), &AccountType::ClientInstrument)
        {
            self.ledger.cost_basis(account.id, strategy)
        } else {
            Decimal::ZERO
        }
    }

    async fn strategy_positions(&self, _strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal> {
        todo!()
    }

    async fn strategy_positions_notional(&self, _strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal> {
        todo!()
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
                    }

                    break;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_go_long_and_close() {
        let pubsub = PubSub::new(1024);
        let accounting = LedgerAccounting::builder().pubsub(pubsub).build();
        let strategy = test_strategy();
        let personal = test_personal_venue();
        let venue = test_binance_venue();
        let instrument = test_inst_binance_btc_usdt_perp();
        let usdt = Tradable::Asset(instrument.margin_asset.clone());

        // Initial deposit
        accounting
            .deposit(&personal, &venue, &usdt, dec!(10000), &AccountType::ClientMargin)
            .unwrap();

        // Go long: Buy 1 BTC at 1000 USDT
        accounting
            .margin_trade(
                MarketSide::Buy,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(1),
                dec!(1000),
                dec!(0.05),   // 5% margin rate
                dec!(0.0002), // 0.02% commission rate
            )
            .unwrap();

        let user_margin = accounting.ledger.find_or_create(&venue, &usdt, &AccountType::ClientMargin);
        let venue_margin = accounting.ledger.find_or_create(&venue, &usdt, &AccountType::VenueMargin);
        let user_inst = accounting.ledger.find_or_create(
            &venue,
            &Tradable::Instrument(instrument.clone()),
            &AccountType::ClientInstrument,
        );
        let venue_spot = accounting.ledger.find_or_create(&venue, &usdt, &AccountType::VenueSpot);

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(1));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(50)); // 1 * 1000 * 0.05
        assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9949.8)); // 10000 - 50 - 0.2

        // Close: Sell 1 BTC at 1200 USDT
        accounting
            .margin_trade(
                MarketSide::Sell,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(1),
                dec!(1200),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(0));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(0));
        assert_eq!(accounting.ledger.pnl(user_margin.id, &strategy), dec!(200));
        assert_eq!(accounting.ledger.balance(user_margin.id), dec!(10199.5600));
        assert_eq!(accounting.ledger.balance(venue_spot.id), dec!(-199.5600));
    }

    #[test]
    fn test_go_long_reduce_then_close() {
        let pubsub = PubSub::new(1024);
        let accounting = LedgerAccounting::builder().pubsub(pubsub).build();
        let strategy = test_strategy();
        let personal = test_personal_venue();
        let venue = test_binance_venue();
        let instrument = test_inst_binance_btc_usdt_perp();
        let usdt = Tradable::Asset(instrument.margin_asset.clone());

        // Initial deposit
        accounting
            .deposit(&personal, &venue, &usdt, dec!(10000), &AccountType::ClientMargin)
            .unwrap();

        let user_margin = accounting.ledger.find_or_create(&venue, &usdt, &AccountType::ClientMargin);
        let venue_margin = accounting.ledger.find_or_create(&venue, &usdt, &AccountType::VenueMargin);
        let user_inst = accounting.ledger.find_or_create(
            &venue,
            &Tradable::Instrument(instrument.clone()),
            &AccountType::ClientInstrument,
        );

        // Go long: Buy 1 BTC at 1000 USDT
        accounting
            .margin_trade(
                MarketSide::Buy,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(1),
                dec!(1000),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(1));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(50));
        assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9949.8));

        // Reduce: Sell 0.5 BTC at 1200 USDT
        accounting
            .margin_trade(
                MarketSide::Sell,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(0.5),
                dec!(1200),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(0.5));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(30)); // 0.5 * 1200 * 0.05
        assert_eq!(accounting.ledger.pnl(user_margin.id, &strategy), dec!(100)); // (1200 - 1000) * 0.5
        assert_eq!(accounting.ledger.balance(user_margin.id), dec!(10069.68)); // 9949.8 + 20 (freed) - 0.24 (comm) + 100 (PnL)

        // Close: Sell 0.5 BTC at 800 USDT
        accounting
            .margin_trade(
                MarketSide::Sell,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(0.5),
                dec!(800),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(0));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(0));
        assert_eq!(accounting.ledger.pnl(user_margin.id, &strategy), dec!(0)); // 100 - (1000 - 800) * 0.5
        assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9999.6)); // 10069.56 + 30 (freed) - 0.16 (comm) - 100 (PnL)
    }

    #[test]
    fn test_go_short_and_close() {
        let pubsub = PubSub::new(1024);
        let accounting = LedgerAccounting::builder().pubsub(pubsub).build();
        let strategy = test_strategy();
        let personal = test_personal_venue();
        let venue = test_binance_venue();
        let instrument = test_inst_binance_btc_usdt_perp();
        let usdt = Tradable::Asset(instrument.margin_asset.clone());

        // Initial deposit
        accounting
            .deposit(&personal, &venue, &usdt, dec!(10000), &AccountType::ClientMargin)
            .unwrap();

        let user_margin = accounting.ledger.find_or_create(&venue, &usdt, &AccountType::ClientMargin);
        let venue_margin = accounting.ledger.find_or_create(&venue, &usdt, &AccountType::VenueMargin);
        let user_inst = accounting.ledger.find_or_create(
            &venue,
            &Tradable::Instrument(instrument.clone()),
            &AccountType::ClientInstrument,
        );

        // Go short: Sell 1 BTC at 1000 USDT
        accounting
            .margin_trade(
                MarketSide::Sell,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(1),
                dec!(1000),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(-1));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(50));
        assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9949.8));

        // Close: Buy 1 BTC at 800 USDT
        accounting
            .margin_trade(
                MarketSide::Buy,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(1),
                dec!(800),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(0));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(0));
        assert_eq!(accounting.ledger.pnl(user_margin.id, &strategy), dec!(200)); // (1000 - 800) * 1
        assert_eq!(accounting.ledger.balance(user_margin.id), dec!(10199.64)); // 9949.8 + 50 (freed) - 0.16 (comm) + 200 (PnL)
    }

    #[test]
    fn test_go_short_reduce_then_close() {
        let pubsub = PubSub::new(1024);
        let accounting = LedgerAccounting::builder().pubsub(pubsub).build();
        let strategy = test_strategy();
        let personal = test_personal_venue();
        let venue = test_binance_venue();
        let instrument = test_inst_binance_btc_usdt_perp();
        let usdt = Tradable::Asset(instrument.margin_asset.clone());

        // Initial deposit
        accounting
            .deposit(&personal, &venue, &usdt, dec!(10000), &AccountType::ClientMargin)
            .unwrap();

        let user_margin = accounting.ledger.find_or_create(&venue, &usdt, &AccountType::ClientMargin);
        let venue_margin = accounting.ledger.find_or_create(&venue, &usdt, &AccountType::VenueMargin);
        let user_inst = accounting.ledger.find_or_create(
            &venue,
            &Tradable::Instrument(instrument.clone()),
            &AccountType::ClientInstrument,
        );

        // Go short: Sell 1 BTC at 1000 USDT
        accounting
            .margin_trade(
                MarketSide::Sell,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(1),
                dec!(1000),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(-1));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(50));
        assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9949.8));

        // Reduce: Buy 0.5 BTC at 800 USDT
        accounting
            .margin_trade(
                MarketSide::Buy,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(0.5),
                dec!(800),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(-0.5));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(20)); // 0.5 * 800 * 0.05
        assert_eq!(accounting.ledger.pnl(user_margin.id, &strategy), dec!(100)); // (1000 - 800) * 0.5
        assert_eq!(accounting.ledger.balance(user_margin.id), dec!(10079.72)); // 9949.8 + 30 (freed) - 0.16 (comm) + 100 (PnL)

        // Close: Buy 0.5 BTC at 1200 USDT
        accounting
            .margin_trade(
                MarketSide::Buy,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(0.5),
                dec!(1200),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(0));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(0));
        assert_eq!(accounting.ledger.pnl(user_margin.id, &strategy), dec!(0)); // 100 - (1000 - 1200) * 0.5
        assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9999.60)); // 10079.64 + 20 (freed) - 0.24 (comm) - 100 (PnL)
    }

    #[test]
    fn test_go_long_flip_short_flip_long_close() {
        let pubsub = PubSub::new(1024);
        let accounting = LedgerAccounting::builder().pubsub(pubsub).build();
        let strategy = test_strategy();
        let personal = test_personal_venue();
        let venue = test_binance_venue();
        let instrument = test_inst_binance_btc_usdt_perp();
        let usdt = Tradable::Asset(instrument.margin_asset.clone());

        // Initial deposit
        accounting
            .deposit(&personal, &venue, &usdt, dec!(10000), &AccountType::ClientMargin)
            .unwrap();

        let user_margin = accounting.ledger.find_or_create(&venue, &usdt, &AccountType::ClientMargin);
        let venue_margin = accounting.ledger.find_or_create(&venue, &usdt, &AccountType::VenueMargin);
        let user_inst = accounting.ledger.find_or_create(
            &venue,
            &Tradable::Instrument(instrument.clone()),
            &AccountType::ClientInstrument,
        );

        // Go long: Buy 1 BTC at 1000 USDT
        accounting
            .margin_trade(
                MarketSide::Buy,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(1),
                dec!(1000),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(1));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(50));

        // Flip to short: Sell 2 BTC at 1200 USDT
        accounting
            .margin_trade(
                MarketSide::Sell,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(2),
                dec!(1200),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(-1));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(60)); // 1 * 1200 * 0.05
        assert_eq!(accounting.ledger.pnl(user_margin.id, &strategy), dec!(200)); // (1200 - 1000) * 1

        // Flip to long: Buy 2 BTC at 800 USDT
        accounting
            .margin_trade(
                MarketSide::Buy,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(2),
                dec!(800),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(1));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(40)); // 1 * 800 * 0.05
        assert_eq!(accounting.ledger.pnl(user_margin.id, &strategy), dec!(600)); // 200 - (1200 - 800) * 1

        // Close: Sell 1 BTC at 900 USDT
        accounting
            .margin_trade(
                MarketSide::Sell,
                strategy.clone(),
                instrument.clone(),
                None,
                dec!(1),
                dec!(900),
                dec!(0.05),
                dec!(0.0002),
            )
            .unwrap();

        assert_eq!(accounting.ledger.position_amount(user_inst.id, &strategy), dec!(0));
        assert_eq!(accounting.ledger.margin_posted(venue_margin.id, &strategy), dec!(0));
        assert_eq!(accounting.ledger.pnl(user_margin.id, &strategy), dec!(700));
    }
}

// #[test]
//     fn test_successful_margin_trade() {
//         let ledger = Ledger::builder().build();
//         let strategy = test_strategy();
//         let personal_venue = test_personal_venue();
//         let binance_venue = test_binance_venue();
//         let usdt: Tradable = test_usdt_asset().into();
//         let btc_usdt: Tradable = test_inst_binance_btc_usdt_perp().into();

//         let personal = ledger.add_account(personal_venue.clone(), usdt.clone(), AccountType::VenueSpot);

//         let user_margin = ledger.add_account(binance_venue.clone(), usdt.clone(), AccountType::ClientMargin);
//         let user_inst = ledger.add_account(binance_venue.clone(), btc_usdt.clone(), AccountType::ClientInstrument);

//         let venue_spot = ledger.add_account(binance_venue.clone(), usdt.clone(), AccountType::VenueSpot);
//         let venue_margin = ledger.add_account(binance_venue.clone(), usdt.clone(), AccountType::VenueMargin);
//         let venue_inst = ledger.add_account(binance_venue.clone(), btc_usdt, AccountType::VenueInstrument);

//         let leverage = dec!(1);
//         let margin_rate = dec!(0.05);
//         let commission_rate = dec!(0.0002);
//         let starting_amount = dec!(10000);

//         // We post some balance in our margin account
//         ledger.transfer(&personal, &user_margin, starting_amount).unwrap();

//         // First transaction we buy 1 BTC-USDT contract at $5000 with 1x leverage
//         let instrument_amount = dec!(1);
//         let instrument_unit_price = dec!(1000);
//         let position_amount = instrument_amount * instrument_unit_price * leverage;
//         let margin_amount = position_amount * margin_rate;
//         let commission_amount = position_amount * commission_rate;

//         ledger
//             .margin_trade(
//                 strategy.clone(),
//                 user_margin.clone(),
//                 venue_margin.clone(),
//                 user_inst.clone(),
//                 venue_inst.clone(),
//                 user_margin.clone(),
//                 venue_spot.clone(),
//                 venue_spot.clone(),
//                 user_margin.clone(),
//                 margin_amount,
//                 instrument_amount,
//                 instrument_unit_price,
//                 commission_amount,
//                 dec!(0),
//             )
//             .unwrap();

//         assert_eq!(
//             ledger.balance(user_margin.id),
//             starting_amount - margin_amount - commission_amount
//         );

//         info!("Strategy Margin Posted: {}", ledger.margin_posted(venue_margin.id, &strategy));
//         info!("Strategy Balance: {}", ledger.balance(user_inst.id));
//         info!("Strategy Position Amount: {}", ledger.position_amount(user_inst.id, &strategy));
//         info!("Strategy Cost Basis: {}", ledger.cost_basis(user_inst.id, &strategy));
//         info!("Strategy PnL: {}", ledger.pnl(user_margin.id, &strategy));

//         // Second transaction (buy 0.5 BTC at 2000 USDT, reduce short)
//         let instrument_amount = dec!(0.5);
//         let instrument_unit_price = dec!(2000);
//         let current_position = ledger.position_amount(user_inst.id, &strategy);
//         let current_entry_price = if !current_position.is_zero() {
//             ledger.cost_basis(user_inst.id, &strategy) / current_position
//         } else {
//             dec!(0)
//         };
//         let new_position = current_position + instrument_amount;

//         // Calculate PnL
//         let pnl = calculate_pnl(current_position, current_entry_price, instrument_unit_price, instrument_amount);

//         // Calculate margin action
//         let (margin_amount, is_posting) =
//             calculate_margin_action(current_position, new_position, margin_rate, instrument_unit_price);
//         info!("Margin Action: margin_amount={}, is_posting={}", margin_amount, is_posting);
//         let current_posted_margin = ledger.margin_posted(venue_margin.id, &strategy);
//         info!("Current Posted Margin: {}", current_posted_margin);
//         // Calculate margin amount
//         let margin = if is_posting {
//             margin_amount - current_posted_margin
//         } else {
//             current_posted_margin - margin_amount
//         };

//         let commission_amount = instrument_amount * instrument_unit_price * commission_rate;

//         ledger
//             .margin_trade(
//                 strategy.clone(),
//                 venue_margin.clone(),
//                 user_margin.clone(),
//                 venue_inst.clone(),
//                 user_inst.clone(),
//                 user_margin.clone(),
//                 venue_spot.clone(),
//                 venue_spot.clone(),
//                 user_margin.clone(),
//                 margin,
//                 instrument_amount,
//                 instrument_unit_price,
//                 commission_amount,
//                 pnl.abs(),
//             )
//             .unwrap();

//         info!("Strategy Margin Posted: {}", ledger.margin_posted(venue_margin.id, &strategy));
//         info!("Strategy Balance: {}", ledger.balance(user_inst.id));
//         info!("Strategy Position Amount: {}", ledger.position_amount(user_inst.id, &strategy));
//         info!("Strategy Cost Basis: {}", ledger.cost_basis(user_inst.id, &strategy));
//         info!("Strategy PnL: {}", ledger.pnl(user_margin.id, &strategy));

//         // Now we sell half our position at a loss
//         let instrument_amount = dec!(0.5);
//         let instrument_unit_price = dec!(1000);
//         let position_amount = instrument_amount * instrument_unit_price * leverage;
//         let commission_amount = position_amount * commission_rate;

//         // Get current balance & cost
//         let current_amount = ledger.position_amount(user_inst.id, &strategy);
//         info!("Instrument Balance: {}", current_amount);
//         let current_cost = ledger.cost_basis(user_inst.id, &strategy);
//         info!("Current Position: {}", current_cost);

//         // Calculate margin
//         let posted_margin = ledger.margin_posted(venue_margin.id, &strategy);
//         let f = instrument_amount / current_amount;
//         let freed_margin_amount = (posted_margin * f).abs();
//         info!("Freed Margin: {}", freed_margin_amount);

//         // Calculate PnL
//         let avg_price = if current_amount > Decimal::ZERO {
//             current_cost / current_amount
//         } else {
//             Decimal::ZERO
//         };
//         let diff_price = instrument_unit_price - avg_price;
//         let pnl = diff_price * instrument_amount;
//         info!("PNL: {}", pnl);

//         ledger
//             .margin_trade(
//                 strategy.clone(),
//                 venue_margin.clone(),
//                 user_margin.clone(),
//                 venue_inst.clone(),
//                 user_inst.clone(),
//                 user_margin.clone(),
//                 venue_spot.clone(),
//                 venue_spot.clone(),
//                 user_margin.clone(),
//                 freed_margin_amount.abs(),
//                 instrument_amount,
//                 instrument_unit_price,
//                 commission_amount,
//                 pnl.abs(),
//             )
//             .unwrap();

//         info!("Strategy Margin Posted: {}", ledger.margin_posted(venue_margin.id, &strategy));
//         info!("Strategy Balance: {}", ledger.balance(user_inst.id));
//         info!("Strategy Position Amount: {}", ledger.position_amount(user_inst.id, &strategy));
//         info!("Strategy Cost Basis: {}", ledger.cost_basis(user_inst.id, &strategy));
//         info!("Strategy PnL: {}", ledger.pnl(user_margin.id, &strategy));

//         // let transfers = ledger.get_transfers();
//         // assert_eq!(transfers.len(), 10);
//     }
