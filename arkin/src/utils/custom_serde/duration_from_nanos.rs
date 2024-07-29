use serde::{Deserialize, Deserializer, Serializer};
use time::Duration;

pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let nanos = duration.whole_nanoseconds() as i64;
    serializer.serialize_i64(nanos)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let nanos = i64::deserialize(deserializer)?;
    Ok(Duration::nanoseconds(nanos))
}
