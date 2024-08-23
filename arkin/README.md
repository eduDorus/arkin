# arkin.

## License

This software is proprietary and not open-source. For more information on licensing, please contact [Your Contact Information].

## Database
### Setup (TimescaleDB)
```bash
docker run -d -v "$(pwd)/timescaledb/data:/var/lib/postgresql/data" -v "$(pwd)/timescaledb/logs:/var/logs" -e POSTGRES_DB=arkin -e POSTGRES_USER=arkin_admin -e POSTGRES_PASSWORD=<PASSWORD> -p 5432:5432 --user ${UID}:${GID} --name timescaledb-server timescale/timescaledb:latest-pg16
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