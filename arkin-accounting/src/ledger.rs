use parking_lot::RwLock;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::AccountingError;

/// The in-memory ledger tracks accounts and can apply sets of transfers atomically.
#[derive(Debug, TypedBuilder)]
pub struct Ledger {
    // For simplicity, we'll hold accounts in a HashMap
    #[builder(default = RwLock::new(HashMap::new()))]
    accounts: RwLock<HashMap<Uuid, Arc<Account>>>,
    // We store completed transfers here, or you could store them in a DB, etc.
    #[builder(default = RwLock::new(Vec::new()))]
    transfers: RwLock<Vec<Arc<Transfer>>>,
}

impl Ledger {
    /// Adds an account to the ledger, returns its ID.
    pub fn add_account(&self, name: &str, asset: Tradable, account_type: AccountType) -> Arc<Account> {
        let mut lock: parking_lot::lock_api::RwLockWriteGuard<'_, parking_lot::RawRwLock, HashMap<Uuid, Arc<Account>>> =
            self.accounts.write();
        let account = Arc::new(
            Account::builder()
                .name(name.into())
                .asset(asset.clone())
                .account_type(account_type)
                .build(),
        );
        let id = account.id;
        lock.insert(id, account.clone());
        // Emit event for persistence
        // self.pubsub.publish(Event::AccountAdded(account.clone()));
        account
    }

    pub fn find_account(&self, account_type: &AccountType, asset: &Tradable) -> Option<Arc<Account>> {
        let accounts = self.accounts.read();
        accounts
            .values()
            .find(|acct| acct.account_type == *account_type && acct.asset == *asset)
            .cloned()
    }

    pub fn find_or_create_account(&self, account_type: &AccountType, asset: &Tradable) -> Arc<Account> {
        if let Some(acct) = self.find_account(&account_type, &asset) {
            acct
        } else {
            self.add_account(&format!("{}_{}", account_type, asset), asset.clone(), account_type.clone())
        }
    }

    pub fn get_account(&self, account_id: Uuid) -> Option<Arc<Account>> {
        let lock = self.accounts.read();
        lock.get(&account_id).cloned()
    }

    pub fn get_accounts(&self) -> Vec<Arc<Account>> {
        let lock = self.accounts.read();
        lock.values().cloned().collect()
    }

    pub fn get_balance(&self, account_id: Uuid) -> Decimal {
        let transfers = self.transfers.read();
        let mut balance = Decimal::ZERO;
        for t in transfers.iter() {
            if t.credit_account.id == account_id {
                balance += t.amount;
            }
            if t.debit_account.id == account_id {
                balance -= t.amount;
            }
        }
        balance
    }

    pub fn get_position(&self, account_id: Uuid) -> Decimal {
        let transfers = self.transfers.read();
        let mut position = Decimal::ZERO;
        for t in transfers.iter() {
            if t.credit_account.id == account_id {
                position += t.amount * t.unit_price;
            }
            if t.debit_account.id == account_id {
                position -= t.amount * t.unit_price;
            }
        }
        position
    }

    pub fn get_transfers(&self) -> Vec<Arc<Transfer>> {
        let lock = self.transfers.read();
        lock.iter().cloned().collect()
    }

    pub fn transfer(
        &self,
        debit_account: Arc<Account>,
        credit_account: Arc<Account>,
        amount: Decimal,
    ) -> Result<(), AccountingError> {
        let transfer = Arc::new(
            Transfer::builder()
                .debit_account(debit_account)
                .credit_account(credit_account)
                .amount(amount)
                .transfer_type(TransferType::Deposit)
                .unit_price(Decimal::ONE)
                .build(),
        );
        self.apply_transfers(&[transfer])
    }

