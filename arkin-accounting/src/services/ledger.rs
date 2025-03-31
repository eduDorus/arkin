use async_trait::async_trait;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::{collections::HashMap, sync::Arc};
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{ledger::Ledger, Accounting, AccountingError, AccountingService};

#[derive(TypedBuilder)]
pub struct LedgerAccounting {
    pubsub: PubSubHandle,
    #[builder(default = Ledger::builder().build())]
    ledger: Ledger,
}

impl LedgerAccounting {
    pub async fn deposit(
        &self,
        debit_venue: &Arc<Venue>,
        credit_venue: &Arc<Venue>,
        asset: &Tradable,
        amount: Decimal,
        account_type: &AccountType,
    ) -> Result<(), AccountingError> {
        let debit_account = self
            .find_or_create_account(debit_venue, &asset, &AccountOwner::Venue, &AccountType::Spot)
            .await;
        let credit_account = self
            .find_or_create_account(credit_venue, &asset, &AccountOwner::User, account_type)
            .await;

        let event_time = self.pubsub.current_time().await;
        let transfers = self.ledger.transfer(event_time, &debit_account, &credit_account, amount)?;
        self.pubsub.publish(transfers).await;
        Ok(())
    }

    pub async fn withdraw(
        &self,
        debit_venue: &Arc<Venue>,
        credit_venue: &Arc<Venue>,
        asset: &Tradable,
        amount: Decimal,
        account_type: &AccountType,
    ) -> Result<(), AccountingError> {
        let debit_account = self
            .find_or_create_account(debit_venue, asset, &AccountOwner::User, account_type)
            .await;
        let credit_account = self
            .find_or_create_account(credit_venue, asset, &AccountOwner::Venue, &AccountType::Spot)
            .await;

        let event_time = self.pubsub.current_time().await;
        let transfers = self.ledger.transfer(event_time, &debit_account, &credit_account, amount)?;
        self.pubsub.publish(transfers).await;
        Ok(())
    }

    pub async fn exchange(
        &self,
        venue: Arc<Venue>,
        debit_asset: Tradable,
        credit_asset: Tradable,
        debit_amount: Decimal,
        credit_amount: Decimal,
    ) -> Result<(), AccountingError> {
        let transfer_group_id = Uuid::new_v4();
        let event_time = self.pubsub.current_time().await;

        let debit_account = self
            .find_or_create_account(&venue, &debit_asset, &AccountOwner::User, &AccountType::Spot)
            .await;
        let venue_credit_account = self
            .find_or_create_account(&venue, &debit_asset, &AccountOwner::Venue, &AccountType::Spot)
            .await;

        let t1 = Transfer::builder()
            .event_time(event_time)
            .transfer_group_id(transfer_group_id)
            .asset(debit_asset)
            .debit_account(debit_account)
            .credit_account(venue_credit_account)
            .amount(debit_amount)
            .unit_price(Decimal::ONE)
            .transfer_type(TransferType::Exchange)
            .build()
            .into();

        let venue_debit_account = self
            .find_or_create_account(&venue, &credit_asset, &AccountOwner::Venue, &AccountType::Spot)
            .await;
        let credit_account = self
            .find_or_create_account(&venue, &credit_asset, &AccountOwner::User, &AccountType::Spot)
            .await;

        let t2 = Transfer::builder()
            .event_time(event_time)
            .transfer_group_id(transfer_group_id)
            .asset(credit_asset)
            .debit_account(venue_debit_account)
            .credit_account(credit_account)
            .amount(credit_amount)
            .unit_price(Decimal::ONE)
            .transfer_type(TransferType::Exchange)
            .build()
            .into();

        let transfers = self.ledger.apply_transfers(&[t1, t2])?;
        self.pubsub.publish(transfers).await;
        Ok(())
    }

