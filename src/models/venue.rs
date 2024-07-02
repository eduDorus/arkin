use std::{fmt, str::FromStr};

use super::errors::ModelError;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Venue {
    Binance,
}

impl fmt::Display for Venue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Venue::Binance => write!(f, "Binance"),
        }
    }
}

impl FromStr for Venue {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Binance" => Ok(Venue::Binance),
            _ => Err(ModelError::UnknownVenueError(s.into())),
        }
    }
}
