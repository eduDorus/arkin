# arkin.

## License

This software is proprietary and not open-source. For more information on licensing, please contact [Your Contact Information].

## Stuff
- https://web.stanford.edu/~boyd/papers/pdf/markowitz.pdf


## Download
### Market Data
```
cargo run -r download --instruments BTCUSDT,SOLUSDT,ETHUSDT --venue binance-usdm --channel agg-trades --start '2025-04-22 00:00' --end '2025-06-06 00:00'
cargo run -r download --instruments BTCUSDT,SOLUSDT,ETHUSDT --venue binance-usdm --channel ticks --start '2025-04-22 00:00' --end '2025-06-06 00:00'
```

#### BINANCE SPOT
cargo run -r download -v binance-usdm-futures -c agg-trades -i spot --start "2024-09-01 00:00" --end "2025-09-01 01:00" --dry-run

#### BINANCE PERPETUAL
cargo run -r download -v binance-usdm-futures -c agg-trades -i perpetual --start "2024-09-01 00:00" --end "2025-09-01 01:00" --dry-run

#### OKX SPOT

# Persistence

## Postgres

## Clickhouse

### Sync between DBs
Enter the container
```bash
docker exec -it arkin-clickhouse clickhouse client
```

Sync from the remote
```sql
INSERT INTO arkin.ticks SELECT * FROM remote('192.168.100.100', 'arkin', 'ticks', 'arkin_admin', 'test1234') WHERE event_time BETWEEN '2023-01-01 00:00:00' AND '2025-06-06 00:00:00';
```
