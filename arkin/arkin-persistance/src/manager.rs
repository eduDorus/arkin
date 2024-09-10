use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool, Pool, Postgres, Transaction,
};

use crate::config::DatabaseConfig;

#[async_trait]
pub trait Persistable {
    async fn save(&self, tx: &mut Transaction<Postgres>) -> Result<()>;
}

#[async_trait]
pub trait Readable {
    async fn read(&self, pool: &Pool<Postgres>) -> Result<()>;
}

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

    pub async fn insert(&self, event: &dyn Persistable) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        event.save(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn batch_insert(&self, events: Vec<&dyn Persistable>) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        for event in events {
            event.save(&mut tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
