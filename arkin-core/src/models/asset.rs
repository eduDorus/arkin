use std::fmt;

use derive_builder::Builder;
use sqlx::Type;
use strum::Display;
use uuid::Uuid;

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "asset_type", rename_all = "snake_case")]
pub enum AssetType {
    Crypto,
    Stock,
    Forex,
    Commodity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Builder)]
#[builder(setter(into))]
pub struct Asset {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub asset_type: AssetType,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}