    pub async fn margin_trade(
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
        let event_time = self.pubsub.current_time().await;
        let venue = instrument.venue.clone();
        let inst_asset = Tradable::Instrument(instrument.clone());
        let margin_asset = Tradable::Asset(instrument.margin_asset.clone());
        let commission_asset = Tradable::Asset(commission_asset.unwrap_or_else(|| instrument.margin_asset.clone()));

        //  Find or create necessary accounts
        let user_margin = self
            .find_or_create_account(&venue, &margin_asset, &AccountOwner::User, &AccountType::Margin)
            .await;
        let venue_margin = self
            .find_or_create_account(&venue, &margin_asset, &AccountOwner::Venue, &AccountType::Margin)
            .await;
        let user_inst = self
            .find_or_create_account(&venue, &inst_asset, &AccountOwner::User, &AccountType::Instrument)
            .await;
        let venue_inst = self
            .find_or_create_account(&venue, &inst_asset, &AccountOwner::Venue, &AccountType::Instrument)
            .await;
        let venue_spot = self
            .find_or_create_account(&venue, &commission_asset, &AccountOwner::Venue, &AccountType::Spot)
            .await;

        let (cost_basis, current_position) = self.ledger.current_position(&strategy, Some(&instrument));
        info!("Cost Basis: {}, Current Position {}", cost_basis, current_position);
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

        let entry_price = if !current_position.is_zero() {
            cost_basis / current_position.abs()
        } else {
            Decimal::ZERO
        };
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
        let margin_delta = if amount_closed > Decimal::ZERO && current_position.signum() == new_position.signum() {
            let current_margin = self.ledger.margin_posted(&strategy, Some(&instrument));
            let closing_margin = current_margin * (amount_closed / current_position.abs());
            -closing_margin
        } else if amount_closed.is_zero()
            && (current_position.signum() == new_position.signum() || current_position.is_zero())
        {
            let posting = new_position.abs() * price * margin_rate;
            posting
        } else {
            let posting = new_position.abs() * price * margin_rate;
            let current_margin = self.ledger.margin_posted(&strategy, Some(&instrument));
            let closing_margin = current_margin * (amount_closed / current_position.abs());
            posting - closing_margin
        };
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
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(user_margin.asset.clone())
                    .strategy(Some(strategy.clone()))
                    .instrument(Some(instrument.clone()))
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
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(venue_margin.asset.clone())
                    .strategy(Some(strategy.clone()))
                    .instrument(Some(instrument.clone()))
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
                .event_time(event_time)
                .transfer_group_id(transfer_group_id)
                .asset(user_margin.asset.clone())
                .strategy(Some(strategy.clone()))
                .instrument(Some(instrument.clone()))
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
                .event_time(event_time)
                .transfer_group_id(transfer_group_id)
                .asset(debit_inst.asset.clone())
                .strategy(Some(strategy.clone()))
                .instrument(Some(instrument.clone()))
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
                        .event_time(event_time)
                        .transfer_group_id(transfer_group_id)
                        .asset(venue_spot.asset.clone())
                        .strategy(Some(strategy.clone()))
                        .instrument(Some(instrument.clone()))
                        .debit_account(venue_spot.clone())
                        .credit_account(user_margin.clone())
                        .amount(pnl)
                        .unit_price(Decimal::ONE)
                        .transfer_type(TransferType::Pnl)
                        .build(),
                ));
            } else if pnl < dec!(0) {
                // Loss: user_margin -> venue_spot
                transfers.push(Arc::new(
                    Transfer::builder()
                        .event_time(event_time)
                        .transfer_group_id(transfer_group_id)
                        .asset(user_margin.asset.clone())
                        .strategy(Some(strategy.clone()))
                        .instrument(Some(instrument.clone()))
                        .debit_account(user_margin.clone())
                        .credit_account(venue_spot.clone())
                        .amount(pnl.abs())
                        .unit_price(Decimal::ONE)
                        .transfer_type(TransferType::Pnl)
                        .build(),
                ));
            }
        }

        for t in &transfers {
            info!("Transfers:");
            info!(" - {}", t);
        }

        // Apply transfers atomically
        let transfers = self.ledger.apply_transfers(&transfers)?;
        self.pubsub.publish(transfers).await;
        Ok(())
    }

    async fn find_or_create_account(
        &self,
        venue: &Arc<Venue>,
        asset: &Tradable,
        owner: &AccountOwner,
        account_type: &AccountType,
    ) -> Arc<Account> {
        if let Some(account) = self.ledger.find_account(venue, asset, owner, account_type) {
            account
        } else {
            let account = self
                .ledger
                .add_account(venue.clone(), asset.clone(), owner.clone(), account_type.clone());

            let event_time = self.pubsub.current_time().await;
            let account_update: Arc<AccountUpdate> = AccountUpdate::builder()
                .event_time(event_time)
                .account(account.clone())
                .build()
                .into();
            self.pubsub.publish(account_update).await;
            account
        }
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
            InstrumentType::Perpetual => {
                debug!("Portfolio processing perpetual order: {}", order);
                self.margin_trade(
                    order.side,
                    order.strategy.clone(),
                    order.instrument.clone(),
                    order.commission_asset.clone(),
                    order.last_fill_quantity,
                    order.last_fill_price,
                    dec!(0.05),
                    dec!(0.0002),
                )
                .await?;
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
    async fn balance(&self, venue: &Arc<Venue>) -> HashMap<Arc<Asset>, Decimal> {
        let accounts = self.ledger.accounts();
        let mut balances = HashMap::new();
        for account in accounts {
            if &account.venue == venue
                && account.account_type == AccountType::Spot
                && account.owner == AccountOwner::User
            {
                if let Tradable::Asset(asset) = &account.asset {
                    let balance = self.ledger.balance(account.id);
                    let entry = balances.entry(asset.clone()).or_insert(Decimal::ZERO);
                    *entry += balance;
                }
            }
        }
        balances
    }

    async fn margin_balance(&self, venue: &Arc<Venue>) -> HashMap<Arc<Asset>, Decimal> {
        let accounts = self.ledger.accounts();
        let mut balances = HashMap::new();
        for account in accounts {
            if &account.venue == venue && account.account_type == AccountType::Margin {
                if let Tradable::Asset(asset) = &account.asset {
                    let balance = self.ledger.balance(account.id);
                    let entry = balances.entry(asset.clone()).or_insert(Decimal::ZERO);
                    *entry += balance;
                }
            }
        }
        balances
    }

    async fn available_margin_balance(&self, venue: &Arc<Venue>) -> HashMap<Arc<Asset>, Decimal> {
        let accounts = self.ledger.accounts();
        let mut balances = HashMap::new();
        for account in accounts {
            if &account.venue == venue
                && account.account_type == AccountType::Margin
                && account.owner == AccountOwner::User
            {
                if let Tradable::Asset(asset) = &account.asset {
                    let balance = self.ledger.balance(account.id);
                    let entry = balances.entry(asset.clone()).or_insert(Decimal::ZERO);
                    *entry += balance;
                }
            }
        }
        balances
    }

    /// Returns the total balance of an asset on a specific venue.
    async fn asset_balance(&self, venue: &Arc<Venue>, asset: &Arc<Asset>) -> Decimal {
        let tradable = Tradable::Asset(asset.clone());
        let account = self
            .find_or_create_account(&venue, &tradable, &AccountOwner::User, &AccountType::Spot)
            .await;

        self.ledger.balance(account.id)
    }

    async fn asset_margin_balance(&self, venue: &Arc<Venue>, asset: &Arc<Asset>) -> Decimal {
        let tradable = Tradable::Asset(asset.clone());
        let client_margin = self
            .find_or_create_account(&venue, &tradable, &AccountOwner::User, &AccountType::Margin)
            .await;
        let venue_margin = self
            .find_or_create_account(&venue, &tradable, &AccountOwner::Venue, &AccountType::Margin)
            .await;

        let mut balance = Decimal::ZERO;
        balance += self.ledger.balance(client_margin.id);
        balance += self.ledger.balance(venue_margin.id);
        balance
    }

    async fn asset_available_margin_balance(&self, venue: &Arc<Venue>, asset: &Arc<Asset>) -> Decimal {
        let tradable = Tradable::Asset(asset.clone());
        let account = self
            .find_or_create_account(&venue, &tradable, &AccountOwner::User, &AccountType::Margin)
            .await;

        self.ledger.balance(account.id)
    }

    // --- Position Queries (Global) ---

    /// Returns all open positions across all instruments globally.
    async fn position(&self, venue: &Arc<Venue>) -> HashMap<Arc<Instrument>, Decimal> {
        let accounts = self.ledger.accounts();
        let mut positions = HashMap::new();
        for account in accounts {
            if &account.venue == venue
                && account.account_type == AccountType::Instrument
                && account.owner == AccountOwner::User
            {
                if let Tradable::Instrument(instrument) = &account.asset {
                    let balance = self.ledger.balance(account.id);
                    let entry = positions.entry(instrument.clone()).or_insert(Decimal::ZERO);
                    *entry += balance;
                }
            }
        }
        positions
    }

    async fn position_value(&self, _venue: &Arc<Venue>) -> HashMap<Arc<Instrument>, Decimal> {
        todo!()
        // let accounts = self.ledger.accounts();
        // let mut positions = HashMap::new();
        // for account in accounts {
        //     // Trade balance
        //     if &account.venue == venue && account.account_type == AccountType::ClientInstrument {
        //         if let Tradable::Instrument(instrument) = &account.asset {
        //             let balance = self.ledger.strategy_cost_basis(account.id);
        //             let entry = positions.entry(instrument.clone()).or_insert(Decimal::ZERO);
        //             *entry += balance;
        //         }
        //     }
        // }
        // positions
    }

    async fn instrument_position(&self, instrument: &Arc<Instrument>) -> Decimal {
        let venue = instrument.venue.clone();
        let tradable = Tradable::Instrument(instrument.clone());
        let account = self
            .find_or_create_account(&venue, &tradable, &AccountOwner::User, &AccountType::Instrument)
            .await;

        self.ledger.balance(account.id)
    }

    // --- Strategy-Specific Queries ---

    async fn strategy_position(&self, strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal> {
        let accounts = self.ledger.accounts();
        let mut positions = HashMap::new();
        for account in accounts {
            if account.account_type == AccountType::Instrument && account.owner == AccountOwner::User {
                if let Tradable::Instrument(instrument) = &account.asset {
                    let balance = self.ledger.strategy_balance(&strategy, Some(instrument));
                    let entry = positions.entry(instrument.clone()).or_insert(Decimal::ZERO);
                    *entry += balance;
                }
            }
        }
        positions
    }

    async fn strategy_position_value(&self, _strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal> {
        todo!()
        // let accounts = self.ledger.accounts();
        // let mut positions = HashMap::new();
        // for account in accounts {
        //     if account.account_type == AccountType::ClientInstrument {
        //         if let Tradable::Instrument(instrument) = &account.asset {
        //             let balance = self.ledger.strategy_cost_basis(account.id, &strategy);
        //             let entry = positions.entry(instrument.clone()).or_insert(Decimal::ZERO);
        //             *entry += balance;
        //         }
        //     }
        // }
        // positions
    }

    async fn strategy_instrument_position(&self, strategy: &Arc<Strategy>, instrument: &Arc<Instrument>) -> Decimal {
        self.ledger.strategy_balance(strategy, Some(instrument))
    }

    async fn strategy_instrument_position_value(
        &self,
        strategy: &Arc<Strategy>,
        instrument: &Arc<Instrument>,
    ) -> Decimal {
        self.ledger.strategy_net_value(strategy, Some(instrument))
    }

    async fn strategy_realized_pnl(&self, strategy: &Arc<Strategy>) -> Decimal {
        let accounts = self.ledger.accounts();
        let mut pnl = Decimal::ZERO;
        for account in accounts {
            if account.owner == AccountOwner::User
                && (account.account_type == AccountType::Margin || account.account_type == AccountType::Spot)
            {
                let balance = self.ledger.strategy_pnl(strategy, None);
                pnl += balance;
            }
        }
        pnl
    }
}

#[async_trait]
impl AccountingService for LedgerAccounting {}

#[async_trait]
impl RunnableService for LedgerAccounting {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        loop {
            select! {
                Some(event) = self.pubsub.recv() => {
                    match event {
                        Event::VenueOrderFillUpdate(order) => {
                          self.order_update(order).await.unwrap_or_else(|e| {
                            error!("Failed to process order update: {}", e);
                          });
                        },
                        Event::Finished => {
                            break;
                        }
                        _ => {}
                    }
                    self.pubsub.ack().await;
                }
                _ = shutdown.cancelled() => {
                    debug!("Accounting shutting down...");
                    let transfers = self.ledger.get_transfers();
                    for t in transfers {
                        debug!(" - {}", t);
                    }

                    let accounts = self.ledger.accounts();
                    for account in accounts {
                        debug!("BALANCE {}: {}", account, self.ledger.balance(account.id));
                    }

                    break;
                }
            }
        }
        info!("Accounting service stopped.");
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use test_log::test;

//     #[test(tokio::test)]
//     async fn test_multi_strategy_multi_instrument() {
//         let pubsub = PubSub::new(1024);
//         let accounting = LedgerAccounting::builder().pubsub(pubsub).build();
//         let strategy_1 = test_strategy_1();
//         let strategy_2 = test_strategy_2();

//         let personal = test_personal_venue();
//         let venue = test_binance_venue();
//         let inst_btc = test_inst_binance_btc_usdt_perp();
//         let inst_eth = test_inst_binance_eth_usdt_perp();
//         let usdt = Tradable::Asset(test_usdt_asset());

//         // Initial deposit
//         accounting
//             .deposit(&personal, &venue, &usdt, dec!(10000), &AccountType::Margin)
//             .await
//             .unwrap();

//         let user_margin = accounting
//             .ledger
//             .find_or_create_account(&venue, &usdt, &AccountOwner::User, &AccountType::Margin);

//         // Go long strategy 1: Buy 1 BTC at 1000 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy_1.clone(),
//                 inst_btc.clone(),
//                 None,
//                 dec!(1),
//                 dec!(1000),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         // Go long strategy 2: Buy 1 ETH at 2000 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy_2.clone(),
//                 inst_eth.clone(),
//                 None,
//                 dec!(1),
//                 dec!(2000),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy_1, Some(&inst_btc)), dec!(1));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_1, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_2, Some(&inst_btc)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_2, Some(&inst_eth)), dec!(1));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_1, Some(&inst_btc)), dec!(1000));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_1, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_2, Some(&inst_btc)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_2, Some(&inst_eth)), dec!(2000));
//         assert_eq!(accounting.ledger.margin_posted(&strategy_1, None), dec!(50));
//         assert_eq!(accounting.ledger.margin_posted(&strategy_2, None), dec!(100));
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9849.4));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy_1, None), dec!(0));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy_2, None), dec!(0));

