use std::sync::Arc;

use rust_decimal::Decimal;
use sqlx::PgPool;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::PersistenceError;

#[derive(Debug, Clone)]
pub struct ExecutionOrderDTO {
    pub id: Uuid,
    pub event_time: OffsetDateTime,
    pub strategy_id: Option<Uuid>,
    pub instrument_id: Uuid,
    pub exec_strategy_type: ExecutionStrategyType,
    pub side: MarketSide,
    pub price: Decimal,
    pub quantity: Decimal,
    pub fill_price: Decimal,
    pub filled_quantity: Decimal,
    pub total_commission: Decimal,
    pub status: ExecutionOrderStatus,
    pub updated_at: OffsetDateTime,
}

impl From<ExecutionOrder> for ExecutionOrderDTO {
    fn from(order: ExecutionOrder) -> Self {
        Self {
            id: order.id,
            event_time: order.created_at.into(),
            strategy_id: order.strategy.as_ref().map(|s| s.id),
            instrument_id: order.instrument.id,
            exec_strategy_type: order.exec_strategy_type,
            side: order.side,
            price: order.price,
            quantity: order.quantity,
            fill_price: order.fill_price,
            filled_quantity: order.filled_quantity,
            total_commission: order.total_commission,
            status: order.status,
            updated_at: order.updated_at.into(),
        }
    }
}

impl From<Arc<ExecutionOrder>> for ExecutionOrderDTO {
    fn from(order: Arc<ExecutionOrder>) -> Self {
        Self {
            id: order.id,
            event_time: order.created_at.into(),
            strategy_id: order.strategy.as_ref().map(|s| s.id),
            instrument_id: order.instrument.id,
            exec_strategy_type: order.exec_strategy_type,
            side: order.side,
            price: order.price,
            quantity: order.quantity,
            fill_price: order.fill_price,
            filled_quantity: order.filled_quantity,
            total_commission: order.total_commission,
            status: order.status,
            updated_at: order.updated_at.into(),
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ExecutionOrderRepo {
    pool: PgPool,
    instance: Arc<Instance>,
}

impl ExecutionOrderRepo {
    pub async fn insert(&self, order: ExecutionOrderDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO execution_orders
            (
                id, 
                event_time, 
                instance_id, 
                strategy_id,
                instrument_id, 
                order_type, 
                side, 
                price, 
                quantity, 
                fill_price, 
                filled_quantity, 
                total_commission, 
                status, 
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            order.id,
            order.event_time,
            self.instance.id,
            order.strategy_id,
            order.instrument_id,
            order.exec_strategy_type as ExecutionStrategyType,
            order.side as MarketSide,
            order.price,
            order.quantity,
            order.fill_price,
            order.filled_quantity,
            order.total_commission,
            order.status as ExecutionOrderStatus,
            order.updated_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update(&self, order: ExecutionOrderDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            UPDATE execution_orders
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
            order.status as ExecutionOrderStatus,
            order.updated_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &Uuid) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            DELETE FROM execution_orders
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
