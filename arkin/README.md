# Docker Clickhouse
```bash
docker run -d -v "$(pwd)/clickhouse/logs:/var/log/clickhouse-server" -v "$(pwd)/clickhouse/data:/var/lib/clickhouse" -e CLICKHOUSE_DB=arkin -e CLICKHOUSE_USER=arkin_admin -e CLICKHOUSE_DEFAULT_ACCESS_MANAGEMENT=1 -e CLICKHOUSE_PASSWORD=<PASSWORD> -p 8123:8123 -p 9000:9000 -p 9005:9005 --user ${UID}:${GID} --name clickhouse-server --ulimit nofile=262144:262144 clickhouse/clickhouse-server:latest
```

# Docker TimescaleDB
```bash
docker run -d -v "$(pwd)/timescaledb/data:/var/lib/postgresql/data" -v "$(pwd)/timescaledb/logs:/var/logs" -e POSTGRES_DB=arkin -e POSTGRES_USER=arkin_admin -e POSTGRES_PASSWORD=<PASSWORD> -p 5432:5432 --user ${UID}:${GID} --name timescaledb-server timescale/timescaledb:latest-pg16
```

# Sqlx CLI
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