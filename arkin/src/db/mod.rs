mod models;

use crate::{
    config::ClickhouseConfig,
    models::{Event, EventID},
};
use anyhow::Result;
use clickhouse::{insert::Insert, Client, Row};
use models::TickCH;
use serde::Serialize;
use strum::IntoEnumIterator;

impl EventID {
    pub fn table_name(&self) -> &'static str {
        match self {
            EventID::TickUpdate => "ticks",
            EventID::TradeUpdate => "trades",
            EventID::BookUpdate => "book_updates",
            _ => "unknown",
        }
    }
}

#[derive(Clone)]
pub struct ClickhouseConnector {
    pub client: Client,
}

impl ClickhouseConnector {
    pub fn new(config: &ClickhouseConfig) -> Self {
        let client = Client::default()
            .with_url(config.url.to_owned())
            .with_user(config.user.to_owned())
            .with_password(config.password.to_owned())
            .with_database(config.database.to_owned());

        Self { client }
    }

    pub async fn batch_insert<T: Row + Serialize>(&self, events: Vec<Event>) -> Result<()> {
        let mut inserts: Vec<T> = Vec::new();

        for event in events {
            match event {
                Event::TickUpdate(e) => {
                    let trade: TickCH = e.into();
                    inserts.push(trade);
                }
                Event::TradeUpdate(e) => inserts.push(e),
                _ => {}
            }
        }
        for event_type in EventID::iter() {
            let table_name = event_type.table_name();
            let events = events
                .iter()
                .filter(|event| matches!(event.event_type(), event_type))
                .collect::<Vec<_>>();

            if events.is_empty() {
                continue;
            }

            let mut insert: Insert<T> = self.client.insert(table_name)?;
            for event in events {
                insert.write(event).await?;
            }
            insert.end().await?;
        }
        Ok(())
    }

    pub async fn insert<T: Row + Serialize>(&self, table_name: &str, rows: Vec<T>) -> Result<()> {
        let mut inserter = self.client.insert(table_name)?;

        for row in rows {
            inserter.write(&row).await?;
        }

        inserter.end().await?;
        Ok(())
    }
}
