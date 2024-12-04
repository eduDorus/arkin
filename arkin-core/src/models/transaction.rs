use std::{fmt, sync::Arc};

use typed_builder::TypedBuilder;
use rust_decimal::Decimal;
use sqlx::Type;
use strum::Display;
use time::OffsetDateTime;
use uuid::Uuid;

use super::{Asset, Instrument, Portfolio};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "transaction_type", rename_all = "snake_case")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Buy,
    Sell,
    Collateral,
    Dividend,
    Fee,
    Settlement,
    Interest,
    Funding,
    Liquidation,
    Transfer,
    Rebate,
    Adjustment,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypedBuilder)]

pub struct Transaction {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub event_time: OffsetDateTime,
    pub transaction_group_id: Uuid,
    pub portfolio: Arc<Portfolio>,
    pub asset: Option<Arc<Asset>>,
    pub instrument: Option<Arc<Instrument>>,
    pub transaction_type: TransactionType,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
    pub total_value: Decimal,
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {}{} {} {} {} {}",
            self.event_time,
            self.portfolio,
            self.asset.as_ref().map(|asset| asset.symbol.as_str()).unwrap_or(""),
            self.instrument
                .as_ref()
                .map(|instrument| instrument.symbol.as_str())
                .unwrap_or(""),
            self.transaction_type,
            self.price.as_ref().map(|price| price.to_string()).unwrap_or("".to_string()),
            self.quantity,
            self.total_value
        )
    }
}
