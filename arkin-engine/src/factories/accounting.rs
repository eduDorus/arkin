use std::sync::Arc;

use clap::ValueEnum;
use rust_decimal_macros::dec;
use strum::Display;

use arkin_accounting::prelude::*;
use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use uuid::Uuid;

#[derive(Debug, Display, Clone, ValueEnum)]
pub enum AccountingServiceType {
    Ledger,
}

pub struct AccountingFactory {}

impl AccountingFactory {
    pub async fn init(
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
        accouting_type: &AccountingServiceType,
    ) -> Arc<dyn AccountingService> {
        // let config = load::<AccountingConfig>();
        let portfolio: Arc<dyn AccountingService> = match accouting_type {
            AccountingServiceType::Ledger => {
                let personal_venue = persistence
                    .venue_store
                    .read_by_id(&Uuid::parse_str("b8b9dcf2-77ea-4d24-964e-8243bb7298ea").expect("Failed to parse UUID"))
                    .await
                    .expect("Failed to read the initial venue from the database");
                let binance_venue = persistence
                    .venue_store
                    .read_by_id(&Uuid::parse_str("48adfe42-29fb-4402-888a-0204bf417e32").expect("Failed to parse UUID"))
                    .await
                    .expect("Failed to read the initial venue from the database");
                let asset = persistence
                    .asset_store
                    .read_by_symbol("USDT")
                    .await
                    .expect("Failed to read asset from DB");

                let accounting = LedgerAccounting::builder()
                    .pubsub(pubsub.handle("LedgerAccounting").await)
                    .build();
                accounting
                    .deposit(
                        &personal_venue,
                        &binance_venue,
                        &asset.into(),
                        dec!(100_000),
                        &AccountType::Margin,
                    )
                    .await
                    .expect("Failed to deposit initial funds");

                Arc::new(accounting)
            }
        };
        portfolio
    }
}
