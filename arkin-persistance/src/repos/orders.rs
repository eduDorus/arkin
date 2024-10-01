use super::DBManager;
use crate::models::ExecutionOrder;
use anyhow::Result;
use rust_decimal::Decimal;
use time::OffsetDateTime;

#[derive(sqlx::FromRow)]
struct OrderRow {
    event_time: OffsetDateTime,
    instrument_type: String,
    venue: String,
    base: String,
    quote: String,
    maturity: Option<OffsetDateTime>,
    strike: Option<Decimal>,
    option_type: Option<String>,
    order_id: i64,
    strategy_id: String,
    order_type: String,
    price: Option<Decimal>,
    avg_fill_price: Option<Decimal>,
    quantity: Decimal,
    quantity_filled: Decimal,
    status: String,
}

impl From<ExecutionOrder> for OrderRow {
    fn from(order: ExecutionOrder) -> Self {
        Self {
            event_time: order.event_time,
            instrument_type: order.instrument.instrument_type().to_string(),
            venue: order.instrument.venue().to_string(),
            base: order.instrument.base().to_string(),
            quote: order.instrument.quote().to_string(),
            maturity: order.instrument.maturity().map(|m| m.value()),
            strike: order.instrument.strike().map(|s| s.value()),
            option_type: order.instrument.option_type().map(|ot| ot.to_string()),
            order_id: order.order_id as i64,
            strategy_id: order.strategy_id.to_string(),
            order_type: order.order_type.to_string(),
            price: order.price.map(|p| p.value()),
            avg_fill_price: order.avg_fill_price.map(|p| p.value()),
            quantity: order.quantity.value(),
            quantity_filled: order.quantity_filled.value(),
            status: order.status.to_string(),
        }
    }
}

impl DBManager {
    pub async fn insert_order(&self, order: ExecutionOrder) -> Result<()> {
        let order = OrderRow::from(order);
        sqlx::query!(
            r#"
            INSERT INTO orders (event_time, instrument_type, venue, base, quote, maturity, strike, option_type, order_id, strategy_id, order_type, price, avg_fill_price, quantity, quantity_filled, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
            order.event_time,
            order.instrument_type,
            order.venue,
            order.base,
            order.quote,
            order.maturity,
            order.strike,
            order.option_type,
            order.order_id,
            order.strategy_id,
            order.order_type,
            order.price,
            order.avg_fill_price,
            order.quantity,
            order.quantity_filled,
            order.status,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::prelude::*;
    use time::OffsetDateTime;

    use super::*;
    use crate::{
        config,
        models::{Instrument, OrderStatus, OrderType, Venue},
    };

    #[tokio::test]
    #[ignore]
    async fn test_insert_order() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let order = ExecutionOrder {
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into()),
            order_id: 1,
            strategy_id: "test".into(),
            order_type: OrderType::Limit,
            price: Some(Decimal::new(10000, 2).into()),
            avg_fill_price: Some(Decimal::new(9990, 2).into()),
            quantity: Decimal::new(105, 1).into(),
            quantity_filled: Decimal::new(105, 1).into(),
            status: OrderStatus::Filled,
        };

        manager.insert_order(order).await.unwrap();

        // Check that the order was inserted
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM orders")
            .fetch_one(&manager.pool)
            .await
            .expect("SQLX failed to fetch row");
        assert_eq!(row.0, 1)
    }
}
