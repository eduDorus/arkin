#![allow(dead_code)]
use std::sync::Arc;

use arkin_core::prelude::*;
use async_trait::async_trait;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use tracing::{error, info, instrument, warn};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(TypedBuilder)]
pub struct Accounting {
    #[builder(default = String::from("accounting"))]
    identifier: String,
    time: Arc<dyn SystemTime>,
    publisher: Arc<dyn Publisher>,
    #[builder(default = Ledger::new())]
    ledger: Arc<Ledger>,
}

impl Accounting {
    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn initial_account_update(&self, update: &AccountUpdate) {
        info!(target: "accounting", "received initial account update");

        let event_time = self.time.now().await;
        let transfer_group_id = Uuid::new_v4();
        let mut transfers = Vec::new();

        // Apply balances first
        for bal in &update.balances {
            let venue = bal.venue.to_owned();
            let asset = Tradable::Asset(bal.asset.clone());
            let user_account = self.ledger.find_or_create_account(
                &venue,
                &asset,
                AccountOwner::User,
                bal.account_type,
                self.time.now().await,
            );
            let current_balance = self.ledger.balance(user_account.id).await;
            let diff = bal.quantity - current_balance;
            if diff == Decimal::ZERO {
                continue;
            }
            let obe = self.ledger.find_or_create_account(
                &venue,
                &asset,
                AccountOwner::Venue,
                AccountType::Equity,
                self.time.now().await,
            );
            let (debit, credit, amount) = if diff > Decimal::ZERO {
                (obe, user_account, diff)
            } else {
                (user_account, obe, -diff)
            };
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(asset)
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(Decimal::ONE)
                    .transfer_type(TransferType::Initial)
                    .build()
                    .into(),
            );
        }

        // Then positions
        for pos in &update.positions {
            let venue = pos.instrument.venue.to_owned();
            let instrument_asset = Tradable::Asset(pos.instrument.base_asset.clone());
            let quote_asset = Tradable::Asset(pos.instrument.quote_asset.clone());
            let position_account = self.ledger.find_or_create_account(
                &venue,
                &instrument_asset,
                AccountOwner::User,
                AccountType::Instrument,
                self.time.now().await,
            );
            let qty_diff = pos.quantity - self.ledger.balance(position_account.id).await;
            let realized_pnl_account = self.ledger.find_or_create_account(
                &venue,
                &quote_asset,
                AccountOwner::User,
                AccountType::Margin,
                self.time.now().await,
            );
            let realized_diff = pos.realized_pnl - self.ledger.balance(realized_pnl_account.id).await;
            let unrealized_pnl_account = self.ledger.find_or_create_account(
                &venue,
                &quote_asset,
                AccountOwner::User,
                AccountType::Equity,
                self.time.now().await,
            );
            let unrealized_diff = pos.unrealized_pnl - self.ledger.balance(unrealized_pnl_account.id).await;

            if qty_diff != Decimal::ZERO {
                let venue_position = self.ledger.find_or_create_account(
                    &venue,
                    &instrument_asset,
                    AccountOwner::Venue,
                    AccountType::Instrument,
                    self.time.now().await,
                );
                let (debit, credit, amount) = if qty_diff > Decimal::ZERO {
                    (venue_position.clone(), position_account.clone(), qty_diff)
                } else {
                    (position_account.clone(), venue_position.clone(), -qty_diff)
                };
                transfers.push(
                    Transfer::builder()
                        .event_time(event_time)
                        .transfer_group_id(transfer_group_id)
                        .asset(instrument_asset.clone())
                        .debit_account(debit)
                        .credit_account(credit)
                        .amount(amount)
                        .unit_price(pos.entry_price)
                        .transfer_type(TransferType::Initial)
                        .build()
                        .into(),
                );
            }

            if realized_diff != Decimal::ZERO {
                let obe = self.ledger.find_or_create_account(
                    &venue,
                    &quote_asset,
                    AccountOwner::Venue,
                    AccountType::Equity,
                    self.time.now().await,
                );
                let (debit, credit, amount) = if realized_diff > Decimal::ZERO {
                    (obe.clone(), realized_pnl_account.clone(), realized_diff)
                } else {
                    (realized_pnl_account.clone(), obe.clone(), -realized_diff)
                };
                transfers.push(
                    Transfer::builder()
                        .event_time(event_time)
                        .transfer_group_id(transfer_group_id)
                        .asset(quote_asset.clone())
                        .debit_account(debit)
                        .credit_account(credit)
                        .amount(amount)
                        .unit_price(Decimal::ONE)
                        .transfer_type(TransferType::Initial)
                        .build()
                        .into(),
                );
            }

            if unrealized_diff != Decimal::ZERO {
                let obe = self.ledger.find_or_create_account(
                    &venue,
                    &quote_asset,
                    AccountOwner::Venue,
                    AccountType::Equity,
                    self.time.now().await,
                );
                let (debit, credit, amount) = if unrealized_diff > Decimal::ZERO {
                    (obe.clone(), unrealized_pnl_account.clone(), unrealized_diff)
                } else {
                    (unrealized_pnl_account.clone(), obe.clone(), -unrealized_diff)
                };
                transfers.push(
                    Transfer::builder()
                        .event_time(event_time)
                        .transfer_group_id(transfer_group_id)
                        .asset(quote_asset.clone())
                        .debit_account(debit)
                        .credit_account(credit)
                        .amount(amount)
                        .unit_price(Decimal::ONE)
                        .transfer_type(TransferType::Initial)
                        .build()
                        .into(),
                );
            }
        }

