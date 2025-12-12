#![allow(dead_code)]
use std::sync::Arc;

use arkin_core::prelude::*;
use async_trait::async_trait;
use rust_decimal::prelude::*;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
enum UpdateMode {
    Initial,
    Recon,
}

impl UpdateMode {
    fn warn_diffs(&self) -> bool {
        matches!(self, UpdateMode::Recon)
    }

    fn force_full(&self) -> bool {
        matches!(self, UpdateMode::Initial | UpdateMode::Recon)
    }
}

pub struct Accounting {
    ledger: Arc<Ledger>,
}

impl Accounting {
    pub fn new(ledger: Arc<Ledger>) -> Self {
        Self { ledger }
    }

    async fn process_account_update(&self, ctx: Arc<CoreCtx>, update: &VenueAccountUpdate, mode: UpdateMode) {
        info!(target: "accounting", "processing account update: reason {}", update);
        let time = ctx.now().await;

        let venue = update.venue.clone();
        let transfer_group_id = Uuid::new_v4();
        let transfer_group_type = match mode {
            UpdateMode::Initial => TransferGroupType::Initial,
            UpdateMode::Recon => TransferGroupType::Reconciliation,
        };
        let mut transfers = Vec::new();
        let mut diffs = Vec::new(); // (item: String, diff, current) for warns

        let user_account = self
            .ledger
            .find_or_create_account(&venue, AccountOwner::User, AccountType::Margin, time)
            .await;
        let venue_account = self
            .ledger
            .find_or_create_account(&venue, AccountOwner::Venue, AccountType::Margin, time)
            .await;

        // Balances (margin deltas only)
        for bal in &update.balances {
            let asset = bal.asset.clone();
            let current = self.ledger.balance(&user_account, &asset).await;
            let diff = if mode.force_full() {
                bal.quantity - current
            } else {
                bal.quantity_change
            };
            diffs.push((format!("Asset {}", asset), diff, current));
            if diff == Decimal::ZERO && !mode.force_full() {
                continue;
            }
            let (debit, credit, amount) = if diff > Decimal::ZERO {
                (venue_account.clone(), user_account.clone(), diff)
            } else {
                (user_account.clone(), venue_account.clone(), -diff)
            };
            transfers.push(
                Transfer::builder()
                    .transfer_group_id(transfer_group_id)
                    .transfer_group_type(transfer_group_type)
                    .transfer_type(TransferType::Margin)
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(Decimal::ONE)
                    .strategy(None)
                    .instrument(None)
                    .asset(Some(asset))
                    .created(time)
                    .build()
                    .into(),
            );
        }

        // Positions
        for pos in &update.positions {
            let instr = pos.instrument.clone();
            let (_entry_price, qty_current) = self.ledger.get_position(&user_account, &instr).await;
            let qty_diff = pos.quantity - qty_current; // Absolute sync always
            info!(target: "accounting", "position change for {} from {} to {} = {}", instr,  qty_current, pos.quantity, qty_diff);
            diffs.push((format!("Instr {}", instr.symbol), qty_diff, qty_current));
            if !qty_diff.is_zero() {
                let (debit, credit, amount) = if qty_diff > Decimal::ZERO {
                    (venue_account.clone(), user_account.clone(), qty_diff)
                } else {
                    (user_account.clone(), venue_account.clone(), -qty_diff)
                };
                transfers.push(
                    Transfer::builder()
                        .transfer_group_id(transfer_group_id)
                        .transfer_group_type(transfer_group_type)
                        .transfer_type(TransferType::Transfer)
                        .debit_account(debit)
                        .credit_account(credit)
                        .amount(amount.abs())
                        .unit_price(pos.entry_price)
                        .strategy(None)
                        .instrument(Some(instr.clone()))
                        .asset(None)
                        .created(time)
                        .build()
                        .into(),
                );
            }

            // Warn on unrealized (no transfer)
            if mode.warn_diffs() && !pos.unrealized_pnl.is_zero() {
                warn!(target: "accounting", "Unrealized PNL {} for {} (not transferred)", pos.unrealized_pnl, instr.symbol);
            }
        }

        if !transfers.is_empty() {
            let batch = TransferBatch::builder().event_time(time).transfers(transfers).build();
            let res = self.ledger.apply_transfers(batch.into()).await;
            match res {
                Ok(_) => {
                    info!(target: "accounting", "Account update applied (mode: {:?})", mode);
                    if mode.warn_diffs() {
                        for (item, diff, current) in diffs {
                            let threshold = current.abs() * dec!(0.0001);
                            if diff.abs() > threshold {
                                warn!(target: "accounting", "Large diff for {}: {} (threshold {})", item, diff, threshold);
                            }
                        }
                    }
                }
                Err(e) => error!(target: "accounting", "Failed account update: {}", e),
            }
        } else if mode.force_full() {
            info!(target: "accounting", "Account in sync (no changes)");
        }
    }

