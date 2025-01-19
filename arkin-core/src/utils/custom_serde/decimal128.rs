use rust_decimal::prelude::*;
use serde::{Deserialize, Deserializer, Serializer};

const SCALE_FACTOR: i128 = 100_000_000;
pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Multiply by 10^8 so we can represent `value` as an integer.
    // Example: 1234.5678 => 1234567800000 in i128 if scale=8
    let scaled = value
        .checked_mul(Decimal::from_i128(SCALE_FACTOR).unwrap())
        .ok_or_else(|| serde::ser::Error::custom("Decimal overflow when scaling"))?;

    let bits = scaled
        .to_i128()
        .ok_or_else(|| serde::ser::Error::custom("Decimal is out of i128 range"))?;

    serializer.serialize_i128(bits)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let bits = i128::deserialize(deserializer)?;
    Ok(Decimal::from_i128_with_scale(bits, SCALE_FACTOR as u32))
}
