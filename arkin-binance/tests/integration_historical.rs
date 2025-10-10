// use std::fmt;
// use std::io::{Cursor, Read};
// use std::time::Duration;

// use anyhow::Result;
// use csv::ReaderBuilder;
// use futures::{stream, StreamExt};
// use reqwest::get;
// use rust_decimal::Decimal;
// use serde::{Deserialize, Deserializer};
// use time::format_description;
// use time::{macros::utc_datetime, UtcDateTime};
// use tokio::pin;
// use tracing::info;
// use zip::ZipArchive;

// use arkin_core::prelude::*;

// const MAX_CONCURRENT: usize = 1;

// const USDM_BASE_URL: &str = "https://data.binance.vision/data/futures/um/daily";
// const CM_BASE_URL: &str = "https://data.binance.vision/data/futures/cm/daily";
// const SPOT_BASE_URL: &str = "https://data.binance.vision/data/spot/daily";

// pub enum BinanceChannel {
//     AggTrades,
//     Metrics,
//     MarkPriceKlines,
//     IndexPriceKlines,
// }

// impl fmt::Display for BinanceChannel {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             BinanceChannel::AggTrades => write!(f, "aggTrades"),
//             BinanceChannel::Metrics => write!(f, "metrics"),
//             BinanceChannel::MarkPriceKlines => write!(f, "markPriceKlines"),
//             BinanceChannel::IndexPriceKlines => write!(f, "indexPriceKlines"),
//         }
//     }
// }

// impl BinanceChannel {
//     pub fn is_kline(&self) -> bool {
//         matches!(self, BinanceChannel::MarkPriceKlines | BinanceChannel::IndexPriceKlines)
//     }
// }

// #[derive(Debug, Deserialize)]
// pub struct BinanceFuturesAggTrade {
//     pub agg_trade_id: u64,
//     pub price: Decimal,
//     pub quantity: Decimal,
//     pub first_trade_id: u64,
//     pub last_trade_id: u64,
//     #[serde(with = "custom_serde::timestamp")]
//     pub transact_time: UtcDateTime,
//     pub is_buyer_maker: bool,
// }

// fn datetime_from_string<'de, D>(deserializer: D) -> Result<UtcDateTime, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let s: &str = Deserialize::deserialize(deserializer)?;
//     let fmt =
//         format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").map_err(serde::de::Error::custom)?;
//     UtcDateTime::parse(s, &fmt).map_err(serde::de::Error::custom)
// }

// #[derive(Debug, Deserialize)]
// pub struct BinanceFuturesMetrics {
//     #[serde(deserialize_with = "datetime_from_string")]
//     pub create_time: UtcDateTime,
//     pub symbol: String,
//     pub sum_open_interest: Decimal,
//     pub sum_open_interest_value: Decimal,
//     pub count_toptrader_long_short_ratio: Option<Decimal>,
//     pub sum_toptrader_long_short_ratio: Option<Decimal>,
//     pub count_long_short_ratio: Option<Decimal>,
//     pub sum_taker_long_short_vol_ratio: Option<Decimal>,
// }

// // open_time	open	high	low	close	volume	close_time	quote_volume	count	taker_buy_volume	taker_buy_quote_volume	ignore
// #[derive(Debug, Deserialize)]
// pub struct BinanceFuturesMarkPriceKline {
//     #[serde(with = "custom_serde::timestamp")]
//     pub open_time: UtcDateTime,
//     pub open: Decimal,
//     pub high: Decimal,
//     pub low: Decimal,
//     pub close: Decimal,
//     pub volume: Decimal,
//     #[serde(with = "custom_serde::timestamp")]
//     pub close_time: UtcDateTime,
//     pub quote_volume: Decimal,
//     pub count: u64,
//     pub taker_buy_volume: Decimal,
//     pub taker_buy_quote_volume: Decimal,
//     pub ignore: String,
// }