    // Wrappers
    pub async fn handle_initial_account_update(&self, ctx: Arc<CoreCtx>, update: &VenueAccountUpdate) {
        self.process_account_update(ctx, update, UpdateMode::Initial).await;
    }

    pub async fn handle_reconcile_account_update(&self, ctx: Arc<CoreCtx>, update: &VenueAccountUpdate) {
        self.process_account_update(ctx, update, UpdateMode::Recon).await;
    }

    async fn handle_fill_update(&self, ctx: Arc<CoreCtx>, update: &VenueOrder) {
        info!(target: "accounting", "processing trade update for order {}", update.id);
        let time = ctx.now().await;

        let transfer_group_id = Uuid::new_v4();
        let transfer_group_type = TransferGroupType::Trade;
        let venue = update.instrument.venue.clone();
        let mut transfers = Vec::new();

        let user_account = self
            .ledger
            .find_or_create_account(&venue, AccountOwner::User, AccountType::Margin, time)
            .await;
        let venue_account = self
            .ledger
            .find_or_create_account(&venue, AccountOwner::Venue, AccountType::Margin, time)
            .await;
        let instrument = update.instrument.clone();
        let margin_asset = instrument.margin_asset.clone();
        let side_sign: Decimal = update.side.into();
        let quantity = update.last_fill_quantity;
        let price = update.last_fill_price;
        let commission = update.commission;

        // Query current (strat-specific if available)
        let (entry, current_qty) = if let Some(strat) = update.strategy.clone() {
            self.ledger.get_strategy_position(&user_account, strat, &instrument).await
        } else {
            self.ledger.get_position(&user_account, &instrument).await
        };

        let qty_delta = quantity * side_sign;

        // Calc closed amt/PNL
        let amt_closed = if current_qty.signum() != qty_delta.signum() {
            current_qty.abs().min(quantity)
        } else {
            dec!(0)
        };
        let pnl = if amt_closed > dec!(0) {
            let pnl_sign = if current_qty > dec!(0) {
                dec!(1)
            } else {
                dec!(-1)
            };
            (price - entry) * amt_closed * pnl_sign
        } else {
            dec!(0)
        };

        // Margin delta
        let leverage = dec!(10); // From config or query
        let margin_release = if amt_closed > dec!(0) {
            amt_closed * entry / leverage
        } else {
            dec!(0)
        };
        let margin_post = if qty_delta.abs() > amt_closed {
            (qty_delta.abs() - amt_closed) * price / leverage
        } else {
            dec!(0)
        };
        let margin_delta = margin_release - margin_post;

        // Margin transfer if delta !=0
        if !margin_delta.is_zero() {
            let (debit, credit, amount) = if margin_delta > dec!(0) {
                (venue_account.clone(), user_account.clone(), margin_delta)
            } else {
                (user_account.clone(), venue_account.clone(), -margin_delta)
            };
            transfers.push(
                Transfer::builder()
                    .transfer_group_id(transfer_group_id)
                    .transfer_group_type(transfer_group_type)
                    .transfer_type(TransferType::Margin)
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(Decimal::ONE)
                    .strategy(update.strategy.clone())
                    .instrument(Some(instrument.clone()))
                    .asset(Some(margin_asset.clone()))
                    .created(time)
                    .build()
                    .into(),
            );
        }

        // Position transfer
        if !qty_delta.is_zero() {
            let (debit, credit, amount) = if qty_delta > dec!(0) {
                (venue_account.clone(), user_account.clone(), qty_delta)
            } else {
                (user_account.clone(), venue_account.clone(), -qty_delta)
            };
            transfers.push(
                Transfer::builder()
                    .transfer_group_id(transfer_group_id)
                    .transfer_group_type(transfer_group_type)
                    .transfer_type(TransferType::Transfer)
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount.abs())
                    .unit_price(price) // Fill price for new
                    .strategy(update.strategy.clone())
                    .instrument(Some(instrument.clone()))
                    .asset(Some(margin_asset.clone()))
                    .created(time)
                    .build()
                    .into(),
            );
        }

        // Commission (as before, with instrument for attribution)
        if !commission.is_zero() {
            transfers.push(
                Transfer::builder()
                    .transfer_group_id(transfer_group_id)
                    .transfer_group_type(transfer_group_type)
                    .transfer_type(TransferType::Commission)
                    .debit_account(user_account.clone())
                    .credit_account(venue_account.clone())
                    .amount(commission.abs())
                    .unit_price(Decimal::ONE)
                    .strategy(update.strategy.clone())
                    .instrument(Some(instrument.clone()))
                    .asset(Some(margin_asset.clone()))
                    .created(time)
                    .build()
                    .into(),
            );
        }

