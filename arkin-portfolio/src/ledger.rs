use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use strum::Display;
use time::OffsetDateTime;
use tracing::info;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::PortfolioError;

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

#[derive(Debug, Display, Clone, PartialEq)]
pub enum AccountType {
    ExchangeWallet,
    Strategy(Arc<Strategy>),
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
        write!(f, "{}: {} {} {}", self.name, self.balance, self.asset, self.account_type)
    }
}

#[derive(Debug, Display, Clone, PartialEq)]
pub enum TransferType {
    Deposit,
    Withdrawal,
    Trade,
    Exchange,
    Margin,
    Fee,
    Interest,
    Funding,
    Settlement,
    Liquidation,
    Rebate,
    Adjustment,
    Other,
}

/// A single same-currency "transfer" in double-entry style.
/// In TigerBeetle's lingo, this is one row in the ledger for a single currency.
#[derive(Debug, Clone, TypedBuilder)]
pub struct Transfer {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    /// The event time of this transfer.
    #[builder(default = OffsetDateTime::now_utc())]
    pub event_time: OffsetDateTime,
    /// The currency must be the same for both `debit_account_id` and `credit_account_id`.
    pub asset: Tradable,
    /// The account that is debited (balance goes down).
    pub debit_account: Account,
    /// The account that is credited (balance goes up).
    pub credit_account: Account,
    /// The amount of this transfer.
    pub amount: Decimal,
    /// Transfer type (e.g. deposit, withdrawal, trade, etc.)
    pub transfer_type: TransferType,
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
            self.transfer_type, self.debit_account.name, self.credit_account.name, self.amount
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
    pub fn apply_transfers(&self, transfers: &[Transfer]) -> Result<(), PortfolioError> {
        let mut accounts_lock = self.accounts.lock().unwrap();
        let mut tx_log_lock = self.transfers.lock().unwrap();

        // 1. Validate all transfers first (currency matching, sufficient balances, etc.)
        for t in transfers {
            let debit_acct = accounts_lock
                .get(&t.debit_account.id)
                .ok_or(PortfolioError::DebitAccountNotFound(t.debit_account.name.clone()))?;
            let credit_acct = accounts_lock
                .get(&t.credit_account.id)
                .ok_or(PortfolioError::CreditAccountNotFound(t.credit_account.name.clone()))?;

            if debit_acct.asset != credit_acct.asset || debit_acct.asset != t.asset {
                return Err(PortfolioError::CurrencyMismatch(t.clone()));
            }
            if !debit_acct.account_type.eq(&AccountType::LiquidityProvider) {
                if debit_acct.balance < t.amount {
                    return Err(PortfolioError::InsufficientBalance(t.clone()));
                }
            }
            if t.amount <= dec!(0) {
                return Err(PortfolioError::InvalidTransferAmount(t.clone()));
            }
        }

        // 2. All validations passed, apply them in memory.
        for t in transfers {
            // Debit side
            let debit_acct = accounts_lock
                .get_mut(&t.debit_account.id)
                .ok_or(PortfolioError::DebitAccountNotFound(t.debit_account.name.clone()))?;
            debit_acct.balance -= t.amount;
            // Credit side
            let credit_acct = accounts_lock
                .get_mut(&t.credit_account.id)
                .ok_or(PortfolioError::CreditAccountNotFound(t.credit_account.name.clone()))?;
            credit_acct.balance += t.amount;

            // Record the transfer
            info!("Applying transfer: {}", t);
            tx_log_lock.push(t.clone());
        }

        Ok(())
    }

    /// Returns a snapshot of all transfers for debugging.
    pub fn list_transfers(&self) -> Vec<Transfer> {
        self.transfers.lock().unwrap().clone()
    }

