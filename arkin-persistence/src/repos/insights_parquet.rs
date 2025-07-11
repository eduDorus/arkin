use std::{fmt, sync::Arc};

use arrow::{
    array::{Float64Builder, RecordBatch, StringBuilder, TimestampSecondBuilder},
    datatypes::{DataType, Field, Schema, TimeUnit},
};
use object_store::{local::LocalFileSystem, path::Path};
use parquet::{
    arrow::{async_writer::ParquetObjectWriter, AsyncArrowWriter},
    basic::{Compression, ZstdLevel},
    file::properties::WriterProperties,
};
use rust_decimal::prelude::*;
use tokio::sync::Mutex;
use tracing::info;
use typed_builder::TypedBuilder;

use crate::PersistenceError;

use super::InsightDTO;

#[derive(Clone, TypedBuilder)]
pub struct InsightsParquetRepo {
    schema: Arc<Schema>,
    writer: Arc<Mutex<AsyncArrowWriter<ParquetObjectWriter>>>,
}

impl fmt::Debug for InsightsParquetRepo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InsightsParquetRepo").finish()
    }
}

impl InsightsParquetRepo {
    pub async fn new(output_path: &str) -> Result<Self, PersistenceError> {
        // Initialize schema only once
        let schema = Arc::new(Schema::new(vec![
            Field::new("event_time", DataType::Timestamp(TimeUnit::Second, None), false),
            Field::new("pipeline_id", DataType::Utf8, false),
            Field::new("instrument_id", DataType::Utf8, true),
            Field::new("feature_id", DataType::Utf8, false),
            Field::new("value", DataType::Float64, false),
        ]));

        let store = Arc::new(LocalFileSystem::new_with_prefix("/Users/dj/repos/arkin/data/parquet/insights/").unwrap());
        let object_store_writer = ParquetObjectWriter::new(store.clone(), Path::from(output_path));

        let writer_props = WriterProperties::builder()
            .set_compression(Compression::ZSTD(ZstdLevel::try_new(3).unwrap()))
            .build();

        // Create and hold onto the writer
        let writer = AsyncArrowWriter::try_new(object_store_writer, schema.clone(), Some(writer_props)).unwrap();
        let writer = Arc::new(Mutex::new(writer));
        Ok(Self { schema, writer })
    }

    pub async fn insert_batch(&self, insights: &[InsightDTO]) -> Result<(), PersistenceError> {
        // Pre-allocate builders with appropriate capacity
        let capacity = insights.len();
        let mut event_time_builder = TimestampSecondBuilder::with_capacity(capacity);
        let mut pipeline_id_builder = StringBuilder::with_capacity(capacity, capacity * 36); // rough guess for UUID length
        let mut instrument_id_builder = StringBuilder::with_capacity(capacity, capacity * 36);
        let mut feature_id_builder = StringBuilder::with_capacity(capacity, capacity * 20); // Adjust as needed
        let mut value_builder = Float64Builder::with_capacity(capacity);

        for insight in insights {
            event_time_builder.append_value(insight.event_time.unix_timestamp());
            pipeline_id_builder.append_value(insight.pipeline_id.to_string().as_str());

            match insight.instrument_id {
                Some(uuid_val) => instrument_id_builder.append_value(uuid_val.to_string().as_str()),
                None => instrument_id_builder.append_null(),
            }

            feature_id_builder.append_value(&insight.feature_id);
            value_builder.append_value(insight.value.to_f64().unwrap());
        }

        let event_time = event_time_builder.finish();
        let pipeline_id = pipeline_id_builder.finish();
        let instrument_id = instrument_id_builder.finish();
        let feature_id = feature_id_builder.finish();
        let value = value_builder.finish();

        let batch = RecordBatch::try_new(
            self.schema.clone(),
            vec![
                Arc::new(event_time),
                Arc::new(pipeline_id),
                Arc::new(instrument_id),
                Arc::new(feature_id),
                Arc::new(value),
            ],
        )
        .unwrap();

        let mut writer = self.writer.lock().await;
        writer.write(&batch).await.unwrap();

        Ok(())
    }

    pub async fn close(&self) -> Result<(), PersistenceError> {
        let mut writer = self.writer.lock().await;
        writer.finish().await.unwrap();
        info!("Parquet writer closed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repos::InsightDTO;
    use rust_decimal::Decimal;
    use time::UtcDateTime;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_insert_batch() {
        let repo = InsightsParquetRepo::new("data.parquet").await.unwrap();

        let insights = vec![
            InsightDTO {
                event_time: UtcDateTime::now(),
                pipeline_id: Uuid::new_v4(),
                instrument_id: Some(Uuid::new_v4()),
                feature_id: "feature1".to_string(),
                value: Decimal::new(123, 2),
            },
            InsightDTO {
                event_time: UtcDateTime::now(),
                pipeline_id: Uuid::new_v4(),
                instrument_id: Some(Uuid::new_v4()),
                feature_id: "feature2".to_string(),
                value: Decimal::new(456, 2),
            },
        ];

        repo.insert_batch(&insights).await.unwrap();
        repo.close().await.unwrap();
    }
}
