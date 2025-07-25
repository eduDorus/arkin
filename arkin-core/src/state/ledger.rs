use dashmap::DashMap;
use rust_decimal::prelude::*;
use std::sync::Arc;
use thiserror::Error;
use time::UtcDateTime;
use tokio::sync::RwLock;
use tracing::info;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{
    Account, AccountOwner, AccountType, Asset, Event, Instrument, InstrumentType, Publisher, Strategy, Transfer,
    TransferBatch, TransferGroupType, TransferType, Venue,
};

#[derive(Error, Debug)]
pub enum AccountingError {
    #[error("Missing strategy information on strategy account creation")]
    MissingStrategy,

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Currency mismatch in transfer: {0}")]
    CurrencyMismatch(Arc<Transfer>),

    #[error("Insufficient balance in debit account: {0}")]
    InsufficientBalance(Arc<Transfer>),

    #[error("Invalid balance: {0}")]
    InvalidBalance(Decimal),

    #[error("Transfer amount must be > 0: {0}")]
    InvalidTransferAmount(Arc<Transfer>),

    #[error("Debit account not found: {0}")]
    DebitAccountNotFound(String),

    #[error("Credit account not found: {0}")]
    CreditAccountNotFound(String),

    // #[error("Liquidity Account not found for {0}: {1}")]
    // LiquidityAccountNotFound(Arc<Venue>, Tradable),

    // #[error("Venue Account not found for {0}: {1}")]
    // VenueAccountNotFound(Arc<Venue>, Tradable),

    // #[error("Strategy Account not found for {0}: {1}")]
    // StrategyAccountNotFound(Arc<Strategy>, Tradable),
    #[error("Invalid exchange: {0}")]
    InvalidExchange(String),

    #[error("Invalid instrument: {0}")]
    UnsupportedInstrumentType(InstrumentType),

    #[error("Same account found for transaction: {0}")]
    SameAccount(Arc<Transfer>),

    #[error("Missing asset transaction: {0}")]
    MissingAsset(Arc<Asset>),
}

/// The in-memory ledger tracks accounts and can apply sets of transfers atomically.
#[derive(TypedBuilder)]
pub struct Ledger {
    publisher: Arc<dyn Publisher>,
    // For simplicity, we'll hold accounts in a HashMap
    #[builder(default)]
    accounts: DashMap<Uuid, Arc<Account>>,
    // We store completed transfers here, or you could store them in a DB, etc.
    #[builder(default = RwLock::new(Vec::new()))]
    transfers: RwLock<Vec<Arc<Transfer>>>,
    // Cached balances for O(1) access
    #[builder(default)]
    balances: DashMap<(Arc<Account>, Arc<Asset>), Decimal>,
    #[builder(default)]
    positions: DashMap<(Arc<Account>, Arc<Instrument>), (Decimal, Decimal)>, // (Entry Price, Quantity)
    #[builder(default)]
    strategy_positions: DashMap<(Arc<Account>, Arc<Strategy>, Arc<Instrument>), (Decimal, Decimal)>, // (Entry Price, Quantity)
}

impl Ledger {
    pub fn new(publisher: Arc<dyn Publisher>) -> Arc<Self> {
        Self {
            publisher,
            accounts: DashMap::new(),
            transfers: RwLock::new(Vec::new()),
            balances: DashMap::new(),
            positions: DashMap::new(),
            strategy_positions: DashMap::new(),
        }
        .into()
    }

    /// Adds an account to the ledger and returns it.
    pub async fn add_account(
        &self,
        venue: Arc<Venue>,
        owner: AccountOwner,
        account_type: AccountType,
        timestamp: UtcDateTime,
    ) -> Arc<Account> {
        match self.find_account(&venue, owner, account_type) {
            Some(account) => account,
            None => {
                let account: Arc<Account> = Account::builder()
                    .id(Uuid::new_v4())
                    .venue(venue)
                    .owner(owner)
                    .account_type(account_type)
                    .created(timestamp)
                    .updated(timestamp)
                    .build()
                    .into();
                self.accounts.insert(account.id, account.clone());
                info!("Added account: {} {}", account.id, account);

                self.publisher.publish(Event::NewAccount(account.clone().into())).await;
                account
            }
        }
    }

