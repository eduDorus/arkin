#![allow(dead_code)]
use std::time::Duration;

use arkin_core::load;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool,
};

use crate::PersistenceConfig;

pub fn connect_database() -> PgPool {
    let config = load::<PersistenceConfig>();
    let db_config = config.database.clone();
    let conn_options = PgConnectOptions::new()
        .host(&db_config.host)
        .port(db_config.port)
        .username(&db_config.user)
        .password(&db_config.password)
        .database(&db_config.database)
        .ssl_mode(PgSslMode::Prefer);

    let pool = PgPoolOptions::new()
        .min_connections(db_config.min_connections)
        .max_connections(db_config.max_connections)
        .idle_timeout(Duration::from_secs(db_config.idle_timeout))
        .acquire_timeout(Duration::from_secs(db_config.acquire_timeout))
        .max_lifetime(Duration::from_secs(db_config.max_lifetime))
        .connect_lazy_with(conn_options);
    pool
}