        // Realized PNL (as before)
        if !pnl.is_zero() {
            let (debit, credit, amount) = if pnl > dec!(0) {
                (venue_account.clone(), user_account.clone(), pnl)
            } else {
                (user_account.clone(), venue_account.clone(), -pnl)
            };
            transfers.push(
                Transfer::builder()
                    .transfer_group_id(transfer_group_id)
                    .transfer_group_type(transfer_group_type)
                    .transfer_type(TransferType::Pnl)
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(Decimal::ONE)
                    .strategy(update.strategy.clone())
                    .instrument(Some(instrument))
                    .asset(Some(margin_asset))
                    .created(time)
                    .build()
                    .into(),
            );
        }

        if !transfers.is_empty() {
            let batch = TransferBatch::builder().event_time(time).transfers(transfers).build();
            if let Err(e) = self.ledger.apply_transfers(batch.clone().into()).await {
                error!(target: "accounting", "Failed trade update: {}", e);
                for transfer in batch.transfers {
                    error!(target: "accounting", " - : {}", transfer);
                }
            } else {
                info!(target: "accounting", "Trade update applied");
            }
        }
    }
}

#[async_trait]
impl Runnable for Accounting {
    async fn handle_event(&self, core_ctx: Arc<CoreCtx>, event: Event) {
        match &event {
            // Account
            Event::InitialAccountUpdate(au) => self.handle_initial_account_update(core_ctx, au).await,
            Event::ReconcileAccountUpdate(au) => self.handle_reconcile_account_update(core_ctx, au).await,
            Event::VenueOrderFill(tu) => self.handle_fill_update(core_ctx, tu).await,
            e => warn!(target: "accounting", "received unused event {}", e),
        }
    }

    async fn teardown(&self, _ctx: Arc<ServiceCtx>, _core_ctx: Arc<CoreCtx>) {
        self.ledger.dump_state(1000).await;
    }
}

// #[cfg(test)]
// mod tests {
//     use std::time::Duration;

//     use super::*;
//     use arkin_core::test_utils::{MockPublisher, MockTime};
//     use uuid::Uuid;

//     #[tokio::test]
//     #[test_log::test]
//     async fn test_account_sync() {
//         let time = MockTime::new();
//         let publisher = MockPublisher::new();
//         let ledger = Ledger::new(publisher.clone());
//         let accounting = Accounting::builder()
//             .identifier("test".to_string())
//             .time(time.clone())
//             .publisher(publisher.clone())
//             .ledger(ledger.clone())
//             .build();

//         let venue = test_binance_venue();
//         let usdt = test_usdt_asset();
//         let btc_perp = test_inst_binance_btc_usdt_perp(); // Assume leverage=20
//         let eth_perp = test_inst_binance_eth_usdt_perp(); // Assume leverage=20

//         let user_account = ledger
//             .find_or_create_account(&venue, AccountOwner::User, AccountType::Margin, time.now().await)
//             .await;

//         // Step 1: Initial (100k USDT margin, no positions)
//         let initial = VenueAccountUpdate::builder()
//             .id(Uuid::new_v4())
//             .event_time(time.now().await)
//             .venue(venue.clone())
//             .balances(vec![BalanceUpdate::builder()
//                 .event_time(time.now().await)
//                 .venue(venue.clone())
//                 .asset(usdt.clone())
//                 .account_type(AccountType::Margin)
//                 .quantity_change(dec!(0))
//                 .quantity(dec!(100000))
//                 .build()])
//             .positions(vec![])
//             .reason("INITIAL".to_string())
//             .build();

//         accounting.handle_event(Event::ReconcileAccountUpdate(initial.into())).await;

//         assert_eq!(ledger.balance(&user_account, &usdt).await, dec!(100000));
//         ledger.dump_state(100).await;

//         // Step 2: Long BTC perp (qty=1 @10000, unreal=0, deduct margin=10000/20=500)
//         time.advance_time_by(Duration::from_secs(1)).await;
//         let after_long = VenueAccountUpdate::builder()
//             .id(Uuid::new_v4())
//             .event_time(time.now().await)
//             .venue(venue.clone())
//             .balances(vec![BalanceUpdate::builder()
//                 .event_time(time.now().await)
//                 .venue(venue.clone())
//                 .asset(usdt.clone())
//                 .account_type(AccountType::Margin)
//                 .quantity_change(dec!(-500))
//                 .quantity(dec!(99500))
//                 .build()])
//             .positions(vec![PositionUpdate::builder()
//                 .event_time(time.now().await)
//                 .instrument(btc_perp.clone())
//                 .account_type(AccountType::Margin)
//                 .entry_price(dec!(10000))
//                 .quantity(dec!(1))
//                 .realized_pnl(dec!(0))
//                 .unrealized_pnl(dec!(0))
//                 .position_side(PositionSide::Long)
//                 .build()])
//             .reason("ORDER".to_string())
//             .build();
//         accounting.handle_event(Event::VenueAccountUpdate(after_long.into())).await;

