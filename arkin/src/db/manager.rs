use crate::{config::DatabaseConfig, models::Event};
use anyhow::Result;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool,
};
use std::time::Duration;
use tracing::{error, info};

pub struct DBManager {
    pub pool: PgPool,
}

impl DBManager {
    pub async fn from_config(config: &DatabaseConfig) -> Self {
        let conn_options = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.user)
            .password(&config.password)
            .database(&config.database)
            .ssl_mode(PgSslMode::Prefer);

        let res = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .idle_timeout(Duration::from_secs(config.idle_timeout))
            .connect_with(conn_options)
            .await;

        let pool = match res {
            Ok(pool) => {
                info!("Connected to database");
                pool
            }
            Err(e) => panic!("SQLX failed to connect to database: {}", e),
        };

        Self { pool }
    }

    pub async fn test(&self) {
        // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
        let row: (i64,) = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_one(&self.pool)
            .await
            .expect("SQLX failed to fetch row");

        assert_eq!(row.0, 150);
    }

    pub async fn add_event(&self, event: Event) -> Result<()> {
        match event {
            Event::Tick(t) => self.insert_tick(t).await?,
            Event::Trade(t) => self.insert_trade(t).await?,
            Event::Order(o) => self.insert_order(o).await?,
            Event::Fill(f) => self.insert_fill(f).await?,
            Event::Signal(s) => self.insert_signal(s).await?,
            Event::Allocation(a) => self.insert_allocation(a).await?,
            _ => {
                error!("Event type not supported: {}", event.event_type());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    #[tokio::test]
    async fn test_db_manager() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;
        manager.test().await;
    }
}
