#![allow(unused)]
use serde::{de, Deserialize, Deserializer, Serializer};
use time::OffsetDateTime;

/// Serialize a `time::OffsetDateTime` to a nanosecond representation.
pub fn serialize<S>(datetime: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let nanos: i64 = datetime.unix_timestamp_nanos() as i64;
    serializer.serialize_i64(nanos)
}

/// Deserialize a `time::OffsetDateTime` from a variable-length timestamp.
pub fn deserialize<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let input = TimestampInput::deserialize(deserializer)?;

    let timestamp_str = match input {
        TimestampInput::Int128(i) => i.to_string(),
        TimestampInput::Int64(i) => i.to_string(),
        TimestampInput::Str(s) => s,
    };

    let num_digits = timestamp_str.len();
    let timestamp_nanos = if num_digits < 19 {
        let zeros_to_add = 19 - num_digits;
        let filler = "0".repeat(zeros_to_add);
        timestamp_str + &filler
    } else {
        timestamp_str
    };

    let timestamp_nanos = timestamp_nanos.parse::<i128>().map_err(de::Error::custom)?;
    OffsetDateTime::from_unix_timestamp_nanos(timestamp_nanos).map_err(de::Error::custom)
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TimestampInput {
    Str(String),
    Int128(i128),
    Int64(i64),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde_json;
    use time::OffsetDateTime;

    // Assuming MyStruct is the struct that uses the custom serialization/deserialization
    #[derive(Serialize, Deserialize)]
    struct MyStruct {
        #[serde(with = "crate::utils::timestamp")]
        timestamp: OffsetDateTime,
    }

    #[test]
    fn test_timestamp_serialization_deserialization() {
        // Example timestamp in milliseconds
        let ms_timestamp = 9223372036854i64;
        let timestamp_str = format!("{{\"timestamp\":{}}}", ms_timestamp);

        // Test deserialization
        let deserialized: MyStruct = serde_json::from_str(&timestamp_str).expect("Deserialization failed");
        assert_eq!(deserialized.timestamp.unix_timestamp(), ms_timestamp / 1000);

        // Test serialization
        let serialized = serde_json::to_string(&deserialized).expect("Serialization failed");
        let expected_serialized = format!("{{\"timestamp\":{}000000}}", ms_timestamp);
        assert_eq!(serialized, expected_serialized);
    }

    #[test]
    fn test_timestamp_serialization_deserialization_string() {
        // Example timestamp in milliseconds
        let ms_timestamp = "9223372036854";
        let timestamp_str = format!("{{\"timestamp\":{}}}", ms_timestamp);

        // Test deserialization
        let deserialized: MyStruct = serde_json::from_str(&timestamp_str).expect("Deserialization failed");

        // Test serialization
        let serialized = serde_json::to_string(&deserialized).expect("Serialization failed");
        let expected_serialized = format!("{{\"timestamp\":{}000000}}", ms_timestamp);
        assert_eq!(serialized, expected_serialized);
    }
}