    /// Finds an account by venue, asset, and account type.
    pub fn find_account(
        &self,
        venue: &Arc<Venue>,
        owner: AccountOwner,
        account_type: AccountType,
    ) -> Option<Arc<Account>> {
        self.accounts
            .iter()
            .find(|a| a.venue == *venue && a.owner == owner && a.account_type == account_type)
            .map(|e| e.value().clone())
    }

    /// Finds an account by venue, asset, and account type, or creates it if it doesn't exist.
    pub async fn find_or_create_account(
        &self,
        venue: &Arc<Venue>,
        owner: AccountOwner,
        account_type: AccountType,
        timestamp: UtcDateTime,
    ) -> Arc<Account> {
        match self.find_account(venue, owner, account_type) {
            Some(account) => account,
            None => self.add_account(venue.clone(), owner, account_type, timestamp).await,
        }
    }

    /// Returns an account by ID.
    pub fn get_account(&self, account_id: Uuid) -> Option<Arc<Account>> {
        self.accounts.get(&account_id).map(|e| e.clone())
    }

    /// Returns all accounts in the ledger.
    pub fn list_accounts(&self) -> Vec<Arc<Account>> {
        self.accounts.iter().map(|e| e.clone()).collect()
    }

    /// Returns the global balance for an account.
    /// This is the sum of all debit and credit transfers from the account.
    pub async fn balance(&self, account: &Arc<Account>, asset: &Arc<Asset>) -> Decimal {
        let key = (account.to_owned(), asset.to_owned());
        if let Some(bal) = self.balances.get(&key) {
            *bal.value()
        } else {
            let txs = self.transfers.read().await;
            let computed = txs
                .iter()
                .filter(|t| {
                    t.has_asset(&asset) && (t.debit_account.id == account.id || t.credit_account.id == account.id)
                })
                .fold(dec!(0), |acc, t| {
                    if t.credit_account.id == account.id {
                        acc + t.amount
                    } else {
                        acc - t.amount
                    }
                });
            self.balances.insert(key, computed);
            computed
        }
    }

    pub async fn get_position(&self, account: &Arc<Account>, instrument: &Arc<Instrument>) -> (Decimal, Decimal) {
        let key = (account.to_owned(), instrument.to_owned());
        if let Some(pos) = self.positions.get(&key) {
            *pos.value()
        } else {
            let txs = self.transfers.read().await;
            let mut entry = dec!(0);
            let mut qty = dec!(0);
            let mut sorted_txs = txs
                .iter()
                .filter(|tx| {
                    tx.has_instrument(instrument)
                        && tx.transfer_type == TransferType::Transfer
                        && (&tx.debit_account == account || &tx.credit_account == account)
                })
                .cloned()
                .collect::<Vec<_>>();
            sorted_txs.sort_by_key(|tx| tx.created); // Ensure order
            for tx in sorted_txs {
                let qty_sign = if &tx.credit_account == account {
                    dec!(1)
                } else {
                    dec!(-1)
                };
                let qty_delta = tx.amount * qty_sign;
                (entry, qty) = self.update_position(entry, qty, qty_delta, tx.unit_price);
            }
            self.positions.insert(key, (entry, qty));
            (entry, qty)
        }
    }

    pub async fn get_strategy_position(
        &self,
        account: &Arc<Account>,
        strategy: Arc<Strategy>,
        instrument: &Arc<Instrument>,
    ) -> (Decimal, Decimal) {
        let key = (account.to_owned(), strategy.to_owned(), instrument.to_owned());
        if let Some(pos) = self.strategy_positions.get(&key) {
            *pos.value()
        } else {
            let txs = self.transfers.read().await;
            let mut entry = dec!(0);
            let mut qty = dec!(0);
            let mut sorted_txs = txs
                .iter()
                .filter(|tx| {
                    tx.has_instrument(instrument)
                        && tx.transfer_type == TransferType::Transfer
                        && tx.strategy.as_ref() == Some(&strategy)
                        && (&tx.debit_account == account || &tx.credit_account == account)
                })
                .cloned()
                .collect::<Vec<_>>();
            sorted_txs.sort_by_key(|tx| tx.created);
            for tx in sorted_txs {
                let qty_sign = if &tx.credit_account == account {
                    dec!(1)
                } else {
                    dec!(-1)
                };
                let qty_delta = tx.amount * qty_sign;
                (entry, qty) = self.update_position(entry, qty, qty_delta, tx.unit_price);
            }
            self.strategy_positions.insert(key, (entry, qty));
            (entry, qty)
        }
    }

