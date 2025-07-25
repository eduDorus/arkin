use std::sync::Arc;

use rust_decimal::Decimal;

use arkin_core::prelude::*;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{context::PersistenceContext, PersistenceError};

#[derive(Debug, Clone)]
pub struct VenueOrderDTO {
    pub id: VenueOrderId,
    pub strategy_id: Option<Uuid>,
    pub instrument_id: Uuid,
    pub side: MarketSide,
    pub order_type: VenueOrderType,
    pub time_in_force: VenueOrderTimeInForce,
    pub price: Decimal,
    pub quantity: Decimal,
    pub last_fill_price: Decimal,
    pub last_fill_quantity: Decimal,
    pub last_fill_commission: Decimal,
    pub filled_price: Decimal,
    pub filled_quantity: Decimal,
    pub commission_asset_id: Option<Uuid>,
    pub commission: Decimal,
    pub status: VenueOrderStatus,
    pub created: OffsetDateTime,
    pub updated: OffsetDateTime,
}

impl From<VenueOrder> for VenueOrderDTO {
    fn from(order: VenueOrder) -> Self {
        Self {
            id: order.id,
            strategy_id: order.strategy.map(|o| o.id),
            instrument_id: order.instrument.id,
            side: order.side,
            order_type: order.order_type,
            time_in_force: order.time_in_force,
            price: order.price,
            quantity: order.quantity,
            last_fill_price: order.last_fill_price,
            last_fill_quantity: order.last_fill_quantity,
            last_fill_commission: order.last_fill_commission,
            filled_price: order.filled_price,
            filled_quantity: order.filled_quantity,
            commission_asset_id: order.commission_asset.as_ref().map(|asset| asset.id),
            commission: order.commission,
            status: order.status,
            created: order.created.into(),
            updated: order.updated.into(),
        }
    }
}

impl From<Arc<VenueOrder>> for VenueOrderDTO {
    fn from(order: Arc<VenueOrder>) -> Self {
        Self {
            id: order.id,
            strategy_id: order.strategy.as_ref().map(|o| o.id),
            instrument_id: order.instrument.id,
            side: order.side,
            order_type: order.order_type,
            time_in_force: order.time_in_force,
            price: order.price,
            quantity: order.quantity,
            last_fill_price: order.last_fill_price,
            last_fill_quantity: order.last_fill_quantity,
            last_fill_commission: order.last_fill_commission,
            filled_price: order.filled_price,
            filled_quantity: order.filled_quantity,
            commission_asset_id: order.commission_asset.as_ref().map(|asset| asset.id),
            commission: order.commission,
            status: order.status,
            created: order.created.into(),
            updated: order.updated.into(),
        }
    }
}

pub async fn insert(ctx: &PersistenceContext, order: VenueOrderDTO) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            INSERT INTO venue_orders
            (
                id, 
                instance_id,
                strategy_id, 
                instrument_id, 
                side, 
                order_type, 
                time_in_force, 
                price, 
                quantity, 
                last_fill_price,
                last_fill_quantity,
                last_fill_commission,
                filled_price, 
                filled_quantity, 
                commission_asset_id,
                commission, 
                status, 
                created,
                updated
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#,
        order.id,
        ctx.instance.id,
        order.strategy_id,
        order.instrument_id,
        order.side as MarketSide,
        order.order_type as VenueOrderType,
        order.time_in_force as VenueOrderTimeInForce,
        order.price,
        order.quantity,
        order.last_fill_price,
        order.last_fill_quantity,
        order.last_fill_commission,
        order.filled_price,
        order.filled_quantity,
        order.commission_asset_id,
        order.commission,
        order.status as VenueOrderStatus,
        order.created,
        order.updated,
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}

pub async fn update(ctx: &PersistenceContext, order: VenueOrderDTO) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            UPDATE venue_orders
            SET
                last_fill_price = $2,
                last_fill_quantity = $3,
                last_fill_commission = $4,
                filled_price = $5,
                filled_quantity = $6,
                commission_asset_id = $7,
                commission = $8,
                status = $9,
                updated = $10
            WHERE id = $1
            "#,
        order.id,
        order.last_fill_price,
        order.last_fill_quantity,
        order.last_fill_commission,
        order.filled_price,
        order.filled_quantity,
        order.commission_asset_id,
        order.commission,
        order.status as VenueOrderStatus,
        order.updated,
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}

pub async fn delete(ctx: &PersistenceContext, id: VenueOrderId) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            DELETE FROM venue_orders
            WHERE id = $1
            "#,
        id
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}
