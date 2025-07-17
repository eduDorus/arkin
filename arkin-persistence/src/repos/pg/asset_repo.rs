use std::sync::Arc;

use arkin_core::prelude::*;
use sqlx::prelude::*;
use uuid::Uuid;

use crate::{context::PersistenceContext, PersistenceError};

#[derive(FromRow)]
pub struct AssetDTO {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub asset_type: AssetType,
}

impl From<Arc<Asset>> for AssetDTO {
    fn from(asset: Arc<Asset>) -> Self {
        Self {
            id: asset.id,
            symbol: asset.symbol.clone(),
            name: asset.name.clone(),
            asset_type: asset.asset_type.clone(),
        }
    }
}

impl From<AssetDTO> for Arc<Asset> {
    fn from(asset: AssetDTO) -> Self {
        let asset = Asset {
            id: asset.id,
            symbol: asset.symbol,
            name: asset.name,
            asset_type: asset.asset_type,
        };
        Arc::new(asset)
    }
}

pub async fn insert(ctx: &PersistenceContext, asset: AssetDTO) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            INSERT INTO assets 
            (
                id, 
                symbol, 
                name,
                asset_type
            ) VALUES ($1, $2, $3, $4)
            "#,
        asset.id,
        asset.symbol,
        asset.name,
        asset.asset_type as AssetType,
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<AssetDTO, PersistenceError> {
    let asset = sqlx::query_as!(
        AssetDTO,
        r#"
            SELECT
                id,
                symbol,
                name,
                asset_type AS "asset_type:AssetType"
            FROM assets
            WHERE id = $1
            "#,
        id,
    )
    .fetch_optional(&ctx.pg_pool)
    .await?;

    match asset {
        Some(asset) => Ok(asset),
        None => Err(PersistenceError::NotFound),
    }
}

pub async fn read_by_symbol(ctx: &PersistenceContext, symbol: &str) -> Result<AssetDTO, PersistenceError> {
    let asset = sqlx::query_as!(
        AssetDTO,
        r#"
            SELECT
                id,
                symbol,
                name,
                asset_type AS "asset_type:AssetType"
            FROM assets
            WHERE symbol = $1
            "#,
        symbol,
    )
    .fetch_optional(&ctx.pg_pool)
    .await?;

    match asset {
        Some(asset) => Ok(asset),
        None => Err(PersistenceError::NotFound),
    }
}