//         // Go long strategy 1: Buy 1 ETH at 2200 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy_1.clone(),
//                 inst_eth.clone(),
//                 None,
//                 dec!(1),
//                 dec!(2200),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         // Go long strategy 2: Buy 1 BTC at 1200 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy_2.clone(),
//                 inst_btc.clone(),
//                 None,
//                 dec!(1),
//                 dec!(1200),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy_1, Some(&inst_btc)), dec!(1));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_1, Some(&inst_eth)), dec!(1));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_2, Some(&inst_btc)), dec!(1));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_2, Some(&inst_eth)), dec!(1));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_1, Some(&inst_btc)), dec!(1000));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_1, Some(&inst_eth)), dec!(2200));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_2, Some(&inst_btc)), dec!(1200));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_2, Some(&inst_eth)), dec!(2000));
//         assert_eq!(accounting.ledger.margin_posted(&strategy_1, None), dec!(160));
//         assert_eq!(accounting.ledger.margin_posted(&strategy_2, None), dec!(160));
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9678.72));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy_1, None), dec!(0));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy_2, None), dec!(0));

//         // Reduce strategy 1: Sell 1 ETH at 1800 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy_1.clone(),
//                 inst_eth.clone(),
//                 None,
//                 dec!(1),
//                 dec!(1800),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         // Reduce strategy 2: Sell 1 ETh at 1800 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy_2.clone(),
//                 inst_eth.clone(),
//                 None,
//                 dec!(1),
//                 dec!(1800),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy_1, Some(&inst_btc)), dec!(1));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_1, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_2, Some(&inst_btc)), dec!(1));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_2, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_1, Some(&inst_btc)), dec!(1000));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_1, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_2, Some(&inst_btc)), dec!(1200));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_2, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.margin_posted(&strategy_1, None), dec!(50));
//         assert_eq!(accounting.ledger.margin_posted(&strategy_2, None), dec!(60));
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9288.00));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy_1, None), dec!(-400));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy_2, None), dec!(-200));

