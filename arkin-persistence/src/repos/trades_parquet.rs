use core::fmt;
use std::sync::Arc;

use arrow::{
    array::AsArray,
    datatypes::{Decimal128Type, Int64Type, TimestampMicrosecondType},
};
use datafusion::{execution::object_store::ObjectStoreUrl, prelude::*, scalar::ScalarValue};
use object_store::local::LocalFileSystem;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::PersistenceError;

use super::TradeDTO;

#[derive(Clone, TypedBuilder)]
pub struct TradeParquetRepo {
    ctx: Arc<SessionContext>,
}

impl fmt::Debug for TradeParquetRepo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TradeParquetRepo").finish()
    }
}

impl TradeParquetRepo {
    pub async fn new() -> Result<Self, PersistenceError> {
        // create local execution context
        let ctx = SessionContext::new();

        let store_url = ObjectStoreUrl::parse("file://").unwrap();
        let store = Arc::new(LocalFileSystem::new());

        ctx.register_object_store(store_url.as_ref(), store);

        ctx.register_parquet(
            "trades",
            "/Users/dj/repos/arkin/data/parquet/trades/trades.parquet",
            ParquetReadOptions::default(),
        )
        .await
        .unwrap();

        Ok(Self { ctx: Arc::new(ctx) })
    }

    pub async fn insert(&self, _trade: TradeDTO) -> Result<(), PersistenceError> {
        Ok(())
    }

    pub async fn insert_batch(&self, _trades: Vec<TradeDTO>) -> Result<(), PersistenceError> {
        Ok(())
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<TradeDTO>, PersistenceError> {
        // instrument_id as FixedSizeBinary(16)
        let instrument_id_exprs: Vec<_> = instrument_ids
            .iter()
            .map(|id| {
                let bytes = id.as_bytes();
                lit(ScalarValue::FixedSizeBinary(16, Some(bytes.to_vec())))
            })
            .collect();

        let from_us = from.unix_timestamp() * 1_000_000 + i64::from(from.microsecond());
        let to_us = to.unix_timestamp() * 1_000_000 + i64::from(to.microsecond());

        let from_val = ScalarValue::TimestampMicrosecond(Some(from_us), None);
        let to_val = ScalarValue::TimestampMicrosecond(Some(to_us), None);

        let time_expr = col("event_time").gt_eq(lit(from_val)).and(col("event_time").lt_eq(lit(to_val)));

        let instrument_expr = datafusion::logical_expr::in_list(col("instrument_id"), instrument_id_exprs, false);

        let filter_expr = time_expr.and(instrument_expr);

        let sort_expr = vec![col("event_time").sort(true, true)];

        let df = self
            .ctx
            .table("trades")
            .await
            .unwrap()
            .filter(filter_expr)
            .unwrap()
            // .sort(sort_expr)
            // .unwrap()
            .select(vec![
                col("event_time"),
                col("instrument_id"),
                col("trade_id"),
                col("side"),
                col("price"),
                col("quantity"),
            ])
            .unwrap()
            .collect()
            .await
            .unwrap();

        let mut results = Vec::new();

        for batch in df {
            debug!("{:?}", batch.schema());
            let schema = batch.schema();

            let event_time_idx = schema.index_of("event_time").unwrap();
            let instrument_id_idx = schema.index_of("instrument_id").unwrap();
            let trade_id_idx = schema.index_of("trade_id").unwrap();
            let side_idx = schema.index_of("side").unwrap();
            let price_idx = schema.index_of("price").unwrap();
            let quantity_idx = schema.index_of("quantity").unwrap();
            debug!("{:?}", batch.column(event_time_idx).data_type());
            debug!("{:?}", batch.column(instrument_id_idx).data_type());
            debug!("{:?}", batch.column(trade_id_idx).data_type());
            debug!("{:?}", batch.column(side_idx).data_type());
            debug!("{:?}", batch.column(price_idx).data_type());
            debug!("{:?}", batch.column(quantity_idx).data_type());

            let event_time_arr = batch.column(event_time_idx).as_primitive::<TimestampMicrosecondType>();
            let instrument_id_arr = batch.column(instrument_id_idx).as_fixed_size_binary();
            let trade_id_arr = batch.column(trade_id_idx).as_primitive::<Int64Type>();
            let side_arr = batch.column(side_idx).as_string_view();
            let price_arr = batch.column(price_idx).as_primitive::<Decimal128Type>();
            let quantity_arr = batch.column(quantity_idx).as_primitive::<Decimal128Type>();

            for i in 0..batch.num_rows() {
                let us_val = event_time_arr.value(i);
                let nanos = us_val as i128 * 1000;
                let odt = OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap();

                let bytes = instrument_id_arr.value(i);
                let inst_id = Uuid::from_bytes(bytes.try_into().unwrap());

                let trade_id = trade_id_arr.value(i);

                let side_str = side_arr.value(i);
                let side = match side_str {
                    "buy" => MarketSide::Buy,
                    "sell" => MarketSide::Sell,
                    _ => panic!("Handle unknown side"),
                };

                let price_val = Decimal::from_i128_with_scale(price_arr.value(i), 8).normalize();
                let quantity_val = Decimal::from_i128_with_scale(quantity_arr.value(i), 8).normalize();

                let trade = TradeDTO {
                    event_time: odt,
                    instrument_id: inst_id,
                    trade_id,
                    side,
                    price: price_val,
                    quantity: quantity_val,
                };

                results.push(trade);
            }
        }

        Ok(results)
    }
}