    /// Returns the total margin posted for the current position.
    pub async fn margin_posted(&self, strategy: Option<Arc<Strategy>>, instrument: Option<Arc<Instrument>>) -> Decimal {
        let transfers = self.transfers.read().await;
        transfers
            .iter()
            .filter(|t| {
                t.has_transfer_type(&TransferType::Margin)
                    && match strategy.as_ref() {
                        Some(s) => t.has_strategy(s),
                        None => true,
                    }
                    && match instrument.as_ref() {
                        Some(i) => t.has_instrument(i),
                        None => true,
                    }
            })
            .fold(Decimal::ZERO, |acc, t| {
                if t.debit_account.is_user_account() {
                    acc + t.amount // Margin posted to venue
                } else {
                    acc - t.amount // Margin released from venue
                }
            })
    }

    /// Returns all transfers in the ledger.
    /// This can be quite expensive and should only be used for debugging or reporting.
    pub async fn get_transfers(&self) -> Vec<Arc<Transfer>> {
        let lock = self.transfers.read().await;
        lock.iter().cloned().collect()
    }

    /// Performs a single same-currency transfer **atomically**:
    /// This is a helper function since transfers are quite common.
    pub async fn transfer(
        &self,
        event_time: UtcDateTime,
        debit_account: &Arc<Account>,
        credit_account: &Arc<Account>,
        asset: &Arc<Asset>,
        amount: Decimal,
    ) -> Result<(), AccountingError> {
        let transfer_group_type = if credit_account.is_venue_account() {
            TransferGroupType::Deposit
        } else {
            TransferGroupType::Withdrawal
        };
        let transfer = Arc::new(
            Transfer::builder()
                .transfer_group_id(Uuid::new_v4())
                .transfer_group_type(transfer_group_type)
                .transfer_type(TransferType::Transfer)
                .debit_account(debit_account.clone())
                .credit_account(credit_account.clone())
                .amount(amount)
                .unit_price(Decimal::ONE)
                .strategy(None)
                .instrument(None)
                .asset(Some(asset.clone()))
                .created(event_time)
                .build(),
        );
        let batch = TransferBatch::builder()
            .event_time(event_time)
            .transfers(vec![transfer])
            .build();
        self.apply_transfers(batch.into()).await
    }

    fn update_position(
        &self,
        current_entry: Decimal,
        current_qty: Decimal,
        qty_delta: Decimal,
        price: Decimal,
    ) -> (Decimal, Decimal) {
        let amt_closed = if current_qty.signum() != qty_delta.signum() {
            current_qty.abs().min(qty_delta.abs())
        } else {
            dec!(0)
        };
        let new_qty = current_qty + qty_delta;
        let new_entry = if new_qty == dec!(0) {
            dec!(0)
        } else if amt_closed == current_qty.abs() {
            price
        } else if amt_closed > dec!(0) {
            current_entry
        } else {
            if current_qty.is_zero() {
                price
            } else {
                (current_entry * current_qty.abs() + price * qty_delta.abs()) / new_qty.abs()
            }
        };
        (new_entry, new_qty)
    }

