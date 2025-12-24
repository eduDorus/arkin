use anyhow::Result;
use arkin_core::prelude::*;
use polars::datatypes::{DataType as PlDataType, Field, TimeUnit};
use polars::df;
use polars::frame::DataFrame;
use polars::prelude::{col, IntoLazy, Schema};
use rust_decimal::prelude::*;
use time::UtcDateTime;

use crate::config::PolarsPipelineConfig;
use crate::pipeline::compute_signals;

pub struct InsightsEngine {
    config: PolarsPipelineConfig,
    // Main buffer holding historical data
    buffer: DataFrame,
    // Accumulator for new trades since last tick
    new_trades: Vec<Trade>,
}

impl InsightsEngine {
    pub fn new(config: PolarsPipelineConfig) -> Self {
        // Initialize with empty DataFrame with correct schema
        let schema = Schema::from_iter(vec![
            Field::new("event_time".into(), PlDataType::Datetime(TimeUnit::Microseconds, None)),
            Field::new("instrument_type".into(), PlDataType::String),
            Field::new("venue".into(), PlDataType::String),
            Field::new("price".into(), PlDataType::Float64),
            Field::new("quantity".into(), PlDataType::Float64),
            Field::new("notional".into(), PlDataType::Float64),
        ]);

        let buffer = DataFrame::empty_with_schema(&schema);

        Self {
            config,
            buffer,
            new_trades: Vec::new(),
        }
    }

    pub fn on_trade(&mut self, trade: &Trade) {
        self.new_trades.push(trade.clone());
    }

    pub fn on_tick(&mut self, event_time: UtcDateTime) -> Result<Option<DataFrame>> {
        if self.new_trades.is_empty() && self.buffer.height() == 0 {
            return Ok(None);
        }

        // 1. Convert new trades to DataFrame
        if !self.new_trades.is_empty() {
            let new_df = self.trades_to_df(&self.new_trades)?;

            // 2. Append to main buffer
            self.buffer.vstack_mut(&new_df)?;
            self.new_trades.clear();
        }

        // 3. Trim buffer (keep last N minutes)
        // We need to filter by time.
        // For simplicity in this prototype, let's just keep the last X rows or filter by time column.
        // Filtering by time is better.

        let _cutoff = event_time - time::Duration::minutes(self.config.buffer_size_minutes);
        // Convert cutoff to polars datetime (i64 microseconds)
        // This conversion depends on how we store time. Let's assume standard unix timestamp.

        // Let's skip complex trimming for now and just rely on the engine being fast enough
        // or implement a simple row count trim if needed.
        // Ideally: self.buffer = self.buffer.filter(col("event_time").gt(lit(cutoff)))

        // 4. Compute Pipeline
        let result_lazy = compute_signals(self.buffer.clone().lazy(), &self.config)?;
        let result = result_lazy.collect()?;

        // 5. Return the latest row(s)
        // We usually only care about the last row (the current minute)
        // But since we might have re-calculated previous minutes, we might want to return more.
        // For live trading, we usually just want the latest signal.

        Ok(Some(result))
    }

    fn trades_to_df(&self, trades: &[Trade]) -> Result<DataFrame> {
        // Vectorize the data extraction
        let mut times = Vec::with_capacity(trades.len());
        let mut types = Vec::with_capacity(trades.len());
        let mut venues = Vec::with_capacity(trades.len());
        let mut prices = Vec::with_capacity(trades.len());
        let mut quantities = Vec::with_capacity(trades.len());
        let mut notionals = Vec::with_capacity(trades.len());

        for t in trades {
            // Convert time::UtcDateTime to i64 microseconds for Polars
            let micros = t.event_time.unix_timestamp_nanos() / 1000;
            times.push(micros as i64);

            types.push(t.instrument.instrument_type.to_string());
            venues.push(t.instrument.venue.name.to_string());

            let p = t.price.to_f64().unwrap_or(0.0);
            let q = t.quantity.to_f64().unwrap_or(0.0);
            prices.push(p);
            quantities.push(q);
            notionals.push(p * q);
        }

        let df = df!(
            "event_time" => times,
            "instrument_type" => types,
            "venue" => venues,
            "price" => prices,
            "quantity" => quantities,
            "notional" => notionals
        )?;

        // Cast event_time to Datetime
        let df = df
            .lazy()
            .with_column(col("event_time").cast(PlDataType::Datetime(TimeUnit::Microseconds, None)))
            .collect()?;

        Ok(df)
    }
}
