use anyhow::Result;
use sqlx::PgPool;

use arkin_core::prelude::*;

#[derive(Debug)]
pub struct VenueOrdersRepo {
    pool: PgPool,
}

impl VenueOrdersRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, order: VenueOrder) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO venue_orders
            (
                id, 
                instance_id, 
                instrument_id, 
                side, 
                order_type, 
                time_in_force, 
                price, 
                quantity, 
                fill_price, 
                filled_quantity, 
                total_commission, 
                status, 
                created_at, 
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            order.id,
            order.instance.id,
            order.instrument.id,
            order.side as MarketSide,
            order.order_type as VenueOrderType,
            order.time_in_force as VenueOrderTimeInForce,
            order.price,
            order.quantity,
            order.fill_price,
            order.filled_quantity,
            order.total_commission,
            order.status as VenueOrderStatus,
            order.created_at,
            order.updated_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update(&self, order: VenueOrder) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE venue_orders
            SET
                fill_price = $2,
                filled_quantity = $3,
                total_commission = $4,
                status = $5,
                updated_at = $6
            WHERE id = $1
            "#,
            order.id,
            order.fill_price,
            order.filled_quantity,
            order.total_commission,
            order.status as VenueOrderStatus,
            order.updated_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: VenueOrderId) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM venue_orders
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
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
    use uuid::Uuid;

    #[test(tokio::test)]
    async fn test_venue_order_repo() {
        let pool = connect_database();
        let repo = VenueOrdersRepo::new(pool);

        let instance = test_instance();
        let instrument = test_inst_binance_btc_usdt_perp();

        let mut order = VenueOrderBuilder::default()
            .id(Uuid::new_v4())
            .instance(instance.clone())
            .instrument(instrument.clone())
            .order_type(VenueOrderType::Market)
            .side(MarketSide::Buy)
            .price(None)
            .quantity(dec!(1))
            .status(VenueOrderStatus::New)
            .build()
            .unwrap();
        repo.insert(order.clone()).await.unwrap();

        order.status = VenueOrderStatus::Placed;
        order.updated_at = OffsetDateTime::now_utc();
        repo.update(order.clone()).await.unwrap();

        order.fill_price = dec!(110);
        order.filled_quantity = dec!(1);
        order.total_commission = dec!(0.2);
        order.status = VenueOrderStatus::Filled;
        order.updated_at = OffsetDateTime::now_utc();
        repo.update(order.clone()).await.unwrap();

        // repo.delete(order.id).await.unwrap();
    }
}
