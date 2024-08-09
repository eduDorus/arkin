use rust_decimal::prelude::*;
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, Sub};
use time::OffsetDateTime;

use crate::constants;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Maturity(OffsetDateTime);

impl Maturity {
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

impl From<OffsetDateTime> for Maturity {
    fn from(maturity: OffsetDateTime) -> Self {
        Maturity(maturity)
    }
}

impl fmt::Display for Maturity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted = self.0.format(constants::TIMESTAMP_FORMAT).expect("Unable to format expiry");
        write!(f, "{}", formatted)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Price(Decimal);

impl Price {
    pub fn value(&self) -> Decimal {
        self.0
    }
}

impl From<f64> for Price {
    fn from(price: f64) -> Self {
        Price(Decimal::from_f64(price).expect("Failed to convert f64 to Decimal"))
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

impl Sub<Price> for Price {
    type Output = Decimal;

    fn sub(self, rhs: Price) -> Decimal {
        self.0 - rhs.0
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Quantity(Decimal);

impl Quantity {
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_positive(&self) -> bool {
        self.0.is_sign_positive()
    }

    pub fn is_negative(&self) -> bool {
        self.0.is_sign_negative()
    }

    pub fn value(&self) -> Decimal {
        self.0
    }

    pub fn abs(&self) -> Self {
        Self::from(self.0.abs())
    }

    fn round(decimal: Decimal) -> Decimal {
        decimal.round_dp(8)
    }
}

impl From<f64> for Quantity {
    fn from(quantity: f64) -> Self {
        let decimal = Decimal::from_f64(quantity).expect("Failed to convert f64 to Decimal");
        Quantity(Self::round(decimal))
    }
}

impl From<Decimal> for Quantity {
    fn from(quantity: Decimal) -> Self {
        Quantity(Self::round(quantity))
    }
}

impl From<Quantity> for f64 {
    fn from(quantity: Quantity) -> f64 {
        quantity.0.to_f64().expect("Failed to convert Decimal to f64")
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<Quantity> for Quantity {
    type Output = Quantity;

    fn add(self, rhs: Quantity) -> Quantity {
        Quantity::from(self.0 + rhs.0)
    }
}

impl Sub<Quantity> for Quantity {
    type Output = Quantity;

    fn sub(self, rhs: Quantity) -> Quantity {
        Quantity::from(self.0 - rhs.0)
    }
}

impl AddAssign<Quantity> for Quantity {
    fn add_assign(&mut self, rhs: Quantity) {
        self.0 = Self::round(self.0 + rhs.0);
    }
}

impl Mul<Price> for Quantity {
    type Output = Notional;

    fn mul(self, rhs: Price) -> Notional {
        Notional::from(self.0 * rhs.value())
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
        Notional::from(self.0 / rhs.value())
    }
}

impl Div<Quantity> for Quantity {
    type Output = Decimal;

    fn div(self, rhs: Quantity) -> Decimal {
        self.0 / rhs.0
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Notional(Decimal);

impl Notional {
    pub fn value(&self) -> Decimal {
        self.0
    }

    pub fn to_f64(&self) -> f64 {
        self.0.to_f64().expect("Failed to convert Decimal to f64")
    }

    pub fn abs(&self) -> Self {
        Self::from(self.0.abs())
    }

    fn round(decimal: Decimal) -> Decimal {
        decimal.round_dp(2)
    }
}

impl From<f64> for Notional {
    fn from(notional: f64) -> Self {
        let decimal = Decimal::from_f64(notional).expect("Failed to convert f64 to Decimal");
        Notional(Self::round(decimal))
    }
}

impl From<Decimal> for Notional {
    fn from(notional: Decimal) -> Self {
        Notional(Self::round(notional))
    }
}

impl fmt::Display for Notional {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<Notional> for Notional {
    type Output = Notional;

    fn add(self, rhs: Notional) -> Notional {
        Notional::from(self.0 + rhs.0)
    }
}

impl AddAssign<Notional> for Notional {
    fn add_assign(&mut self, rhs: Notional) {
        self.0 = Self::round(self.0 + rhs.0);
    }
}

impl Mul<Decimal> for Notional {
    type Output = Notional;

    fn mul(self, rhs: Decimal) -> Notional {
        Notional::from(self.0 * rhs)
    }
}

impl Div<Quantity> for Notional {
    type Output = Price;

    fn div(self, rhs: Quantity) -> Price {
        Price(self.0 / rhs.value())
    }
}

impl Div<Price> for Notional {
    type Output = Quantity;

    fn div(self, rhs: Price) -> Quantity {
        Quantity(self.0 / rhs.value())
    }
}

impl Div<Notional> for Notional {
    type Output = Quantity;

    fn div(self, rhs: Notional) -> Quantity {
        Quantity(self.0 / rhs.0)
    }
}

impl Sub<Notional> for Notional {
    type Output = Notional;

    fn sub(self, rhs: Notional) -> Notional {
        Notional::from(self.0 - rhs.0)
    }
}

impl Sum<Notional> for Notional {
    fn sum<I: Iterator<Item = Notional>>(iter: I) -> Notional {
        iter.fold(Notional(Decimal::from(0)), |acc, x| acc + x)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Weight(Decimal);

impl Weight {
    pub fn value(&self) -> Decimal {
        self.0
    }
}

impl From<f64> for Weight {
    fn from(weight: f64) -> Self {
        match weight {
            weight if weight < -1.0 => Weight((-1).into()),
            weight if weight > 1.0 => Weight(1.into()),
            _ => Weight(Decimal::from_f64(weight).expect("Failed to convert f64 to Decimal")),
        }
    }
}

impl From<Decimal> for Weight {
    fn from(weight: Decimal) -> Self {
        match weight {
            weight if weight < Decimal::from(-1) => Weight(Decimal::from(-1)),
            weight if weight > Decimal::from(1) => Weight(Decimal::from(1)),
            _ => Weight(weight),
        }
    }
}

impl fmt::Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.round_dp(2))
    }
}
