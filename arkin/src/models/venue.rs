use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use super::errors::ModelError;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Venue {
    Simulation,
    Binance,
}

impl fmt::Display for Venue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Venue::Simulation => write!(f, "simulation"),
            Venue::Binance => write!(f, "binance"),
        }
    }
}

impl FromStr for Venue {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "simulation" => Ok(Venue::Simulation),
            "binance" => Ok(Venue::Binance),
            _ => Err(ModelError::UnknownVenueError(s.into())),
        }
    }
}