        if !transfers.is_empty() {
            let res = self.ledger.apply_transfers(&transfers).await;
            match res {
                Ok(_) => info!(target: "accounting", "Initial account update applied"),
                Err(e) => error!(target: "accounting", "Failed initial account update: {}", e),
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn initial_balance_update(&self, update: &BalanceUpdate) {
        info!(target: "accounting", "received initial balance update");

        let venue = update.venue.to_owned();
        let asset = Tradable::Asset(update.asset.clone());

        let user_account = self.ledger.find_or_create_account(
            &venue,
            &asset,
            AccountOwner::User,
            update.account_type,
            self.time.now().await,
        );

        let current_balance = self.ledger.balance(user_account.id).await;
        let diff = update.quantity - current_balance;

        if diff == Decimal::ZERO {
            info!(target: "accounting", "Initial balance already set");
            return;
        }

        let obe_account = self.ledger.find_or_create_account(
            &venue,
            &asset,
            AccountOwner::Venue,
            AccountType::Equity,
            self.time.now().await,
        );

        let event_time = self.time.now().await;
        let transfer_group_id = Uuid::new_v4();

        let (debit_account, credit_account, amount) = if diff > Decimal::ZERO {
            (obe_account.clone(), user_account.clone(), diff)
        } else {
            (user_account.clone(), obe_account.clone(), -diff)
        };

        let transfer = Transfer::builder()
            .event_time(event_time)
            .transfer_group_id(transfer_group_id)
            .asset(asset)
            .debit_account(debit_account)
            .credit_account(credit_account)
            .amount(amount)
            .unit_price(Decimal::ONE)
            .transfer_type(TransferType::Initial)
            .build()
            .into();

        let res = self.ledger.apply_transfers(&[transfer]).await;
        match res {
            Ok(_) => info!(target: "accounting", "Initial balance set: {}", update.quantity),
            Err(e) => error!(target: "accounting", "Failed initial balance: {}", e),
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn initial_position_update(&self, update: &PositionUpdate) {
        info!(target: "accounting", "received initial position update");

        let venue = update.instrument.venue.to_owned(); // Assume added to PositionUpdate
        let instrument_asset = Tradable::Asset(update.instrument.base_asset.clone()); // Base for quantity
        let quote_asset = Tradable::Asset(update.instrument.quote_asset.clone()); // For PNL (assume USDT)

        // Position quantity account
        let position_account = self.ledger.find_or_create_account(
            &venue,
            &instrument_asset,
            AccountOwner::User,
            AccountType::Instrument,
            self.time.now().await,
        );

        let current_quantity = self.ledger.balance(position_account.id).await; // Assume balance tracks qty
        let qty_diff = update.quantity - current_quantity;

        // Realized PNL equity
        let realized_pnl_account = self.ledger.find_or_create_account(
            &venue,
            &quote_asset,
            AccountOwner::User,
            AccountType::Margin, // Name it "RealizedPNL" via extension if needed
            self.time.now().await,
        );

        // Unrealized PNL equity
        let unrealized_pnl_account = self.ledger.find_or_create_account(
            &venue,
            &quote_asset,
            AccountOwner::User,
            AccountType::Equity, // "UnrealizedPNL"
            self.time.now().await,
        );

        let event_time = self.time.now().await;
        let transfer_group_id = Uuid::new_v4();
        let mut transfers = Vec::new();

        // Quantity diff (use venue counterpart for position entry)
        if qty_diff != Decimal::ZERO {
            let venue_position_account = self.ledger.find_or_create_account(
                &venue,
                &instrument_asset,
                AccountOwner::Venue,
                AccountType::Instrument,
                self.time.now().await,
            );
            let (debit, credit, amount) = if qty_diff > Decimal::ZERO {
                (venue_position_account.clone(), position_account.clone(), qty_diff)
            } else {
                (position_account.clone(), venue_position_account.clone(), -qty_diff)
            };
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(instrument_asset.clone())
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(update.entry_price) // Use entry for cost basis
                    .transfer_type(TransferType::Initial)
                    .build()
                    .into(),
            );
        }

        // Realized PNL (assume no current; full set)
        if update.realized_pnl != Decimal::ZERO {
            let obe_account = self.ledger.find_or_create_account(
                &venue,
                &quote_asset,
                AccountOwner::Venue,
                AccountType::Equity,
                self.time.now().await,
            );
            let (debit, credit, amount) = if update.realized_pnl > Decimal::ZERO {
                (obe_account.clone(), realized_pnl_account.clone(), update.realized_pnl)
            } else {
                (realized_pnl_account.clone(), obe_account.clone(), -update.realized_pnl)
            };
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(quote_asset.clone())
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(Decimal::ONE)
                    .transfer_type(TransferType::Initial)
                    .build()
                    .into(),
            );
        }

        // Unrealized PNL (similar)
        if update.unrealized_pnl != Decimal::ZERO {
            let obe_account = self.ledger.find_or_create_account(
                &venue,
                &quote_asset,
                AccountOwner::Venue,
                AccountType::Equity,
                self.time.now().await,
            );
            let (debit, credit, amount) = if update.unrealized_pnl > Decimal::ZERO {
                (obe_account.clone(), unrealized_pnl_account.clone(), update.unrealized_pnl)
            } else {
                (unrealized_pnl_account.clone(), obe_account.clone(), -update.unrealized_pnl)
            };
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(quote_asset)
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(Decimal::ONE)
                    .transfer_type(TransferType::Initial)
                    .build()
                    .into(),
            );
        }

        if !transfers.is_empty() {
            let res = self.ledger.apply_transfers(&transfers).await;
            match res {
                Ok(_) => info!(target: "accounting", "Initial position set"),
                Err(e) => error!(target: "accounting", "Failed initial position: {}", e),
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn reconcile_account_update(&self, update: &AccountUpdate) {
        info!(target: "accounting", "received reconcile account update");

        let event_time = self.time.now().await;
        let transfer_group_id = Uuid::new_v4();
        let mut transfers = Vec::new();
        let mut diffs = Vec::new(); // To collect diffs for warnings

        // Apply balances first
        for bal in &update.balances {
            let venue = bal.venue.to_owned();
            let asset = Tradable::Asset(bal.asset.clone());
            let user_account = self.ledger.find_or_create_account(
                &venue,
                &asset,
                AccountOwner::User,
                bal.account_type,
                self.time.now().await,
            );
            let current_balance = self.ledger.balance(user_account.id).await;
            let diff = bal.quantity - current_balance;
            diffs.push((asset.clone(), diff, current_balance));
            if diff == Decimal::ZERO {
                continue;
            }
            let recon = self.ledger.find_or_create_account(
                &venue,
                &asset,
                AccountOwner::Venue,
                AccountType::Equity,
                self.time.now().await,
            );

            let (debit, credit, amount) = if diff > Decimal::ZERO {
                (recon, user_account, diff)
            } else {
                (user_account, recon, -diff)
            };
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(asset)
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(Decimal::ONE)
                    .transfer_type(TransferType::Reconciliation)
                    .build()
                    .into(),
            );
        }

        // Then positions
        for pos in &update.positions {
            let venue = pos.instrument.venue.to_owned();
            let instrument_asset = Tradable::Instrument(pos.instrument.clone());
            let quote_asset = Tradable::Asset(pos.instrument.quote_asset.clone());
            let position_account = self.ledger.find_or_create_account(
                &venue,
                &instrument_asset,
                AccountOwner::User,
                AccountType::Instrument,
                self.time.now().await,
            );
            let qty_diff = pos.quantity - self.ledger.balance(position_account.id).await;
            let realized_pnl_account = self.ledger.find_or_create_account(
                &venue,
                &quote_asset,
                AccountOwner::User,
                AccountType::Margin,
                self.time.now().await,
            );
            let realized_diff = pos.realized_pnl;
            let unrealized_pnl_account = self.ledger.find_or_create_account(
                &venue,
                &quote_asset,
                AccountOwner::User,
                AccountType::Equity,
                self.time.now().await,
            );
            let unrealized_diff = pos.unrealized_pnl;

            diffs.push((
                instrument_asset.clone(),
                qty_diff,
                self.ledger.balance(position_account.id).await,
            ));
            diffs.push((
                quote_asset.clone(),
                realized_diff,
                self.ledger.balance(realized_pnl_account.id).await,
            ));
            diffs.push((
                quote_asset.clone(),
                unrealized_diff,
                self.ledger.balance(unrealized_pnl_account.id).await,
            ));

            if qty_diff != Decimal::ZERO {
                let venue_position = self.ledger.find_or_create_account(
                    &venue,
                    &instrument_asset,
                    AccountOwner::Venue,
                    AccountType::Instrument,
                    self.time.now().await,
                );
                let (debit, credit, amount) = if qty_diff > Decimal::ZERO {
                    (venue_position.clone(), position_account.clone(), qty_diff)
                } else {
                    (position_account.clone(), venue_position.clone(), -qty_diff)
                };
                transfers.push(
                    Transfer::builder()
                        .event_time(event_time)
                        .transfer_group_id(transfer_group_id)
                        .asset(instrument_asset.clone())
                        .debit_account(debit)
                        .credit_account(credit)
                        .amount(amount)
                        .unit_price(pos.entry_price)
                        .transfer_type(TransferType::Reconciliation)
                        .build()
                        .into(),
                );
            }

            if realized_diff != Decimal::ZERO {
                let recon = self.ledger.find_or_create_account(
                    &venue,
                    &quote_asset,
                    AccountOwner::Venue,
                    AccountType::Equity,
                    self.time.now().await,
                );
                let (debit, credit, amount) = if realized_diff > Decimal::ZERO {
                    (recon.clone(), realized_pnl_account.clone(), realized_diff)
                } else {
                    (realized_pnl_account.clone(), recon.clone(), -realized_diff)
                };
                transfers.push(
                    Transfer::builder()
                        .event_time(event_time)
                        .transfer_group_id(transfer_group_id)
                        .asset(quote_asset.clone())
                        .debit_account(debit)
                        .credit_account(credit)
                        .amount(amount)
                        .unit_price(Decimal::ONE)
                        .transfer_type(TransferType::Reconciliation)
                        .build()
                        .into(),
                );
            }

            if unrealized_diff != Decimal::ZERO {
                let recon = self.ledger.find_or_create_account(
                    &venue,
                    &quote_asset,
                    AccountOwner::Venue,
                    AccountType::Equity,
                    self.time.now().await,
                );
                let (debit, credit, amount) = if unrealized_diff > Decimal::ZERO {
                    (recon.clone(), unrealized_pnl_account.clone(), unrealized_diff)
                } else {
                    (unrealized_pnl_account.clone(), recon.clone(), -unrealized_diff)
                };
                transfers.push(
                    Transfer::builder()
                        .event_time(event_time)
                        .transfer_group_id(transfer_group_id)
                        .asset(quote_asset.clone())
                        .debit_account(debit)
                        .credit_account(credit)
                        .amount(amount)
                        .unit_price(Decimal::ONE)
                        .transfer_type(TransferType::Reconciliation)
                        .build()
                        .into(),
                );
            }
        }

        if !transfers.is_empty() {
            let res = self.ledger.apply_transfers(&transfers).await;
            match res {
                Ok(_) => {
                    info!(target: "accounting", "Reconcile account update applied");
                    for (asset, diff, current) in diffs {
                        let threshold = current.abs() * dec!(0.0001);
                        if diff.abs() > threshold {
                            warn!(target: "accounting", "Large recon diff for {}: {} (threshold {})", asset, diff, threshold);
                        }
                    }
                }
                Err(e) => error!(target: "accounting", "Failed recon account update: {}", e),
            }
        } else {
            info!(target: "accounting", "Account in sync");
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn reconcile_balance_update(&self, update: &BalanceUpdate) {
        info!(target: "accounting", "received reconcile balance update");

        let venue = update.venue.to_owned();
        let asset = Tradable::Asset(update.asset.clone());

        let user_account = self.ledger.find_or_create_account(
            &venue,
            &asset,
            AccountOwner::User,
            update.account_type,
            self.time.now().await,
        );

        let current_balance = self.ledger.balance(user_account.id).await;
        let diff = update.quantity - current_balance;

        if diff == Decimal::ZERO {
            info!(target: "accounting", "Balance in sync");
            return;
        }

        let recon_account = self.ledger.find_or_create_account(
            &venue,
            &asset,
            AccountOwner::Venue,
            AccountType::Equity,
            self.time.now().await,
        );

        let event_time = self.time.now().await;
        let transfer_group_id = Uuid::new_v4();

        let (debit_account, credit_account, amount) = if diff > Decimal::ZERO {
            (recon_account.clone(), user_account.clone(), diff)
        } else {
            (user_account.clone(), recon_account.clone(), -diff)
        };

        let transfer = Transfer::builder()
            .event_time(event_time)
            .transfer_group_id(transfer_group_id)
            .asset(asset)
            .debit_account(debit_account)
            .credit_account(credit_account)
            .amount(amount)
            .unit_price(Decimal::ONE)
            .transfer_type(TransferType::Reconciliation)
            .build()
            .into();

        let res = self.ledger.apply_transfers(&[transfer]).await;
        match res {
            Ok(_) => {
                info!(target: "accounting", "Reconciliation applied: diff {}", diff);
                let threshold = current_balance * dec!(0.0001);
                if diff.abs() > threshold {
                    warn!(target: "accounting", "Large reconciliation diff: {} (threshold {})", diff, threshold);
                }
            }
            Err(e) => error!(target: "accounting", "Failed reconciliation: {}", e),
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn reconcile_position_update(&self, update: &PositionUpdate) {
        info!(target: "accounting", "received reconcile position update");

        let venue = update.instrument.venue.to_owned();
        let instrument_asset = Tradable::Asset(update.instrument.base_asset.clone());
        let quote_asset = Tradable::Asset(update.instrument.quote_asset.clone());

        let position_account = self.ledger.find_or_create_account(
            &venue,
            &instrument_asset,
            AccountOwner::User,
            AccountType::Instrument,
            self.time.now().await,
        );

        let current_quantity = self.ledger.balance(position_account.id).await;
        let qty_diff = update.quantity - current_quantity;

        let realized_pnl_account = self.ledger.find_or_create_account(
            &venue,
            &quote_asset,
            AccountOwner::User,
            AccountType::Equity,
            self.time.now().await,
        );

        let current_realized = self.ledger.balance(realized_pnl_account.id).await; // Assume tracks cumulative
        let realized_diff = update.realized_pnl - current_realized;

        let unrealized_pnl_account = self.ledger.find_or_create_account(
            &venue,
            &quote_asset,
            AccountOwner::User,
            AccountType::Equity,
            self.time.now().await,
        );

        let current_unrealized = self.ledger.balance(unrealized_pnl_account.id).await;
        let unrealized_diff = update.unrealized_pnl - current_unrealized;

        let event_time = self.time.now().await;
        let transfer_group_id = Uuid::new_v4();
        let mut transfers = Vec::new();

        if qty_diff != Decimal::ZERO {
            let venue_position_account = self.ledger.find_or_create_account(
                &venue,
                &instrument_asset,
                AccountOwner::Venue,
                AccountType::Instrument,
                self.time.now().await,
            );
            let (debit, credit, amount) = if qty_diff > Decimal::ZERO {
                (venue_position_account.clone(), position_account.clone(), qty_diff)
            } else {
                (position_account.clone(), venue_position_account.clone(), -qty_diff)
            };
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(instrument_asset.clone())
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(update.entry_price)
                    .transfer_type(TransferType::Reconciliation)
                    .build()
                    .into(),
            );
        }

        if realized_diff != Decimal::ZERO {
            let recon_account = self.ledger.find_or_create_account(
                &venue,
                &quote_asset,
                AccountOwner::Venue,
                AccountType::Equity,
                self.time.now().await,
            );
            let (debit, credit, amount) = if realized_diff > Decimal::ZERO {
                (recon_account.clone(), realized_pnl_account.clone(), realized_diff)
            } else {
                (realized_pnl_account.clone(), recon_account.clone(), -realized_diff)
            };
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(quote_asset.clone())
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(Decimal::ONE)
                    .transfer_type(TransferType::Reconciliation)
                    .build()
                    .into(),
            );
        }

        if unrealized_diff != Decimal::ZERO {
            let recon_account = self.ledger.find_or_create_account(
                &venue,
                &quote_asset,
                AccountOwner::Venue,
                AccountType::Equity,
                self.time.now().await,
            );
            let (debit, credit, amount) = if unrealized_diff > Decimal::ZERO {
                (recon_account.clone(), unrealized_pnl_account.clone(), unrealized_diff)
            } else {
                (unrealized_pnl_account.clone(), recon_account.clone(), -unrealized_diff)
            };
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(quote_asset)
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(Decimal::ONE)
                    .transfer_type(TransferType::Reconciliation)
                    .build()
                    .into(),
            );
        }

        if !transfers.is_empty() {
            let res = self.ledger.apply_transfers(&transfers).await;
            match res {
                Ok(_) => info!(target: "accounting", "Position reconciled"),
                Err(e) => error!(target: "accounting", "Failed position reconciliation: {}", e),
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn account_update(&self, update: &AccountUpdate) {
        info!(target: "accounting", "received account update: reason {}", update.reason);

        let event_time = self.time.now().await;
        let transfer_group_id = Uuid::new_v4();
        let mut transfers = Vec::new();

        // Apply balances first
        for bal in &update.balances {
            // Mirror balance_update logic, but collect transfers
            if bal.quantity_change == Decimal::ZERO {
                info!(target: "accounting", "received balance update with no change");
                continue;
            }
            let venue = bal.venue.to_owned();
            let asset = Tradable::Asset(bal.asset.clone());
            let user_account = self.ledger.find_or_create_account(
                &venue,
                &asset,
                AccountOwner::User,
                bal.account_type,
                self.time.now().await,
            );
            let venue_funding = self.ledger.find_or_create_account(
                &venue,
                &asset,
                AccountOwner::Venue,
                AccountType::Equity,
                self.time.now().await,
            );
            let (debit, credit, amount) = if bal.quantity_change > Decimal::ZERO {
                (venue_funding, user_account, bal.quantity_change)
            } else {
                (user_account, venue_funding, -bal.quantity_change)
            };
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(asset)
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(Decimal::ONE)
                    .transfer_type(TransferType::Trade) // TODO: Probably depending on update reason
                    .build()
                    .into(),
            );
        }

        // Then positions
        for pos in &update.positions {
            // Mirror position_update logic, collect transfers
            let venue = pos.instrument.venue.to_owned();
            let instrument_asset = Tradable::Instrument(pos.instrument.clone());
            let quote_asset = Tradable::Asset(pos.instrument.quote_asset.clone());
            let position_account = self.ledger.find_or_create_account(
                &venue,
                &instrument_asset,
                AccountOwner::User,
                AccountType::Instrument,
                self.time.now().await,
            );
            let qty_diff = pos.quantity - self.ledger.balance(position_account.id).await;
            let realized_pnl_account = self.ledger.find_or_create_account(
                &venue,
                &quote_asset,
                AccountOwner::User,
                AccountType::Margin,
                self.time.now().await,
            );
            let realized_diff = pos.realized_pnl;
            let unrealized_pnl_account = self.ledger.find_or_create_account(
                &venue,
                &quote_asset,
                AccountOwner::User,
                AccountType::Equity,
                self.time.now().await,
            );
            let unrealized_diff = pos.unrealized_pnl;

            if qty_diff != Decimal::ZERO {
                let venue_position = self.ledger.find_or_create_account(
                    &venue,
                    &instrument_asset,
                    AccountOwner::Venue,
                    AccountType::Instrument,
                    self.time.now().await,
                );
                let (debit, credit, amount) = if qty_diff > Decimal::ZERO {
                    (venue_position.clone(), position_account.clone(), qty_diff)
                } else {
                    (position_account.clone(), venue_position.clone(), -qty_diff)
                };
                transfers.push(
                    Transfer::builder()
                        .event_time(event_time)
                        .transfer_group_id(transfer_group_id)
                        .asset(instrument_asset)
                        .debit_account(debit)
                        .credit_account(credit)
                        .amount(amount)
                        .unit_price(pos.entry_price)
                        .transfer_type(TransferType::Trade)
                        .build()
                        .into(),
                );
            }

            if realized_diff != Decimal::ZERO {
                let funding = self.ledger.find_or_create_account(
                    &venue,
                    &quote_asset,
                    AccountOwner::Venue,
                    AccountType::Equity,
                    self.time.now().await,
                );
                let (debit, credit, amount) = if realized_diff > Decimal::ZERO {
                    (funding.clone(), realized_pnl_account.clone(), realized_diff)
                } else {
                    (realized_pnl_account.clone(), funding.clone(), -realized_diff)
                };
                transfers.push(
                    Transfer::builder()
                        .event_time(event_time)
                        .transfer_group_id(transfer_group_id)
                        .asset(quote_asset.clone())
                        .debit_account(debit)
                        .credit_account(credit)
                        .amount(amount)
                        .unit_price(Decimal::ONE)
                        .transfer_type(TransferType::Pnl)
                        .build()
                        .into(),
                );
            }

            if unrealized_diff != Decimal::ZERO {
                let funding = self.ledger.find_or_create_account(
                    &venue,
                    &quote_asset,
                    AccountOwner::Venue,
                    AccountType::Equity,
                    self.time.now().await,
                );
                let (debit, credit, amount) = if unrealized_diff > Decimal::ZERO {
                    (funding.clone(), unrealized_pnl_account.clone(), unrealized_diff)
                } else {
                    (unrealized_pnl_account.clone(), funding.clone(), -unrealized_diff)
                };
                transfers.push(
                    Transfer::builder()
                        .event_time(event_time)
                        .transfer_group_id(transfer_group_id)
                        .asset(quote_asset)
                        .debit_account(debit)
                        .credit_account(credit)
                        .amount(amount)
                        .unit_price(Decimal::ONE)
                        .transfer_type(TransferType::UnrealizedPNL)
                        .build()
                        .into(),
                );
            }
        }

        if !transfers.is_empty() {
            let res = self.ledger.apply_transfers(&transfers).await;
            match res {
                Ok(_) => info!(target: "accounting", "Account update applied"),
                Err(e) => error!(target: "accounting", "Failed account update: {}", e),
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn balance_update(&self, update: &BalanceUpdate) {
        info!(target: "accounting", "received balance update");

        if update.quantity_change == Decimal::ZERO {
            info!(target: "accounting", "received balance update with no change");
            return;
        }

        let venue = update.venue.to_owned();
        let asset = Tradable::Asset(update.asset.clone());

        let user_account = self.ledger.find_or_create_account(
            &venue,
            &asset,
            AccountOwner::User,
            update.account_type,
            self.time.now().await,
        );

        let venue_funding_account = self.ledger.find_or_create_account(
            &venue,
            &asset,
            AccountOwner::Venue,
            AccountType::Equity,
            self.time.now().await,
        );

        let event_time = self.time.now().await;
        let transfer_group_id = Uuid::new_v4();

        let (debit_account, credit_account, amount) = if update.quantity_change > Decimal::ZERO {
            (venue_funding_account, user_account, update.quantity_change)
        } else {
            (user_account, venue_funding_account, -update.quantity_change)
        };

        let transfer = Transfer::builder()
            .event_time(event_time)
            .transfer_group_id(transfer_group_id)
            .asset(asset)
            .debit_account(debit_account)
            .credit_account(credit_account)
            .amount(amount)
            .unit_price(Decimal::ONE)
            .transfer_type(TransferType::Adjustment)
            .build()
            .into();

        let res = self.ledger.apply_transfers(&[transfer]).await;
        match res {
            Ok(_) => info!(target: "accounting", "Balance adjustment applied"),
            Err(e) => error!(target: "accounting", "Failed balance adjustment: {}", e),
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn position_update(&self, update: &PositionUpdate) {
        info!(target: "accounting", "received position update");

        let venue = update.instrument.venue.to_owned();
        let instrument_asset = Tradable::Asset(update.instrument.base_asset.clone());
        let quote_asset = Tradable::Asset(update.instrument.quote_asset.clone());

        let position_account = self.ledger.find_or_create_account(
            &venue,
            &instrument_asset,
            AccountOwner::User,
            AccountType::Instrument,
            self.time.now().await,
        );

        let current_quantity = self.ledger.balance(position_account.id).await;
        let qty_diff = update.quantity - current_quantity;

        let realized_pnl_account = self.ledger.find_or_create_account(
            &venue,
            &quote_asset,
            AccountOwner::User,
            AccountType::Margin,
            self.time.now().await,
        );

        let current_realized = self.ledger.balance(realized_pnl_account.id).await;
        let realized_diff = update.realized_pnl - current_realized;

        let unrealized_pnl_account = self.ledger.find_or_create_account(
            &venue,
            &quote_asset,
            AccountOwner::User,
            AccountType::Equity,
            self.time.now().await,
        );

        let current_unrealized = self.ledger.balance(unrealized_pnl_account.id).await;
        let unrealized_diff = update.unrealized_pnl - current_unrealized;

        let event_time = self.time.now().await;
        let transfer_group_id = Uuid::new_v4();
        let mut transfers = Vec::new();

        if qty_diff != Decimal::ZERO {
            let venue_position_account = self.ledger.find_or_create_account(
                &venue,
                &instrument_asset,
                AccountOwner::Venue,
                AccountType::Instrument,
                self.time.now().await,
            );

            let (debit, credit, amount) = if qty_diff > Decimal::ZERO {
                (venue_position_account.clone(), position_account.clone(), qty_diff)
            } else {
                (position_account.clone(), venue_position_account.clone(), -qty_diff)
            };
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(instrument_asset.clone())
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(update.entry_price)
                    .transfer_type(TransferType::Adjustment)
                    .build()
                    .into(),
            );
        }

        if realized_diff != Decimal::ZERO {
            let funding_account = self.ledger.find_or_create_account(
                &venue,
                &quote_asset,
                AccountOwner::Venue,
                AccountType::Equity,
                self.time.now().await,
            );
            let (debit, credit, amount) = if realized_diff > Decimal::ZERO {
                (funding_account.clone(), realized_pnl_account.clone(), realized_diff)
            } else {
                (realized_pnl_account.clone(), funding_account.clone(), -realized_diff)
            };
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(quote_asset.clone())
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(Decimal::ONE)
                    .transfer_type(TransferType::Pnl)
                    .build()
                    .into(),
            );
        }

        if unrealized_diff != Decimal::ZERO {
            let funding_account = self.ledger.find_or_create_account(
                &venue,
                &quote_asset,
                AccountOwner::Venue,
                AccountType::Equity,
                self.time.now().await,
            );
            let (debit, credit, amount) = if unrealized_diff > Decimal::ZERO {
                (funding_account.clone(), unrealized_pnl_account.clone(), unrealized_diff)
            } else {
                (unrealized_pnl_account.clone(), funding_account.clone(), -unrealized_diff)
            };
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(quote_asset)
                    .debit_account(debit)
                    .credit_account(credit)
                    .amount(amount)
                    .unit_price(Decimal::ONE)
                    .transfer_type(TransferType::Pnl)
                    .build()
                    .into(),
            );
        }

        if !transfers.is_empty() {
            let res = self.ledger.apply_transfers(&transfers).await;
            match res {
                Ok(_) => info!(target: "accounting", "Position update applied"),
                Err(e) => error!(target: "accounting", "Failed position update: {}", e),
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn venue_order_fill(&self, order: &VenueOrder) {
        let transfer_group_id = Uuid::new_v4();
        let event_time = self.time.now().await;

        let venue = order.instrument.venue.clone();
        let (credit_asset, debit_asset) = if order.side == MarketSide::Buy {
            (
                Tradable::Asset(order.instrument.base_asset.clone()),
                Tradable::Asset(order.instrument.quote_asset.clone()),
            )
        } else {
            (
                Tradable::Asset(order.instrument.quote_asset.clone()),
                Tradable::Asset(order.instrument.base_asset.clone()),
            )
        };

        let (credit_amount, debit_amount) = if order.side == MarketSide::Buy {
            (order.quantity, order.quantity * order.price)
        } else {
            (order.quantity * order.price, order.quantity)
        };

        let mut transfers = vec![];

        // Debit transfer
        let debit_account = self.ledger.find_or_create_account(
            &venue,
            &debit_asset,
            AccountOwner::User,
            AccountType::Spot,
            self.time.now().await,
        );
        let credit_account = self.ledger.find_or_create_account(
            &venue,
            &debit_asset,
            AccountOwner::Venue,
            AccountType::Spot,
            self.time.now().await,
        );

        transfers.push(
            Transfer::builder()
                .event_time(event_time)
                .transfer_group_id(transfer_group_id)
                .asset(debit_asset.clone())
                .debit_account(debit_account)
                .credit_account(credit_account)
                .amount(debit_amount)
                .unit_price(Decimal::ONE)
                .transfer_type(TransferType::Exchange)
                .build()
                .into(),
        );

        // Credit transfer
        let debit_account = self.ledger.find_or_create_account(
            &venue,
            &credit_asset,
            AccountOwner::Venue,
            AccountType::Spot,
            self.time.now().await,
        );
        let credit_account = self.ledger.find_or_create_account(
            &venue,
            &credit_asset,
            AccountOwner::User,
            AccountType::Spot,
            self.time.now().await,
        );

        transfers.push(
            Transfer::builder()
                .event_time(event_time)
                .transfer_group_id(transfer_group_id)
                .asset(credit_asset)
                .debit_account(debit_account)
                .credit_account(credit_account)
                .amount(credit_amount)
                .unit_price(Decimal::ONE)
                .transfer_type(TransferType::Exchange)
                .build()
                .into(),
        );

        // Commission (assume order has commission field; stub 0.1% quote)
        let commission = (debit_amount * dec!(0.001)).max(dec!(0.01)); // Placeholder
        if commission > Decimal::ZERO {
            let user_quote_account = self.ledger.find_or_create_account(
                &venue,
                &Tradable::Asset(order.instrument.quote_asset.clone()),
                AccountOwner::User,
                AccountType::Spot,
                self.time.now().await,
            );
            let commission_account = self.ledger.find_or_create_account(
                &venue,
                &Tradable::Asset(order.instrument.quote_asset.clone()),
                AccountOwner::Venue,
                AccountType::Equity, // Commission expense
                self.time.now().await,
            );
            transfers.push(
                Transfer::builder()
                    .event_time(event_time)
                    .transfer_group_id(transfer_group_id)
                    .asset(Tradable::Asset(order.instrument.quote_asset.clone()))
                    .debit_account(user_quote_account)
                    .credit_account(commission_account)
                    .amount(commission)
                    .unit_price(Decimal::ONE)
                    .transfer_type(TransferType::Commission)
                    .build()
                    .into(),
            );
        }

        let res = self.ledger.apply_transfers(&transfers).await;
        match res {
            Ok(_) => info!(target: "accounting", "Transfers applied successfully"),
            Err(e) => error!(target: "accounting", "Failed to apply transfers: {}", e),
        }
    }
}

#[async_trait]
impl Runnable for Accounting {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            // Account
            Event::InitialAccountUpdate(au) => self.initial_account_update(au).await,
            Event::ReconcileAccountUpdate(au) => self.reconcile_account_update(au).await,
            Event::AccountUpdate(au) => self.account_update(au).await,

            // Balance
            Event::InitialBalanceUpdate(u) => self.initial_balance_update(u).await,
            Event::ReconcileBalanceUpdate(u) => self.reconcile_balance_update(u).await,
            Event::ReconcilePositionUpdate(u) => self.reconcile_position_update(u).await,

            // Position
            Event::InitialPositionUpdate(u) => self.initial_position_update(u).await,
            Event::BalanceUpdate(u) => self.balance_update(u).await,
            Event::PositionUpdate(u) => self.position_update(u).await,

            // Fill (Will probably delete since this is handled with account updates)
            Event::VenueOrderFill(vo) => self.venue_order_fill(vo).await,
            e => warn!(target: "accounting", "received unused event {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use arkin_core::test_utils::{MockPublisher, MockTime};
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    #[tokio::test]
    #[test_log::test]
    async fn test_account_sync() {
        let time = MockTime::new();
        let publisher = MockPublisher::new();
        let ledger = Ledger::new();
        let accounting = Accounting::builder()
            .identifier("test".to_string())
            .time(time.to_owned())
            .publisher(publisher.to_owned())
            .ledger(ledger.to_owned())
            .build();

        let venue = test_binance_venue();
        let usdt = test_usdt_asset();
        let btc_perp = test_inst_binance_btc_usdt_perp(); // Assume leverage=20
        let eth_perp = test_inst_binance_eth_usdt_perp(); // Assume leverage=20

        // Step 1: Initial (100k USDT margin, no positions)
        let initial = AccountUpdate::builder()
            .id(Uuid::new_v4())
            .event_time(time.now().await)
            .balances(vec![BalanceUpdate::builder()
                .event_time(time.now().await)
                .venue(venue.clone())
                .asset(usdt.clone())
                .account_type(AccountType::Margin)
                .quantity_change(dec!(0))
                .quantity(dec!(100000))
                .build()])
            .positions(vec![])
            .reason("INITIAL".to_string())
            .build();
        accounting.handle_event(Event::InitialAccountUpdate(initial.into())).await;

        let margin_acct = ledger.find_or_create_account(
            &venue,
            &Tradable::Asset(usdt.clone()),
            AccountOwner::User,
            AccountType::Margin,
            time.now().await,
        );
        assert_eq!(ledger.balance(margin_acct.id).await, dec!(100000));
        ledger.dump_state(100).await;

        // Step 2: Long BTC perp (qty=1 @10000, unreal=0, deduct margin=10000/20=500)
        time.advance_time_by(Duration::from_secs(1)).await;
        let after_long = AccountUpdate::builder()
            .id(Uuid::new_v4())
            .event_time(time.now().await)
            .balances(vec![BalanceUpdate::builder()
                .event_time(time.now().await)
                .venue(venue.clone())
                .asset(usdt.clone())
                .account_type(AccountType::Margin)
                .quantity_change(dec!(-500))
                .quantity(dec!(99500))
                .build()])
            .positions(vec![PositionUpdate::builder()
                .event_time(time.now().await)
                .instrument(btc_perp.clone())
                .account_type(AccountType::Margin)
                .entry_price(dec!(10000))
                .quantity(dec!(1))
                .realized_pnl(dec!(0))
                .unrealized_pnl(dec!(0))
                .position_side(PositionSide::Long)
                .build()])
            .reason("ORDER".to_string())
            .build();
        accounting.handle_event(Event::AccountUpdate(after_long.into())).await;

        let btc_pos_acct = ledger.find_or_create_account(
            &venue,
            &Tradable::Instrument(btc_perp.clone()),
            AccountOwner::User,
            AccountType::Instrument,
            time.now().await,
        );
        assert_eq!(ledger.balance(btc_pos_acct.id).await, dec!(1));
        assert_eq!(ledger.balance(margin_acct.id).await, dec!(99500));
        ledger.dump_state(100).await;

        // Step 3: Short ETH perp (qty=-2 @2000, unreal=0, deduct margin=abs(-2)*2000/20=200)
        time.advance_time_by(Duration::from_secs(1)).await;
        let after_short = AccountUpdate::builder()
            .id(Uuid::new_v4())
            .event_time(time.now().await)
            .balances(vec![BalanceUpdate::builder()
                .event_time(time.now().await)
                .venue(venue.clone())
                .asset(usdt.clone())
                .account_type(AccountType::Margin)
                .quantity_change(dec!(-200))
                .quantity(dec!(99300))
                .build()])
            .positions(vec![PositionUpdate::builder()
                .event_time(time.now().await)
                .instrument(eth_perp.clone())
                .account_type(AccountType::Margin)
                .entry_price(dec!(2000))
                .quantity(dec!(-2))
                .realized_pnl(dec!(0))
                .unrealized_pnl(dec!(0))
                .position_side(PositionSide::Short)
                .build()])
            .reason("ORDER".to_string())
            .build();
        accounting.handle_event(Event::AccountUpdate(after_short.into())).await;

        let eth_pos_acct = ledger.find_or_create_account(
            &venue,
            &Tradable::Instrument(eth_perp.clone()),
            AccountOwner::User,
            AccountType::Instrument,
            time.now().await,
        );
        assert_eq!(ledger.balance(eth_pos_acct.id).await, dec!(-2));
        assert_eq!(ledger.balance(margin_acct.id).await, dec!(99300));
        ledger.dump_state(100).await;

        // Step 4: Reconciliation (small mismatch, e.g., balance 99301 due to funding fee, BTC qty 1.001 drift)
        time.advance_time_by(Duration::from_secs(1)).await;
        let recon = AccountUpdate::builder()
            .id(Uuid::new_v4())
            .event_time(time.now().await)
            .balances(vec![BalanceUpdate::builder()
                .event_time(time.now().await)
                .venue(venue.clone())
                .asset(usdt.clone())
                .account_type(AccountType::Margin)
                .quantity_change(dec!(1))
                .quantity(dec!(99301))
                .build()])
            .positions(vec![PositionUpdate::builder()
                .event_time(time.now().await)
                .instrument(btc_perp.clone())
                .account_type(AccountType::Margin)
                .entry_price(dec!(10000))
                .quantity(dec!(1.001))
                .realized_pnl(dec!(0))
                .unrealized_pnl(dec!(0))
                .position_side(PositionSide::Long)
                .build()])
            .reason("RECON".to_string())
            .build();
        accounting.handle_event(Event::ReconcileAccountUpdate(recon.into())).await;

        assert_eq!(ledger.balance(margin_acct.id).await, dec!(99301));
        assert_eq!(ledger.balance(btc_pos_acct.id).await, dec!(1.001));
        ledger.dump_state(100).await;

        // Step 5: Sell BTC long with gain (close to qty=0, realized +1000, release margin 10000/20=500 + gain to margin)
        time.advance_time_by(Duration::from_secs(1)).await;
        let sell_btc = AccountUpdate::builder()
            .id(Uuid::new_v4())
            .event_time(time.now().await)
            .balances(vec![BalanceUpdate::builder()
                .event_time(time.now().await)
                .venue(venue.clone())
                .asset(usdt.clone())
                .account_type(AccountType::Margin)
                .quantity_change(dec!(500)) // Released 500
                .quantity(dec!(99801))
                .build()])
            .positions(vec![PositionUpdate::builder()
                .event_time(time.now().await)
                .instrument(btc_perp.clone())
                .account_type(AccountType::Margin)
                .entry_price(dec!(0)) // Closed
                .quantity(dec!(0))
                .realized_pnl(dec!(1000))
                .unrealized_pnl(dec!(0))
                .position_side(PositionSide::Long)
                .build()])
            .reason("ORDER".to_string())
            .build();
        accounting.handle_event(Event::AccountUpdate(sell_btc.into())).await;

        assert_eq!(ledger.balance(btc_pos_acct.id).await, dec!(0));
        assert_eq!(ledger.balance(margin_acct.id).await, dec!(100801));
        ledger.dump_state(100).await;

        // Step 6: Buy back ETH short with loss (close to qty=0, realized -500, release margin 4000/20=200 - loss from margin)
        time.advance_time_by(Duration::from_secs(1)).await;
        let buy_eth = AccountUpdate::builder()
            .id(Uuid::new_v4())
            .event_time(time.now().await)
            .balances(vec![BalanceUpdate::builder()
                .event_time(time.now().await)
                .venue(venue.clone())
                .asset(usdt.clone())
                .account_type(AccountType::Margin)
                .quantity_change(dec!(200)) // Released 200
                .quantity(dec!(101001))
                .build()])
            .positions(vec![PositionUpdate::builder()
                .event_time(time.now().await)
                .instrument(eth_perp.clone())
                .account_type(AccountType::Margin)
                .entry_price(dec!(0))
                .quantity(dec!(0))
                .realized_pnl(dec!(-500))
                .unrealized_pnl(dec!(0))
                .position_side(PositionSide::Short)
                .build()])
            .reason("ORDER".to_string())
            .build();
        accounting.handle_event(Event::AccountUpdate(buy_eth.into())).await;

        assert_eq!(ledger.balance(eth_pos_acct.id).await, dec!(0));
        assert_eq!(ledger.balance(margin_acct.id).await, dec!(100501));
        ledger.dump_state(100).await;

        // Final ledger check (dump or asserts)
        let realized_pnl_acct = ledger.find_or_create_account(
            &venue,
            &Tradable::Asset(usdt.clone()),
            AccountOwner::User,
            AccountType::Margin,
            time.now().await,
        );
        assert_eq!(ledger.balance(realized_pnl_acct.id).await, dec!(100501)); // Net +1000 -500
        ledger.dump_state(100).await; // For visibility
    }

    #[tokio::test]
    #[test_log::test]
    #[ignore]
    async fn test_accounting() {
        let time = MockTime::new();
        let publisher = MockPublisher::new();
        let ledger = Ledger::new();
        let accounting = Accounting::builder()
            .identifier("test".to_string())
            .time(time.to_owned())
            .publisher(publisher.to_owned())
            .ledger(ledger.to_owned())
            .build();

        // Initial zero assertion
        let usdt_account = ledger.find_or_create_account(
            &test_binance_venue(),
            &Tradable::Asset(test_usdt_asset()),
            AccountOwner::User,
            AccountType::Margin,
            time.now().await,
        );
        assert_eq!(ledger.balance(usdt_account.id).await, dec!(0));

        let init_balance_update_1 = BalanceUpdate::builder()
            .id(Uuid::new_v4())
            .event_time(time.now().await)
            .venue(test_binance_venue())
            .account_type(AccountType::Margin)
            .asset(test_usdt_asset())
            .quantity_change(dec!(0))
            .quantity(dec!(100000))
            .build();

        accounting
            .handle_event(Event::InitialBalanceUpdate(init_balance_update_1.clone().into()))
            .await;

        assert_eq!(ledger.balance(usdt_account.id).await, dec!(100000));

        let init_position_update_1 = PositionUpdate::builder()
            .id(Uuid::new_v4())
            .event_time(time.now().await)
            .account_type(AccountType::Margin)
            .instrument(test_inst_binance_btc_usdt_perp())
            .entry_price(dec!(10000))
            .quantity(dec!(1))
            .realized_pnl(dec!(0))
            .unrealized_pnl(dec!(100))
            .position_side(PositionSide::Long)
            .build();

        accounting
            .handle_event(Event::InitialPositionUpdate(init_position_update_1.clone().into()))
            .await;

        let btc_position_account = ledger.find_or_create_account(
            &test_binance_venue(),
            &Tradable::Asset(test_btc_asset()), // Assume base
            AccountOwner::User,
            AccountType::Instrument,
            time.now().await,
        );
        assert_eq!(ledger.balance(btc_position_account.id).await, dec!(1));

        let unrealized_pnl_account = ledger.find_or_create_account(
            &test_binance_venue(),
            &Tradable::Asset(test_usdt_asset()),
            AccountOwner::User,
            AccountType::Equity,
            time.now().await,
        );
        assert_eq!(ledger.balance(unrealized_pnl_account.id).await, dec!(100));

        // We realize some pnl
        let init_position_update_2 = PositionUpdate::builder()
            .id(Uuid::new_v4())
            .event_time(time.now().await)
            .account_type(AccountType::Margin)
            .instrument(test_inst_binance_btc_usdt_perp())
            .entry_price(dec!(10000))
            .quantity(dec!(0.5))
            .realized_pnl(dec!(50))
            .unrealized_pnl(dec!(50))
            .position_side(PositionSide::Long)
            .build();

        accounting
            .handle_event(Event::PositionUpdate(init_position_update_2.clone().into()))
            .await;

        let btc_position_account = ledger.find_or_create_account(
            &test_binance_venue(),
            &Tradable::Asset(test_btc_asset()), // Assume base
            AccountOwner::User,
            AccountType::Instrument,
            time.now().await,
        );
        assert_eq!(ledger.balance(btc_position_account.id).await, dec!(0.5));

        let unrealized_pnl_account = ledger.find_or_create_account(
            &test_binance_venue(),
            &Tradable::Asset(test_usdt_asset()),
            AccountOwner::User,
            AccountType::Equity,
            time.now().await,
        );
        assert_eq!(ledger.balance(unrealized_pnl_account.id).await, dec!(50));

        let margin_account = ledger.find_or_create_account(
            &test_binance_venue(),
            &Tradable::Asset(test_usdt_asset()),
            AccountOwner::User,
            AccountType::Margin,
            time.now().await,
        );
        assert_eq!(ledger.balance(margin_account.id).await, dec!(100050));
        ledger.dump_state(100).await;
    }
}

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
