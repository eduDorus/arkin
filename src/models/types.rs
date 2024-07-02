use core::fmt;

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;

use crate::constants;

use super::errors::ModelError;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Expiry(OffsetDateTime);

impl Expiry {
    pub fn new(maturity: OffsetDateTime) -> Self {
        Expiry(maturity)
    }

    pub fn time_to_maturity_in_years(&self) -> f64 {
        let now = OffsetDateTime::now_utc();
        let duration = self.0 - now;
        duration.whole_seconds() as f64 / 60.0 / 60.0 / 24.0 / 365.0
    }
}

impl fmt::Display for Expiry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted = self.0.format(constants::TIMESTAMP_FORMAT).expect("Unable to format expiry");
        write!(f, "{}", formatted)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Price(Decimal);

impl Price {
    pub fn new(price: Decimal) -> Result<Self> {
        if price >= Decimal::ZERO {
            Ok(Price(price))
        } else {
            Err(ModelError::PriceError("Price cannot be negative".into()).into())
        }
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Quantity(Decimal);

impl Quantity {
    pub fn new(quantity: Decimal) -> Self {
        Quantity(quantity)
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
