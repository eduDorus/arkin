use parking_lot::RwLock;
use rust_decimal::prelude::*;
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
    /// Adds an account to the ledger and returns it.
    pub fn add_account(&self, venue: Arc<Venue>, asset: Tradable, account_type: AccountType) -> Arc<Account> {
        match self.find_account(&venue, &asset, &account_type) {
            Some(account) => account,
            None => {
                let account: Arc<Account> = Account::builder()
                    .venue(venue)
                    .asset(asset.clone())
                    .account_type(account_type)
                    .build()
                    .into();
                self.accounts.write().insert(account.id.clone(), account.clone());
                account
            }
        }
    }

    /// Finds an account by venue, asset, and account type.
    pub fn find_account(
        &self,
        venue: &Arc<Venue>,
        asset: &Tradable,
        account_type: &AccountType,
    ) -> Option<Arc<Account>> {
        let accounts = self.accounts.read();
        accounts
            .values()
            .find(|a| a.venue == *venue && a.asset == *asset && a.account_type == *account_type)
            .cloned()
    }

    /// Finds an account by venue, asset, and account type, or creates it if it doesn't exist.
    pub fn find_or_create(&self, venue: &Arc<Venue>, asset: &Tradable, account_type: &AccountType) -> Arc<Account> {
        match self.find_account(venue, asset, account_type) {
            Some(account) => account,
            None => self.add_account(venue.clone(), asset.clone(), account_type.clone()),
        }
    }

    /// Returns an account by ID.
    pub fn account(&self, account_id: Uuid) -> Option<Arc<Account>> {
        let lock = self.accounts.read();
        lock.get(&account_id).cloned()
    }

    /// Returns all accounts in the ledger.
    pub fn accounts(&self) -> Vec<Arc<Account>> {
        let lock = self.accounts.read();
        lock.values().cloned().collect()
    }

    /// Returns the global balance for an account.
    /// This is the sum of all debit and credit transfers from the account.
    pub fn balance(&self, account_id: Uuid) -> Decimal {
        let transfers = self.transfers.read();
        transfers.iter().fold(Decimal::ZERO, |acc, t| {
            if t.credit_account.id == account_id {
                acc + t.amount
            } else if t.debit_account.id == account_id {
                acc - t.amount
            } else {
                acc
            }
        })
    }

    /// Returns the net position amount for an account and strategy.
    pub fn position_amount(&self, account_id: Uuid, strategy: &Arc<Strategy>) -> Decimal {
        let transfers = self.transfers.read();
        transfers
            .iter()
            .filter(|t| t.transfer_type == TransferType::Trade && t.strategy.as_ref() == Some(strategy))
            .fold(Decimal::ZERO, |acc, t| {
                if t.credit_account.id == account_id {
                    acc + t.amount
                } else if t.debit_account.id == account_id {
                    acc - t.amount
                } else {
                    acc
                }
            })
    }

    /// Returns the net PnL for an account and strategy.
    pub fn pnl(&self, account_id: Uuid, strategy: &Arc<Strategy>) -> Decimal {
        let transfers = self.transfers.read();
        transfers
            .iter()
            .filter(|t| t.transfer_type == TransferType::PnL && t.strategy.as_ref() == Some(strategy))
            .fold(Decimal::ZERO, |acc, t| {
                if t.credit_account.id == account_id {
                    acc + t.amount
                } else if t.debit_account.id == account_id {
                    acc - t.amount
                } else {
                    acc
                }
            })
    }

    /// Returns the total cost basis for the current position.
    /// Returns the average entry price per unit for the current position.
    pub fn cost_basis(&self, account_id: Uuid, strategy: &Arc<Strategy>) -> Decimal {
        let current_position = self.position_amount(account_id, strategy);

        // If the position is zero, return zero
        if current_position == Decimal::ZERO {
            return Decimal::ZERO;
        }

        let transfers = self.transfers.read();
        let mut total_cost = Decimal::ZERO;
        let mut total_amount = Decimal::ZERO;
        let mut running_position = Decimal::ZERO;

        for t in transfers
            .iter()
            .filter(|t| t.transfer_type == TransferType::Trade && t.strategy.as_ref() == Some(strategy))
        {
            let amount = t.amount; // Amount is always positive
            let is_buy = t.debit_account.id == account_id; // Buy: debit to account
            let tx_position_change = if is_buy { amount } else { -amount }; // Buy increases, sell decreases position

            // Current position before this transaction
            let position_before = running_position;
            running_position += tx_position_change;

            if position_before.is_zero() {
                // Starting a new position
                total_cost = amount * t.unit_price;
                total_amount = amount;
            } else if position_before.is_sign_positive() {
                if is_buy {
                    // Adding to long position
                    total_cost += amount * t.unit_price;
                    total_amount += amount;
                } else {
                    // Selling from long position
                    if running_position.is_sign_positive() || running_position.is_zero() {
                        // Still long or flat
                        let avg_cost = total_cost / total_amount;
                        total_cost -= amount * avg_cost;
                        total_amount -= amount;
                    } else {
                        // Crossing from long to short
                        let amount_to_close = position_before; // Amount to reduce to zero
                        let excess_sell = amount - amount_to_close; // Amount that starts short
                        let avg_cost = total_cost / total_amount;
                        total_cost -= amount_to_close * avg_cost;
                        total_amount -= amount_to_close;
                        // Reset and start short position
                        total_cost = excess_sell * t.unit_price;
                        total_amount = excess_sell;
                    }
                }
            } else {
                // position_before is negative (short)
                if !is_buy {
                    // Adding to short position (sell)
                    total_cost += amount * t.unit_price;
                    total_amount += amount;
                } else {
                    // Buying to cover short position
                    if running_position.is_sign_negative() || running_position.is_zero() {
                        // Still short or flat
                        let avg_cost = total_cost / total_amount;
                        total_cost -= amount * avg_cost;
                        total_amount -= amount;
                    } else {
                        // Crossing from short to long
                        let amount_to_close = -position_before; // Amount to cover short
                        let excess_buy = amount - amount_to_close; // Amount that starts long
                        let avg_cost = total_cost / total_amount;
                        total_cost -= amount_to_close * avg_cost;
                        total_amount -= amount_to_close;
                        // Reset and start long position
                        total_cost = excess_buy * t.unit_price;
                        total_amount = excess_buy;
                    }
                }
            }

            // Ensure total_amount doesn't go negative due to rounding
            if total_amount < Decimal::ZERO {
                total_cost = Decimal::ZERO;
                total_amount = Decimal::ZERO;
            }
        }

        info!("Total cost: {}, Total amount: {}", total_cost, total_amount);

        if !total_amount.is_zero() {
            total_cost / total_amount
        } else {
            Decimal::ZERO
        }
    }

    /// Returns the total margin posted for the current position.
    pub fn margin_posted(&self, account_id: Uuid, strategy: &Arc<Strategy>) -> Decimal {
        let transfers = self.transfers.read();
        transfers
            .iter()
            .filter(|t| {
                t.transfer_type == TransferType::Margin && t.strategy.as_ref().map(|s| s.id) == Some(strategy.id)
            })
            .fold(Decimal::ZERO, |acc, t| {
                if t.credit_account.id == account_id {
                    acc + t.amount // Margin posted to venue
                } else if t.debit_account.id == account_id {
                    acc - t.amount // Margin released from venue
                } else {
                    acc
                }
            })
    }

    /// Returns all transfers in the ledger.
    /// This can be quite expensive and should only be used for debugging or reporting.
    pub fn get_transfers(&self) -> Vec<Arc<Transfer>> {
        let lock = self.transfers.read();
        lock.iter().cloned().collect()
    }

    /// Performs a single same-currency transfer **atomically**:
    /// This is a helper function since transfers are quite common.
    pub fn transfer(
        &self,
        debit_account: &Arc<Account>,
        credit_account: &Arc<Account>,
        amount: Decimal,
    ) -> Result<(), AccountingError> {
        let transfer = Arc::new(
            Transfer::builder()
                .transfer_group_id(Uuid::new_v4())
                .strategy(None)
                .debit_account(debit_account.clone())
                .credit_account(credit_account.clone())
                .amount(amount)
                .transfer_type(TransferType::Deposit)
                .unit_price(Decimal::ONE)
                .build(),
        );
        self.apply_transfers(&[transfer])
    }

    /// Performs one or more same-currency transfers **atomically**:
    /// - All succeed or all fail if any validation fails (e.g. insufficient balance).
    /// - For double-entry: each Transfer has a `debit_account_id` and `credit_account_id`.
    ///
    /// Returns an error if any of the transfers are invalid.
    pub fn apply_transfers(&self, transfers: &[Arc<Transfer>]) -> Result<(), AccountingError> {
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
            if t.debit_account.account_type == AccountType::ClientSpot
                || t.debit_account.account_type == AccountType::ClientMargin
            {
                if self.balance(t.debit_account.id) < t.amount {
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_successful_transfer() {
        let ledger = Ledger::builder().build();

        let personal_venue = test_personal_venue();
        let binance_venue = test_binance_venue();

        // let strategy = test_strategy();
        let usdt = test_usdt_asset();

        // Create Personal account for USD
        let account_l = ledger.add_account(personal_venue.clone(), usdt.clone().into(), AccountType::VenueSpot);

        // Create two Strategy accounts for USD
        let account_a = ledger.add_account(binance_venue.clone(), usdt.clone().into(), AccountType::ClientSpot);
        let account_b = ledger.add_account(binance_venue.clone(), usdt.clone().into(), AccountType::ClientMargin);

        let amount = dec!(100);
        ledger.transfer(&account_l, &account_a, amount).unwrap();
        ledger.transfer(&account_a, &account_b, amount).unwrap();
        let half_amount = amount / dec!(2);
        ledger.transfer(&account_b, &account_a, half_amount).unwrap();

        // Verify balances
        assert_eq!(ledger.balance(account_l.id), -amount); // -100 USD
        assert_eq!(ledger.balance(account_a.id), half_amount); // +50 USD
        assert_eq!(ledger.balance(account_b.id), half_amount); // +50 USD

        // Verify transfer record
        let transfers = ledger.get_transfers();
        assert_eq!(transfers.len(), 3);
        let t = &transfers[1];
        assert_eq!(t.debit_account.id, account_a.id);
        assert_eq!(t.credit_account.id, account_b.id);
        assert_eq!(t.amount, amount);
        assert_eq!(t.transfer_type, TransferType::Deposit);
        assert_eq!(t.unit_price, dec!(1));
    }

    #[test]
    fn test_insufficient_balance_client_spot() {
        let ledger = Ledger::builder().build();
        let personal_venue = test_personal_venue();
        let binance_venue = test_binance_venue();
        let usdt = test_usdt_asset();

        let account_l = ledger.add_account(personal_venue.clone(), usdt.clone().into(), AccountType::VenueSpot);
        let account_a = ledger.add_account(binance_venue.clone(), usdt.clone().into(), AccountType::ClientSpot);

        ledger.transfer(&account_l, &account_a, dec!(1000)).unwrap();
        let result = ledger.transfer(&account_a, &account_l, dec!(1001));
        assert!(matches!(result, Err(AccountingError::InsufficientBalance(_))));

        assert_eq!(ledger.balance(account_l.id), dec!(-1000));
        assert_eq!(ledger.balance(account_a.id), dec!(1000));
    }

    #[test]
    fn test_invalid_amount() {
        let ledger = Ledger::builder().build();
        let binance_venue = test_binance_venue();
        let usdt = test_usdt_asset();

        let account_a = ledger.add_account(binance_venue.clone(), usdt.clone().into(), AccountType::ClientSpot);
        let account_b = ledger.add_account(binance_venue.clone(), usdt.clone().into(), AccountType::ClientMargin);

        let result_zero = ledger.transfer(&account_a, &account_b, dec!(0));
        assert!(matches!(result_zero, Err(AccountingError::InvalidTransferAmount(_))));

        let result_negative = ledger.transfer(&account_a, &account_b, dec!(-10));
        assert!(matches!(result_negative, Err(AccountingError::InvalidTransferAmount(_))));
    }

    #[test]
    fn test_currency_mismatch() {
        let ledger = Ledger::builder().build();
        let binance_venue = test_binance_venue();
        let usdt = test_usdt_asset();
        let btc = test_btc_asset();

        let account_usd = ledger.add_account(binance_venue.clone(), usdt.clone().into(), AccountType::ClientSpot);
        let account_btc = ledger.add_account(binance_venue.clone(), btc.clone().into(), AccountType::ClientSpot);

        let result = ledger.transfer(&account_usd, &account_btc, dec!(100));
        assert!(matches!(result, Err(AccountingError::CurrencyMismatch(_))));
    }

    #[test]
    fn test_same_account() {
        let ledger = Ledger::builder().build();
        let binance_venue = test_binance_venue();
        let usdt = test_usdt_asset();

        let account_a = ledger.add_account(binance_venue.clone(), usdt.clone().into(), AccountType::ClientSpot);
        let result = ledger.transfer(&account_a, &account_a, dec!(100));
        assert!(matches!(result, Err(AccountingError::SameAccount(_))));
    }
}
