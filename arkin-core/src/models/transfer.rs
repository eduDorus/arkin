use std::{fmt, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use strum::Display;
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{AccountQuery, Asset, AssetQuery, EventPayload, InstrumentQuery, PersistenceReader, StrategyQuery};

use super::{Account, Instrument, Strategy};

#[derive(Debug, Display, Clone, Copy, PartialEq, Type, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "transfer_group_type", rename_all = "snake_case")]
pub enum TransferGroupType {
    Initial,
    Deposit,
    Withdrawal,
    Trade,
    Exchange,
    Interest,
    Funding,
    Settlement,
    Liquidation,
    Adjustment,
    Reconciliation,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Type, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "transfer_type", rename_all = "snake_case")]
pub enum TransferType {
    Transfer,
    Pnl,
    UnrealizedPnl,
    Margin,
    Commission,
    Rebate,
    Reconciliation,
}

/// A single same-currency "transfer" in double-entry style.
/// In TigerBeetle's lingo, this is one row in the ledger for a single currency.
#[derive(Debug, Clone, TypedBuilder)]
pub struct Transfer {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    /// The ID of the transfer group this transfer belongs to.
    pub transfer_group_id: Uuid,
    /// Type of transfer group
    pub transfer_group_type: TransferGroupType,
    /// Transfer type (e.g. deposit, withdrawal, trade, etc.)
    pub transfer_type: TransferType,
    /// The account that is debited (balance goes down).
    pub debit_account: Arc<Account>,
    /// The account that is credited (balance goes up).
    pub credit_account: Arc<Account>,
    /// The amount of this transfer.
    pub amount: Decimal,
    /// The Unit Price of the asset being transferred.
    pub unit_price: Decimal,
    /// strategy that initiated this transfer.
    pub strategy: Option<Arc<Strategy>>,
    /// Instrument that this transfer is related to.
    pub instrument: Option<Arc<Instrument>>,
    /// The asset being transferred.
    pub asset: Option<Arc<Asset>>,
    /// The time of this transfer.
    pub created: UtcDateTime,
}

impl Transfer {
    pub fn has_transfer_type(&self, transfer_type: &TransferType) -> bool {
        &self.transfer_type == transfer_type
    }

    pub fn has_strategy(&self, strategy: &Arc<Strategy>) -> bool {
        self.strategy.as_ref() == Some(strategy)
    }

    pub fn has_instrument(&self, instrument: &Arc<Instrument>) -> bool {
        self.instrument.as_ref() == Some(instrument)
    }

    pub fn has_asset(&self, asset: &Arc<Asset>) -> bool {
        self.asset.as_ref() == Some(asset)
    }
}

#[async_trait]
impl EventPayload for Transfer {
    type Dto = TransferDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let debit_account = persistence
            .get_account(&AccountQuery::builder().id(dto.debit_account_id).build())
            .await
            .context(format!("Failed to get debit account with id {}", dto.debit_account_id))?;
        let credit_account = persistence
            .get_account(&AccountQuery::builder().id(dto.credit_account_id).build())
            .await
            .context(format!("Failed to get credit account with id {}", dto.credit_account_id))?;

        let strategy = if let Some(sid) = dto.strategy_id {
            persistence.get_strategy(&StrategyQuery::builder().id(sid).build()).await.ok()
        } else {
            None
        };

        let instrument = if let Some(iid) = dto.instrument_id {
            persistence
                .get_instrument(&InstrumentQuery::builder().id(iid).build())
                .await
                .ok()
        } else {
            None
        };

        let asset = if let Some(aid) = dto.asset_id {
            persistence.get_asset(&AssetQuery::builder().id(aid).build()).await.ok()
        } else {
            None
        };

        Ok(Transfer::builder()
            .id(dto.id)
            .transfer_group_id(dto.transfer_group_id)
            .transfer_group_type(dto.transfer_group_type)
            .transfer_type(dto.transfer_type)
            .debit_account(debit_account)
            .credit_account(credit_account)
            .amount(dto.amount)
            .unit_price(dto.unit_price)
            .strategy(strategy)
            .instrument(instrument)
            .asset(asset)
            .created(dto.created)
            .build())
    }
}

impl fmt::Display for Transfer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}: {}{} {}{} @ {}",
            self.created,
            self.transfer_type,
            // self.debit_account,
            // self.credit_account,
            if self.debit_account.is_user_account() {
                "-"
            } else {
                "+"
            },
            self.amount,
            match &self.asset {
                Some(asset) => format!("{}", asset),
                None => "".to_string(),
            },
            match &self.instrument {
                Some(inst) => format!("{}", inst),
                None => "".to_string(),
            },
            self.unit_price
        )
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct TransferBatch {
    pub event_time: UtcDateTime,
    pub transfers: Vec<Arc<Transfer>>,
}

#[async_trait]
impl EventPayload for TransferBatch {
    type Dto = TransferBatchDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let mut transfers = Vec::new();
        for t in dto.transfers {
            transfers.push(Arc::new(Transfer::from_dto(t, persistence.clone()).await?));
        }
        Ok(TransferBatch::builder()
            .event_time(dto.event_time)
            .transfers(transfers)
            .build())
    }
}

impl fmt::Display for TransferBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "event_time={} transfers_count={}", self.event_time, self.transfers.len())
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransferDto {
    pub id: Uuid,
    pub transfer_group_id: Uuid,
    pub transfer_group_type: TransferGroupType,
    pub transfer_type: TransferType,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub amount: Decimal,
    pub unit_price: Decimal,
    pub strategy_id: Option<Uuid>,
    pub instrument_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub created: UtcDateTime,
}

impl From<Transfer> for TransferDto {
    fn from(transfer: Transfer) -> Self {
        Self {
            id: transfer.id,
            transfer_group_id: transfer.transfer_group_id,
            transfer_group_type: transfer.transfer_group_type,
            transfer_type: transfer.transfer_type,
            debit_account_id: transfer.debit_account.id,
            credit_account_id: transfer.credit_account.id,
            amount: transfer.amount,
            unit_price: transfer.unit_price,
            strategy_id: transfer.strategy.map(|s| s.id),
            instrument_id: transfer.instrument.map(|i| i.id),
            asset_id: transfer.asset.map(|a| a.id),
            created: transfer.created,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransferBatchDto {
    pub event_time: UtcDateTime,
    pub transfers: Vec<TransferDto>,
}

impl From<TransferBatch> for TransferBatchDto {
    fn from(batch: TransferBatch) -> Self {
        Self {
            event_time: batch.event_time,
            transfers: batch.transfers.iter().map(|t| t.as_ref().clone().into()).collect(),
        }
    }
}