    /// Performs one or more same-currency transfers **atomically**:
    /// - All succeed or all fail if any validation fails (e.g. insufficient balance).
    /// - For double-entry: each Transfer has a `debit_account_id` and `credit_account_id`.
    ///
    /// Returns an error if any of the transfers are invalid.
    pub async fn apply_transfers(&self, transfer_batch: Arc<TransferBatch>) -> Result<(), AccountingError> {
        let mut tx_log_lock = self.transfers.write().await;

        for t in transfer_batch.transfers.iter() {
            if t.debit_account.id == t.credit_account.id {
                return Err(AccountingError::SameAccount(t.clone()));
            }

            if t.amount <= Decimal::ZERO {
                return Err(AccountingError::InvalidTransferAmount(t.clone()));
            }

            if t.debit_account.is_user_account()
                && (t.debit_account.account_type == AccountType::Spot
                    || t.debit_account.account_type == AccountType::Margin)
            {
                if let Some(asset) = t.asset.clone() {
                    let key = (t.debit_account.to_owned(), asset.to_owned());
                    let bal = if let Some(b) = self.balances.get(&key) {
                        *b
                    } else {
                        let computed = tx_log_lock
                            .iter()
                            .filter(|tx| {
                                tx.has_asset(&asset)
                                    && (tx.debit_account.id == t.debit_account.id
                                        || tx.credit_account.id == t.debit_account.id)
                            })
                            .fold(dec!(0), |acc, tx| {
                                if tx.credit_account.id == t.debit_account.id {
                                    acc + tx.amount
                                } else {
                                    acc - tx.amount
                                }
                            });
                        self.balances.insert(key, computed);
                        computed
                    };
                    if bal < t.amount {
                        return Err(AccountingError::InsufficientBalance(t.clone()));
                    }
                }

                // Only check positions if TransferType::Transfer (qty only)
                if t.transfer_type == TransferType::Transfer {
                    if let Some(inst) = t.instrument.clone() {
                        let key = (t.debit_account.to_owned(), inst.to_owned());
                        if self.positions.get(&key).is_none() {
                            let computed_qty = tx_log_lock
                                .iter()
                                .filter(|tx| {
                                    tx.has_instrument(&inst)
                                        && tx.transfer_type == TransferType::Transfer
                                        && (tx.debit_account.id == t.debit_account.id
                                            || tx.credit_account.id == t.debit_account.id)
                                })
                                .fold(dec!(0), |acc, tx| {
                                    if tx.credit_account.id == t.debit_account.id {
                                        acc + tx.amount
                                    } else {
                                        acc - tx.amount
                                    }
                                });
                            self.positions.insert(key, (dec!(0), computed_qty));
                            // Init entry=0
                        };
                    }
                }
            }
        }

        for t in transfer_batch.transfers.iter() {
            info!("Applying transfer: {}", t);
            tx_log_lock.push(t.clone());

            // Always update balances if asset (PnL/comm/margin)
            if let Some(asset) = t.asset.clone() {
                let d_key = (t.debit_account.to_owned(), asset.to_owned());
                let mut db = self.balances.entry(d_key).or_insert(dec!(0));
                *db -= t.amount;

                let c_key = (t.credit_account.to_owned(), asset.to_owned());
                let mut cb = self.balances.entry(c_key).or_insert(dec!(0));
                *cb += t.amount;
            }

            // Only update positions if TransferType::Transfer
            if t.transfer_type == TransferType::Transfer {
                if let Some(inst) = t.instrument.clone() {
                    // Determine direction (user receives positive qty on buy/credit)
                    let is_credit_user = t.credit_account.is_user_account();
                    let qty_sign = if is_credit_user { dec!(1) } else { dec!(-1) };
                    let qty_delta = t.amount * qty_sign;

                    // Overall user position
                    let user_key = if is_credit_user {
                        t.credit_account.clone()
                    } else {
                        t.debit_account.clone()
                    };
                    let (current_entry, current_qty) = *self
                        .positions
                        .entry((user_key.clone(), inst.clone()))
                        .or_insert((dec!(0), dec!(0)));
                    let (new_entry, new_qty) =
                        self.update_position(current_entry, current_qty, qty_delta, t.unit_price);
                    self.positions.insert((user_key.clone(), inst.clone()), (new_entry, new_qty));

                    // Strategy if present
                    if let Some(strat) = t.strategy.clone() {
                        let strat_key = (user_key.clone(), strat, inst.clone());
                        let (current_strat_entry, current_strat_qty) =
                            *self.strategy_positions.entry(strat_key.clone()).or_insert((dec!(0), dec!(0)));
                        let (new_strat_entry, new_strat_qty) =
                            self.update_position(current_strat_entry, current_strat_qty, qty_delta, t.unit_price);
                        self.strategy_positions.insert(strat_key, (new_strat_entry, new_strat_qty));
                    }

                    // Venue mirror
                    let venue_key = if is_credit_user {
                        t.debit_account.clone()
                    } else {
                        t.credit_account.clone()
                    };
                    self.positions.insert((venue_key, inst), (new_entry, -new_qty));
                    // Entry positive, qty negative
                }
            }
        }

        self.publisher.publish(Event::NewTransferBatch(transfer_batch)).await;
        Ok(())
    }

