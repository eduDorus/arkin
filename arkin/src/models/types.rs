use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul};

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;

use crate::constants;

use super::errors::ModelError;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Maturity(OffsetDateTime);

impl Maturity {
    pub fn new(maturity: OffsetDateTime) -> Self {
        Maturity(maturity)
    }

    pub fn time_to_maturity_in_years(&self) -> f64 {
        let now = OffsetDateTime::now_utc();
        let duration = self.0 - now;
        duration.whole_seconds() as f64 / 60.0 / 60.0 / 24.0 / 365.0
    }

    pub fn timestamp_in_ms(&self) -> i64 {
        self.0.unix_timestamp()
    }

    pub fn value(&self) -> OffsetDateTime {
        self.0
    }
}

impl fmt::Display for Maturity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted = self.0.format(constants::TIMESTAMP_FORMAT).expect("Unable to format expiry");
        write!(f, "{}", formatted)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

impl From<Decimal> for Price {
    fn from(price: Decimal) -> Self {
        Price(price)
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.round_dp(2))
    }
}

impl Add<Price> for Price {
    type Output = Price;

    fn add(self, rhs: Price) -> Price {
        Price(self.0 + rhs.0)
    }
}

impl AddAssign<Price> for Price {
    fn add_assign(&mut self, rhs: Price) {
        self.0 += rhs.0;
    }
}

impl Mul<Quantity> for Price {
    type Output = Notional;

    fn mul(self, rhs: Quantity) -> Notional {
        Notional(self.0 * rhs.value())
    }
}

impl Div<Notional> for Price {
    type Output = Quantity;

    fn div(self, rhs: Notional) -> Quantity {
        Quantity(self.0 / rhs.value())
    }
}

impl Div<Quantity> for Price {
    type Output = Notional;

    fn div(self, rhs: Quantity) -> Notional {
        Notional(self.0 / rhs.value())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Quantity(Decimal);

impl Quantity {
    pub fn new(quantity: Decimal) -> Self {
        Quantity(quantity)
    }

    pub fn value(&self) -> Decimal {
        self.0
    }

    pub fn abs(&self) -> Self {
        self.0.abs().into()
    }
}

impl From<Decimal> for Quantity {
    fn from(quantity: Decimal) -> Self {
        Quantity(quantity)
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AddAssign<Quantity> for Quantity {
    fn add_assign(&mut self, rhs: Quantity) {
        self.0 += rhs.0;
    }
}

impl Mul<Price> for Quantity {
    type Output = Notional;

    fn mul(self, rhs: Price) -> Notional {
        Notional(self.0 * rhs.value())
    }
}

impl Div<Notional> for Quantity {
    type Output = Price;

    fn div(self, rhs: Notional) -> Price {
        Price(self.0 / rhs.value())
    }
}

impl Div<Price> for Quantity {
    type Output = Notional;

    fn div(self, rhs: Price) -> Notional {
        Notional(self.0 / rhs.value())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Notional(Decimal);

impl Notional {
    pub fn new(notional: Decimal) -> Self {
        Notional(notional)
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

impl From<Decimal> for Notional {
    fn from(notional: Decimal) -> Self {
        Notional(notional)
    }
}

impl fmt::Display for Notional {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AddAssign<Notional> for Notional {
    fn add_assign(&mut self, rhs: Notional) {
        self.0 += rhs.0;
    }
}

impl Div<Quantity> for Notional {
    type Output = Price;

    fn div(self, rhs: Quantity) -> Price {
        Price(self.0 / rhs.value())
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Weight(Decimal);

impl Weight {
    pub fn new(weight: Decimal) -> Self {
        // Check if between -1 and 1 else clip
        match weight {
            weight if weight < Decimal::from(-1) => Weight(Decimal::from(-1)),
            weight if weight > Decimal::from(1) => Weight(Decimal::from(1)),
            _ => Weight(weight),
        }
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

impl fmt::Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.round_dp(2))
    }
}