//         // Reduce strategy 1: Sell 0.5 BTC at 3000 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy_1.clone(),
//                 inst_btc.clone(),
//                 None,
//                 dec!(0.5),
//                 dec!(3000),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         // Reduce strategy 2: Sell 0.5 BTC at 3000 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy_2.clone(),
//                 inst_btc.clone(),
//                 None,
//                 dec!(0.5),
//                 dec!(3000),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy_1, Some(&inst_btc)), dec!(0.5));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_1, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_2, Some(&inst_btc)), dec!(0.5));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_2, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_1, Some(&inst_btc)), dec!(500));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_1, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_2, Some(&inst_btc)), dec!(600));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_2, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.margin_posted(&strategy_1, None), dec!(25));
//         assert_eq!(accounting.ledger.margin_posted(&strategy_2, None), dec!(30));
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(11242.40));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy_1, None), dec!(600));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy_2, None), dec!(700));

//         // increase strategy 1: Buy 0.5 BTC at 1700 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy_1.clone(),
//                 inst_btc.clone(),
//                 None,
//                 dec!(0.5),
//                 dec!(1700),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         // increase strategy 2: Buy 0.5 BTC at 1700 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy_2.clone(),
//                 inst_btc.clone(),
//                 None,
//                 dec!(0.5),
//                 dec!(1700),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy_1, Some(&inst_btc)), dec!(1));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_1, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_2, Some(&inst_btc)), dec!(1));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_2, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_1, Some(&inst_btc)), dec!(1350));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_1, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_2, Some(&inst_btc)), dec!(1450));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_2, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.margin_posted(&strategy_1, None), dec!(0));
//         assert_eq!(accounting.ledger.margin_posted(&strategy_2, None), dec!(0));
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(11897.06));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy_1, None), dec!(950));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy_2, None), dec!(950));

