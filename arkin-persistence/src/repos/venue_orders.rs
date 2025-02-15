use std::sync::Arc;

use rust_decimal::Decimal;
use sqlx::PgPool;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::PersistenceError;

#[derive(Debug, Clone)]
pub struct VenueOrderDTO {
    pub id: VenueOrderId,
    pub portfolio_id: Uuid,
    pub instrument_id: Uuid,
    pub side: MarketSide,
    pub order_type: VenueOrderType,
    pub time_in_force: VenueOrderTimeInForce,
    pub price: Decimal,
    pub quantity: Decimal,
    pub fill_price: Decimal,
    pub filled_quantity: Decimal,
    pub total_commission: Decimal,
    pub status: VenueOrderStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<VenueOrder> for VenueOrderDTO {
    fn from(order: VenueOrder) -> Self {
        Self {
            id: order.id,
            portfolio_id: order.portfolio.id,
            instrument_id: order.instrument.id,
            side: order.side,
            order_type: order.order_type,
            time_in_force: order.time_in_force,
            price: order.price,
            quantity: order.quantity,
            fill_price: order.filled_price,
            filled_quantity: order.filled_quantity,
            total_commission: order.commission,
            status: order.status,
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}

impl From<Arc<VenueOrder>> for VenueOrderDTO {
    fn from(order: Arc<VenueOrder>) -> Self {
        Self {
            id: order.id,
            portfolio_id: order.portfolio.id,
            instrument_id: order.instrument.id,
            side: order.side,
            order_type: order.order_type,
            time_in_force: order.time_in_force,
            price: order.price,
            quantity: order.quantity,
            fill_price: order.filled_price,
            filled_quantity: order.filled_quantity,
            total_commission: order.commission,
            status: order.status,
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct VenueOrderRepo {
    pool: PgPool,
}

impl VenueOrderRepo {
    pub async fn insert(&self, order: VenueOrderDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO venue_orders
            (
                id, 
                portfolio_id, 
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
            order.portfolio_id,
            order.instrument_id,
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

    pub async fn update(&self, order: VenueOrderDTO) -> Result<(), PersistenceError> {
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

    pub async fn delete(&self, id: VenueOrderId) -> Result<(), PersistenceError> {
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