//         assert_eq!(ledger.get_position(&user_account, &btc_perp).await, dec!(1));
//         assert_eq!(ledger.balance(&user_account, &usdt).await, dec!(99500));
//         ledger.dump_state(100).await;

//         // Step 3: Short ETH perp (qty=-2 @2000, unreal=0, deduct margin=abs(-2)*2000/20=200)
//         time.advance_time_by(Duration::from_secs(1)).await;
//         let after_short = VenueAccountUpdate::builder()
//             .id(Uuid::new_v4())
//             .event_time(time.now().await)
//             .venue(venue.clone())
//             .balances(vec![BalanceUpdate::builder()
//                 .event_time(time.now().await)
//                 .venue(venue.clone())
//                 .asset(usdt.clone())
//                 .account_type(AccountType::Margin)
//                 .quantity_change(dec!(-200))
//                 .quantity(dec!(99300))
//                 .build()])
//             .positions(vec![PositionUpdate::builder()
//                 .event_time(time.now().await)
//                 .instrument(eth_perp.clone())
//                 .account_type(AccountType::Margin)
//                 .entry_price(dec!(2000))
//                 .quantity(dec!(-2))
//                 .realized_pnl(dec!(0))
//                 .unrealized_pnl(dec!(0))
//                 .position_side(PositionSide::Short)
//                 .build()])
//             .reason("ORDER".to_string())
//             .build();
//         accounting.handle_event(Event::VenueAccountUpdate(after_short.into())).await;

//         assert_eq!(ledger.get_position(&user_account, &eth_perp).await, dec!(-2));
//         assert_eq!(ledger.balance(&user_account, &usdt).await, dec!(99300));
//         ledger.dump_state(100).await;

//         // Step 4: Reconciliation (small mismatch, e.g., balance 99301 due to funding fee, BTC qty 1.001 drift)
//         time.advance_time_by(Duration::from_secs(1)).await;
//         let recon = VenueAccountUpdate::builder()
//             .id(Uuid::new_v4())
//             .event_time(time.now().await)
//             .venue(venue.clone())
//             .balances(vec![BalanceUpdate::builder()
//                 .event_time(time.now().await)
//                 .venue(venue.clone())
//                 .asset(usdt.clone())
//                 .account_type(AccountType::Margin)
//                 .quantity_change(dec!(1))
//                 .quantity(dec!(99301))
//                 .build()])
//             .positions(vec![PositionUpdate::builder()
//                 .event_time(time.now().await)
//                 .instrument(btc_perp.clone())
//                 .account_type(AccountType::Margin)
//                 .entry_price(dec!(10000))
//                 .quantity(dec!(1.001))
//                 .realized_pnl(dec!(0))
//                 .unrealized_pnl(dec!(0))
//                 .position_side(PositionSide::Long)
//                 .build()])
//             .reason("RECON".to_string())
//             .build();
//         accounting.handle_event(Event::ReconcileAccountUpdate(recon.into())).await;

//         assert_eq!(ledger.balance(&user_account, &usdt).await, dec!(99301));
//         assert_eq!(ledger.get_position(&user_account, &btc_perp).await, dec!(1.001));
//         ledger.dump_state(100).await;

//         // Step 5: Sell BTC long with gain (close to qty=0, realized +1000, release margin 10000/20=500 + gain to margin)
//         time.advance_time_by(Duration::from_secs(1)).await;
//         let sell_btc = VenueAccountUpdate::builder()
//             .id(Uuid::new_v4())
//             .event_time(time.now().await)
//             .venue(venue.clone())
//             .balances(vec![BalanceUpdate::builder()
//                 .event_time(time.now().await)
//                 .venue(venue.clone())
//                 .asset(usdt.clone())
//                 .account_type(AccountType::Margin)
//                 .quantity_change(dec!(500)) // Released 500
//                 .quantity(dec!(99801))
//                 .build()])
//             .positions(vec![PositionUpdate::builder()
//                 .event_time(time.now().await)
//                 .instrument(btc_perp.clone())
//                 .account_type(AccountType::Margin)
//                 .entry_price(dec!(0)) // Closed
//                 .quantity(dec!(0))
//                 .realized_pnl(dec!(1000))
//                 .unrealized_pnl(dec!(0))
//                 .position_side(PositionSide::Long)
//                 .build()])
//             .reason("ORDER".to_string())
//             .build();
//         accounting.handle_event(Event::VenueAccountUpdate(sell_btc.into())).await;

