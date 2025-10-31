use anyhow::Result;
use time::{Duration, Time, UtcDateTime};

#[derive(Clone, Copy)]
pub enum Frequency {
    Hourly,
    HalfDaily,
    Daily,
}

fn next_boundary(dt: UtcDateTime, frequency: &Frequency) -> UtcDateTime {
    match frequency {
        Frequency::Daily => {
            // Get the date and move to the next day at midnight
            let next_date = dt.date() + Duration::days(1);
            next_date.midnight().as_utc()
        }
        Frequency::HalfDaily => {
            // If before noon, move to noon; otherwise, move to the next day at midnight
            if dt.hour() < 12 {
                dt.replace_time(Time::from_hms(12, 0, 0).unwrap())
            } else {
                let next_date = dt.date() + Duration::days(1);
                next_date.midnight().as_utc()
            }
        }
        Frequency::Hourly => {
            // If exactly on the hour, move to the next hour
            if dt.minute() == 0 && dt.second() == 0 && dt.nanosecond() == 0 {
                dt + Duration::hours(1)
            } else {
                // Otherwise, truncate to the current hour and add one
                let truncated = dt.replace_time(Time::from_hms(dt.hour(), 0, 0).unwrap());
                truncated + Duration::hours(1)
            }
        }
    }
}

pub fn datetime_chunks(
    start: UtcDateTime,
    end: UtcDateTime,
    frequency: Frequency,
) -> Result<Vec<(UtcDateTime, UtcDateTime)>, anyhow::Error> {
    if start > end {
        anyhow::bail!("Start date cannot be greater than end date");
    }

    let mut chunks = Vec::new();
    let mut current = start;

    while current < end {
        let next = next_boundary(current, &frequency);
        // If next is at or before end, use it as the chunk end; otherwise, use end
        let chunk_end = if next <= end { next } else { end };
        chunks.push((current, chunk_end));
        current = next;
        // Break if we've reached the end to avoid adding an empty chunk
        if chunk_end == end {
            break;
        }
    }

    // If start == end, the loop won't run, returning an empty Vec, which is reasonable
    Ok(chunks)
}

