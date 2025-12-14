use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_core::PersistenceError;

use crate::context::PersistenceContext;
use crate::repos::ch::audit_repo::{self, AuditClickhouseDTO};

pub async fn create_table(ctx: &PersistenceContext) -> Result<(), PersistenceError> {
    audit_repo::create_table(ctx).await
}

pub async fn insert(ctx: &PersistenceContext, event: Arc<Event>) -> Result<(), PersistenceError> {
    let dto = AuditClickhouseDTO {
        event_time: event.timestamp().into(),
        instance_id: ctx.instance.id,
        event_type: event.event_type().to_string(),
        message: event.to_string(),
    };
    audit_repo::insert(ctx, dto).await
}

pub async fn insert_batch(ctx: &PersistenceContext, events: &[Event]) -> Result<(), PersistenceError> {
    if events.is_empty() {
        return Ok(());
    }

    let instance_id = ctx.instance.id;
    let dtos: Vec<AuditClickhouseDTO> = events
        .iter()
        .map(|event| {
            let event_type = event.event_type().to_string();
            let message = event.to_string();
            AuditClickhouseDTO {
                event_time: event.timestamp().into(),
                instance_id,
                event_type,
                message,
            }
        })
        .collect();

    audit_repo::insert_batch(ctx, &dtos).await
}