//         assert_eq!(ledger.get_position(&user_account, &btc_perp).await, dec!(0));
//         assert_eq!(ledger.balance(&user_account, &usdt).await, dec!(100801));
//         ledger.dump_state(100).await;

//         // Step 6: Buy back ETH short with loss (close to qty=0, realized -500, release margin 4000/20=200 - loss from margin)
//         time.advance_time_by(Duration::from_secs(1)).await;
//         let buy_eth = VenueAccountUpdate::builder()
//             .id(Uuid::new_v4())
//             .event_time(time.now().await)
//             .venue(venue.clone())
//             .balances(vec![BalanceUpdate::builder()
//                 .event_time(time.now().await)
//                 .venue(venue.clone())
//                 .asset(usdt.clone())
//                 .account_type(AccountType::Margin)
//                 .quantity_change(dec!(200)) // Released 200
//                 .quantity(dec!(101001))
//                 .build()])
//             .positions(vec![PositionUpdate::builder()
//                 .event_time(time.now().await)
//                 .instrument(eth_perp.clone())
//                 .account_type(AccountType::Margin)
//                 .entry_price(dec!(0))
//                 .quantity(dec!(0))
//                 .realized_pnl(dec!(-500))
//                 .unrealized_pnl(dec!(0))
//                 .position_side(PositionSide::Short)
//                 .build()])
//             .reason("ORDER".to_string())
//             .build();
//         accounting.handle_event(Event::VenueAccountUpdate(buy_eth.into())).await;

//         assert_eq!(ledger.get_position(&user_account, &eth_perp).await, dec!(0));
//         assert_eq!(ledger.balance(&user_account, &usdt).await, dec!(100501));
//         ledger.dump_state(100).await;
//     }
// }

// #[tokio::test]
// #[test_log::test]
// #[ignore]
// async fn test_accounting() {
//     let time = MockTime::new();
//     let publisher = MockPublisher::new();
//     let ledger = Ledger::new();
//     let accounting = Accounting::builder()
//         .identifier("test".to_string())
//         .time(time.clone())
//         .publisher(publisher.clone())
//         .ledger(ledger.clone())
//         .build();

//     // Initial zero assertion
//     let usdt_account = ledger.find_or_create_account(
//         &test_binance_venue(),
//         &Tradable::Asset(test_usdt_asset()),
//         AccountOwner::User,
//         AccountType::Margin,
//         time.now().await,
//     );
//     assert_eq!(ledger.balance(usdt_account.id).await, dec!(0));

//     let init_balance_update_1 = BalanceUpdate::builder()
//         .id(Uuid::new_v4())
//         .event_time(time.now().await)
//         .venue(test_binance_venue())
//         .account_type(AccountType::Margin)
//         .asset(test_usdt_asset())
//         .quantity_change(dec!(0))
//         .quantity(dec!(100000))
//         .build();

//     accounting
//         .handle_event(Event::InitialBalanceUpdate(init_balance_update_1.clone().into()))
//         .await;

//     assert_eq!(ledger.balance(usdt_account.id).await, dec!(100000));

//     let init_position_update_1 = PositionUpdate::builder()
//         .id(Uuid::new_v4())
//         .event_time(time.now().await)
//         .account_type(AccountType::Margin)
//         .instrument(test_inst_binance_btc_usdt_perp())
//         .entry_price(dec!(10000))
//         .quantity(dec!(1))
//         .realized_pnl(dec!(0))
//         .unrealized_pnl(dec!(100))
//         .position_side(PositionSide::Long)
//         .build();

//     accounting
//         .handle_event(Event::InitialPositionUpdate(init_position_update_1.clone().into()))
//         .await;

//     let btc_position_account = ledger.find_or_create_account(
//         &test_binance_venue(),
//         &Tradable::Asset(test_btc_asset()), // Assume base
//         AccountOwner::User,
//         AccountType::Instrument,
//         time.now().await,
//     );
//     assert_eq!(ledger.balance(btc_position_account.id).await, dec!(1));

//     let unrealized_pnl_account = ledger.find_or_create_account(
//         &test_binance_venue(),
//         &Tradable::Asset(test_usdt_asset()),
//         AccountOwner::User,
//         AccountType::Equity,
//         time.now().await,
//     );
//     assert_eq!(ledger.balance(unrealized_pnl_account.id).await, dec!(100));

//     // We realize some pnl
//     let init_position_update_2 = PositionUpdate::builder()
//         .id(Uuid::new_v4())
//         .event_time(time.now().await)
//         .account_type(AccountType::Margin)
//         .instrument(test_inst_binance_btc_usdt_perp())
//         .entry_price(dec!(10000))
//         .quantity(dec!(0.5))
//         .realized_pnl(dec!(50))
//         .unrealized_pnl(dec!(50))
//         .position_side(PositionSide::Long)
//         .build();