pub fn datetime_range_minute(start: UtcDateTime, end: UtcDateTime) -> Result<Vec<UtcDateTime>> {
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

pub fn datetime_range_hourly(start: UtcDateTime, end: UtcDateTime) -> Result<Vec<UtcDateTime>> {
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

pub fn datetime_range_daily(start: UtcDateTime, end: UtcDateTime) -> Result<Vec<UtcDateTime>> {
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

pub fn datetime_range_weekly(start: UtcDateTime, end: UtcDateTime) -> Result<Vec<UtcDateTime>> {
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

pub fn round_to_minute(datetime: UtcDateTime) -> Result<UtcDateTime> {
    let mut datetime = datetime;
    datetime = datetime.replace_second(0)?;
    datetime = datetime.replace_nanosecond(0)?;
    Ok(datetime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;
    use tracing::info;

    #[test]
    fn test_datetime_range_minute() {
        let start = datetime!(2023 - 06 - 09 12:23:03 UTC).to_utc();
        let end = datetime!(2023 - 06 - 09 12:26:03 UTC).to_utc();

        let dates = datetime_range_minute(start, end).unwrap();

        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], datetime!(2023 - 06 - 09 12:23:00 UTC).to_utc());
        assert_eq!(dates[1], datetime!(2023 - 06 - 09 12:24:00 UTC).to_utc());
        assert_eq!(dates[2], datetime!(2023 - 06 - 09 12:25:00 UTC).to_utc());
        assert_eq!(dates[3], datetime!(2023 - 06 - 09 12:26:00 UTC).to_utc());
    }

    #[test]
    fn test_datetime_range_minute_same_tme() {
        let start = datetime!(2023 - 06 - 09 12:23:03 UTC).to_utc();
        let end = datetime!(2023 - 06 - 09 12:23:03 UTC).to_utc();

        let dates = datetime_range_minute(start, end).unwrap();

        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], datetime!(2023 - 06 - 09 12:23:00 UTC).to_utc());
    }

    #[test]
    fn test_datetime_range_hourly() {
        let start = datetime!(2023 - 06 - 09 12:23:03 UTC).to_utc();
        let end = datetime!(2023 - 06 - 09 15:43:13 UTC).to_utc();

        let dates = datetime_range_hourly(start, end).unwrap();

        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], datetime!(2023 - 06 - 09 12:00:00 UTC).to_utc());
        assert_eq!(dates[1], datetime!(2023 - 06 - 09 13:00:00 UTC).to_utc());
        assert_eq!(dates[2], datetime!(2023 - 06 - 09 14:00:00 UTC).to_utc());
        assert_eq!(dates[3], datetime!(2023 - 06 - 09 15:00:00 UTC).to_utc());
    }

    #[test]
    fn test_datetime_range_daily() {
        let start = datetime!(2023 - 06 - 09 12:23:03 UTC).to_utc();
        let end = datetime!(2023 - 06 - 11 22:23:03 UTC).to_utc();

        let dates = datetime_range_daily(start, end).unwrap();

        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0], datetime!(2023 - 06 - 09 00:00:00 UTC).to_utc());
        assert_eq!(dates[1], datetime!(2023 - 06 - 10 00:00:00 UTC).to_utc());
        assert_eq!(dates[2], datetime!(2023 - 06 - 11 00:00:00 UTC).to_utc());
    }

    #[test]
    fn test_datetime_range_weekly() {
        let start = datetime!(2023 - 01 - 01 12:23:03 UTC).to_utc();
        let end = datetime!(2023 - 02 - 15 22:23:03 UTC).to_utc();

        let dates = datetime_range_weekly(start, end).unwrap();

        assert_eq!(dates.len(), 8);
        info!("{:?}", dates);
    }

    #[test]
    fn test_round_to_minute() {
        let datetime = datetime!(2023 - 06 - 09 12:23:03.430239483 UTC).to_utc();
        let rounded = round_to_minute(datetime).unwrap();
        assert_eq!(rounded, datetime!(2023 - 06 - 09 12:23:00 UTC).to_utc());
    }

    #[test]
    fn test_datetime_range_chunker() {
        let start = datetime!(2023 - 06 - 09 12:23:03 UTC).to_utc();
        let end = datetime!(2023 - 06 - 11 22:23:03 UTC).to_utc();

        let dates = datetime_chunks(start, end, Frequency::Hourly).unwrap();
        assert_eq!(dates.len(), 59);
        assert_eq!(dates[0].0, start);
        assert_eq!(dates[0].1, datetime!(2023 - 06 - 09 13:00:00 UTC).to_utc());
        assert_eq!(dates[dates.len() - 1].0, datetime!(2023 - 06 - 11 22:00:00 UTC).to_utc());
        assert_eq!(dates[dates.len() - 1].1, end);

        let dates = datetime_chunks(start, end, Frequency::HalfDaily).unwrap();
        assert_eq!(dates.len(), 5);
        assert_eq!(dates[0].0, start);
        assert_eq!(dates[0].1, datetime!(2023 - 06 - 10 00:00:00 UTC).to_utc());
        assert_eq!(dates[dates.len() - 1].0, datetime!(2023 - 06 - 11 12:00:00 UTC).to_utc());
        assert_eq!(dates[dates.len() - 1].1, end);

        let dates = datetime_chunks(start, end, Frequency::Daily).unwrap();
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0].0, start);
        assert_eq!(dates[0].1, datetime!(2023 - 06 - 10 00:00:00 UTC).to_utc());
        assert_eq!(dates[dates.len() - 1].0, datetime!(2023 - 06 - 11 00:00:00 UTC).to_utc());
        assert_eq!(dates[dates.len() - 1].1, end);
    }
}