//         // Reduce strategy 1: Sell 1. BTC at 2000 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy_1.clone(),
//                 inst_btc.clone(),
//                 None,
//                 dec!(1),
//                 dec!(2000),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         // Reduce strategy 2: Sell 1. BTC at 2000 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy_2.clone(),
//                 inst_btc.clone(),
//                 None,
//                 dec!(1),
//                 dec!(2000),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy_1, Some(&inst_btc)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_1, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_2, Some(&inst_btc)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_balance(&strategy_2, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_1, Some(&inst_btc)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_1, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_2, Some(&inst_btc)), dec!(0));
//         assert_eq!(accounting.ledger.strategy_cost_basis(&strategy_2, Some(&inst_eth)), dec!(0));
//         assert_eq!(accounting.ledger.margin_posted(&strategy_1, None), dec!(0));
//         assert_eq!(accounting.ledger.margin_posted(&strategy_2, None), dec!(0));
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(13096.26));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy_1, None), dec!(1600.0));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy_2, None), dec!(1500.0));
//     }

//     #[test(tokio::test)]
//     async fn test_go_long_and_close() {
//         let pubsub = PubSub::new(1024);
//         let accounting = LedgerAccounting::builder().pubsub(pubsub).build();
//         let strategy = test_strategy_1();
//         let personal = test_personal_venue();
//         let venue = test_binance_venue();
//         let instrument = test_inst_binance_btc_usdt_perp();
//         let margin_asset = instrument.margin_asset.clone();
//         let usdt = Tradable::Asset(instrument.margin_asset.clone());