//     accounting
//         .handle_event(Event::PositionUpdate(init_position_update_2.clone().into()))
//         .await;

//     let btc_position_account = ledger.find_or_create_account(
//         &test_binance_venue(),
//         &Tradable::Asset(test_btc_asset()), // Assume base
//         AccountOwner::User,
//         AccountType::Instrument,
//         time.now().await,
//     );
//     assert_eq!(ledger.balance(btc_position_account.id).await, dec!(0.5));

//     let unrealized_pnl_account = ledger.find_or_create_account(
//         &test_binance_venue(),
//         &Tradable::Asset(test_usdt_asset()),
//         AccountOwner::User,
//         AccountType::Equity,
//         time.now().await,
//     );
//     assert_eq!(ledger.balance(unrealized_pnl_account.id).await, dec!(50));

//     let margin_account = ledger.find_or_create_account(
//         &test_binance_venue(),
//         &Tradable::Asset(test_usdt_asset()),
//         AccountOwner::User,
//         AccountType::Margin,
//         time.now().await,
//     );
//     assert_eq!(ledger.balance(margin_account.id).await, dec!(100050));
//     ledger.dump_state(100).await;
// }
// }

// #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
// pub async fn venue_order_fill(&self, order: &VenueOrder) {
//     let transfer_group_id = Uuid::new_v4();
//     let event_time = self.time.now().await;

//     let venue = order.instrument.venue.clone();
//     let (credit_asset, debit_asset) = if order.side == MarketSide::Buy {
//         (order.instrument.base_asset.clone(), order.instrument.quote_asset.clone())
//     } else {
//         (order.instrument.quote_asset.clone(), order.instrument.base_asset.clone())
//     };

//     let (credit_amount, debit_amount) = if order.side == MarketSide::Buy {
//         (order.quantity, order.quantity * order.price)
//     } else {
//         (order.quantity * order.price, order.quantity)
//     };

//     let mut transfers = vec![];

//     // Debit transfer
//     let debit_account =
//         self.ledger
//             .find_or_create_account(&venue, AccountOwner::User, AccountType::Spot, self.time.now().await);
//     let credit_account =
//         self.ledger
//             .find_or_create_account(&venue, AccountOwner::Venue, AccountType::Spot, self.time.now().await);

//     transfers.push(
//         Transfer::builder()
//             .event_time(event_time)
//             .transfer_group_id(transfer_group_id)
//             .debit_account(debit_account)
//             .credit_account(credit_account)
//             .amount(debit_amount)
//             .unit_price(Decimal::ONE)
//             .transfer_type(TransferType::Exchange)
//             .asset(debit_asset.clone())
//             .instrument(None)
//             .strategy(None)
//             .build()
//             .into(),
//     );

//     // Credit transfer
//     let debit_account = self.ledger.find_or_create_account(
//         &venue,
//         &credit_asset,
//         AccountOwner::Venue,
//         AccountType::Spot,
//         self.time.now().await,
//     );
//     let credit_account = self.ledger.find_or_create_account(
//         &venue,
//         &credit_asset,
//         AccountOwner::User,
//         AccountType::Spot,
//         self.time.now().await,
//     );

//     transfers.push(
//         Transfer::builder()
//             .event_time(event_time)
//             .transfer_group_id(transfer_group_id)
//             .asset(credit_asset)
//             .debit_account(debit_account)
//             .credit_account(credit_account)
//             .amount(credit_amount)
//             .unit_price(Decimal::ONE)
//             .transfer_type(TransferType::Exchange)
//             .build()
//             .into(),
//     );

//     // Commission (assume order has commission field; stub 0.1% quote)
//     let commission = (debit_amount * dec!(0.001)).max(dec!(0.01)); // Placeholder
//     if commission > Decimal::ZERO {
//         let user_quote_account = self.ledger.find_or_create_account(
//             &venue,
//             &Tradable::Asset(order.instrument.quote_asset.clone()),
//             AccountOwner::User,
//             AccountType::Spot,
//             self.time.now().await,
//         );
//         let commission_account = self.ledger.find_or_create_account(
//             &venue,
//             &Tradable::Asset(order.instrument.quote_asset.clone()),
//             AccountOwner::Venue,
//             AccountType::Equity, // Commission expense
//             self.time.now().await,
//         );
//         transfers.push(
//             Transfer::builder()
//                 .event_time(event_time)
//                 .transfer_group_id(transfer_group_id)
//                 .asset(Tradable::Asset(order.instrument.quote_asset.clone()))
//                 .debit_account(user_quote_account)
//                 .credit_account(commission_account)
//                 .amount(commission)
//                 .unit_price(Decimal::ONE)
//                 .transfer_type(TransferType::Commission)
//                 .build()
//                 .into(),
//         );
//     }

