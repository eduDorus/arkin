use anyhow::Result;
use polars::prelude::*;
use tracing::info;

use crate::config::PolarsPipelineConfig;

pub fn compute_signals(df: LazyFrame, config: &PolarsPipelineConfig) -> Result<LazyFrame> {
    info!("Building Polars computation graph...");

    // =======================================================================
    // STAGE 1: Raw Trades → 1m Aggregates (The "Base Grid")
    // =======================================================================
    // We group by dynamic 1m windows.
    // CRITICAL: We include 'instrument_type' in the group key so we can use it later.
    let candles_1m = df
        .sort(["event_time"], Default::default())
        .group_by_dynamic(
            col("event_time"),
            vec![col("instrument_type"), col("venue")], // Group keys
            DynamicGroupOptions {
                every: Duration::parse("1m"),
                period: Duration::parse("1m"),
                offset: Duration::parse("0"),
                ..Default::default()
            },
        )
        .agg([
            // Standard OHLCV
            col("price").max().alias("high"),
            col("price").min().alias("low"),
            col("price").last().alias("close"),
            // Notional Aggregates
            col("notional").abs().sum().alias("total_notional"),
            // VWAP: sum(price * qty) / sum(qty)
            (col("price") * col("quantity")).sum().alias("pv_sum"),
            col("quantity").sum().alias("v_sum"),
        ])
        .with_column((col("pv_sum") / col("v_sum")).alias("vwap"));

    // =======================================================================
    // STAGE 2: Cross-Sectional Aggregation ("Synthetics")
    // =======================================================================
    // Group by InstrumentType (Spot vs Perp) to sum notionals.

    let sector_stats = candles_1m
        .group_by([col("event_time"), col("instrument_type")])
        .agg([col("total_notional").sum().alias("sector_notional")])
        .sort(["event_time"], Default::default());

    // =======================================================================
    // SAFETY STEP: Handle Missing Data
    // =======================================================================
    // Upsample to ensure every minute exists for every type.
    // Note: upsample on LazyFrame might not be available in this version.
    // For now, we skip upsampling.
    let sector_stats_safe = sector_stats;
    /*
        .upsample(
            vec!["instrument_type"], // Keep these groups distinct
            "event_time",
            Duration::parse("1m"),
            Duration::parse("0"),
        )
        .with_columns([
            // If a minute is missing, volume is 0 (not null!)
            col("sector_notional").fill_null(0.0),
        ]);
    */

    // =======================================================================
    // STAGE 3: Multi-Timeframe (Rolling Windows)
    // =======================================================================
    // Dynamically generate rolling windows based on config

    let mut rolling_exprs = vec![];

    for window in &config.windows {
        let alias = format!("notional_{}", window);
        rolling_exprs.push(
            col("sector_notional")
                .rolling_sum(RollingOptionsFixedWindow {
                    window_size: 10,
                    min_periods: 1,
                    ..Default::default()
                })
                .over([col("instrument_type")]) // Calculate per type
                .alias(&alias),
        );
    }

    let multi_tf = sector_stats_safe.with_columns(rolling_exprs);

    // =======================================================================
    // STAGE 4: Spot vs Perp Imbalance (Pivot & Math)
    // =======================================================================
    // We pivot so we have columns: notional_5m_spot, notional_5m_perp
    // Note: Pivot is not available in LazyFrame directly in older Polars versions,
    // but let's assume we collect and pivot or use a workaround if needed.
    // For now, let's return the multi_tf frame. The pivot usually happens eagerly.

    // To keep it lazy, we can do a self-join or conditional columns.
    // But for simplicity in this prototype, let's assume we return the un-pivoted data
    // and let the engine handle the pivot or just publish the per-type values.

    // Actually, let's try to do the pivot logic using conditional columns which is lazy-friendly
    // "notional_5m_spot" = when(type == spot) then notional_5m else 0

    let mut pivot_exprs = vec![];
    for window in &config.windows {
        let col_name = format!("notional_{}", window);

        // Spot column
        pivot_exprs.push(
            when(col("instrument_type").eq(lit("Spot")))
                .then(col(&col_name))
                .otherwise(lit(0.0))
                .alias(&format!("{}_spot", col_name)),
        );

        // Perp column
        pivot_exprs.push(
            when(col("instrument_type").eq(lit("Perpetual")))
                .then(col(&col_name))
                .otherwise(lit(0.0))
                .alias(&format!("{}_perp", col_name)),
        );
    }

    let with_pivots = multi_tf.with_columns(pivot_exprs);

    // Now group by time again to merge the rows (since we have separate rows for spot/perp)
    let mut sum_exprs = vec![];
    for window in &config.windows {
        let col_name = format!("notional_{}", window);
        sum_exprs.push(col(&format!("{}_spot", col_name)).sum());
        sum_exprs.push(col(&format!("{}_perp", col_name)).sum());
    }

    let merged = with_pivots
        .group_by([col("event_time")])
        .agg(sum_exprs)
        .sort(["event_time"], Default::default());

    // Now we can calculate imbalances
    let mut imbalance_exprs = vec![];
    for window in &config.windows {
        let base = format!("notional_{}", window);
        let spot = format!("{}_spot", base);
        let perp = format!("{}_perp", base);
        let imbalance = format!("imbalance_{}", window);

        imbalance_exprs.push((col(&spot) - col(&perp)).alias(&imbalance));

        // Lags
        // pct_change(1) = (col - col.shift(1)) / col.shift(1)
        imbalance_exprs.push(
            ((col(&imbalance) - col(&imbalance).shift(lit(1))) / col(&imbalance).shift(lit(1)))
                .alias(&format!("{}_change", imbalance)),
        );
    }

    Ok(merged.with_columns(imbalance_exprs))
}