    /// Dumps the current state of the ledger as a formatted string for debugging.
    /// Includes all accounts with balances and the last N transfers.
    pub async fn dump_state(&self, max_transfers: usize) {
        let mut accts: Vec<_> = self.accounts.iter().map(|e| (e.key().clone(), e.value().clone())).collect();
        accts.sort_by_key(|(id, _)| *id); // Deterministic order

        // User Accounts section with multi-asset balances and positions
        println!("=== User Balances ===");
        for (id, acct) in accts.iter().filter(|(_, a)| a.is_user_account()) {
            println!("Account: {}", acct);

            // Collect and log asset balances
            let mut asset_bals: Vec<(Arc<Asset>, Decimal)> = self
                .balances
                .iter()
                .filter_map(|ref_multi| {
                    if ref_multi.key().0.id == *id {
                        Some((ref_multi.key().1.clone(), *ref_multi.value()))
                    } else {
                        None
                    }
                })
                .collect();
            asset_bals.sort_by_key(|(a, _)| a.symbol.clone());
            for (asset, bal) in asset_bals {
                println!("  Asset {}: {}", asset.symbol, bal);
            }

            // Collect and log instrument positions (qty @ entry)
            let mut instr_pos: Vec<(Arc<Instrument>, (Decimal, Decimal))> = self
                .positions
                .iter()
                .filter_map(|ref_multi| {
                    if ref_multi.key().0.id == *id {
                        Some((ref_multi.key().1.clone(), *ref_multi.value()))
                    } else {
                        None
                    }
                })
                .collect();
            instr_pos.sort_by_key(|(i, _)| i.symbol.clone());
            for (instr, (entry, qty)) in instr_pos {
                println!("  Position {}: {} @ {}", instr.symbol, qty, entry);
            }
        }

        // Strategy Positions section
        println!("=== Strategy Positions ===");
        let mut strat_pos: Vec<((Arc<Account>, Arc<Strategy>, Arc<Instrument>), (Decimal, Decimal))> =
            self.strategy_positions.iter().map(|e| (e.key().clone(), *e.value())).collect();
        strat_pos.sort_by_key(|(k, _)| (k.0.id, k.1.id, k.2.symbol.clone()));
        for ((acct, strat, instr), (entry, qty)) in strat_pos {
            println!(
                "  Account: {}, Strategy: {}, Position {}: {} @ {}",
                acct, strat.id, instr.symbol, qty, entry
            );
        }

        // Venue Accounts section similar (qty @ entry)
        println!("=== Venue Balances ===");
        for (id, acct) in accts.iter().filter(|(_, a)| a.is_venue_account()) {
            println!("Account: {}", acct);

            let mut asset_bals: Vec<(Arc<Asset>, Decimal)> = self
                .balances
                .iter()
                .filter_map(|ref_multi| {
                    if ref_multi.key().0.id == *id {
                        Some((ref_multi.key().1.clone(), *ref_multi.value()))
                    } else {
                        None
                    }
                })
                .collect();
            asset_bals.sort_by_key(|(a, _)| a.symbol.clone());
            for (asset, bal) in asset_bals {
                println!("  Asset {}: {}", asset.symbol, bal);
            }

            let mut instr_pos: Vec<(Arc<Instrument>, (Decimal, Decimal))> = self
                .positions
                .iter()
                .filter_map(|ref_multi| {
                    if ref_multi.key().0.id == *id {
                        Some((ref_multi.key().1.clone(), *ref_multi.value()))
                    } else {
                        None
                    }
                })
                .collect();
            instr_pos.sort_by_key(|(i, _)| i.symbol.clone());
            for (instr, (entry, qty)) in instr_pos {
                println!("  Position {}: {} @ {}", instr.symbol, qty, entry);
            }
        }

        // Transfers section (last max_transfers, oldest first)
        println!("=== Recent Transfers (Oldest First) ===");
        let tx_lock = self.transfers.read().await;
        let mut all_tx: Vec<_> = tx_lock.iter().cloned().collect();
        all_tx.sort_by_key(|t| t.created);
        let recent: Vec<_> = all_tx.iter().take(max_transfers).cloned().collect();
        for tx in recent {
            println!("{}", tx);
        }
        if all_tx.len() > max_transfers {
            println!("... ({} more transfers omitted)", all_tx.len() - max_transfers);
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use rust_decimal_macros::dec;

//     use crate::{prelude::MockTime, test_utils, SystemTime};

//     #[tokio::test]
//     #[test_log::test]
//     async fn test_successful_transfer() {
//         let time = MockTime::new();
//         let ledger = Ledger::builder().build();

//         let personal_venue = test_utils::test_personal_venue();
//         let binance_venue = test_utils::test_binance_venue();

//         // let strategy = test_strategy();
//         let usdt = test_utils::test_usdt_asset();

//         // Create Personal account for USD
//         let account_l =
//             ledger.add_account(personal_venue.clone(), AccountOwner::Venue, AccountType::Spot, time.now().await);

//         // Create two Strategy accounts for USD
//         let account_a =
//             ledger.add_account(binance_venue.clone(), AccountOwner::User, AccountType::Spot, time.now().await);
//         let account_b =
//             ledger.add_account(binance_venue.clone(), AccountOwner::User, AccountType::Margin, time.now().await);

//         let amount = dec!(100);
//         ledger
//             .transfer(UtcDateTime::now(), &account_l, &account_a, &usdt, amount)
//             .await
//             .unwrap();
//         ledger
//             .transfer(UtcDateTime::now(), &account_a, &account_b, &usdt, amount)
//             .await
//             .unwrap();
//         let half_amount = amount / dec!(2);
//         ledger
//             .transfer(UtcDateTime::now(), &account_b, &account_a, &usdt, half_amount)
//             .await
//             .unwrap();

//         // Verify balances
//         assert_eq!(ledger.balance(account_l.id).await, -amount); // -100 USD
//         assert_eq!(ledger.balance(account_a.id).await, half_amount); // +50 USD
//         assert_eq!(ledger.balance(account_b.id).await, half_amount); // +50 USD

//         // Verify transfer record
//         let transfers = ledger.get_transfers().await;
//         assert_eq!(transfers.len(), 3);
//         let t = &transfers[1];
//         assert_eq!(t.debit_account.id, account_a.id);
//         assert_eq!(t.credit_account.id, account_b.id);
//         assert_eq!(t.amount, amount);
//         assert_eq!(t.transfer_type, TransferType::Transfer);
//         assert_eq!(t.unit_price, dec!(1));
//     }

//     #[tokio::test]
//     #[test_log::test]
//     async fn test_insufficient_balance_client_spot() {
//         let time = MockTime::new();
//         let ledger = Ledger::builder().build();
//         let personal_venue = test_utils::test_personal_venue();
//         let binance_venue = test_utils::test_binance_venue();
//         let usdt = test_utils::test_usdt_asset();

//         // Create Personal account for USD
//         let account_l =
//             ledger.add_account(personal_venue.clone(), AccountOwner::Venue, AccountType::Spot, time.now().await);

//         // Create two Strategy accounts for USD
//         let account_a =
//             ledger.add_account(binance_venue.clone(), AccountOwner::User, AccountType::Spot, time.now().await);

//         ledger
//             .transfer(UtcDateTime::now(), &account_l, &account_a, &usdt, dec!(1000))
//             .await
//             .unwrap();
//         let result = ledger.transfer(UtcDateTime::now(), &account_a, &account_l, dec!(1001)).await;
//         assert!(matches!(result, Err(AccountingError::InsufficientBalance(_))));

//         assert_eq!(ledger.balance(account_l.id).await, dec!(-1000));
//         assert_eq!(ledger.balance(account_a.id).await, dec!(1000));
//     }

//     #[tokio::test]
//     #[test_log::test]
//     async fn test_invalid_amount() {
//         let time = MockTime::new();
//         let ledger = Ledger::builder().build();
//         let binance_venue = test_utils::test_binance_venue();
//         let usdt = test_utils::test_usdt_asset();

//         // Create two Strategy accounts for USD
//         let account_a = ledger.add_account(
//             binance_venue.clone(),
//             usdt.clone().into(),
//             AccountOwner::User,
//             AccountType::Spot,
//             time.now().await,
//         );
//         let account_b = ledger.add_account(
//             binance_venue.clone(),
//             usdt.clone().into(),
//             AccountOwner::User,
//             AccountType::Margin,
//             time.now().await,
//         );

//         let result_zero = ledger.transfer(UtcDateTime::now(), &account_a, &account_b, dec!(0)).await;
//         assert!(matches!(result_zero, Err(AccountingError::InvalidTransferAmount(_))));

//         let result_negative = ledger.transfer(UtcDateTime::now(), &account_a, &account_b, dec!(-10)).await;
//         assert!(matches!(result_negative, Err(AccountingError::InvalidTransferAmount(_))));
//     }

//     #[tokio::test]
//     #[test_log::test]
//     async fn test_currency_mismatch() {
//         let time = MockTime::new();
//         let ledger = Ledger::builder().build();
//         let binance_venue = test_utils::test_binance_venue();
//         let usdt = test_utils::test_usdt_asset();
//         let btc = test_utils::test_btc_asset();

//         let account_usd = ledger.add_account(
//             binance_venue.clone(),
//             usdt.clone().into(),
//             AccountOwner::User,
//             AccountType::Spot,
//             time.now().await,
//         );
//         let account_btc = ledger.add_account(
//             binance_venue.clone(),
//             btc.clone().into(),
//             AccountOwner::User,
//             AccountType::Spot,
//             time.now().await,
//         );

//         let result = ledger.transfer(UtcDateTime::now(), &account_usd, &account_btc, dec!(100)).await;
//         assert!(matches!(result, Err(AccountingError::CurrencyMismatch(_))));
//     }

//     #[tokio::test]
//     #[test_log::test]
//     async fn test_same_account() {
//         let time = MockTime::new();
//         let ledger = Ledger::builder().build();
//         let binance_venue = test_utils::test_binance_venue();
//         let usdt = test_utils::test_usdt_asset();

//         let account_a = ledger.add_account(
//             binance_venue.clone(),
//             usdt.clone().into(),
//             AccountOwner::User,
//             AccountType::Spot,
//             time.now().await,
//         );
//         let result = ledger.transfer(UtcDateTime::now(), &account_a, &account_a, dec!(100)).await;
//         assert!(matches!(result, Err(AccountingError::SameAccount(_))));
//     }
// }

// /// Returns the net position amount for an account and strategy.
// pub async fn strategy_balance(&self, strategy: &Arc<Strategy>, instrument: Option<&Arc<Instrument>>) -> Decimal {
//     let transfers = self.transfers.read().await;
//     transfers
//         .iter()
//         .filter(|t| {
//             t.has_transfer_type(&TransferType::Trade)
//                 && t.has_strategy(strategy)
//                 && match instrument {
//                     Some(i) => t.has_instrument(i),
//                     None => true,
//                 }
//         })
//         .fold(Decimal::ZERO, |acc, t| {
//             if t.credit_account.is_user_account() {
//                 acc + t.amount
//             } else {
//                 acc - t.amount
//             }
//         })
// }

// /// Returns the net PnL for an account and strategy.
// pub async fn strategy_pnl(&self, strategy: &Arc<Strategy>, instrument: Option<&Arc<Instrument>>) -> Decimal {
//     let transfers = self.transfers.read().await;
//     transfers
//         .iter()
//         .filter(|t| {
//             t.has_transfer_type(&TransferType::Pnl)
//                 && t.has_strategy(strategy)
//                 && match instrument {
//                     Some(i) => t.has_instrument(i),
//                     None => true,
//                 }
//         })
//         .fold(Decimal::ZERO, |acc, t| {
//             if t.credit_account.is_user_account() {
//                 acc + t.amount
//             } else {
//                 acc - t.amount
//             }
//         })
// }

// pub async fn strategy_net_value(&self, strategy: &Arc<Strategy>, instrument: Option<&Arc<Instrument>>) -> Decimal {
//     let (total_cost, total_amount) = self.current_position(strategy, instrument).await;
//     total_cost * total_amount
// }

// pub async fn strategy_cost_basis(&self, strategy: &Arc<Strategy>, instrument: Option<&Arc<Instrument>>) -> Decimal {
//     let (total_cost, _) = self.current_position(strategy, instrument).await;
//     total_cost
// }

// Returns the total cost basis and current amount.
// pub async fn current_position(
//     &self,
//     strategy: &Arc<Strategy>,
//     instrument: Option<&Arc<Instrument>>,
// ) -> (Decimal, Decimal) {
//     let filter = |t: &&Arc<Transfer>| {
//         t.has_transfer_type(&TransferType::Trade)
//             && t.has_strategy(strategy)
//             && match instrument {
//                 Some(i) => t.has_instrument(i),
//                 None => true,
//             }
//     };

//     let transfers = self.transfers.read().await;
//     let mut total_cost = Decimal::ZERO;
//     let mut total_amount = Decimal::ZERO;
//     let mut running_position = Decimal::ZERO;
//     let mut total_amount_signed = Decimal::ZERO;

//     for t in transfers.iter().filter(filter) {
//         let amount = t.amount; // Amount is always positive
//         let is_buy = t.debit_account.is_user_account(); // Buy: debit to account
//         let tx_position_change = if is_buy { amount } else { -amount }; // Buy increases, sell decreases position

//         total_amount_signed += tx_position_change;

//         // Current position before this transaction
//         let position_before = running_position;
//         running_position += tx_position_change;

//         if position_before.is_zero() {
//             // Starting a new position
//             total_cost = amount * t.unit_price;
//             total_amount = amount;
//         } else if position_before.is_sign_positive() {
//             if is_buy {
//                 // Adding to long position
//                 total_cost += amount * t.unit_price;
//                 total_amount += amount;
//             } else {
//                 // Selling from long position
//                 if running_position.is_sign_positive() || running_position.is_zero() {
//                     // Still long or flat
//                     let avg_cost = total_cost / total_amount;
//                     total_cost -= amount * avg_cost;
//                     total_amount -= amount;
//                 } else {
//                     // Crossing from long to short
//                     let amount_to_close = position_before; // Amount to reduce to zero
//                     let excess_sell = amount - amount_to_close; // Amount that starts short
//                     let avg_cost = total_cost / total_amount;
//                     total_cost -= amount_to_close * avg_cost;
//                     total_amount -= amount_to_close;
//                     // Reset and start short position
//                     total_cost = excess_sell * t.unit_price;
//                     total_amount = excess_sell;
//                 }
//             }
//         } else {
//             // position_before is negative (short)
//             if !is_buy {
//                 // Adding to short position (sell)
//                 total_cost += amount * t.unit_price;
//                 total_amount += amount;
//             } else {
//                 // Buying to cover short position
//                 if running_position.is_sign_negative() || running_position.is_zero() {
//                     // Still short or flat
//                     let avg_cost = total_cost / total_amount;
//                     total_cost -= amount * avg_cost;
//                     total_amount -= amount;
//                 } else {
//                     // Crossing from short to long
//                     let amount_to_close = -position_before; // Amount to cover short
//                     let excess_buy = amount - amount_to_close; // Amount that starts long
//                     let avg_cost = total_cost / total_amount;
//                     total_cost -= amount_to_close * avg_cost;
//                     total_amount -= amount_to_close;
//                     // Reset and start long position
//                     total_cost = excess_buy * t.unit_price;
//                     total_amount = excess_buy;
//                 }
//             }
//         }

//         // Ensure total_amount doesn't go negative due to rounding
//         if total_amount < Decimal::ZERO {
//             total_cost = Decimal::ZERO;
//             total_amount = Decimal::ZERO;
//         }
//     }
//     info!("Running position: {}", running_position);

//     (total_cost, -total_amount_signed)
// }
