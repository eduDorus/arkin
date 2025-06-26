#![allow(dead_code)]
use std::time::Duration;

use arkin_core::prelude::load;
use clickhouse::Client;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool,
};

use crate::PersistenceConfig;

pub fn connect_postgres_db() -> PgPool {
    let config = load::<PersistenceConfig>();
    let db_config = config.postgres.clone();
    let conn_options = PgConnectOptions::new()
        .host(&db_config.host)
        .port(db_config.port)
        .username(&db_config.user)
        .password(&db_config.password)
        .database(&db_config.database)
        .ssl_mode(PgSslMode::Prefer);

    PgPoolOptions::new()
        .min_connections(db_config.min_connections)
        .max_connections(db_config.max_connections)
        .idle_timeout(Duration::from_secs(db_config.idle_timeout))
        .acquire_timeout(Duration::from_secs(db_config.acquire_timeout))
        .max_lifetime(Duration::from_secs(db_config.max_lifetime))
        .connect_lazy_with(conn_options)
}

pub fn connect_clickhouse_db() -> Client {
    let config = load::<PersistenceConfig>();
    let ch_config = config.clickhouse.clone();
    Client::default()
        .with_url(format!("http://{}:{}", ch_config.host, ch_config.port))
        .with_compression(clickhouse::Compression::Lz4)
        .with_database(ch_config.database)
        .with_user(ch_config.user)
        .with_password(ch_config.password)
        .with_option("wait_end_of_query", "1")
}