    pub fn exchange(
        &self,
        debit_account: Arc<Account>,
        credit_account: Arc<Account>,
        venue_debit_account: Arc<Account>,
        venue_credit_account: Arc<Account>,
        debit_amount: Decimal,
        credit_amount: Decimal,
    ) -> Result<(), AccountingError> {
        let transfer_group_id = Uuid::new_v4();
        let debit_unit_price = debit_amount / credit_amount;
        let credit_unit_price = credit_amount / debit_amount;

        let t1 = Arc::new(
            Transfer::builder()
                .transfer_group_id(transfer_group_id)
                .debit_account(debit_account)
                .credit_account(venue_debit_account)
                .amount(debit_amount)
                .transfer_type(TransferType::Exchange)
                .unit_price(debit_unit_price)
                .build(),
        );
        let t2 = Arc::new(
            Transfer::builder()
                .transfer_group_id(transfer_group_id)
                .debit_account(venue_credit_account)
                .credit_account(credit_account)
                .amount(credit_amount)
                .transfer_type(TransferType::Exchange)
                .unit_price(credit_unit_price)
                .build(),
        );

        self.apply_transfers(&[t1, t2])
    }

    pub fn margin_trade(
        &self,
        margin_debit_account: Arc<Account>,
        margin_credit_account: Arc<Account>,
        instrument_debit_account: Arc<Account>,
        instrument_credit_account: Arc<Account>,
        commission_debit_account: Arc<Account>,
        commission_credit_account: Arc<Account>,
        margin_amount: Decimal,
        instrument_amount: Decimal,
        instrument_unit_price: Decimal,
        commission_amount: Decimal,
    ) -> Result<(), AccountingError> {
        let transfer_group_id = Uuid::new_v4();

        let t1 = Arc::new(
            Transfer::builder()
                .transfer_group_id(transfer_group_id)
                .debit_account(margin_debit_account)
                .credit_account(margin_credit_account)
                .amount(margin_amount)
                .transfer_type(TransferType::Margin)
                .unit_price(Decimal::ONE)
                .build(),
        );
        let t2 = Arc::new(
            Transfer::builder()
                .transfer_group_id(transfer_group_id)
                .debit_account(commission_debit_account)
                .credit_account(commission_credit_account)
                .amount(commission_amount)
                .transfer_type(TransferType::Commission)
                .unit_price(Decimal::ONE)
                .build(),
        );
        let t3 = Arc::new(
            Transfer::builder()
                .transfer_group_id(transfer_group_id)
                .debit_account(instrument_debit_account)
                .credit_account(instrument_credit_account)
                .amount(instrument_amount)
                .transfer_type(TransferType::Trade)
                .unit_price(instrument_unit_price)
                .build(),
        );
        self.apply_transfers(&[t1, t2, t3])
    }

    /// Performs one or more same-currency transfers **atomically**:
    /// - All succeed or all fail if any validation fails (e.g. insufficient balance).
    /// - For double-entry: each Transfer has a `debit_account_id` and `credit_account_id`.
    ///
    /// Returns an error if any of the transfers are invalid.
    fn apply_transfers(&self, transfers: &[Arc<Transfer>]) -> Result<(), AccountingError> {
        for t in transfers {
            // Check if it is not the same account
            if t.debit_account.id == t.credit_account.id {
                return Err(AccountingError::SameAccount(t.clone()));
            }

            // Check for currency mismatch
            if t.debit_account.asset != t.credit_account.asset {
                return Err(AccountingError::CurrencyMismatch(t.clone()));
            }

            // Check for insufficient balance on exchange wallets
            if matches!(t.debit_account.account_type, AccountType::VenueAccount(_)) {
                if self.get_balance(t.debit_account.id) < t.amount {
                    return Err(AccountingError::InsufficientBalance(t.clone()));
                }
            }

            // Check for invalid transfer amount
            if t.amount <= dec!(0) {
                return Err(AccountingError::InvalidTransferAmount(t.clone()));
            }
        }

        // 2. All validations passed, apply them in memory.
        // This could potentially be a problem since we are validating before we lock
        let mut tx_log_lock = self.transfers.write();
        for t in transfers {
            info!("Applying transfer: {}", t);
            tx_log_lock.push(t.clone());
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use test_log::test;

//     #[test(tokio::test)]
//     async fn test_sample() {
//         let ledger = Ledger::builder().build();

//         let usdt = test_usdt_asset();
//         let bnb = test_bnb_asset();
//         let perp_btc_usdt = test_inst_binance_btc_usdt_perp();

//         let strategy = test_strategy();
//     }
// }
