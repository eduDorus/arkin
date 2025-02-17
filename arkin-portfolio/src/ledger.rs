use core::fmt;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Tradable {
    /// A simple asset (spot currency like BTC, USDT, BNB, etc.)
    Asset(Arc<Asset>),

    /// A derivative instrument (future, option, perpetual, etc.)
    Instrument(Arc<Instrument>),
}

impl fmt::Display for Tradable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tradable::Asset(a) => write!(f, "{}", a.symbol),
            Tradable::Instrument(i) => write!(f, "{}", i.symbol),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccountType {
    ExchangeWallet,
    LiquidityProvider,
}

/// Each account references a specific currency.
/// We'll store the balance as a Decimal, but you could use integer
/// amounts of "cents" or "atomic units" for real usage.
#[derive(Debug, Clone, TypedBuilder)]
pub struct Account {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub asset: Tradable,
    #[builder(default = dec!(0))]
    pub balance: Decimal,
    pub account_type: AccountType,
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} {}", self.name, self.balance, self.asset)
    }
}

/// A single same-currency "transfer" in double-entry style.
/// In TigerBeetle's lingo, this is one row in the ledger for a single currency.
#[derive(Debug, Clone, TypedBuilder)]
pub struct Transfer {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    /// The currency must be the same for both `debit_account_id` and `credit_account_id`.
    pub asset: Tradable,
    /// The account that is debited (balance goes down).
    pub debit_account: Account,
    /// The account that is credited (balance goes up).
    pub credit_account: Account,
    /// The amount of this transfer.
    pub amount: Decimal,
    /// If this transfer is linked to another (for cross-currency exchange),
    /// we can store a link ID.
    #[builder(default = false)]
    pub linked: bool,
}

impl fmt::Display for Transfer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} -> {} {}",
            self.id, self.debit_account.name, self.credit_account.name, self.amount
        )
    }
}

/// The in-memory ledger tracks accounts and can apply sets of transfers atomically.
#[derive(Debug, TypedBuilder)]
pub struct Ledger {
    // For simplicity, we'll hold accounts in a HashMap
    #[builder(default = Mutex::new(HashMap::new()))]
    accounts: Mutex<HashMap<Uuid, Account>>,
    // We store completed transfers here, or you could store them in a DB, etc.
    #[builder(default = Mutex::new(Vec::new()))]
    transfers: Mutex<Vec<Transfer>>,
}

impl Ledger {
    /// Adds an account to the ledger, returns its ID.
    pub fn add_account(&self, name: &str, asset: Tradable, balance: Decimal, account_type: AccountType) -> Account {
        let mut lock = self.accounts.lock().unwrap();
        let account = Account::builder()
            .name(name.into())
            .asset(asset)
            .balance(balance)
            .account_type(account_type)
            .build();
        let id = account.id;
        lock.insert(id, account.clone());
        account
    }

    /// Retrieves an account (immutable) for debugging or inspection.
    pub fn get_account(&self, account_id: Uuid) -> Option<Account> {
        let lock = self.accounts.lock().unwrap();
        lock.get(&account_id).cloned()
    }

    /// Performs one or more same-currency transfers **atomically**:
    /// - All succeed or all fail if any validation fails (e.g. insufficient balance).
    /// - For double-entry: each Transfer has a `debit_account_id` and `credit_account_id`.
    ///
    /// Returns an error if any of the transfers are invalid.
    pub fn apply_transfers(&self, transfers: &[Transfer]) -> Result<(), String> {
        let mut accounts_lock = self.accounts.lock().unwrap();
        let mut tx_log_lock = self.transfers.lock().unwrap();

        // 1. Validate all transfers first (currency matching, sufficient balances, etc.)
        for t in transfers {
            let debit_acct = accounts_lock
                .get(&t.debit_account.id)
                .ok_or_else(|| format!("Debit account not found: {}", t.debit_account.name))?;
            let credit_acct = accounts_lock
                .get(&t.credit_account.id)
                .ok_or_else(|| format!("Credit account not found: {}", t.credit_account.name))?;

            if debit_acct.asset != credit_acct.asset || debit_acct.asset != t.asset {
                return Err(format!("Currency mismatch in transfer: {:?}", t));
            }
            if !debit_acct.account_type.eq(&AccountType::LiquidityProvider) {
                if debit_acct.balance < t.amount {
                    return Err(format!("Insufficient balance in debit account: {:?}", t));
                }
            }
            if t.amount <= dec!(0) {
                return Err(format!("Transfer amount must be > 0: {:?}", t));
            }
        }

        // 2. All validations passed, apply them in memory.
        for t in transfers {
            // Debit side
            let debit_acct = accounts_lock.get_mut(&t.debit_account.id).unwrap();
            debit_acct.balance -= t.amount;
            // Credit side
            let credit_acct = accounts_lock.get_mut(&t.credit_account.id).unwrap();
            credit_acct.balance += t.amount;

            // Record the transfer
            info!("Applying transfer: {}", t);
            tx_log_lock.push(t.clone());
        }

        Ok(())
    }