//         // Initial deposit
//         accounting
//             .deposit(&personal, &venue, &usdt, dec!(10000), &AccountType::Margin)
//             .await
//             .unwrap();

//         // Go long: Buy 1 BTC at 1000 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(1),
//                 dec!(1000),
//                 dec!(0.05),   // 5% margin rate
//                 dec!(0.0002), // 0.02% commission rate
//             )
//             .await
//             .unwrap();

//         let user_margin = accounting
//             .ledger
//             .find_or_create_account(&venue, &usdt, &AccountOwner::User, &AccountType::Margin);
//         let venue_spot = accounting
//             .ledger
//             .find_or_create_account(&venue, &usdt, &AccountOwner::Venue, &AccountType::Spot);

//         // Check balances from ledger
//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(1));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(50)); // 1 * 1000 * 0.05
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9949.8)); // 10000 - 50 - 0.2

//         // TODO: Move this to a separate test
//         // Check margin from accounting
//         let asset_margin_balances = accounting.asset_margin_balance(&venue, &instrument.margin_asset).await;
//         let asset_available_margin_balances = accounting
//             .asset_available_margin_balance(&venue, &instrument.margin_asset)
//             .await;
//         assert_eq!(asset_margin_balances, dec!(9999.8));
//         assert_eq!(asset_available_margin_balances, dec!(9949.8));