// #[derive(Debug, Deserialize)]
// pub struct BinanceFuturesIndexPriceKline {
//     #[serde(with = "custom_serde::timestamp")]
//     pub open_time: UtcDateTime,
//     pub open: Decimal,
//     pub high: Decimal,
//     pub low: Decimal,
//     pub close: Decimal,
//     pub volume: Decimal,
//     #[serde(with = "custom_serde::timestamp")]
//     pub close_time: UtcDateTime,
//     pub quote_volume: Decimal,
//     pub count: u64,
//     pub taker_buy_volume: Decimal,
//     pub taker_buy_quote_volume: Decimal,
//     pub ignore: String,
// }

// // "a": 12345,         // Aggregate trade ID
// // "p": "0.001",       // Price
// // "q": "100",         // Quantity
// // "f": 100,           // First trade ID
// // "l": 105,           // Last trade ID
// // "T": 1672515782136, // Trade time
// // "m": true,          // Is the buyer the market maker?
// // "M": true           // Ignore

// fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let s: &str = Deserialize::deserialize(deserializer)?;
//     match s.to_lowercase().as_str() {
//         "true" => Ok(true),
//         "false" => Ok(false),
//         _ => Err(serde::de::Error::custom(format!("Invalid bool: {}", s))),
//     }
// }

// #[derive(Debug, Deserialize)]
// pub struct BinanceSpotAggTrade {
//     pub agg_trade_id: u64,
//     pub price: Decimal,
//     pub quantity: Decimal,
//     pub first_trade_id: u64,
//     pub last_trade_id: u64,
//     #[serde(with = "custom_serde::timestamp")]
//     pub transact_time: UtcDateTime,
//     #[serde(deserialize_with = "bool_from_string")]
//     pub is_buyer_maker: bool,
//     #[serde(deserialize_with = "bool_from_string")]
//     pub is_best_match: bool,
// }

// pub fn construct_urls(
//     instruments: &[String],
//     start: UtcDateTime,
//     end: UtcDateTime,
//     base_url: &str,
//     channel: BinanceChannel,
// ) -> Vec<String> {
//     // Let's create a vector of dates in "YYYY-MM-DD" format from start to end
//     let mut dates = Vec::new();
//     let mut current_date = start;
//     while current_date <= end {
//         dates.push(
//             current_date
//                 .format(&format_description::parse("[year]-[month]-[day]").unwrap())
//                 .unwrap(),
//         );
//         current_date += Duration::from_secs(86400); // 1 day in seconds
//     }

//     // Create the urls
//     instruments
//         .iter()
//         .flat_map(|instrument| {
//             dates
//                 .iter()
//                 .map(|date| {
//                     // Create URL
//                     if channel.is_kline() {
//                         format!(
//                             "{}/{}/{}/{}/{}-{}-{}.zip",
//                             base_url, channel, instrument, "1m", instrument, "1m", date
//                         )
//                     } else {
//                         format!(
//                             "{}/{}/{}/{}-{}-{}.zip",
//                             base_url, channel, instrument, instrument, channel, date
//                         )
//                     }
//                 })
//                 .collect::<Vec<_>>()
//         })
//         .collect::<Vec<_>>()
// }

// pub async fn download<T>(url: String, has_headers: bool) -> Result<Vec<T>>
// where
//     T: for<'de> Deserialize<'de>,
// {
//     info!("Downloading from URL: {}", url);

//     // Fetch ZIP bytes
//     let resp = get(url).await?;
//     let bytes = resp.bytes().await?;

//     // Unzip in-memory
//     let mut archive = ZipArchive::new(Cursor::new(bytes))?;
//     let mut file = archive.by_index(0)?; // Assume first file is the CSV
//     let mut csv_data = String::new();
//     file.read_to_string(&mut csv_data)?;

//     // Parse CSV
//     let mut rdr = ReaderBuilder::new()
//         .has_headers(has_headers) // Binance aggTrades CSVs are headerless
//         .from_reader(csv_data.as_bytes());

//     // print rows for debugging
//     // for result in rdr.records() {
//     //     let record = result?;
//     //     info!("{:?}", record);
//     // }

//     // Collect all trades, propagating any deserialization errors
//     rdr.deserialize().collect::<Result<_, _>>().map_err(Into::into)
// }