//     let res = self.ledger.apply_transfers(&transfers).await;
//     match res {
//         Ok(_) => info!(target: "accounting", "Transfers applied successfully"),
//         Err(e) => error!(target: "accounting", "Failed to apply transfers: {}", e),
//     }
// }

// pub async fn margin_fill(&self, order: &VenueOrder) {
//     // info!(target: "accounting", "Starting Margin Trade...");
//     // info!(target: "accounting", "Side: {}", side);
//     // info!(target: "accounting", "Price: {}", price);
//     // info!(target: "accounting", "Amount: {}", amount);
//     // info!(target: "accounting", "Margin Rate: {}", margin_rate);
//     // info!(target: "accounting", "Commission Rate: {}", commission_rate);
//     let event_time = self.time.now().await;
//     let instrument = order.instrument.clone();
//     let strategy = order.strategy.clone();
//     let venue = order.instrument.venue.clone();
//     let price = order.last_fill_price;
//     let amount = order.last_fill_quantity;
//     let side = order.side;
//     let margin_rate = dec!(0.05);
//     let inst_asset = Tradable::Instrument(order.instrument.clone());
//     let margin_asset = Tradable::Asset(order.instrument.margin_asset.clone());
//     let commission_asset =
//         Tradable::Asset(order.commission_asset.unwrap_or_else(|| order.instrument.margin_asset.clone()));

//     //  Find or create necessary accounts
//     let user_margin =
//         self.ledger
//             .find_or_create_account(&venue, &margin_asset, &AccountOwner::User, &AccountType::Margin);
//     let venue_margin =
//         self.ledger
//             .find_or_create_account(&venue, &margin_asset, &AccountOwner::Venue, &AccountType::Margin);
//     let user_inst =
//         self.ledger
//             .find_or_create_account(&venue, &inst_asset, &AccountOwner::User, &AccountType::Instrument);
//     let venue_inst =
//         self.ledger
//             .find_or_create_account(&venue, &inst_asset, &AccountOwner::Venue, &AccountType::Instrument);
//     let venue_spot =
//         self.ledger
//             .find_or_create_account(&venue, &commission_asset, &AccountOwner::Venue, &AccountType::Spot);

//     let (cost_basis, current_position) =
//         self.ledger.current_position(&order.strategy.unwrap(), Some(&instrument)).await;
//     info!(target: "accounting", "Cost Basis: {}, Current Position {}", cost_basis, current_position);
//     let new_position = match side {
//         MarketSide::Buy => current_position + amount,
//         MarketSide::Sell => current_position - amount,
//     };
//     info!(target: "accounting", "New Position after {} will be: {}", side, new_position);

//     // Calculate amount closed and PnL
//     let amount_closed = if (current_position > Decimal::ZERO && new_position <= Decimal::ZERO)
//         || (current_position < Decimal::ZERO && new_position >= Decimal::ZERO)
//     {
//         info!(target: "accounting", "Position will fully close: {} -> {}", current_position, new_position);
//         current_position.abs() // Full close before flip
//     } else {
//         info!(target: "accounting", "Position will not close fully: {} -> {}", current_position, new_position);
//         amount.min(current_position.abs()) // Partial close
//     };
//     info!(target: "accounting", "Amount closed: {}", amount_closed);

//     let entry_price = if !current_position.is_zero() {
//         cost_basis / current_position.abs()
//     } else {
//         Decimal::ZERO
//     };
//     info!(target: "accounting", "Entry price from ledger: {}", entry_price);
//     let pnl = if current_position > Decimal::ZERO {
//         info!(target: "accounting", "Calculating PnL for long position");
//         (price - entry_price) * amount_closed
//     } else if current_position < Decimal::ZERO {
//         info!(target: "accounting", "Calculating PnL for short position");
//         (entry_price - price) * amount_closed
//     } else {
//         info!(target: "accounting", "No PnL for flat position");
//         dec!(0)
//     };

//     // Margin adjustments
//     let margin_delta = if amount_closed > Decimal::ZERO && current_position.signum() == new_position.signum() {
//         let current_margin = self.ledger.margin_posted(&strategy, Some(&instrument)).await;
//         let closing_margin = current_margin * (amount_closed / current_position.abs());
//         -closing_margin
//     } else if amount_closed.is_zero()
//         && (current_position.signum() == new_position.signum() || current_position.is_zero())
//     {
//         let posting = new_position.abs() * price * margin_rate;
//         posting
//     } else {
//         let posting = new_position.abs() * price * margin_rate;
//         let current_margin = self.ledger.margin_posted(&strategy, Some(&instrument)).await;
//         let closing_margin = current_margin * (amount_closed / current_position.abs());
//         posting - closing_margin
//     };
//     info!(target: "accounting", "Margin delta: {}", margin_delta);

