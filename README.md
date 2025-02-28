# arkin.

## License

This software is proprietary and not open-source. For more information on licensing, please contact [Your Contact Information].

## Database
### Setup (TimescaleDB)
```bash
docker run -d -v "$(pwd)/timescaledb/data:/var/lib/postgresql/data" -v "$(pwd)/timescaledb/logs:/var/logs" -e POSTGRES_DB=arkin_test -e POSTGRES_USER=arkin_admin -e POSTGRES_PASSWORD=<PASSWORD> -p 5432:5432 --user ${UID}:${GID} --name timescaledb-server timescale/timescaledb:latest-pg16
```

### Sqlx CLI
Install Sqlx CLI
```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

Create a **.env** file in the root directory of the project with the DATABASE_URL
```bash
DATABASE_URL=postgresql://<USERNAME>:<PASSWORD>@<HOST>:<PORT>/<DATABASE>
```

Create migration
```bash
sqlx migrate add <name>
```

Run migration
```bash
sqlx migrate run
```

Rollback migration
```bash
sqlx migrate revert
```

## Grafana
### Setup
```bash
# create a directory for your data
mkdir data

# start grafana with your user id and using the data directory
docker run -d -p 3000:3000 --user ${UID}:${GID} -v "$(pwd)/grafana:/var/lib/grafana" --name=grafana grafana/grafana-oss
```

https://fapi.binance.com/fapi/v1/exchangeInfo


pg2parquet export --host 127.0.0.1 --dbname arkin --output-file trades.parquet -q 'SELECT * FROM trades'




# Get scaler data
## Dedup first just to make sure
```sql
OPTIMIZE TABLE insights FINAL DEDUPLICATE;
```


## Query data
```sql
WITH percentiles AS (
    SELECT
        feature_id,
        quantileExact(0.01)(value) AS percentile_01,
        quantileExact(0.99)(value) AS percentile_99
    FROM insights FINAL
    WHERE event_time BETWEEN '2024-01-01 00:00:00' AND '2025-01-01 00:00:00'
        AND pipeline_id = '5d4e78e0-ef8a-47b1-ad9f-b162021994a3'
        AND insight_type = 'continuous'
    GROUP BY feature_id
),
clipped_values AS (
    SELECT
        i.feature_id,
        LEAST(GREATEST(i.value, p.percentile_01), p.percentile_99) AS clipped_value,
        p.percentile_01,
        p.percentile_99
    FROM insights i FINAL
    JOIN percentiles p ON i.feature_id = p.feature_id
    WHERE i.event_time BETWEEN '2024-01-01 00:00:00' AND '2025-01-01 00:00:00'
        AND i.pipeline_id = '5d4e78e0-ef8a-47b1-ad9f-b162021994a3'
        AND i.insight_type = 'continuous'
),
min_values AS (
    SELECT
        feature_id,
        min(clipped_value) AS min_val
    FROM clipped_values
    GROUP BY feature_id
),
stats AS (
    SELECT
        cv.feature_id,
        quantilesExact(0.25, 0.5, 0.75)(cv.clipped_value) AS q_original,
        quantilesExact(0.25, 0.5, 0.75)(log(cv.clipped_value + IF(m.min_val <= 0, -m.min_val + 1, 0))) AS q_log,
        skewPop(cv.clipped_value) AS skew,
        kurtPop(cv.clipped_value) AS kurtosis,
        m.min_val,
        cv.percentile_01,
        cv.percentile_99
    FROM clipped_values cv
    JOIN min_values m ON cv.feature_id = m.feature_id
    GROUP BY cv.feature_id, m.min_val, cv.percentile_01, cv.percentile_99
)
SELECT
    feature_id,
    IF(abs(skew) > 1, arrayElement(q_log, 2), arrayElement(q_original, 2)) AS median,
    IF(abs(skew) > 1, arrayElement(q_log, 3) - arrayElement(q_log, 1), arrayElement(q_original, 3) - arrayElement(q_original, 1)) AS iqr,
    percentile_01,
    percentile_99,
    abs(skew) > 1 AS is_skewed,
    skew,
    IF(min_val <= 0, -min_val + 1, 0) AS skew_offset,
    kurtosis
FROM stats
ORDER BY feature_id ASC;
```