use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use sqlx::Type;
use strum::Display;
use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::Event;

use super::{Account, Instrument, Strategy, Tradable};

#[derive(Debug, Display, Clone, PartialEq, Type)]
#[sqlx(type_name = "transfer_type", rename_all = "snake_case")]
pub enum TransferType {
    Deposit,
    Withdrawal,
    Trade,
    Pnl,
    Exchange,
    Margin,
    Commission,
    Interest,
    Funding,
    Settlement,
    Liquidation,
    Rebate,
    Adjustment,
}

/// A single same-currency "transfer" in double-entry style.
/// In TigerBeetle's lingo, this is one row in the ledger for a single currency.
#[derive(Debug, Clone, TypedBuilder)]
pub struct Transfer {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    /// The event time of this transfer.
    pub event_time: OffsetDateTime,
    /// The ID of the transfer group this transfer belongs to.
    pub transfer_group_id: Uuid,
    /// The account that is debited (balance goes down).
    pub debit_account: Arc<Account>,
    /// The account that is credited (balance goes up).
    pub credit_account: Arc<Account>,
    /// The asset being transferred.
    pub asset: Tradable,
    /// The amount of this transfer.
    pub amount: Decimal,
    /// The Unit Price of the asset being transferred.
    pub unit_price: Decimal,
    /// Transfer type (e.g. deposit, withdrawal, trade, etc.)
    pub transfer_type: TransferType,
    /// strategy that initiated this transfer.
    #[builder(default)]
    pub strategy: Option<Arc<Strategy>>,
    /// Instrument that this transfer is related to.
    #[builder(default)]
    pub instrument: Option<Arc<Instrument>>,
}

impl Transfer {
    pub fn has_asset(&self, asset: &Tradable) -> bool {
        &self.asset == asset
    }

    pub fn has_type(&self, transfer_type: &TransferType) -> bool {
        &self.transfer_type == transfer_type
    }

    pub fn has_strategy(&self, strategy: &Arc<Strategy>) -> bool {
        match &self.strategy {
            Some(s) => s == strategy,
            None => false,
        }
    }

    pub fn has_instrument(&self, instrument: &Arc<Instrument>) -> bool {
        match &self.instrument {
            Some(i) => i == instrument,
            None => false,
        }
    }
}

impl fmt::Display for Transfer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} -> {} {} @ {}",
            self.transfer_type, self.debit_account, self.credit_account, self.amount, self.unit_price
        )
    }
}

impl From<Arc<Transfer>> for Event {
    fn from(transfer: Arc<Transfer>) -> Self {
        Event::Transfer(transfer)
    }
}

impl From<Vec<Arc<Transfer>>> for Event {
    fn from(transfers: Vec<Arc<Transfer>>) -> Self {
        Event::TransferBatch(transfers)
    }
}