//     //  Calculate commission
//     let commission = amount * price * commission_rate;
//     info!(target: "accounting", "Commission: {}", commission);

//     // Step 7: Create transfers
//     let transfer_group_id = Uuid::new_v4();
//     let mut transfers = Vec::new();

//     // Margin adjustment
//     if margin_delta > dec!(0) {
//         // Post additional margin
//         transfers.push(Arc::new(
//             Transfer::builder()
//                 .event_time(event_time)
//                 .transfer_group_id(transfer_group_id)
//                 .asset(user_margin.asset.clone())
//                 .strategy(Some(strategy.clone()))
//                 .instrument(Some(instrument.clone()))
//                 .debit_account(user_margin.clone())
//                 .credit_account(venue_margin.clone())
//                 .amount(margin_delta)
//                 .unit_price(Decimal::ONE)
//                 .transfer_type(TransferType::Margin)
//                 .build(),
//         ));
//     } else if margin_delta < dec!(0) {
//         // Free margin
//         transfers.push(Arc::new(
//             Transfer::builder()
//                 .event_time(event_time)
//                 .transfer_group_id(transfer_group_id)
//                 .asset(venue_margin.asset.clone())
//                 .strategy(Some(strategy.clone()))
//                 .instrument(Some(instrument.clone()))
//                 .debit_account(venue_margin.clone())
//                 .credit_account(user_margin.clone())
//                 .amount(margin_delta.abs())
//                 .unit_price(Decimal::ONE)
//                 .transfer_type(TransferType::Margin)
//                 .build(),
//         ));
//     }

//     // Commission payment
//     transfers.push(Arc::new(
//         Transfer::builder()
//             .event_time(event_time)
//             .transfer_group_id(transfer_group_id)
//             .asset(user_margin.asset.clone())
//             .strategy(Some(strategy.clone()))
//             .instrument(Some(instrument.clone()))
//             .debit_account(user_margin.clone())
//             .credit_account(venue_spot.clone())
//             .amount(commission)
//             .unit_price(Decimal::ONE)
//             .transfer_type(TransferType::Commission)
//             .build(),
//     ));

//     // Instrument trade
//     let (debit_inst, credit_inst) = if side == MarketSide::Buy {
//         (venue_inst.clone(), user_inst.clone())
//     } else {
//         (user_inst.clone(), venue_inst.clone())
//     };
//     transfers.push(Arc::new(
//         Transfer::builder()
//             .event_time(event_time)
//             .transfer_group_id(transfer_group_id)
//             .asset(debit_inst.asset.clone())
//             .strategy(Some(strategy.clone()))
//             .instrument(Some(instrument.clone()))
//             .debit_account(debit_inst)
//             .credit_account(credit_inst)
//             .amount(amount)
//             .unit_price(price)
//             .transfer_type(TransferType::Trade)
//             .build(),
//     ));

//     // PnL transfer
//     if amount_closed > dec!(0) {
//         if pnl > Decimal::ZERO {
//             // Profit: venue_spot -> user_margin
//             transfers.push(Arc::new(
//                 Transfer::builder()
//                     .event_time(event_time)
//                     .transfer_group_id(transfer_group_id)
//                     .asset(venue_spot.asset.clone())
//                     .strategy(Some(strategy.clone()))
//                     .instrument(Some(instrument.clone()))
//                     .debit_account(venue_spot.clone())
//                     .credit_account(user_margin.clone())
//                     .amount(pnl)
//                     .unit_price(Decimal::ONE)
//                     .transfer_type(TransferType::Pnl)
//                     .build(),
//             ));
//         } else if pnl < dec!(0) {
//             // Loss: user_margin -> venue_spot
//             transfers.push(Arc::new(
//                 Transfer::builder()
//                     .event_time(event_time)
//                     .transfer_group_id(transfer_group_id)
//                     .asset(user_margin.asset.clone())
//                     .strategy(Some(strategy.clone()))
//                     .instrument(Some(instrument.clone()))
//                     .debit_account(user_margin.clone())
//                     .credit_account(venue_spot.clone())
//                     .amount(pnl.abs())
//                     .unit_price(Decimal::ONE)
//                     .transfer_type(TransferType::Pnl)
//                     .build(),
//             ));
//         }
//     }

//     // Apply transfers
//     let res = self.ledger.apply_transfers(&transfers).await;
//     match res {
//         Ok(transfers) => {
//             info!(target: "accounting", "Transfers applied successfully");
//         }
//         Err(e) => {
//             error!(target: "accounting", "Failed to apply transfers: {}", e);
//         }
//     }
//     // self.publisher.publish(Event::TransferNew(transfers)).await;
// }