//         // Check margin from accounting (venue)
//         let margin_balances = accounting.margin_balance(&venue).await;
//         let available_margin_balances = accounting.available_margin_balance(&venue).await;
//         assert_eq!(margin_balances.get(&margin_asset), Some(&dec!(9999.8)));
//         assert_eq!(available_margin_balances.get(&margin_asset), Some(&dec!(9949.8)));

//         // Check position from accounting
//         let position = accounting.position(&venue).await;
//         assert_eq!(position.get(&instrument), Some(&dec!(1)));

//         // Check specific instrument position from accounting
//         let inst_position = accounting.instrument_position(&instrument).await;
//         assert_eq!(inst_position, dec!(1));

//         // Check strategy position from accounting
//         let strategy_position = accounting.strategy_position(&strategy).await;
//         assert_eq!(strategy_position.get(&instrument), Some(&dec!(1)));

//         // Close: Sell 1 BTC at 1200 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(1),
//                 dec!(1200),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(0));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(0));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy, None), dec!(200));
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(10199.56));
//         assert_eq!(accounting.ledger.balance(venue_spot.id), dec!(-199.5600));

//         // Check margin from accounting
//         let asset_margin_balances = accounting.asset_margin_balance(&venue, &instrument.margin_asset).await;
//         let asset_available_margin_balances = accounting
//             .asset_available_margin_balance(&venue, &instrument.margin_asset)
//             .await;
//         assert_eq!(asset_margin_balances, dec!(10199.56));
//         assert_eq!(asset_available_margin_balances, dec!(10199.56));

//         // Check margin from accounting (venue)
//         // TODO: Move this to a separate test
//         let margin_balances = accounting.margin_balance(&venue).await;
//         let available_margin_balances = accounting.available_margin_balance(&venue).await;
//         assert_eq!(margin_balances.get(&margin_asset), Some(&dec!(10199.56)));
//         assert_eq!(available_margin_balances.get(&margin_asset), Some(&dec!(10199.56)));

//         // Check position from accounting
//         let position = accounting.position(&venue).await;
//         assert_eq!(position.get(&instrument), Some(&dec!(0)));
//         assert_eq!(position.len(), 1);

//         // Check specific instrument position from accounting
//         let inst_position = accounting.instrument_position(&instrument).await;
//         assert_eq!(inst_position, dec!(0));

//         // Check strategy position from accounting
//         let strategy_position = accounting.strategy_position(&strategy).await;
//         assert_eq!(strategy_position.get(&instrument), Some(&dec!(0)));

//         // Check realized PnL from accounting
//         let realized_pnl = accounting.strategy_realized_pnl(&strategy).await;
//         assert_eq!(realized_pnl, dec!(200));
//     }

//     #[test(tokio::test)]
//     async fn test_go_long_reduce_then_close() {
//         let pubsub = PubSub::new(1024);
//         let accounting = LedgerAccounting::builder().pubsub(pubsub).build();
//         let strategy = test_strategy_1();
//         let personal = test_personal_venue();
//         let venue = test_binance_venue();
//         let instrument = test_inst_binance_btc_usdt_perp();
//         let usdt = Tradable::Asset(instrument.margin_asset.clone());

//         // Initial deposit
//         accounting
//             .deposit(&personal, &venue, &usdt, dec!(10000), &AccountType::Margin)
//             .await
//             .unwrap();

//         let user_margin = accounting
//             .ledger
//             .find_or_create_account(&venue, &usdt, &AccountOwner::User, &AccountType::Margin);

//         // Go long: Buy 1 BTC at 1000 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(2),
//                 dec!(1000),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(2));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(100));
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9899.6));

//         // Reduce: Sell 0.5 BTC at 1200 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(0.5),
//                 dec!(1200),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(1.5));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(75));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy, None), dec!(100));
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(10024.48));

//         // Close: Sell 0.5 BTC at 800 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(1.5),
//                 dec!(800),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(0));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(0));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy, None), dec!(-200));
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9799.24));
//     }

//     #[test(tokio::test)]
//     async fn test_go_short_and_close() {
//         let pubsub = PubSub::new(1024);
//         let accounting = LedgerAccounting::builder().pubsub(pubsub).build();
//         let strategy = test_strategy_1();
//         let personal = test_personal_venue();
//         let venue = test_binance_venue();
//         let instrument = test_inst_binance_btc_usdt_perp();
//         let usdt = Tradable::Asset(instrument.margin_asset.clone());

