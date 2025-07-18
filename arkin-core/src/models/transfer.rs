use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use sqlx::Type;
use strum::Display;
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::Asset;

use super::{Account, Instrument, Strategy};

#[derive(Debug, Display, Clone, Copy, PartialEq, Type)]
#[sqlx(type_name = "transfer_group_type", rename_all = "snake_case")]
pub enum TransferGroupType {
    Initial,
    Deposit,
    Withdrawal,
    Trade,
    Exchange,
    Interest,
    Funding,
    Settlement,
    Liquidation,
    Adjustment,
    Reconciliation,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Type)]
#[sqlx(type_name = "transfer_type", rename_all = "snake_case")]
pub enum TransferType {
    Transfer,
    Pnl,
    UnrealizedPnl,
    Margin,
    Commission,
    Rebate,
    Reconciliation,
}

/// A single same-currency "transfer" in double-entry style.
/// In TigerBeetle's lingo, this is one row in the ledger for a single currency.
#[derive(Debug, Clone, TypedBuilder)]
pub struct Transfer {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    /// The ID of the transfer group this transfer belongs to.
    pub transfer_group_id: Uuid,
    /// Type of transfer group
    pub transfer_group_type: TransferGroupType,
    /// Transfer type (e.g. deposit, withdrawal, trade, etc.)
    pub transfer_type: TransferType,
    /// The account that is debited (balance goes down).
    pub debit_account: Arc<Account>,
    /// The account that is credited (balance goes up).
    pub credit_account: Arc<Account>,
    /// The amount of this transfer.
    pub amount: Decimal,
    /// The Unit Price of the asset being transferred.
    pub unit_price: Decimal,
    /// strategy that initiated this transfer.
    pub strategy: Option<Arc<Strategy>>,
    /// Instrument that this transfer is related to.
    pub instrument: Option<Arc<Instrument>>,
    /// The asset being transferred.
    pub asset: Option<Arc<Asset>>,
    /// The time of this transfer.
    pub created: UtcDateTime,
}

impl Transfer {
    pub fn has_transfer_type(&self, transfer_type: &TransferType) -> bool {
        &self.transfer_type == transfer_type
    }

    pub fn has_strategy(&self, strategy: &Arc<Strategy>) -> bool {
        self.strategy.as_ref() == Some(strategy)
    }

    pub fn has_instrument(&self, instrument: &Arc<Instrument>) -> bool {
        self.instrument.as_ref() == Some(instrument)
    }

    pub fn has_asset(&self, asset: &Arc<Asset>) -> bool {
        self.asset.as_ref() == Some(&asset)
    }
}

impl fmt::Display for Transfer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}: {} -> {} {}{} {} @ {}",
            self.created,
            self.transfer_type,
            self.debit_account,
            self.credit_account,
            match &self.asset {
                Some(asset) => format!("{}", asset),
                None => "".to_string(),
            },
            match &self.instrument {
                Some(inst) => format!("{}", inst),
                None => "".to_string(),
            },
            self.amount,
            self.unit_price
        )
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct TransferGroup {
    pub transfers: Vec<Arc<Transfer>>,
    pub created: UtcDateTime,
}