// #[tokio::test]
// #[test_log::test]
// async fn test_binance_usdm_agg_trades() -> Result<()> {
//     info!("Testing Binance Usdm AggTrade download");
//     let instruments = vec![
//         "BTCUSDT".to_string(),
//         "ETHUSDT".to_string(),
//         "BNBUSDT".to_string(),
//         "XRPUSDT".to_string(),
//         "SOLUSDT".to_string(),
//     ];

//     let start = utc_datetime!(2025 - 10 - 01 00:00:00);
//     let end = utc_datetime!(2025 - 10 - 03 00:00:00);

//     let stream = stream::iter(construct_urls(
//         &instruments,
//         start,
//         end,
//         USDM_BASE_URL,
//         BinanceChannel::AggTrades,
//     ))
//     .map(|url| download::<BinanceFuturesAggTrade>(url, true))
//     .buffer_unordered(MAX_CONCURRENT);
//     pin!(stream);
//     while let Some(result) = stream.next().await {
//         match result {
//             Ok(trades) => info!("Stream trade count: {}", trades.len()),
//             Err(e) => info!("Stream error: {}", e),
//         }
//     }

//     Ok(())
// }

// #[tokio::test]
// #[test_log::test]
// async fn test_binance_usdm_metrics() -> Result<()> {
//     info!("Testing Binance UsdM metrics download");
//     let instruments = vec![
//         "BTCUSDT".to_string(),
//         "ETHUSDT".to_string(),
//         "BNBUSDT".to_string(),
//         "XRPUSDT".to_string(),
//         "SOLUSDT".to_string(),
//     ];

//     let start = utc_datetime!(2025 - 10 - 01 00:00:00);
//     let end = utc_datetime!(2025 - 10 - 03 00:00:00);

//     let stream = stream::iter(construct_urls(&instruments, start, end, USDM_BASE_URL, BinanceChannel::Metrics))
//         .map(|url| download::<BinanceFuturesMetrics>(url, true))
//         .buffer_unordered(MAX_CONCURRENT);
//     pin!(stream);
//     while let Some(result) = stream.next().await {
//         match result {
//             Ok(metrics) => {
//                 info!("Stream metrics count: {}", metrics.len());
//             }
//             Err(e) => info!("Stream error: {}", e),
//         }
//     }

//     Ok(())
// }

// #[tokio::test]
// #[test_log::test]
// async fn test_binance_usdm_mark_price_klines() -> Result<()> {
//     info!("Testing Binance UsdM mark price klines download");
//     let instruments = vec![
//         "BTCUSDT".to_string(),
//         "ETHUSDT".to_string(),
//         "BNBUSDT".to_string(),
//         "XRPUSDT".to_string(),
//         "SOLUSDT".to_string(),
//     ];

//     let start = utc_datetime!(2025 - 10 - 01 00:00:00);
//     let end = utc_datetime!(2025 - 10 - 03 00:00:00);

//     let stream = stream::iter(construct_urls(
//         &instruments,
//         start,
//         end,
//         USDM_BASE_URL,
//         BinanceChannel::MarkPriceKlines,
//     ))
//     .map(|url| download::<BinanceFuturesMarkPriceKline>(url, true))
//     .buffer_unordered(MAX_CONCURRENT);
//     pin!(stream);
//     while let Some(result) = stream.next().await {
//         match result {
//             Ok(metrics) => {
//                 info!("Stream metrics count: {}", metrics.len());
//             }
//             Err(e) => info!("Stream error: {}", e),
//         }
//     }

//     Ok(())
// }

// #[tokio::test]
// #[test_log::test]
// async fn test_binance_usdm_index_price_klines() -> Result<()> {
//     info!("Testing Binance UsdM mark price klines download");
//     let instruments = vec![
//         "BTCUSDT".to_string(),
//         "ETHUSDT".to_string(),
//         "BNBUSDT".to_string(),
//         "XRPUSDT".to_string(),
//         "SOLUSDT".to_string(),
//     ];

//     let start = utc_datetime!(2025 - 10 - 01 00:00:00);
//     let end = utc_datetime!(2025 - 10 - 03 00:00:00);

