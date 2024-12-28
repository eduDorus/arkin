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


https://fapi.binance.com/fapi/v1/exchangeInfo


pg2parquet export --host 127.0.0.1 --dbname arkin --output-file trades.parquet -q 'SELECT * FROM trades'