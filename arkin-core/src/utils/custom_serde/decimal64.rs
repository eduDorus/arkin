use std::sync::LazyLock;

use rust_decimal::prelude::*;
use serde::{Deserialize, Deserializer, Serializer};

// 8 decimal places
const SCALE_FACTOR: i128 = 100_000_000;
static SCALE_FACTOR_DEC: LazyLock<Decimal> = LazyLock::new(|| Decimal::from_i128(SCALE_FACTOR).unwrap());

pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Multiply by 10^8 so we can represent `value` as an integer.
    // Example: 1234.5678 => 1234567800000 in i128 if scale=8
    let scaled = value
        .checked_mul(*SCALE_FACTOR_DEC)
        .ok_or_else(|| serde::ser::Error::custom("Decimal overflow when scaling"))?;

    let bits = scaled
        .to_i64()
        .ok_or_else(|| serde::ser::Error::custom("Decimal is out of i64 range"))?;

    serializer.serialize_i64(bits)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    // Here we do the opposit of above, divide by 10^8
    let bits = i64::deserialize(deserializer)?;
    let value = Decimal::from_i64(bits).ok_or_else(|| serde::de::Error::custom("Decimal is out of i64 range"))?;
    let scaled = value
        .checked_div(*SCALE_FACTOR_DEC)
        .ok_or_else(|| serde::de::Error::custom("Decimal overflow when descaling"))?;
    Ok(scaled)
}
