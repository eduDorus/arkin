use anyhow::Result;
use derive_builder::Builder;
use sqlx::PgPool;
use tracing::debug;

use arkin_core::prelude::*;

use crate::BIND_LIMIT;

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct VenueOrderFillsRepo {
    pool: PgPool,
}

impl VenueOrderFillsRepo {
    pub async fn insert(&self, order: VenueOrderFill) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO venue_order_fills
            (
                event_time, 
                instance_id, 
                venue_order_id, 
                instrument_id, 
                side, 
                price, 
                quantity,
                commission
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            order.event_time,
            order.instance.id,
            order.venue_order.id,
            order.instrument.id,
            order.side as MarketSide,
            order.price,
            order.quantity,
            order.commission,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, fills: Vec<VenueOrderFill>) -> Result<()> {
        // Build batched insert queries
        for batch in fills.chunks(BIND_LIMIT / 8) {
            // Create a query builder
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO venue_order_fills
                (
                    event_time, 
                    instance_id,
                    venue_order_id, 
                    instrument_id, 
                    side, 
                    price, 
                    quantity,
                    commission
                ) ",
            );

            // Note that `.into_iter()` wasn't needed here since `users` is already an iterator.
            query_builder.push_values(batch, |mut b, order| {
                // If you wanted to bind these by-reference instead of by-value,
                // you'd need an iterator that yields references that live as long as `query_builder`,
                // e.g. collect it to a `Vec` first.
                b.push_bind(order.event_time)
                    .push_bind(order.instance.id)
                    .push_bind(order.venue_order.id)
                    .push_bind(order.instrument.id)
                    .push_bind(order.side as MarketSide)
                    .push_bind(order.price)
                    .push_bind(order.quantity)
                    .push_bind(order.commission);
            });

            let query = query_builder.build();

            query.execute(&self.pool).await?;
        }
        debug!("Saved {} venue orders", fills.len());
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::test_utils::connect_database;

    use super::*;
    use rust_decimal_macros::dec;
    use test_log::test;
    use time::OffsetDateTime;

    #[test(tokio::test)]
    async fn test_venue_order_fill_repo() {
        let pool = connect_database();
        let repo = VenueOrderFillsRepoBuilder::default().pool(pool).build().unwrap();

        let instrument = test_inst_binance_btc_usdt_perp();
        let instance = test_instance();
        let order = test_venue_order();

        let fill = VenueOrderFillBuilder::default()
            .event_time(OffsetDateTime::now_utc())
            .instance(instance.clone())
            .venue_order(order.clone())
            .instrument(instrument.clone())
            .price(dec!(110))
            .quantity(dec!(1))
            .commission(dec!(0.51))
            .side(MarketSide::Buy)
            .build()
            .unwrap();
        repo.insert(fill).await.unwrap();
    }
}
