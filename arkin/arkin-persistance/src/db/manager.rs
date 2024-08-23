use std::time::Duration;

use anyhow::Result;
use arkin_common::prelude::*;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool,
};
use tracing::error;

use crate::config::DatabaseConfig;

pub struct DBManager {
    pub pool: PgPool,
}

impl DBManager {
    pub fn from_config(config: &DatabaseConfig) -> Self {
        let conn_options = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.user)
            .password(&config.password)
            .database(&config.database)
            .ssl_mode(PgSslMode::Prefer);

        let pool = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .idle_timeout(Duration::from_secs(config.idle_timeout))
            .connect_lazy_with(conn_options);

        Self { pool }
    }

    pub async fn insert(&self, event: Event) -> Result<()> {
        match event {
            Event::Tick(t) => self.insert_tick(t).await?,
            Event::Trade(t) => self.insert_trade(t).await?,
            _ => {
                error!("Event type not supported: {}", event.event_type());
            }
        }
        Ok(())
    }

    pub async fn batch_insert(&self, events: &[Event]) -> Result<()> {
        let ticks = events
            .iter()
            .filter_map(|e| match e {
                Event::Tick(t) => Some(t),
                _ => None,
            })
            .cloned()
            .collect::<Vec<_>>();
        self.insert_ticks_batch(ticks).await?;
        Ok(())
    }
}
