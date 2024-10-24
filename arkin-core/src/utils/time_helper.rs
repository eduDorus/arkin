use anyhow::Result;
use time::{Duration, OffsetDateTime, Time};

pub fn datetime_range_minute(start: OffsetDateTime, end: OffsetDateTime) -> Result<Vec<OffsetDateTime>> {
    if start > end {
        anyhow::bail!("Start date cannot be greater than end date");
    }

    // Calculate the number of minutes for capacity allocation
    let duration_in_minutes = (end.unix_timestamp() - start.unix_timestamp()) / 60;
    let mut datetimes = Vec::with_capacity(duration_in_minutes as usize);

    // Adjust start time to the beginning of the minute
    let adjusted_start = start.replace_time(Time::from_hms(start.hour(), start.minute(), 0)?);

    let mut current = adjusted_start;
    while current < end {
        datetimes.push(current);
        current += Duration::minutes(1);
    }

    Ok(datetimes)
}

pub fn datetime_range_hourly(start: OffsetDateTime, end: OffsetDateTime) -> Result<Vec<OffsetDateTime>> {
    if start > end {
        anyhow::bail!("Start date cannot be greater than end date");
    }

    let duration_in_hours = (end.unix_timestamp() - start.unix_timestamp()) / 3600;
    let mut datetimes = Vec::with_capacity(duration_in_hours as usize);

    let adjusted_start = start.replace_time(Time::from_hms(start.hour(), 0, 0)?);

    let mut current = adjusted_start;
    while current < end {
        datetimes.push(current);
        current += Duration::hours(1);
    }

    Ok(datetimes)
}

pub fn datetime_range_daily(start: OffsetDateTime, end: OffsetDateTime) -> Result<Vec<OffsetDateTime>> {
    if start > end {
        anyhow::bail!("Start date cannot be greater than end date");
    }

    let duration_in_days = (end.unix_timestamp() - start.unix_timestamp()) / 86400;
    let mut datetimes = Vec::with_capacity(duration_in_days as usize);

    let adjusted_start = start.replace_time(Time::MIDNIGHT);

    let mut current = adjusted_start;
    while current < end {
        datetimes.push(current);
        current += Duration::days(1);
    }

    Ok(datetimes)
}

pub fn datetime_range_weekly(start: OffsetDateTime, end: OffsetDateTime) -> Result<Vec<OffsetDateTime>> {
    if start > end {
        anyhow::bail!("Start date cannot be greater than end date");
    }

    let duration_in_weeks = ((end.unix_timestamp() - start.unix_timestamp()) as f64 / 604800.0).ceil() as usize;
    let mut datetimes = Vec::with_capacity(duration_in_weeks);

    let days_to_subtract = start.weekday().number_from_monday() as i64 - 1;
    let adjusted_start = start.replace_time(Time::MIDNIGHT) - Duration::days(days_to_subtract);

    let mut current = adjusted_start;
    while current < end {
        datetimes.push(current);
        current += Duration::weeks(1);
    }

    Ok(datetimes)
}

pub fn round_to_minute(datetime: OffsetDateTime) -> Result<OffsetDateTime> {
    let mut datetime = datetime;
    datetime = datetime.replace_second(0)?;
    datetime = datetime.replace_nanosecond(0)?;
    Ok(datetime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_datetime_range_minute() {
        let start = datetime!(2023 - 06 - 09 12:23:03).assume_utc();
        let end = datetime!(2023 - 06 - 09 12:26:03).assume_utc();

        let dates = datetime_range_minute(start, end).unwrap();

        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], datetime!(2023 - 06 - 09 12:23:00).assume_utc());
        assert_eq!(dates[1], datetime!(2023 - 06 - 09 12:24:00).assume_utc());
        assert_eq!(dates[2], datetime!(2023 - 06 - 09 12:25:00).assume_utc());
        assert_eq!(dates[3], datetime!(2023 - 06 - 09 12:26:00).assume_utc());
    }

    #[test]
    fn test_datetime_range_minute_same_tme() {
        let start = datetime!(2023 - 06 - 09 12:23:03).assume_utc();
        let end = datetime!(2023 - 06 - 09 12:23:03).assume_utc();

        let dates = datetime_range_minute(start, end).unwrap();

        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], datetime!(2023 - 06 - 09 12:23:00).assume_utc());
    }

    #[test]
    fn test_datetime_range_hourly() {
        let start = datetime!(2023 - 06 - 09 12:23:03).assume_utc();
        let end = datetime!(2023 - 06 - 09 15:43:13).assume_utc();

        let dates = datetime_range_hourly(start, end).unwrap();

        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], datetime!(2023 - 06 - 09 12:00:00).assume_utc());
        assert_eq!(dates[1], datetime!(2023 - 06 - 09 13:00:00).assume_utc());
        assert_eq!(dates[2], datetime!(2023 - 06 - 09 14:00:00).assume_utc());
        assert_eq!(dates[3], datetime!(2023 - 06 - 09 15:00:00).assume_utc());
    }

    #[test]
    fn test_datetime_range_daily() {
        let start = datetime!(2023 - 06 - 09 12:23:03).assume_utc();
        let end = datetime!(2023 - 06 - 11 22:23:03).assume_utc();

        let dates = datetime_range_daily(start, end).unwrap();

        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0], datetime!(2023 - 06 - 09 00:00:00).assume_utc());
        assert_eq!(dates[1], datetime!(2023 - 06 - 10 00:00:00).assume_utc());
        assert_eq!(dates[2], datetime!(2023 - 06 - 11 00:00:00).assume_utc());
    }

    #[test]
    fn test_datetime_range_weekly() {
        let start = datetime!(2023 - 01 - 01 12:23:03).assume_utc();
        let end = datetime!(2023 - 02 - 15 22:23:03).assume_utc();

        let dates = datetime_range_weekly(start, end).unwrap();

        assert_eq!(dates.len(), 8);
        println!("{:?}", dates);
    }

    #[test]
    fn test_round_to_minute() {
        let datetime = datetime!(2023 - 06 - 09 12:23:03.430239483).assume_utc();
        let rounded = round_to_minute(datetime).unwrap();
        assert_eq!(rounded, datetime!(2023 - 06 - 09 12:23:00).assume_utc());
    }
}