//         // Initial deposit
//         accounting
//             .deposit(&personal, &venue, &usdt, dec!(10000), &AccountType::Margin)
//             .await
//             .unwrap();

//         let user_margin = accounting
//             .ledger
//             .find_or_create_account(&venue, &usdt, &AccountOwner::User, &AccountType::Margin);

//         // Go short: Sell 1 BTC at 1000 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(1),
//                 dec!(1000),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(-1));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(50));
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9949.8));

//         // Close: Buy 1 BTC at 800 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(1),
//                 dec!(800),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(0));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(0));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy, None), dec!(200)); // (1000 - 800) * 1
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(10199.64)); // 9949.8 + 50 (freed) - 0.16 (comm) + 200 (PnL)
//     }

//     #[test(tokio::test)]
//     async fn test_go_short_reduce_then_close() {
//         let pubsub = PubSub::new(1024);
//         let accounting = LedgerAccounting::builder().pubsub(pubsub).build();
//         let strategy = test_strategy_1();
//         let personal = test_personal_venue();
//         let venue = test_binance_venue();
//         let instrument = test_inst_binance_btc_usdt_perp();
//         let usdt = Tradable::Asset(instrument.margin_asset.clone());

//         // Initial deposit
//         accounting
//             .deposit(&personal, &venue, &usdt, dec!(10000), &AccountType::Margin)
//             .await
//             .unwrap();

//         let user_margin = accounting
//             .ledger
//             .find_or_create_account(&venue, &usdt, &AccountOwner::User, &AccountType::Margin);

//         // Go short: Sell 1 BTC at 1000 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(1),
//                 dec!(1000),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(-1));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(50));
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9949.8));

//         // Reduce: Buy 0.5 BTC at 800 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(0.5),
//                 dec!(800),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(-0.5));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(25.0)); // 0.5 * 800 * 0.05
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy, None), dec!(100)); // (1000 - 800) * 0.5
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(10074.72)); // 9949.8 + 30 (freed) - 0.16 (comm) + 100 (PnL)

//         // Close: Buy 0.5 BTC at 1200 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(0.5),
//                 dec!(1200),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(0));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(0));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy, None), dec!(0)); // 100 - (1000 - 1200) * 0.5
//         assert_eq!(accounting.ledger.balance(user_margin.id), dec!(9999.60)); // 10079.64 + 20 (freed) - 0.24 (comm) - 100 (PnL)
//     }

//     #[test(tokio::test)]
//     async fn test_go_long_flip_short_flip_long_close() {
//         let pubsub = PubSub::new(1024);
//         let accounting = LedgerAccounting::builder().pubsub(pubsub).build();
//         let strategy = test_strategy_1();
//         let personal = test_personal_venue();
//         let venue = test_binance_venue();
//         let instrument = test_inst_binance_btc_usdt_perp();
//         let usdt = Tradable::Asset(instrument.margin_asset.clone());

//         // Initial deposit
//         accounting
//             .deposit(&personal, &venue, &usdt, dec!(10000), &AccountType::Margin)
//             .await
//             .unwrap();

//         // Go long: Buy 1 BTC at 1000 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(1),
//                 dec!(1000),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(1));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(50));

//         // Flip to short: Sell 2 BTC at 1200 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(2),
//                 dec!(1200),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(-1));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(60)); // 1 * 1200 * 0.05
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy, None), dec!(200)); // (1200 - 1000) * 1

//         // Flip to long: Buy 2 BTC at 800 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Buy,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(2),
//                 dec!(800),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(1));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(40)); // 1 * 800 * 0.05
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy, None), dec!(600)); // 200 - (1200 - 800) * 1

//         // Close: Sell 1 BTC at 900 USDT
//         accounting
//             .margin_trade(
//                 MarketSide::Sell,
//                 strategy.clone(),
//                 instrument.clone(),
//                 None,
//                 dec!(1),
//                 dec!(900),
//                 dec!(0.05),
//                 dec!(0.0002),
//             )
//             .await
//             .unwrap();

//         assert_eq!(accounting.ledger.strategy_balance(&strategy, None), dec!(0));
//         assert_eq!(accounting.ledger.margin_posted(&strategy, None), dec!(0));
//         assert_eq!(accounting.ledger.strategy_pnl(&strategy, None), dec!(700));
//     }
// }
