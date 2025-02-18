use parking_lot::RwLock;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::PortfolioError;

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
        let mut lock = self.accounts.write();
        let account = Arc::new(
            Account::builder()
                .name(name.into())
                .asset(asset)
                .account_type(account_type)
                .build(),
        );
        let id = account.id;
        lock.insert(id, account.clone());
        account
    }

    /// Retrieves an account (immutable) for debugging or inspection.
    pub fn get_account(&self, account_id: Uuid) -> Option<Arc<Account>> {
        let lock = self.accounts.read();
        lock.get(&account_id).cloned()
    }

    pub fn get_liquidity_provider_account(
        &self,
        venue: &Arc<Venue>,
        asset: &Tradable,
    ) -> Result<Arc<Account>, PortfolioError> {
        let lock = self.accounts.read();
        for account in lock.values() {
            if account.account_type.eq(&AccountType::LiquidityProvider(venue.clone())) && account.asset.eq(&asset) {
                return Ok(account.clone());
            }
        }
        Err(PortfolioError::LiquidityAccountNotFound(venue.clone(), asset.clone()))
    }

    pub fn get_venue_account(&self, venue: &Arc<Venue>, asset: &Tradable) -> Result<Arc<Account>, PortfolioError> {
        let lock = self.accounts.read();
        for account in lock.values() {
            if account.account_type.eq(&AccountType::ExchangeWallet(venue.clone())) && account.asset.eq(&asset) {
                return Ok(account.clone());
            }
        }
        Err(PortfolioError::VenueAccountNotFound(venue.clone(), asset.clone()))
    }

    pub fn get_strategy_account(
        &self,
        strategy: &Arc<Strategy>,
        asset: &Tradable,
    ) -> Result<Arc<Account>, PortfolioError> {
        let lock = self.accounts.read();
        for account in lock.values() {
            if account.account_type.eq(&AccountType::Strategy(strategy.clone())) && account.asset.eq(&asset) {
                return Ok(account.clone());
            }
        }
        Err(PortfolioError::StrategyAccountNotFound(strategy.clone(), asset.clone()))
    }

    /// Computes current balance for a given account by replaying all transfers.
    pub fn get_balance(&self, account_id: Uuid) -> Result<Decimal, PortfolioError> {
        // We want to sum all credits minus all debits for this account
        let transfers = self.transfers.read();
        let mut balance = Decimal::ZERO;

        for t in transfers.iter() {
            // If this account is credited
            if t.credit_account.id == account_id {
                balance += t.amount;
            }
            // If this account is debited
            if t.debit_account.id == account_id {
                balance -= t.amount;
            }
        }

        Ok(balance)
    }

    /// Return the current ledger balances in assets
    pub fn get_balances(&self) -> HashMap<Tradable, Decimal> {
        let accounts_lock = self.accounts.read();
        let mut balances = HashMap::new();
        for account in accounts_lock.values() {
            if matches!(account.account_type, AccountType::LiquidityProvider(_)) {
                continue;
            }
            let balance = self.get_balance(account.id).unwrap();
            let entry = balances.entry(account.asset.clone()).or_insert(Decimal::ZERO);
            *entry += balance;
        }
        balances
    }

    /// Performs one or more same-currency transfers **atomically**:
    /// - All succeed or all fail if any validation fails (e.g. insufficient balance).
    /// - For double-entry: each Transfer has a `debit_account_id` and `credit_account_id`.
    ///
    /// Returns an error if any of the transfers are invalid.
    pub fn apply_transfers(&self, transfers: &[Arc<Transfer>]) -> Result<(), PortfolioError> {
        for t in transfers {
            // Check for currency mismatch
            if t.debit_account.asset != t.credit_account.asset || t.debit_account.asset != t.asset {
                return Err(PortfolioError::CurrencyMismatch(t.clone()));
            }

            // Check for insufficient balance on exchange wallets
            if matches!(t.debit_account.account_type, AccountType::ExchangeWallet(_)) {
                if self.get_balance(t.debit_account.id)? < t.amount {
                    return Err(PortfolioError::InsufficientBalance(t.clone()));
                }
            }

            // Check for invalid transfer amount
            if t.amount <= dec!(0) {
                return Err(PortfolioError::InvalidTransferAmount(t.clone()));
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

    /// Returns a snapshot of all transfers for debugging.
    pub fn list_transfers(&self) -> Vec<Arc<Transfer>> {
        self.transfers.read().clone()
    }

    /// Print all accounts for debugging.
    pub fn list_accounts(&self) {
        let accounts_lock = self.accounts.read();
        info!("---------------- Accounts ----------------");
        for account in accounts_lock.values() {
            if matches!(account.account_type, AccountType::LiquidityProvider(_)) {
                continue;
            }
            info!("{}", account);
        }
    }

    /// Return the current ledger positions in instruments
    pub fn positions(&self) -> HashMap<Arc<Instrument>, Decimal> {
        let accounts_lock = self.accounts.read();
        let mut positions = HashMap::new();
        for account in accounts_lock.values() {
            if matches!(account.account_type, AccountType::LiquidityProvider(_)) {
                continue;
            }
            if let Tradable::Instrument(instrument) = &account.asset {
                let balance = self.get_balance(account.id).unwrap();
                let entry = positions.entry(instrument.clone()).or_insert(Decimal::ZERO);
                *entry += balance;
            }
        }
        positions
    }

    /// Return the strategy positions
    pub fn strategy_positions(&self, strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal> {
        let strategy_account = AccountType::Strategy(strategy.clone());
        let accounts_lock = self.accounts.read();
        let mut positions = HashMap::new();
        for account in accounts_lock.values() {
            if !account.account_type.eq(&strategy_account) {
                continue;
            }
            if let Tradable::Instrument(instrument) = &account.asset {
                let balance = self.get_balance(account.id).unwrap();
                positions.insert(instrument.clone(), balance);
            }
        }
        positions
    }

    /// Transfer funds into an account deposit
    pub fn deposit(
        &self,
        debit_account: &Arc<Account>,
        credit_account: &Arc<Account>,
        amount: Decimal,
    ) -> Result<(), PortfolioError> {
        info!("Deposit: {} -> {} = {}", debit_account, credit_account, amount);
        let transfer = Transfer::builder()
            .asset(debit_account.asset.clone())
            .debit_account(debit_account.clone())
            .credit_account(credit_account.clone())
            .amount(amount)
            .transfer_type(TransferType::Deposit)
            .build()
            .into();
        self.apply_transfers(&[transfer])
    }

    /// Exchange currency between two accounts
    pub fn exchange(
        &self,
        from_account: &Arc<Account>,
        to_account: &Arc<Account>,
        amount: Decimal,
        rate: Decimal,
    ) -> Result<(), PortfolioError> {
        // Get the venue from the account wallet type
        let venue = match &from_account.account_type {
            AccountType::ExchangeWallet(venue) => venue,
            _ => {
                return Err(PortfolioError::InvalidExchange(
                    "Exchange only possible for exchange wallets".into(),
                ))
            }
        };
        let from_exchange_account = self.get_liquidity_provider_account(venue, &from_account.asset)?;
        let to_exchange_account = self.get_liquidity_provider_account(venue, &to_account.asset)?;
        let exchange_amount = amount / rate;
        info!(
            "Exchange: {} = {} -> {} = {} @ {}",
            from_account.asset, amount, to_account.asset, exchange_amount, rate
        );

        let t1 = Transfer::builder()
            .asset(from_account.asset.clone())
            .debit_account(from_account.clone())
            .credit_account(from_exchange_account.clone())
            .amount(amount)
            .transfer_type(TransferType::Exchange)
            .linked(true)
            .build()
            .into();

        let t2 = Transfer::builder()
            .asset(to_account.asset.clone())
            .debit_account(to_exchange_account.clone())
            .credit_account(to_account.clone())
            .amount(exchange_amount)
            .transfer_type(TransferType::Exchange)
            .build()
            .into();

        self.apply_transfers(&[t1, t2])
    }

    /// Merges "open/close" logic into one function for *isolated margin*
    /// by reading the last fill from a `VenueOrder`.
    ///
    /// - We check old_position vs new_position (in absolute contracts).
    /// - If new_position > old_position, we deposit additional margin.
    /// - If new_position < old_position, we free margin back to the user.
    /// - Commission is deducted from the user's margin account.
    /// - The instrument itself is transferred from venue -> user (buy) or user -> venue (sell).
    ///
    /// `order.instrument.margin_asset` is used as the margin currency (e.g. USDT).
    /// If the order is partial or repeated, call this each time `last_fill_quantity` changes.
    pub fn trade_perpetual_isolated_from_order(&self, order: &Arc<VenueOrder>) -> Result<(), PortfolioError> {
        info!("Trade Perpetual Isolated: {}", order);
        // 1) Basic fields from the order
        let side = order.side;
        let fill_price = order.last_fill_price;
        let fill_qty = order.last_fill_quantity; // how many contracts user is adding/closing
        let commission_amount = order.last_fill_commission;
        let instrument = &order.instrument;

        // If nothing was filled, do nothing
        if fill_qty <= dec!(0) {
            return Ok(());
        }

        // 2) Identify accounts via ledger lookups
        let strategy = &order.strategy; // e.g. user or strategy ID
        let venue = &instrument.venue;

        // Margin is the instrument's margin_asset
        let margin_asset = Tradable::Asset(instrument.margin_asset.clone());

        let user_margin_acct = self.get_venue_account(venue, &margin_asset)?;
        let venue_margin_acct = self.get_liquidity_provider_account(venue, &margin_asset)?;

        // Commission uses the same margin asset by default; if the order has a
        // separate commission_asset, you'd handle it here:
        let commission_asset = match &order.commission_asset {
            Some(asset) => Tradable::Asset(asset.clone()),
            None => margin_asset.clone(),
        };
        let user_commission_acct = self.get_venue_account(venue, &commission_asset)?;
        let venue_commission_acct = self.get_liquidity_provider_account(venue, &commission_asset)?;

        // Instrument accounts
        let user_instrument_acct = self.get_strategy_account(strategy, &Tradable::Instrument(instrument.clone()))?;
        let venue_instrument_acct =
            self.get_liquidity_provider_account(venue, &Tradable::Instrument(instrument.clone()))?;

        // 3) Old position in user_instrument_acct
        let old_position = self.get_balance(user_instrument_acct.id)?;
        // e.g. +2 => user is long 2 contracts; -1 => short 1

        // 4) Calculate new_position
        let new_position = if side == MarketSide::Buy {
            old_position + fill_qty
        } else {
            old_position - fill_qty
        };

        // 5) Compute old vs new margin
        // For a simple approach: margin_required = |position| * fill_price * contract_size
        let contract_size = instrument.contract_size;
        let old_margin_required = old_position.abs() * fill_price * contract_size;
        let new_margin_required = new_position.abs() * fill_price * contract_size;
        let margin_diff = new_margin_required - old_margin_required;

        // 6) Build Commission Transfer
        let commission_transfer = Transfer::builder()
            .asset(commission_asset.clone())
            .debit_account(user_commission_acct.clone())
            .credit_account(venue_commission_acct.clone())
            .amount(commission_amount)
            .transfer_type(TransferType::Commission)
            .build()
            .into();

        // 7) Build Instrument Transfer
        let (instrument_debit_acct, instrument_credit_acct) = if side == MarketSide::Buy {
            // venue -> user
            (venue_instrument_acct.clone(), user_instrument_acct.clone())
        } else {
            // user -> venue
            (user_instrument_acct.clone(), venue_instrument_acct.clone())
        };

        let instrument_transfer = Transfer::builder()
            .asset(Tradable::Instrument(instrument.clone()))
            .debit_account(instrument_debit_acct)
            .credit_account(instrument_credit_acct)
            .amount(fill_qty)
            .transfer_type(TransferType::Trade)
            .build()
            .into();

        // 8) Margin transfers: if margin_diff > 0 => deposit more, if < 0 => free up
        let margin_transfers = if margin_diff > Decimal::ZERO {
            // deposit additional margin
            let t = Transfer::builder()
                .asset(margin_asset.clone())
                .debit_account(user_margin_acct.clone())
                .credit_account(venue_margin_acct.clone())
                .amount(margin_diff)
                .transfer_type(TransferType::Margin)
                .build()
                .into();
            vec![t]
        } else if margin_diff < Decimal::ZERO {
            // free margin back
            let freed_margin = margin_diff.abs();
            let t = Transfer::builder()
                .asset(margin_asset.clone())
                .debit_account(venue_margin_acct.clone())
                .credit_account(user_margin_acct.clone())
                .amount(freed_margin)
                .transfer_type(TransferType::Margin)
                .build()
                .into();
            vec![t]
        } else {
            // no margin change
            vec![]
        };

        // 9) Combine all transfers
        let mut all_transfers = vec![];
        all_transfers.extend(margin_transfers.into_iter());
        all_transfers.push(commission_transfer);
        all_transfers.push(instrument_transfer);

        // 10) Apply them atomically
        self.apply_transfers(&all_transfers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;
    use time::OffsetDateTime;
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

        let binance_venue = test_binance_venue();
        let personal_venue = test_personal_venue();

        let s1 = test_strategy();
        let s2 = test_strategy_crossover();

        // Create some accounts
        let _a_btc = ledger.add_account("Main-BTC", btc.clone(), AccountType::ExchangeWallet(binance_venue.clone())); // Enough for the example
        let a_bnb = ledger.add_account("Main-BNB", bnb.clone(), AccountType::ExchangeWallet(binance_venue.clone())); // Enough for the example
        let a_usdt = ledger.add_account("Main-USDT", usdt.clone(), AccountType::ExchangeWallet(binance_venue.clone())); // Enough for the example
        let _s_1_btc_usdt =
            ledger.add_account("Strategy-1-BTC-USDT", btc_usdt.clone(), AccountType::Strategy(s1.clone()));
        let _s_2_btc_usdt =
            ledger.add_account("Strategy-2-BTC-USDT", btc_usdt.clone(), AccountType::Strategy(s2.clone()));
        let _s_2_eth_usdt =
            ledger.add_account("Strategy-2-ETH-USDT", eth_usdt.clone(), AccountType::Strategy(s2.clone()));

        // Liquidity provider accounts
        let l_personal_usdt = ledger.add_account(
            "Personal-Fund-USDT",
            usdt.clone(),
            AccountType::LiquidityProvider(personal_venue.clone()),
        );
        let _l_btc = ledger.add_account(
            "Binance-BTC",
            btc.clone(),
            AccountType::LiquidityProvider(binance_venue.clone()),
        );
        let _l_bnb = ledger.add_account(
            "Binance-BNB",
            bnb.clone(),
            AccountType::LiquidityProvider(binance_venue.clone()),
        );
        let _l_usdt = ledger.add_account(
            "Binance-USDT",
            usdt.clone(),
            AccountType::LiquidityProvider(binance_venue.clone()),
        );
        let _l_btc_usdt = ledger.add_account(
            "Binance-BTC-USDT",
            btc_usdt.clone(),
            AccountType::LiquidityProvider(binance_venue.clone()),
        );
        let _l_eth_usdt = ledger.add_account(
            "Binance-ETH-USDT",
            eth_usdt.clone(),
            AccountType::LiquidityProvider(binance_venue.clone()),
        );

        // Deposti 100k USDT
        ledger.deposit(&l_personal_usdt, &a_usdt, dec!(100000))?;

        ledger.exchange(&a_usdt, &a_bnb, dec!(5000), dec!(600))?;

        let mut order = VenueOrder::builder()
            .strategy(s2.clone())
            .instrument(test_inst_binance_btc_usdt_perp())
            .side(MarketSide::Buy)
            .quantity(dec!(1.0))
            .price(dec!(50000.0))
            .order_type(VenueOrderType::Market)
            .commission_asset(Some(test_bnb_asset()))
            .build();

        ledger.trade_perpetual_isolated_from_order(&Arc::new(order.clone()))?;

        order.add_fill(OffsetDateTime::now_utc(), dec!(5001), dec!(0.5), dec!(0.1));

        ledger.trade_perpetual_isolated_from_order(&Arc::new(order.clone()))?;

        order.add_fill(OffsetDateTime::now_utc(), dec!(4990), dec!(0.5), dec!(0.13));

        ledger.trade_perpetual_isolated_from_order(&Arc::new(order.clone()))?;

        let mut order = VenueOrder::builder()
            .strategy(s2.clone())
            .instrument(test_inst_binance_btc_usdt_perp())
            .side(MarketSide::Sell)
            .quantity(dec!(1.0))
            .price(dec!(50000.0))
            .order_type(VenueOrderType::Market)
            .commission_asset(Some(test_bnb_asset()))
            .build();

        ledger.trade_perpetual_isolated_from_order(&Arc::new(order.clone()))?;

        order.add_fill(OffsetDateTime::now_utc(), dec!(5001), dec!(0.5), dec!(0.1));

        ledger.trade_perpetual_isolated_from_order(&Arc::new(order.clone()))?;

        order.add_fill(OffsetDateTime::now_utc(), dec!(4990), dec!(0.5), dec!(0.13));

        ledger.trade_perpetual_isolated_from_order(&Arc::new(order.clone()))?;

        // ledger.trade_perpetual(
        //     &MarketSide::Sell,
        //     &s1,
        //     &test_inst_binance_btc_usdt_perp(),
        //     dec!(2000),
        //     dec!(0.02),
        //     dec!(0.4),
        //     Some(&bnb),
        // )?;

        // ledger.trade_perpetual(
        //     &MarketSide::Sell,
        //     &s2,
        //     &test_inst_binance_btc_usdt_perp(),
        //     dec!(4000),
        //     dec!(0.04),
        //     dec!(0.8),
        //     Some(&bnb),
        // )?;

        // ledger.trade_perpetual(
        //     &MarketSide::Sell,
        //     &s2,
        //     &test_inst_binance_eth_usdt_perp(),
        //     dec!(16000),
        //     dec!(0.02),
        //     dec!(3.5),
        //     Some(&bnb),
        // )?;

        // Now let's see final balances:
        ledger.list_accounts();

        // List balances
        let balances = ledger.get_balances();
        info!("Balances:");
        for (asset, balance) in balances {
            info!(" - {}: {}", asset, balance);
        }

        // List positions
        let positions = ledger.positions();
        info!("Positions:");
        for (instrument, position) in positions {
            info!(" - {}: {}", instrument.symbol, position);
        }

        // List strategy positions
        let strategy = test_strategy();
        let strategy_positions = ledger.strategy_positions(&strategy);
        info!("{}", strategy);
        for (instrument, position) in strategy_positions {
            info!(" - {}: {}", instrument.symbol, position);
        }

        let strategy = test_strategy_crossover();
        let strategy_positions = ledger.strategy_positions(&strategy);
        info!("{}", strategy);
        for (instrument, position) in strategy_positions {
            info!(" - {}: {}", instrument.symbol, position);
        }
        Ok(())
    }
}
