use std::{fmt, sync::Arc};

use uuid::Uuid;

use super::{Asset, Instrument};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tradable {
    /// A simple asset (spot currency like BTC, USDT, BNB, etc.)
    Asset(Arc<Asset>),

    /// A derivative instrument (future, option, perpetual, etc.)
    Instrument(Arc<Instrument>),
}

impl Tradable {
    pub fn id(&self) -> Uuid {
        match self {
            Tradable::Asset(a) => a.id,
            Tradable::Instrument(i) => i.id,
        }
    }

    pub fn is_asset(&self) -> bool {
        match self {
            Tradable::Asset(_) => true,
            _ => false,
        }
    }

    pub fn is_instrument(&self) -> bool {
        match self {
            Tradable::Instrument(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Tradable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tradable::Asset(a) => write!(f, "{}", a.symbol.to_uppercase()),
            Tradable::Instrument(i) => write!(f, "{}", i.symbol.to_uppercase()),
        }
    }
}

impl From<Arc<Asset>> for Tradable {
    fn from(asset: Arc<Asset>) -> Self {
        Tradable::Asset(asset)
    }
}

impl From<Arc<Instrument>> for Tradable {
    fn from(instrument: Arc<Instrument>) -> Self {
        Tradable::Instrument(instrument)
    }
}