    /// Print all accounts for debugging.
    pub fn list_accounts(&self) {
        let accounts_lock = self.accounts.lock().unwrap();
        info!("---------------- Accounts ----------------");
        for account in accounts_lock.values() {
            if account.account_type.eq(&AccountType::LiquidityProvider) {
                continue;
            }
            info!("{}", account);
        }
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

    /// Return the strategy positions
    pub fn strategy_positions(&self, strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal> {
        let strategy_account = AccountType::Strategy(strategy.clone());
        let accounts_lock = self.accounts.lock().unwrap();
        let mut positions = HashMap::new();
        for account in accounts_lock.values() {
            if !account.account_type.eq(&strategy_account) {
                continue;
            }
            if let Tradable::Instrument(instrument) = &account.asset {
                positions.insert(instrument.clone(), account.balance);
            }
        }
        positions
    }

    /// Processes the last fill from a venue order, generating relevant ledger transfers.
    ///
    /// Arguments:
    /// - `order`: The VenueOrder with last_fill_price, last_fill_quantity, last_fill_commission, etc.
    /// - `user_quote_acct`: The user's quote currency account (e.g. USDT).
    /// - `exchange_quote_acct`: The exchange's quote currency account (liquidity provider).
    /// - `user_instrument_acct`: The user's instrument account (e.g. BTC-USDT).
    /// - `exchange_instrument_acct`: The exchange's instrument account (liquidity provider).
    /// - `user_fee_acct`: The user's account for paying fees (could be same as quote, or e.g. BNB).
    /// - `exchange_fee_acct`: The exchange's fee-asset account (liquidity provider).
    ///
    /// Returns `Ok(())` or an error if insufficient balance or mismatch.
    pub fn process_venue_order_fill(
        &self,
        order: &VenueOrder,
        user_quote_acct: &Account,
        exchange_quote_acct: &Account,
        user_instrument_acct: &Account,
        exchange_instrument_acct: &Account,
        user_fee_acct: &Account,
        exchange_fee_acct: &Account,
    ) -> Result<(), PortfolioError> {
        // 1) We only do something if there's a last_fill_quantity > 0
        if order.last_fill_quantity <= dec!(0) {
            return Ok(());
        }

        // For clarity:
        let fill_price = order.last_fill_price;
        let fill_qty = order.last_fill_quantity;
        let commission = order.last_fill_commission; // e.g. 0.1 BNB or 5 USDT
        let side = order.side;

        // 2) Build the margin/cost transfer for the fill in the quote currency
        //    e.g. user -> exchange if it's a buy, or exchange -> user if it's a sell
        let cost = fill_price * fill_qty; // ignoring any contract multiplier for simplicity
        let margin_transfer_type = TransferType::Margin; // or Trade, up to your design

        let margin_transfer = if side == MarketSide::Buy {
            // user_quote_acct => exchange_quote_acct
            Transfer::builder()
                .asset(user_quote_acct.asset.clone()) // must match user_quote_acct & exch_quote_acct
                .debit_account(user_quote_acct.clone())
                .credit_account(exchange_quote_acct.clone())
                .amount(cost)
                .transfer_type(margin_transfer_type)
                .build()
        } else {
            // Sell: user receives the quote; exchange pays the user
            Transfer::builder()
                .asset(user_quote_acct.asset.clone())
                .debit_account(exchange_quote_acct.clone())
                .credit_account(user_quote_acct.clone())
                .amount(cost)
                .transfer_type(margin_transfer_type)
                .build()
        };

        // 3) Build the "position" or "instrument" transfer
        //    e.g. exchange_instrument -> user_instrument if it's a buy
        //         user_instrument -> exchange_instrument if it's a sell
        let trade_transfer_type = TransferType::Trade;

        let trade_transfer = if side == MarketSide::Buy {
            // user obtains the instrument
            Transfer::builder()
                .asset(user_instrument_acct.asset.clone())
                .debit_account(exchange_instrument_acct.clone())
                .credit_account(user_instrument_acct.clone())
                .amount(fill_qty)
                .transfer_type(trade_transfer_type)
                .build()
        } else {
            // user_instrument -> exchange_instrument
            Transfer::builder()
                .asset(user_instrument_acct.asset.clone())
                .debit_account(user_instrument_acct.clone())
                .credit_account(exchange_instrument_acct.clone())
                .amount(fill_qty)
                .transfer_type(trade_transfer_type)
                .build()
        };

        // 4) Build a fee transfer if commission > 0
        //    user_fee_acct => exchange_fee_acct
        //    (Both accounts should have the same asset, e.g. BNB or USDT)
        let mut all_transfers = vec![margin_transfer, trade_transfer];

        if commission > dec!(0) {
            let fee_transfer = Transfer::builder()
                .asset(user_fee_acct.asset.clone())
                .debit_account(user_fee_acct.clone())
                .credit_account(exchange_fee_acct.clone())
                .amount(commission)
                .transfer_type(TransferType::Fee)
                .build();
            all_transfers.push(fee_transfer);
        }

        // 5) Apply them atomically
        self.apply_transfers(&all_transfers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;
    use tracing::info;

    #[test(tokio::test)]
    async fn example_exchange() -> Result<(), PortfolioError> {
        // Create the ledger
        let ledger = Ledger::builder().build();

        let btc = Tradable::Asset(test_btc_asset());
        let bnb = Tradable::Asset(test_bnb_asset());
        let usdt = Tradable::Asset(test_usdt_asset());
        let btc_usdt = Tradable::Instrument(test_inst_binance_btc_usdt_perp());
        let eth_usdt = Tradable::Instrument(test_inst_binance_eth_usdt_perp());

        // Create some accounts
        let a_btc = ledger.add_account("Main-BTC", btc.clone(), dec!(0), AccountType::ExchangeWallet); // Enough for the example
        let a_bnb = ledger.add_account("Main-BNB", bnb.clone(), dec!(5), AccountType::ExchangeWallet); // Enough for the example
        let a_usdt = ledger.add_account("Main-USDT", usdt.clone(), dec!(100_000), AccountType::ExchangeWallet);
        let s_1_btc_usdt = ledger.add_account(
            "Strategy-1-BTC-USDT",
            btc_usdt.clone(),
            dec!(0),
            AccountType::Strategy(test_strategy()),
        );
        let s_2_btc_usdt = ledger.add_account(
            "Strategy-2-BTC-USDT",
            btc_usdt.clone(),
            dec!(0),
            AccountType::Strategy(test_strategy_crossover()),
        );
        let s_2_eth_usdt = ledger.add_account(
            "Strategy-2-ETH-USDT",
            eth_usdt.clone(),
            dec!(0),
            AccountType::Strategy(test_strategy_crossover()),
        );

        // Liquidity provider accounts
        let l_btc = ledger.add_account("Binance-BTC", btc.clone(), dec!(0), AccountType::LiquidityProvider);
        let l_bnb = ledger.add_account("Binance-BNB", bnb.clone(), dec!(0), AccountType::LiquidityProvider);
        let l_usdt = ledger.add_account("Binance-USDT", usdt.clone(), dec!(0), AccountType::LiquidityProvider);
        let l_btc_usdt =
            ledger.add_account("Binance-BTC-USDT", btc_usdt.clone(), dec!(0), AccountType::LiquidityProvider);
        let l_eth_usdt =
            ledger.add_account("Binance-ETH-USDT", eth_usdt.clone(), dec!(0), AccountType::LiquidityProvider);

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
            .transfer_type(TransferType::Exchange)
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
            .transfer_type(TransferType::Exchange)
            .build();

        // We apply them as a batch atomically:
        ledger.apply_transfers(&[t1, t2])?;

        // Now do a "buy 1 BTC-PERP contract" for 2000 USDT margin
        let t1 = Transfer::builder()
            .asset(usdt.clone())
            .debit_account(a_usdt.clone())
            .credit_account(l_usdt.clone())
            .amount(dec!(4000))
            .transfer_type(TransferType::Margin)
            .build();

        // We pay the trading fees in BNB
        let t2 = Transfer::builder()
            .asset(bnb.clone())
            .debit_account(a_bnb.clone())
            .credit_account(l_bnb.clone())
            .amount(dec!(0.1))
            .transfer_type(TransferType::Fee)
            .build();

        let t3 = Transfer::builder()
            .asset(btc_usdt.clone())
            .debit_account(l_btc_usdt.clone())
            .credit_account(s_1_btc_usdt.clone())
            .amount(dec!(2))
            .transfer_type(TransferType::Trade)
            .build();

        ledger.apply_transfers(&[t1, t2, t3])?;

        // Now do a "buy 1 BTC-PERP contract" for 2000 USDT margin
        let t1 = Transfer::builder()
            .asset(usdt.clone())
            .debit_account(a_usdt.clone())
            .credit_account(l_usdt.clone())
            .amount(dec!(2300))
            .transfer_type(TransferType::Margin)
            .build();

        // We pay the trading fees in BNB
        let t2 = Transfer::builder()
            .asset(bnb.clone())
            .debit_account(a_bnb.clone())
            .credit_account(l_bnb.clone())
            .amount(dec!(0.2))
            .transfer_type(TransferType::Fee)
            .build();

        let t3 = Transfer::builder()
            .asset(eth_usdt.clone())
            .debit_account(l_eth_usdt.clone())
            .credit_account(s_2_eth_usdt.clone())
            .amount(dec!(12))
            .transfer_type(TransferType::Trade)
            .build();

        ledger.apply_transfers(&[t1, t2, t3])?;

        // Now do a "buy 1 BTC-PERP contract" for 2000 USDT margin
        let t1 = Transfer::builder()
            .asset(usdt.clone())
            .debit_account(a_usdt.clone())
            .credit_account(l_usdt.clone())
            .amount(dec!(7000))
            .transfer_type(TransferType::Margin)
            .build();

        // We pay the trading fees in BNB
        let t2 = Transfer::builder()
            .asset(bnb.clone())
            .debit_account(a_bnb.clone())
            .credit_account(l_bnb.clone())
            .amount(dec!(0.15))
            .transfer_type(TransferType::Fee)
            .build();

        let t3 = Transfer::builder()
            .asset(btc_usdt.clone())
            .debit_account(l_btc_usdt.clone())
            .credit_account(s_2_btc_usdt.clone())
            .amount(dec!(0.5))
            .transfer_type(TransferType::Trade)
            .build();

        ledger.apply_transfers(&[t1, t2, t3])?;

        // Now let's see final balances:
        ledger.list_accounts();

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

        // List strategy positions
        let strategy = test_strategy();
        let strategy_positions = ledger.strategy_positions(&strategy);
        info!("{}", strategy);
        for (instrument, position) in strategy_positions {
            info!("{}: {}", instrument.symbol, position);
        }

        let strategy = test_strategy_crossover();
        let strategy_positions = ledger.strategy_positions(&strategy);
        info!("{}", strategy);
        for (instrument, position) in strategy_positions {
            info!("{}: {}", instrument.symbol, position);
        }
        Ok(())
    }
}
