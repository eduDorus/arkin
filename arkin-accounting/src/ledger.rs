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
    pub fn add_account(
        &self,
        venue: Arc<Venue>,
        asset: Tradable,
        strategy: Option<Arc<Strategy>>,
        account_type: AccountType,
    ) -> Result<Arc<Account>, AccountingError> {
        // Check if account already exists
        if let Some(account) = self.find_account(&venue, &asset, strategy.as_ref(), &account_type) {
            return Ok(account);
        }

        // Check if we make a strategy account that we have a strategy
        if account_type == AccountType::Strategy && strategy.is_none() {
            return Err(AccountingError::MissingStrategy);
        }

        // Create a new account and insert it into the HashMap
        let account = Arc::new(
            Account::builder()
                .venue(venue)
                .asset(asset.clone())
                .strategy(strategy)
                .account_type(account_type)
                .build(),
        );
        let id = account.id;
        self.accounts.write().insert(id, account.clone());
        // Emit event for persistence
        // self.pubsub.publish(Event::AccountAdded(account.clone()));
        Ok(account)
    }

    pub fn find_account(
        &self,
        venue: &Arc<Venue>,
        asset: &Tradable,
        strategy: Option<&Arc<Strategy>>,
        account_type: &AccountType,
    ) -> Option<Arc<Account>> {
        let accounts = self.accounts.read();
        accounts
            .values()
            .find(|a| {
                a.venue == *venue
                    && a.asset == *asset
                    && a.account_type == *account_type
                    && match strategy {
                        Some(s) => a.strategy.as_ref() == Some(s),
                        None => a.strategy.is_none(),
                    }
            })
            .cloned()
    }

    pub fn find_or_create_account(
        &self,
        venue: &Arc<Venue>,
        asset: &Tradable,
        strategy: Option<&Arc<Strategy>>,
        account_type: &AccountType,
    ) -> Result<Arc<Account>, AccountingError> {
        if let Some(acct) = self.find_account(venue, asset, strategy, account_type) {
            Ok(acct)
        } else {
            self.add_account(venue.clone(), asset.clone(), strategy.cloned(), account_type.clone())
        }
    }

    pub fn account(&self, account_id: Uuid) -> Option<Arc<Account>> {
        let lock = self.accounts.read();
        lock.get(&account_id).cloned()
    }

    pub fn accounts(&self) -> Vec<Arc<Account>> {
        let lock = self.accounts.read();
        lock.values().cloned().collect()
    }

    pub fn balance(&self, account_id: Uuid) -> Decimal {
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

    pub fn position(&self, account_id: Uuid) -> Decimal {
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
        debit_account: &Arc<Account>,
        credit_account: &Arc<Account>,
        amount: Decimal,
    ) -> Result<(), AccountingError> {
        let transfer = Arc::new(
            Transfer::builder()
                .debit_account(debit_account.clone())
                .credit_account(credit_account.clone())
                .amount(amount)
                .transfer_type(TransferType::Deposit)
                .unit_price(Decimal::ONE)
                .build(),
        );
        self.apply_transfers(&[transfer])
    }

    pub fn exchange(
        &self,
        debit_account: &Arc<Account>,
        credit_account: &Arc<Account>,
        venue_debit_account: &Arc<Account>,
        venue_credit_account: &Arc<Account>,
        debit_amount: Decimal,
        credit_amount: Decimal,
    ) -> Result<(), AccountingError> {
        let transfer_group_id = Uuid::new_v4();
        let debit_unit_price = debit_amount / credit_amount;
        let credit_unit_price = credit_amount / debit_amount;

        let t1 = Arc::new(
            Transfer::builder()
                .transfer_group_id(transfer_group_id)
                .debit_account(debit_account.clone())
                .credit_account(venue_debit_account.clone())
                .amount(debit_amount)
                .transfer_type(TransferType::Exchange)
                .unit_price(debit_unit_price)
                .build(),
        );
        let t2 = Arc::new(
            Transfer::builder()
                .transfer_group_id(transfer_group_id)
                .debit_account(venue_credit_account.clone())
                .credit_account(credit_account.clone())
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
        pnl: Decimal,
    ) -> Result<(), AccountingError> {
        let transfer_group_id = Uuid::new_v4();

        let t1 = Arc::new(
            Transfer::builder()
                .transfer_group_id(transfer_group_id)
                .debit_account(margin_debit_account.clone())
                .credit_account(margin_credit_account.clone())
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
        match pnl {
            pnl if pnl > dec!(0) => {
                let t4 = Arc::new(
                    Transfer::builder()
                        .transfer_group_id(transfer_group_id)
                        .debit_account(margin_debit_account)
                        .credit_account(margin_credit_account)
                        .amount(pnl)
                        .transfer_type(TransferType::PnL)
                        .unit_price(Decimal::ONE)
                        .build(),
                );
                self.apply_transfers(&[t1, t2, t3, t4])
            }
            pnl if pnl < dec!(0) => {
                let t4 = Arc::new(
                    Transfer::builder()
                        .transfer_group_id(transfer_group_id)
                        .debit_account(margin_credit_account)
                        .credit_account(margin_debit_account)
                        .amount(pnl.abs())
                        .transfer_type(TransferType::PnL)
                        .unit_price(Decimal::ONE)
                        .build(),
                );
                self.apply_transfers(&[t1, t2, t3, t4])
            }
            _ => self.apply_transfers(&[t1, t2, t3]),
        }
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
        let account_l = ledger
            .add_account(personal_venue.clone(), usdt.clone().into(), None, AccountType::VenueSpot)
            .unwrap();

        // Create two Strategy accounts for USD
        let account_a = ledger
            .add_account(binance_venue.clone(), usdt.clone().into(), None, AccountType::ClientSpot)
            .unwrap();
        let account_b = ledger
            .add_account(binance_venue.clone(), usdt.clone().into(), None, AccountType::ClientMargin)
            .unwrap();

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

        let account_l = ledger
            .add_account(personal_venue.clone(), usdt.clone().into(), None, AccountType::VenueSpot)
            .unwrap();
        let account_a = ledger
            .add_account(binance_venue.clone(), usdt.clone().into(), None, AccountType::ClientSpot)
            .unwrap();

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

        let account_a = ledger
            .add_account(binance_venue.clone(), usdt.clone().into(), None, AccountType::ClientSpot)
            .unwrap();
        let account_b = ledger
            .add_account(binance_venue.clone(), usdt.clone().into(), None, AccountType::ClientMargin)
            .unwrap();

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

        let account_usd = ledger
            .add_account(binance_venue.clone(), usdt.clone().into(), None, AccountType::ClientSpot)
            .unwrap();
        let account_btc = ledger
            .add_account(binance_venue.clone(), btc.clone().into(), None, AccountType::ClientSpot)
            .unwrap();

        let result = ledger.transfer(&account_usd, &account_btc, dec!(100));
        assert!(matches!(result, Err(AccountingError::CurrencyMismatch(_))));
    }

    #[test]
    fn test_same_account() {
        let ledger = Ledger::builder().build();
        let binance_venue = test_binance_venue();
        let usdt = test_usdt_asset();

        let account_a = ledger
            .add_account(binance_venue.clone(), usdt.clone().into(), None, AccountType::ClientSpot)
            .unwrap();
        let result = ledger.transfer(&account_a, &account_a, dec!(100));
        assert!(matches!(result, Err(AccountingError::SameAccount(_))));
    }

    #[test]
    fn test_successful_margin_trade() {
        let ledger = Ledger::builder().build();
        let personal_venue = test_personal_venue();
        let binance_venue = test_binance_venue();
        let usdt = test_usdt_asset();
        let strategy = test_strategy();
        let btc_usdt = test_inst_binance_btc_usdt_perp();

        let personal_usdt = ledger
            .add_account(personal_venue.clone(), usdt.clone().into(), None, AccountType::VenueSpot)
            .unwrap();
        let client_margin_usdt = ledger
            .add_account(binance_venue.clone(), usdt.clone().into(), None, AccountType::ClientMargin)
            .unwrap();
        let client_strategy = ledger
            .add_account(
                binance_venue.clone(),
                btc_usdt.clone().into(),
                Some(strategy),
                AccountType::Strategy,
            )
            .unwrap();
        let venue_spot_usdt = ledger
            .add_account(binance_venue.clone(), usdt.clone().into(), None, AccountType::VenueSpot)
            .unwrap();
        let venue_margin_usdt = ledger
            .add_account(binance_venue.clone(), usdt.clone().into(), None, AccountType::VenueMargin)
            .unwrap();
        let venue_btc_usdt = ledger
            .add_account(binance_venue.clone(), btc_usdt.clone().into(), None, AccountType::VenueSpot)
            .unwrap();

        let leverage = dec!(1);
        let starting_amount = dec!(10000);
        let t1_instrument_amount = dec!(1);
        let t1_instrument_unit_price = dec!(5000);
        let t1_position_amount = t1_instrument_amount * t1_instrument_unit_price;
        let t1_margin_amount = t1_instrument_amount * t1_instrument_unit_price / leverage;
        let t1_commission_amount = dec!(10);

        ledger.transfer(&personal_usdt, &client_margin_usdt, starting_amount).unwrap();
        ledger
            .margin_trade(
                client_margin_usdt.clone(),
                venue_margin_usdt.clone(),
                venue_btc_usdt.clone(),
                client_strategy.clone(),
                client_margin_usdt.clone(),
                venue_spot_usdt.clone(),
                t1_margin_amount,
                t1_instrument_amount,
                t1_instrument_unit_price,
                t1_commission_amount,
                dec!(0),
            )
            .unwrap();

        assert_eq!(
            ledger.balance(client_margin_usdt.id),
            starting_amount - t1_margin_amount - t1_commission_amount
        );
        assert_eq!(ledger.balance(venue_margin_usdt.id), t1_margin_amount);
        assert_eq!(ledger.balance(venue_spot_usdt.id), t1_commission_amount);
        assert_eq!(ledger.balance(venue_btc_usdt.id), -t1_instrument_amount);
        assert_eq!(ledger.balance(client_strategy.id), t1_instrument_amount);
        assert_eq!(ledger.position(client_strategy.id), t1_position_amount);

        let instrument_amount = dec!(0.5);
        let instrument_unit_price = dec!(4500);
        let commission_amount = dec!(10);

        let current_margin_posted = ledger.position(venue_margin_usdt.id);
        let current_instrument_amount = ledger.balance(client_strategy.id);
        let diff_margin = current_margin_posted * (instrument_amount / current_instrument_amount);

        let current_position = ledger.position(client_strategy.id);
        let avg_price = current_position / current_instrument_amount;
        let diff_price = instrument_unit_price - avg_price;
        let pnl = diff_price * instrument_amount;
        info!("Current PnL: {}", pnl);

        ledger
            .margin_trade(
                venue_margin_usdt.clone(),
                client_margin_usdt.clone(),
                client_strategy.clone(),
                venue_btc_usdt.clone(),
                client_margin_usdt.clone(),
                venue_spot_usdt.clone(),
                diff_margin.abs(),
                instrument_amount,
                instrument_unit_price,
                commission_amount,
                pnl,
            )
            .unwrap();

        // assert_eq!(ledger.balance(venue_margin_usdt.id), diff_margin);
        assert_eq!(ledger.balance(venue_spot_usdt.id), dec!(20));
        assert_eq!(ledger.balance(venue_btc_usdt.id), -dec!(0.5));
        assert_eq!(ledger.balance(client_strategy.id), dec!(0.5));
        // assert_eq!(ledger.position(client_strategy.id), dec!(500));

        let instrument_amount = dec!(0.5);
        let instrument_unit_price = dec!(4000);
        let commission_amount = dec!(10);

        let current_margin_posted = ledger.position(venue_margin_usdt.id);
        let current_instrument_amount = ledger.balance(client_strategy.id);
        let diff_margin = current_margin_posted * (instrument_amount / current_instrument_amount);

        let current_position = ledger.position(client_strategy.id);
        let avg_price = current_position / current_instrument_amount;
        let diff_price = instrument_unit_price - avg_price;
        let pnl = diff_price * instrument_amount;
        info!("Current PnL: {}", pnl);

        ledger
            .margin_trade(
                venue_margin_usdt.clone(),
                client_margin_usdt.clone(),
                client_strategy.clone(),
                venue_btc_usdt.clone(),
                client_margin_usdt.clone(),
                venue_spot_usdt.clone(),
                diff_margin.abs(),
                instrument_amount,
                instrument_unit_price,
                commission_amount,
                pnl,
            )
            .unwrap();

        // assert_eq!(ledger.balance(venue_margin_usdt.id), diff_margin);
        assert_eq!(ledger.balance(venue_spot_usdt.id), dec!(30));
        assert_eq!(ledger.balance(venue_btc_usdt.id), dec!(0));
        assert_eq!(ledger.balance(client_strategy.id), dec!(0));
        // assert_eq!(ledger.position(client_strategy.id), dec!(500));

        for account in ledger.accounts() {
            info!(
                "Account: {}, Balance: {}, Position: {}",
                account,
                ledger.balance(account.id),
                ledger.position(account.id)
            );
        }

        // let transfers = ledger.get_transfers();
        // assert_eq!(transfers.len(), 10);
    }
}
