use super::DBManager;
use crate::models::{Allocation, Fill, Order, Signal, Tick, Trade};
use anyhow::Result;

impl DBManager {
    pub async fn insert_trade(&self, trade: Trade) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO trades (received_time, event_time, instrument_type, venue, base, quote, maturity, strike, option_type, trade_id, price, quantity, source)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            trade.received_time,
            trade.event_time,
            trade.instrument.instrument_type().to_string(),
            trade.instrument.venue().to_string(),
            trade.instrument.base().to_string(),
            trade.instrument.quote().to_string(),
            trade.instrument.maturity().map(|m| m.value()),
            trade.instrument.strike().map(|s| s.value()),
            trade.instrument.option_type().map(|ot| ot.to_string()),
            trade.trade_id as i64,
            trade.price.value(),
            trade.quantity.value(),
            trade.source.to_string(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_tick(&self, tick: Tick) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO ticks (received_time, event_time, instrument_type, venue, base, quote, maturity, strike, option_type, bid_price, bid_quantity, ask_price, ask_quantity, source)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            tick.received_time,
            tick.event_time,
            tick.instrument.instrument_type().to_string(),
            tick.instrument.venue().to_string(),
            tick.instrument.base().to_string(),
            tick.instrument.quote().to_string(),
            tick.instrument.maturity().map(|m| m.value()),
            tick.instrument.strike().map(|s| s.value()),
            tick.instrument.option_type().map(|ot| ot.to_string()),
            tick.bid_price.value(),
            tick.bid_quantity.value(),
            tick.ask_price.value(),
            tick.ask_quantity.value(),
            tick.source.to_string(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_order(&self, order: Order) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO orders (received_time, event_time, instrument_type, venue, base, quote, maturity, strike, option_type, order_id, strategy_id, order_type, price, avg_fill_price, quantity, quantity_filled, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            "#,
            order.received_time,
            order.event_time,
            order.instrument.instrument_type().to_string(),
            order.instrument.venue().to_string(),
            order.instrument.base().to_string(),
            order.instrument.quote().to_string(),
            order.instrument.maturity().map(|m| m.value()),
            order.instrument.strike().map(|s| s.value()),
            order.instrument.option_type().map(|ot| ot.to_string()),
            order.order_id as i64,
            order.strategy_id,
            order.order_type.to_string(),
            order.price.map(|p| p.value()),
            order.avg_fill_price.map(|p| p.value()),
            order.quantity.value(),
            order.quantity_filled.value(),
            order.status.to_string(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_fill(&self, fill: Fill) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO fills (received_time, event_time, instrument_type, venue, base, quote, maturity, strike, option_type, order_id, strategy_id, price, quantity, commission)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            fill.received_time,
            fill.event_time,
            fill.instrument.instrument_type().to_string(),
            fill.instrument.venue().to_string(),
            fill.instrument.base().to_string(),
            fill.instrument.quote().to_string(),
            fill.instrument.maturity().map(|m| m.value()),
            fill.instrument.strike().map(|s| s.value()),
            fill.instrument.option_type().map(|ot| ot.to_string()),
            fill.order_id.map(|o| o as i64),
            fill.strategy_id,
            fill.price.value(),
            fill.quantity.value(),
            fill.commission.value(),
        )
        .execute(&self.pool).await?;

        Ok(())
    }

    pub async fn insert_signal(&self, signal: Signal) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO signals (received_time, event_time, instrument_type, venue, base, quote, maturity, strike, option_type, strategy_id, signal)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            signal.received_time,
            signal.event_time,
            signal.instrument.instrument_type().to_string(),
            signal.instrument.venue().to_string(),
            signal.instrument.base().to_string(),
            signal.instrument.quote().to_string(),
            signal.instrument.maturity().map(|m| m.value()),
            signal.instrument.strike().map(|s| s.value()),
            signal.instrument.option_type().map(|ot| ot.to_string()),
            signal.strategy_id,
            signal.signal.value(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_allocation(&self, allocation: Allocation) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO allocations (received_time, event_time, instrument_type, venue, base, quote, maturity, strike, option_type, strategy_id, notional)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            allocation.received_time,
            allocation.event_time,
            allocation.instrument.instrument_type().to_string(),
            allocation.instrument.venue().to_string(),
            allocation.instrument.base().to_string(),
            allocation.instrument.quote().to_string(),
            allocation.instrument.maturity().map(|m| m.value()),
            allocation.instrument.strike().map(|s| s.value()),
            allocation.instrument.option_type().map(|ot| ot.to_string()),
            allocation.strategy_id,
            allocation.notional.value(),
        ).execute(&self.pool).await?;

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
        ingestors::IngestorID,
        models::{
            Asset, FutureContract, Instrument, Maturity, Notional, OrderStatus, OrderType, Price, Quantity, Venue,
            Weight,
        },
    };

    #[tokio::test]
    async fn test_insert_trade() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let trade = Trade {
            received_time: OffsetDateTime::now_utc(),
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::Future(FutureContract::new(
                &Venue::Binance,
                &Asset::new("BTC"),
                &Asset::new("USDT"),
                &Maturity::new(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )),
            trade_id: 1,
            price: Price::new(Decimal::new(10000, 2)).unwrap(),
            quantity: Quantity::new(Decimal::new(105, 1)),
            source: IngestorID::Test,
        };

        manager.insert_trade(trade).await.unwrap();

        // Check that the trade was inserted
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM trades")
            .fetch_one(&manager.pool)
            .await
            .expect("SQLX failed to fetch row");
        assert_eq!(row.0, 1)
    }

    #[tokio::test]
    async fn test_insert_tick() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let tick = Tick {
            received_time: OffsetDateTime::now_utc(),
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::Future(FutureContract::new(
                &Venue::Binance,
                &Asset::new("BTC"),
                &Asset::new("USDT"),
                &Maturity::new(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )),
            bid_price: Price::new(Decimal::new(10000, 2)).unwrap(),
            bid_quantity: Quantity::new(Decimal::new(105, 1)),
            ask_price: Price::new(Decimal::new(10001, 2)).unwrap(),
            ask_quantity: Quantity::new(Decimal::new(106, 1)),
            source: IngestorID::Test,
        };

        manager.insert_tick(tick).await.unwrap();

        // Check that the tick was inserted
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ticks")
            .fetch_one(&manager.pool)
            .await
            .expect("SQLX failed to fetch row");
        assert_eq!(row.0, 1)
    }

    #[tokio::test]
    async fn test_insert_order() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let order = Order {
            received_time: OffsetDateTime::now_utc(),
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::Future(FutureContract::new(
                &Venue::Binance,
                &Asset::new("BTC"),
                &Asset::new("USDT"),
                &Maturity::new(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )),
            order_id: 1,
            strategy_id: "test".to_string(),
            order_type: OrderType::Limit,
            price: Some(Price::new(Decimal::new(10000, 2)).unwrap()),
            avg_fill_price: Some(Price::new(Decimal::new(9990, 2)).unwrap()),
            quantity: Quantity::new(Decimal::new(105, 1)),
            quantity_filled: Quantity::new(Decimal::new(105, 1)),
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

    #[tokio::test]
    async fn test_insert_fill() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let fill = Fill {
            received_time: OffsetDateTime::now_utc(),
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::Future(FutureContract::new(
                &Venue::Binance,
                &Asset::new("BTC"),
                &Asset::new("USDT"),
                &Maturity::new(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )),
            order_id: Some(1),
            strategy_id: "test".to_string(),
            price: Price::new(Decimal::new(10000, 2)).unwrap(),
            quantity: Quantity::new(Decimal::new(105, 1)),
            commission: Price::new(Decimal::new(10, 2)).unwrap(),
        };

        manager.insert_fill(fill).await.unwrap();

        // Check that the fill was inserted
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fills")
            .fetch_one(&manager.pool)
            .await
            .expect("SQLX failed to fetch row");
        assert_eq!(row.0, 1)
    }

    #[tokio::test]
    async fn test_insert_signal() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let signal = Signal {
            received_time: OffsetDateTime::now_utc(),
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::Future(FutureContract::new(
                &Venue::Binance,
                &Asset::new("BTC"),
                &Asset::new("USDT"),
                &Maturity::new(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )),
            strategy_id: "test".to_string(),
            signal: Weight::new(Decimal::new(1, 0)),
        };

        manager.insert_signal(signal).await.unwrap();

        // Check that the signal was inserted
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM signals")
            .fetch_one(&manager.pool)
            .await
            .expect("SQLX failed to fetch row");
        assert_eq!(row.0, 1)
    }

    #[tokio::test]
    async fn test_insert_allocation() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let allocation = Allocation {
            received_time: OffsetDateTime::now_utc(),
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::Future(FutureContract::new(
                &Venue::Binance,
                &Asset::new("BTC"),
                &Asset::new("USDT"),
                &Maturity::new(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )),
            strategy_id: "test".to_string(),
            notional: Notional::new(Decimal::new(100000, 2)),
        };

        manager.insert_allocation(allocation).await.unwrap();

        // Check that the allocation was inserted
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM allocations")
            .fetch_one(&manager.pool)
            .await
            .expect("SQLX failed to fetch row");
        assert_eq!(row.0, 1)
    }
}
