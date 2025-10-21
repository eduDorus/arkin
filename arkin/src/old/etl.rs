use clickhouse::Client;
use polars::prelude::DataType as PolarsDataType;
use polars::prelude::*;
use rust_decimal::Decimal;
use std::time::Instant;
use time::{format_description, Duration, OffsetDateTime};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create ClickHouse client
    let client = Client::default()
        .with_url("http://192.168.100.100:8123")
        .with_user("arkin_admin")
        .with_password("test1234")
        .with_database("arkin");

    println!("✅ ClickHouse client created!");

    // Performance test: Load last 3 hours in 1-minute batches
    let now = OffsetDateTime::now_utc();
    let three_hours_ago = now - Duration::hours(3);

    println!("\n🎯 Performance Test: Loading 3 hours of data in 1-minute batches");
    println!(
        "📅 Time range: {} to {}",
        three_hours_ago.format(&format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")?)?,
        now.format(&format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")?)?
    );

    let format = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")?;

    // Start performance timer
    let total_start = Instant::now();

    // Calculate number of batches (180 batches for 3 hours)
    let total_minutes = 180;
    let mut batch_count = 0;
    let mut total_rows = 0;

    // This will hold our aggregated features (1-minute bars with rolling features)
    let mut feature_frames: Vec<DataFrame> = Vec::new();

    println!("\n⏳ Iterative Feature Engineering Workflow...\n");
    println!("📋 Steps per batch:");
    println!("  1. Load 1min of trades");
    println!("  2. Aggregate to notional volume (buy/sell split)");
    println!("  3. Calculate 15min rolling average over aggregations");
    println!("  4. Append to feature DataFrame\n");

    // Iterative feature engineering: Load 1min -> Aggregate -> Calculate rolling features
    for i in 0..total_minutes {
        let batch_start_time = Instant::now();

        let batch_end = three_hours_ago + Duration::minutes(i + 1);
        let batch_start = three_hours_ago + Duration::minutes(i);

        let start_str = batch_start.format(&format)?;
        let end_str = batch_end.format(&format)?;

        // ============================================================
        // STEP 1: Load 1min of trades into DataFrame
        // ============================================================
        let query = format!(
            "SELECT 
                event_time,
                instrument_id,
                trade_id,
                side,
                price,
                quantity
            FROM trades FINAL
            WHERE event_time >= toDateTime64('{}', 3)
              AND event_time < toDateTime64('{}', 3)
            ORDER BY event_time ASC",
            start_str, end_str
        );

        let mut cursor = client.query(&query).fetch::<ClickhouseRow>()?;

        let mut event_times: Vec<i64> = Vec::new();
        let mut instrument_ids: Vec<String> = Vec::new();
        let mut trade_ids: Vec<u64> = Vec::new();
        let mut sides: Vec<i8> = Vec::new();
        let mut prices: Vec<f64> = Vec::new();
        let mut quantities: Vec<f64> = Vec::new();

        while let Some(row) = cursor.next().await? {
            event_times.push((row.event_time.unix_timestamp_nanos() / 1_000_000) as i64);
            instrument_ids.push(row.instrument_id.to_string());
            trade_ids.push(row.trade_id);
            sides.push(row.side);
            prices.push(row.price.to_string().parse::<f64>()?);
            quantities.push(row.quantity.to_string().parse::<f64>()?);
        }

        let batch_rows = event_times.len();

        if batch_rows > 0 {
            // Create DataFrame for this 1-minute batch
            let event_time_series = Series::new("event_time".into(), event_times)
                .cast(&PolarsDataType::Datetime(TimeUnit::Milliseconds, None))?;

            let trades_df = DataFrame::new(vec![
                event_time_series.into(),
                Series::new("instrument_id".into(), instrument_ids).into(),
                Series::new("trade_id".into(), trade_ids).into(),
                Series::new("side".into(), sides).into(),
                Series::new("price".into(), prices).into(),
                Series::new("quantity".into(), quantities).into(),
            ])?;

            // ============================================================
            // STEP 2: Aggregate 1min to notional volume (buy/sell split)
            // ============================================================
            // Notional = price * quantity
            let minute_agg = trades_df
                .lazy()
                .with_columns([(col("price") * col("quantity")).alias("notional")])
                .group_by([col("instrument_id"), col("side")])
                .agg([
                    col("event_time").first().alias("timestamp"),
                    col("notional").sum().alias("notional_volume"),
                    col("trade_id").count().alias("trade_count"),
                ])
                .collect()?;

            // Pivot to have buy_notional and sell_notional as separate columns
            let minute_features = minute_agg
                .lazy()
                .with_columns([lit(batch_start.unix_timestamp() * 1000)
                    .cast(PolarsDataType::Datetime(TimeUnit::Milliseconds, None))
                    .alias("bar_timestamp")])
                .select([
                    col("bar_timestamp"),
                    col("instrument_id"),
                    when(col("side").eq(lit(1)))
                        .then(col("notional_volume"))
                        .otherwise(lit(0.0))
                        .alias("buy_notional_temp"),
                    when(col("side").eq(lit(-1)))
                        .then(col("notional_volume"))
                        .otherwise(lit(0.0))
                        .alias("sell_notional_temp"),
                    when(col("side").eq(lit(1)))
                        .then(col("trade_count"))
                        .otherwise(lit(0))
                        .alias("buy_count_temp"),
                    when(col("side").eq(lit(-1)))
                        .then(col("trade_count"))
                        .otherwise(lit(0))
                        .alias("sell_count_temp"),
                ])
                .group_by([col("bar_timestamp"), col("instrument_id")])
                .agg([
                    col("buy_notional_temp").sum().alias("buy_notional"),
                    col("sell_notional_temp").sum().alias("sell_notional"),
                    col("buy_count_temp").sum().alias("buy_count"),
                    col("sell_count_temp").sum().alias("sell_count"),
                ])
                .with_columns([
                    (col("buy_notional") + col("sell_notional")).alias("total_notional"),
                    (col("buy_count") + col("sell_count")).alias("total_count"),
                ])
                .collect()?;

            // Add this minute's features to our collection
            feature_frames.push(minute_features);
            batch_count += 1;
            total_rows += batch_rows;

            let batch_elapsed = batch_start_time.elapsed();
            if i % 30 == 0 || i == total_minutes - 1 {
                println!(
                    "  Batch {}/{}: {} trades -> {} instruments in {:?}",
                    i + 1,
                    total_minutes,
                    batch_rows,
                    feature_frames.last().unwrap().height(),
                    batch_elapsed
                );
            }
        }
    }

    println!(
        "\n✅ Loaded {} batches with {} total trades in {:?}",
        batch_count,
        total_rows,
        total_start.elapsed()
    );

    // ============================================================
    // STEP 3: Concatenate all 1-minute feature bars
    // ============================================================
    println!("\n🔗 Concatenating {} 1-minute feature bars...", feature_frames.len());
    let concat_start = Instant::now();

    let features_df = if !feature_frames.is_empty() {
        // Concatenate all feature frames
        let mut combined = feature_frames[0].clone();
        for i in 1..feature_frames.len() {
            combined.vstack_mut(&feature_frames[i])?;
        }
        combined
    } else {
        return Err("No data found in the time range".into());
    };

    println!("✅ Concatenation completed in {:?}", concat_start.elapsed());
    println!("📊 Features DataFrame shape: {:?}", features_df.shape());

    // ============================================================
    // STEP 4: Calculate 15-minute rolling average of notional volume
    // ============================================================
    println!("\n📊 Calculating 15-minute rolling features...");
    let rolling_start = Instant::now();

    // Add buy/sell ratio and prepare for rolling calculations
    let features_with_ratio = features_df
        .lazy()
        .sort(["instrument_id", "bar_timestamp"], SortMultipleOptions::default())
        .with_columns([(col("buy_notional") / (col("buy_notional") + col("sell_notional"))).alias("buy_sell_ratio")])
        .collect()?;

    // Manual rolling window calculation per instrument
    // This is necessary in Polars 0.45 since rolling_mean doesn't work with over() directly
    let total_notional_col = features_with_ratio.column("total_notional")?;
    let buy_notional_col = features_with_ratio.column("buy_notional")?;
    let sell_notional_col = features_with_ratio.column("sell_notional")?;
    let instrument_col = features_with_ratio.column("instrument_id")?;

    let mut notional_15min_avg: Vec<f64> = Vec::with_capacity(features_with_ratio.height());
    let mut buy_notional_15min_avg: Vec<f64> = Vec::with_capacity(features_with_ratio.height());
    let mut sell_notional_15min_avg: Vec<f64> = Vec::with_capacity(features_with_ratio.height());

    for i in 0..features_with_ratio.height() {
        let current_instrument = instrument_col.str()?.get(i).unwrap();

        // Find the window start (up to 15 bars back for same instrument)
        let mut window_start = i;
        let mut count = 0;
        while window_start > 0 && count < 15 {
            window_start -= 1;
            if instrument_col.str()?.get(window_start).unwrap() == current_instrument {
                count += 1;
            } else {
                window_start += 1;
                break;
            }
        }

        // Calculate averages for the window
        let mut total_sum = 0.0;
        let mut buy_sum = 0.0;
        let mut sell_sum = 0.0;
        let mut window_count = 0;

        for j in window_start..=i {
            if instrument_col.str()?.get(j).unwrap() == current_instrument {
                total_sum += total_notional_col.f64()?.get(j).unwrap();
                buy_sum += buy_notional_col.f64()?.get(j).unwrap();
                sell_sum += sell_notional_col.f64()?.get(j).unwrap();
                window_count += 1;
            }
        }

        notional_15min_avg.push(total_sum / window_count as f64);
        buy_notional_15min_avg.push(buy_sum / window_count as f64);
        sell_notional_15min_avg.push(sell_sum / window_count as f64);
    }

    // Add rolling columns to dataframe using Polars' efficient column operations
    let df_with_rolling = features_with_ratio
        .lazy()
        .with_columns([
            Series::new("notional_15min_avg".into(), notional_15min_avg).lit(),
            Series::new("buy_notional_15min_avg".into(), buy_notional_15min_avg).lit(),
            Series::new("sell_notional_15min_avg".into(), sell_notional_15min_avg).lit(),
        ])
        .collect()?;

    println!("✅ Rolling features calculated in {:?}", rolling_start.elapsed());

    println!("\n🎉 Feature DataFrame created!");
    println!("{}", df_with_rolling.head(Some(20)));

    println!("\n📈 Final DataFrame shape: {:?}", df_with_rolling.shape());
    println!("📋 Schema:\n{:?}", df_with_rolling.schema());

    // Feature Analysis & Statistics
    if df_with_rolling.height() > 0 {
        println!("\n📊 Feature Analysis...");
        let analysis_start = Instant::now();

        // Analyze features by instrument
        let feature_summary = df_with_rolling
            .clone()
            .lazy()
            .group_by([col("instrument_id")])
            .agg([
                col("total_notional").mean().alias("avg_notional_per_min"),
                col("total_notional").std(1).alias("std_notional"),
                col("notional_15min_avg").last().alias("latest_15min_avg"),
                col("buy_sell_ratio").mean().alias("avg_buy_sell_ratio"),
                col("total_count").sum().alias("total_trades"),
                col("bar_timestamp").count().alias("num_bars"),
            ])
            .sort(
                ["avg_notional_per_min"],
                SortMultipleOptions::default().with_order_descending(true),
            )
            .collect()?;

        println!("✅ Feature analysis computed in {:?}", analysis_start.elapsed());
        println!("\n🔢 Feature Summary by Instrument (sorted by avg notional):");
        println!("{}", feature_summary.head(Some(10)));

        // Show sample of most recent features with rolling values
        let recent_features = df_with_rolling
            .clone()
            .lazy()
            .sort(
                ["bar_timestamp", "instrument_id"],
                SortMultipleOptions::default().with_order_descending(true),
            )
            .select([
                col("bar_timestamp"),
                col("instrument_id"),
                col("total_notional"),
                col("notional_15min_avg"),
                col("buy_notional"),
                col("sell_notional"),
                col("buy_sell_ratio"),
            ])
            .limit(15)
            .collect()?;

        println!("\n📈 Most Recent Features (with 15min rolling avg):");
        println!("{}", recent_features);

        // Validate rolling window calculation
        println!("\n✅ Feature Engineering Summary:");
        println!("  • Total 1-minute bars: {}", df_with_rolling.height());
        println!(
            "  • Unique instruments: {}",
            df_with_rolling.column("instrument_id")?.n_unique()?
        );
        println!("  • Time range covered: {} minutes", feature_frames.len());
        println!("  • Features per bar: {}", df_with_rolling.width());

        // Show sample of final feature DataFrame
        println!("\n📋 Sample of feature DataFrame (last 20 rows):");
        let tail = df_with_rolling.tail(Some(20));
        println!("{}", tail);

        // Show rolling feature stats
        println!("\n📊 Rolling Feature Statistics:");
        let rolling_stats = df_with_rolling
            .clone()
            .lazy()
            .select([
                col("notional_15min_avg").mean().alias("avg_notional_15min"),
                col("notional_15min_avg").std(1).alias("std_notional_15min"),
                col("buy_sell_ratio").mean().alias("avg_buy_sell_ratio"),
                col("buy_sell_ratio").std(1).alias("std_buy_sell_ratio"),
            ])
            .collect()?;
        println!("{}", rolling_stats);
    } else {
        println!("⚠️  No data found in the time range");
    }

    println!("\n🎉 Total elapsed time: {:?}", total_start.elapsed());

    Ok(())
}

// Define a struct to deserialize ClickHouse rows
#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct ClickhouseRow {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    event_time: OffsetDateTime,
    #[serde(with = "clickhouse::serde::uuid")]
    instrument_id: Uuid,
    trade_id: u64,
    side: i8,
    #[serde(with = "arkin_core::prelude::custom_serde::decimal64")]
    price: Decimal,
    #[serde(with = "arkin_core::prelude::custom_serde::decimal64")]
    quantity: Decimal,
}