//     let stream = stream::iter(construct_urls(
//         &instruments,
//         start,
//         end,
//         USDM_BASE_URL,
//         BinanceChannel::IndexPriceKlines,
//     ))
//     .map(|url| download::<BinanceFuturesIndexPriceKline>(url, true))
//     .buffer_unordered(MAX_CONCURRENT);
//     pin!(stream);
//     while let Some(result) = stream.next().await {
//         match result {
//             Ok(metrics) => {
//                 info!("Stream metrics count: {}", metrics.len());
//             }
//             Err(e) => info!("Stream error: {}", e),
//         }
//     }

//     Ok(())
// }

// #[tokio::test]
// #[test_log::test]
// async fn test_binance_coinm_agg_trades() -> Result<()> {
//     info!("Testing Binance CoinM AggTrade download");
//     let instruments = vec![
//         "BTCUSD_PERP".to_string(),
//         "ETHUSD_PERP".to_string(),
//         "BNBUSD_PERP".to_string(),
//         "XRPUSD_PERP".to_string(),
//         "SOLUSD_PERP".to_string(),
//     ];

//     let start = utc_datetime!(2025 - 10 - 01 00:00:00);
//     let end = utc_datetime!(2025 - 10 - 03 00:00:00);

//     let stream = stream::iter(construct_urls(&instruments, start, end, CM_BASE_URL, BinanceChannel::AggTrades))
//         .map(|url| download::<BinanceFuturesAggTrade>(url, true))
//         .buffer_unordered(MAX_CONCURRENT);
//     pin!(stream);
//     while let Some(result) = stream.next().await {
//         match result {
//             Ok(trades) => info!("Stream trade count: {}", trades.len()),
//             Err(e) => info!("Stream error: {}", e),
//         }
//     }

//     Ok(())
// }

// #[tokio::test]
// #[test_log::test]
// async fn test_binance_coinm_metrics() -> Result<()> {
//     info!("Testing Binance CoinM metrics download");
//     let instruments = vec![
//         "BTCUSD_PERP".to_string(),
//         "ETHUSD_PERP".to_string(),
//         "BNBUSD_PERP".to_string(),
//         "XRPUSD_PERP".to_string(),
//         "SOLUSD_PERP".to_string(),
//     ];

//     let start = utc_datetime!(2025 - 10 - 01 00:00:00);
//     let end = utc_datetime!(2025 - 10 - 03 00:00:00);

//     let stream = stream::iter(construct_urls(&instruments, start, end, CM_BASE_URL, BinanceChannel::Metrics))
//         .map(|url| download::<BinanceFuturesMetrics>(url, true))
//         .buffer_unordered(MAX_CONCURRENT);
//     pin!(stream);
//     while let Some(result) = stream.next().await {
//         match result {
//             Ok(metrics) => info!("Stream metrics count: {}", metrics.len()),
//             Err(e) => info!("Stream error: {}", e),
//         }
//     }

//     Ok(())
// }

// #[tokio::test]
// #[test_log::test]
// async fn test_binance_spot_agg_trades() -> Result<()> {
//     info!("Testing Binance Spot AggTrade download");
//     let instruments = vec![
//         "BTCUSDC".to_string(),
//         "ETHUSDC".to_string(),
//         "BNBUSDC".to_string(),
//         "XRPUSDC".to_string(),
//         "SOLUSDC".to_string(),
//     ];

//     let start = utc_datetime!(2025 - 10 - 01 00:00:00);
//     let end = utc_datetime!(2025 - 10 - 03 00:00:00);

//     let stream = stream::iter(construct_urls(
//         &instruments,
//         start,
//         end,
//         SPOT_BASE_URL,
//         BinanceChannel::AggTrades,
//     ))
//     .map(|url| download::<BinanceSpotAggTrade>(url, false))
//     .buffer_unordered(MAX_CONCURRENT);
//     pin!(stream);
//     while let Some(result) = stream.next().await {
//         match result {
//             Ok(trades) => info!("Stream trade count: {}", trades.len()),
//             Err(e) => info!("Stream error: {}", e),
//         }
//     }

//     Ok(())
// }