    /// A convenience helper for single transfer.
    pub fn transfer(
        &self,
        asset: Tradable,
        debit_account: Account,
        credit_account: Account,
        amount: Decimal,
        linked: bool,
    ) -> Result<(), String> {
        let t = Transfer::builder()
            .asset(asset)
            .debit_account(debit_account)
            .credit_account(credit_account)
            .amount(amount)
            .linked(linked)
            .build();
        self.apply_transfers(&[t])
    }

    /// Returns a snapshot of all transfers for debugging.
    pub fn list_transfers(&self) -> Vec<Transfer> {
        self.transfers.lock().unwrap().clone()
    }

    /// Return the current ledger balances in assets
    pub fn balances(&self) -> HashMap<Arc<Asset>, Decimal> {
        let accounts_lock = self.accounts.lock().unwrap();
        let mut balances = HashMap::new();
        for account in accounts_lock.values() {
            if account.account_type.eq(&AccountType::LiquidityProvider) {
                continue;
            }
            if let Tradable::Asset(asset) = &account.asset {
                balances.insert(asset.clone(), account.balance);
            }
        }
        balances
    }

    /// Return the current ledger positions in instruments
    pub fn positions(&self) -> HashMap<Arc<Instrument>, Decimal> {
        let accounts_lock = self.accounts.lock().unwrap();
        let mut positions = HashMap::new();
        for account in accounts_lock.values() {
            if account.account_type.eq(&AccountType::LiquidityProvider) {
                continue;
            }
            if let Tradable::Instrument(instrument) = &account.asset {
                positions.insert(instrument.clone(), account.balance);
            }
        }
        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use test_log::test;
    use tracing::info;

    #[test(tokio::test)]
    async fn example_exchange() -> Result<(), String> {
        // Create the ledger
        let ledger = Ledger::builder().build();

        let btc = Tradable::Asset(test_btc_asset());
        let bnb = Tradable::Asset(test_bnb_asset());
        let usdt = Tradable::Asset(test_usdt_asset());
        let btc_usdt = Tradable::Instrument(test_inst_binance_btc_usdt_perp());

        // Create some accounts
        let a_btc = ledger.add_account("Alice-BTC", btc.clone(), dec!(0), AccountType::ExchangeWallet); // Enough for the example
        let a_bnb = ledger.add_account("Alice-BNB", bnb.clone(), dec!(5), AccountType::ExchangeWallet); // Enough for the example
        let a_usdt = ledger.add_account("Alice-USDT", usdt.clone(), dec!(100_000), AccountType::ExchangeWallet);
        let a_btc_usdt = ledger.add_account("Alice-BTC-USDT", btc_usdt.clone(), dec!(0), AccountType::ExchangeWallet);

        // Liquidity provider accounts
        let l_btc = ledger.add_account("Binance-BTC", btc.clone(), dec!(0), AccountType::LiquidityProvider);
        let l_bnb = ledger.add_account("Binance-BNB", bnb.clone(), dec!(0), AccountType::LiquidityProvider);
        let l_usdt = ledger.add_account("Binance-USDT", usdt.clone(), dec!(0), AccountType::LiquidityProvider);
        let l_btc_usdt =
            ledger.add_account("Binance-BTC-USDT", btc_usdt.clone(), dec!(0), AccountType::LiquidityProvider);

        // Let's say we want to send $100 from A1 (USD) to A2 (INR).
        // Exchange rate: $1.00 = ₹82.42135 => $100 = ₹8242.135
        // We'll multiply by 100 for "integer" style amounts if we want but let's keep decimals simple.

        let usdt_amount = dec!(46200);
        let btc_amount = dec!(0.1);

        // We create two same-currency transfers, "linked" together for atomic exchange:

        // 1) Transfer T1: from A1 to L1 (USD)
        //    Debits A1(USD), credits L1(USD).
        //    linked = true for the first part
        let t1 = Transfer::builder()
            .asset(usdt.clone())
            .debit_account(a_usdt.clone())
            .credit_account(l_usdt.clone())
            .amount(usdt_amount)
            .linked(true)
            .build();

        // 2) Transfer T2: from L2 to A2 (INR)
        //    Debits L2(INR), credits A2(INR).
        //    linked = false for second
        let t2 = Transfer::builder()
            .asset(btc.clone())
            .debit_account(l_btc.clone())
            .credit_account(a_btc.clone())
            .amount(btc_amount)
            .build();

        // We apply them as a batch atomically:
        ledger.apply_transfers(&[t1, t2])?;

        // Now let's see final balances:
        let a_usdt_acct = ledger.get_account(a_usdt.id).unwrap();
        let a_btc_acct = ledger.get_account(a_btc.id).unwrap();
        let l_usdt_acct = ledger.get_account(l_usdt.id).unwrap();
        let l_btc_acct = ledger.get_account(l_btc.id).unwrap();

        info!("Transaction 1:");
        info!("A1 (USDT) after exchange: {}", a_usdt_acct);
        info!("A2 (BTC) after exchange: {}", a_btc_acct);
        info!("L1 (USDT) after exchange: {}", l_usdt_acct);
        info!("L2 (BTC) after exchange: {}", l_btc_acct);
        info!("");

        // Now do a "buy 1 BTC-PERP contract" for 2000 USDT margin
        let t1 = Transfer::builder()
            .asset(usdt.clone())
            .debit_account(a_usdt.clone())
            .credit_account(l_usdt.clone())
            .amount(dec!(4000))
            .build();

        // We pay the trading fees in BNB
        let t2 = Transfer::builder()
            .asset(bnb.clone())
            .debit_account(a_bnb.clone())
            .credit_account(l_bnb.clone())
            .amount(dec!(0.1))
            .build();

        let t3 = Transfer::builder()
            .asset(btc_usdt.clone())
            .debit_account(l_btc_usdt.clone())
            .credit_account(a_btc_usdt.clone())
            .amount(dec!(2))
            .build();

        ledger.apply_transfers(&[t1, t2, t3])?;

        // Now let's see final balances:
        let a_usdt_acct = ledger.get_account(a_usdt.id).unwrap();
        let a_bnb_acct = ledger.get_account(a_bnb.id).unwrap();
        let a_btc_usdt_acct = ledger.get_account(a_btc_usdt.id).unwrap();
        let l_usdt_acct = ledger.get_account(l_usdt.id).unwrap();
        let l_bnb_acct = ledger.get_account(l_bnb.id).unwrap();
        let l_btc_usdt_acct = ledger.get_account(l_btc_usdt.id).unwrap();

        info!("Transaction 2:");
        info!("A (USDT) after exchange: {}", a_usdt_acct);
        info!("A (BNB) after exchange: {}", a_bnb_acct);
        info!("A (BTC-USDT) after exchange: {}", a_btc_usdt_acct);
        info!("L (USDT) after exchange: {}", l_usdt_acct);
        info!("L (BNB) after exchange: {}", l_bnb_acct);
        info!("L (BTC-USDT) after exchange: {}", l_btc_usdt_acct);

        // List balances
        let balances = ledger.balances();
        info!("Balances:");
        for (asset, balance) in balances {
            info!("{}: {}", asset.symbol, balance);
        }

        // List positions
        let positions = ledger.positions();
        info!("Positions:");
        for (instrument, position) in positions {
            info!("{}: {}", instrument.symbol, position);
        }
        Ok(())
    }
}
