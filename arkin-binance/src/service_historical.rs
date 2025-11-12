use std::io::{Cursor, Read};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use csv::ReaderBuilder;
use futures::{stream, StreamExt};
use reqwest::get;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};
use time::format_description;
use time::UtcDateTime;
use tracing::{error, info};
use typed_builder::TypedBuilder;
use zip::ZipArchive;

use arkin_core::prelude::*;

pub const MAX_CONCURRENT: usize = 10;

pub const USDM_BASE_URL: &str = "https://data.binance.vision/data/futures/um/daily";
pub const CM_BASE_URL: &str = "https://data.binance.vision/data/futures/cm/daily";
pub const SPOT_BASE_URL: &str = "https://data.binance.vision/data/spot/daily";

#[derive(Debug)]
pub enum BinanceHistoricalData {
    AggTradeFutures(Vec<BinanceFuturesAggTrade>),
    AggTradeSpot(Vec<BinanceSpotAggTrade>),
    Metrics(Vec<BinanceFuturesMetrics>),
    MarkPriceKline(Vec<BinanceFuturesMarkPriceKline>),
    IndexPriceKline(Vec<BinanceFuturesIndexPriceKline>),
    // Add more as needed
}

#[derive(Debug, Deserialize)]
pub struct BinanceFuturesAggTrade {
    pub agg_trade_id: u64,
    pub price: Decimal,
    pub quantity: Decimal,
    pub first_trade_id: u64,
    pub last_trade_id: u64,
    #[serde(with = "custom_serde::timestamp")]
    pub transact_time: UtcDateTime,
    pub is_buyer_maker: bool,
}

fn datetime_from_string<'de, D>(deserializer: D) -> Result<UtcDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let fmt =
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").map_err(serde::de::Error::custom)?;
    UtcDateTime::parse(s, &fmt).map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
pub struct BinanceFuturesMetrics {
    #[serde(deserialize_with = "datetime_from_string")]
    pub create_time: UtcDateTime,
    pub symbol: String,
    pub sum_open_interest: Decimal,
    pub sum_open_interest_value: Decimal,
    pub count_toptrader_long_short_ratio: Option<Decimal>,
    pub sum_toptrader_long_short_ratio: Option<Decimal>,
    pub count_long_short_ratio: Option<Decimal>,
    pub sum_taker_long_short_vol_ratio: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceFuturesMarkPriceKline {
    #[serde(with = "custom_serde::timestamp")]
    pub open_time: UtcDateTime,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    #[serde(with = "custom_serde::timestamp")]
    pub close_time: UtcDateTime,
    pub quote_volume: Decimal,
    pub count: u64,
    pub taker_buy_volume: Decimal,
    pub taker_buy_quote_volume: Decimal,
    pub ignore: String,
}

#[derive(Debug, Deserialize)]
pub struct BinanceFuturesIndexPriceKline {
    #[serde(with = "custom_serde::timestamp")]
    pub open_time: UtcDateTime,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    #[serde(with = "custom_serde::timestamp")]
    pub close_time: UtcDateTime,
    pub quote_volume: Decimal,
    pub count: u64,
    pub taker_buy_volume: Decimal,
    pub taker_buy_quote_volume: Decimal,
    pub ignore: String,
}

// "a": 12345,         // Aggregate trade ID
// "p": "0.001",       // Price
// "q": "100",         // Quantity
// "f": 100,           // First trade ID
// "l": 105,           // Last trade ID
// "T": 1672515782136, // Trade time
// "m": true,          // Is the buyer the market maker?
// "M": true           // Ignore

fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(serde::de::Error::custom(format!("Invalid bool: {}", s))),
    }
}

#[derive(Debug, Deserialize)]
pub struct BinanceSpotAggTrade {
    pub agg_trade_id: u64,
    pub price: Decimal,
    pub quantity: Decimal,
    pub first_trade_id: u64,
    pub last_trade_id: u64,
    #[serde(with = "custom_serde::timestamp")]
    pub transact_time: UtcDateTime,
    #[serde(deserialize_with = "bool_from_string")]
    pub is_buyer_maker: bool,
    #[serde(deserialize_with = "bool_from_string")]
    pub is_best_match: bool,
}

// pub fn construct_urls(
//     exchange: VenueName,
//     channel: Channel,
//     instruments: &[Arc<Instrument>],
//     start: UtcDateTime,
//     end: UtcDateTime,
// ) -> Vec<(Arc<Instrument>, String)> {
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

//     let base_url = match exchange {
//         VenueName::BinanceUsdmFutures => USDM_BASE_URL,
//         VenueName::BinanceCoinmFutures => CM_BASE_URL,
//         VenueName::BinanceSpot => SPOT_BASE_URL,
//         _ => {
//             error!("Unsupported exchange for historical data: {:?}", exchange);
//             return vec![];
//         }
//     };

//     let channel_str = channel.channel_name_by_venue(exchange);

//     // Create the urls
//     instruments
//         .iter()
//         .flat_map(|instrument| {
//             dates
//                 .iter()
//                 .map(|date| {
//                     // Create URL
//                     // Strip _PERP or _USD suffixes for futures symbols
//                     let venue_symbol =
//                         if exchange == VenueName::BinanceCoinmFutures && channel == Channel::IndexPriceKlines {
//                             instrument.venue_symbol.replace("_PERP", "")
//                         } else {
//                             instrument.venue_symbol.clone()
//                         };

//                     let url = if channel.is_kline() {
//                         format!(
//                             "{}/{}/{}/{}/{}-{}-{}.zip",
//                             base_url, channel_str, venue_symbol, "1m", venue_symbol, "1m", date
//                         )
//                     } else {
//                         format!(
//                             "{}/{}/{}/{}-{}-{}.zip",
//                             base_url, channel_str, venue_symbol, venue_symbol, channel_str, date
//                         )
//                     };
//                     (instrument.clone(), url)
//                 })
//                 .collect::<Vec<_>>()
//         })
//         .collect::<Vec<_>>()
// }

// pub async fn download(exchange: &VenueName, channel: &Channel, url: &str) -> Result<BinanceHistoricalData> {
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
//         .has_headers(channel.has_headers(*exchange)) // Binance aggTrades CSVs are headerless
//         .from_reader(csv_data.as_bytes());

//     // Collect all trades, propagating any deserialization errors
//     // rdr.deserialize().collect::<Result<_, _>>().map_err(Into::into)
//     match (exchange, channel) {
//         (VenueName::BinanceSpot, Channel::AggTrades) => {
//             let data: Vec<BinanceSpotAggTrade> = rdr.deserialize().collect::<Result<_, _>>()?;
//             Ok(BinanceHistoricalData::AggTradeSpot(data))
//         }
//         (_, Channel::AggTrades) => {
//             // Futures variants
//             let data: Vec<BinanceFuturesAggTrade> = rdr.deserialize().collect::<Result<_, _>>()?;
//             Ok(BinanceHistoricalData::AggTradeFutures(data))
//         }
//         (_, Channel::Metrics) => {
//             let data: Vec<BinanceFuturesMetrics> = rdr.deserialize().collect::<Result<_, _>>()?;
//             Ok(BinanceHistoricalData::Metrics(data))
//         }
//         (_, Channel::MarkPriceKlines) => {
//             let data: Vec<BinanceFuturesMarkPriceKline> = rdr.deserialize().collect::<Result<_, _>>()?;
//             Ok(BinanceHistoricalData::MarkPriceKline(data))
//         }
//         (_, Channel::IndexPriceKlines) => {
//             let data: Vec<BinanceFuturesIndexPriceKline> = rdr.deserialize().collect::<Result<_, _>>()?;
//             Ok(BinanceHistoricalData::IndexPriceKline(data))
//         }
//         _ => Err(anyhow::anyhow!("Unsupported exchange/channel: {:?}/{:?}", exchange, channel)),
//     }
// }

// #[derive(Debug, Clone, TypedBuilder)]
// pub struct BinanceHistoricalIngestor {
//     pub venue: VenueName,
//     pub channel: Channel,
//     pub start: UtcDateTime,
//     pub end: UtcDateTime,
// }

// pub async fn download_task(
//     ingestor: Arc<BinanceHistoricalIngestor>,
//     core_ctx: Arc<CoreCtx>,
//     service_ctx: Arc<ServiceCtx>,
// ) {
//     info!(
//         "Starting Binance Historical Ingestor for venue: {:?}, channel: {}, from {} to {}",
//         ingestor.venue, ingestor.channel, ingestor.start, ingestor.end
//     );

//     let venue = match core_ctx.persistence.get_venue_by_name(&ingestor.venue).await {
//         Ok(v) => v,
//         Err(e) => {
//             error!("Error fetching venue {:?}: {}", ingestor.venue, e);
//             return;
//         }
//     };
//     let instruments = match core_ctx.persistence.get_instruments_by_venue(&venue).await {
//         Ok(insts) => insts,
//         Err(e) => {
//             info!("Error fetching instruments for venue {:?}: {}", venue, e);
//             return;
//         }
//     };
//     // let instruments: Vec<String> = instruments.iter().map(|inst| inst.venue_symbol.clone()).collect();
//     let venue_name = ingestor.venue;
//     let channel = ingestor.channel;

//     let mut stream = stream::iter(construct_urls(
//         ingestor.venue,
//         ingestor.channel,
//         &instruments,
//         ingestor.start,
//         ingestor.end,
//     ))
//     .map(|(instrument, url)| async move {
//         let data = download(&venue_name, &channel, &url).await;
//         match data {
//             Ok(data) => Ok((instrument, data)),
//             Err(e) => Err(e),
//         }
//     })
//     .buffer_unordered(MAX_CONCURRENT);

//     let shutdown_token = service_ctx.get_shutdown_token();
//     loop {
//         tokio::select! {
//             _ = shutdown_token.cancelled() => {
//                 info!("Shutdown signal received, stopping download task.");
//                 break;
//             }
//             Some(result) = stream.next() => {
//                 match result {
//                     Ok((inst, data)) => {
//                     match data {
//                         BinanceHistoricalData::AggTradeFutures(trades) => {
//                             info!("Downloaded {} futures agg trades", trades.len());

//                             let trades = trades.into_iter().map(|trade| {
//                                 Arc::new(AggTrade::builder()
//                                     .event_time(trade.transact_time)
//                                     .instrument(inst.clone())
//                                     .trade_id(trade.agg_trade_id)
//                                     .side(if trade.is_buyer_maker { MarketSide::Sell } else { MarketSide::Buy })
//                                     .price(trade.price)
//                                     .quantity(trade.quantity).build())
//                             }).collect::<Vec<_>>();

//                             for trade in trades {
//                               core_ctx.publish(Event::AggTradeUpdate(trade)).await;
//                             }
//                         }
//                         BinanceHistoricalData::AggTradeSpot(trades) => {
//                             info!("Downloaded {} spot agg trades", trades.len());

//                             let trades = trades.into_iter().map(|trade| {
//                                 Arc::new(AggTrade::builder()
//                                     .event_time(trade.transact_time)
//                                     .instrument(inst.clone())
//                                     .trade_id(trade.agg_trade_id)
//                                     .side(if trade.is_buyer_maker { MarketSide::Sell } else { MarketSide::Buy })
//                                     .price(trade.price)
//                                     .quantity(trade.quantity).build())
//                             }).collect::<Vec<_>>();

//                             for trade in trades {
//                               core_ctx.publish(Event::AggTradeUpdate(trade)).await;
//                             }
//                         }
//                         BinanceHistoricalData::Metrics(metrics) => {
//                             info!("Downloaded {} futures metrics", metrics.len());
//                             // Here you would typically process and store the metrics

//                             let metrics = metrics.into_iter().flat_map(|metric| {
//                                 let mut parsed_metrics = vec![];
//                                 let sum_open_interest = Arc::new(Metric::builder()
//                                     .event_time(metric.create_time)
//                                     .instrument(inst.clone())
//                                     .metric_type(MetricType::OpenInterest)
//                                     .value(metric.sum_open_interest)
//                                     .build());
//                                 parsed_metrics.push(sum_open_interest);
//                                 let sum_open_interest_value = Arc::new(Metric::builder()
//                                     .event_time(metric.create_time)
//                                     .instrument(inst.clone())
//                                     .metric_type(MetricType::OpenInterestNotional)
//                                     .value(metric.sum_open_interest_value)
//                                     .build());
//                                 parsed_metrics.push(sum_open_interest_value);
//                                 if let Some(ratio) = metric.count_toptrader_long_short_ratio {
//                                     let count_toptrader_long_short_ratio = Arc::new(Metric::builder()
//                                         .event_time(metric.create_time)
//                                         .instrument(inst.clone())
//                                         .metric_type(MetricType::CountTopTraderLongShortRatio)
//                                         .value(ratio)
//                                         .build());
//                                     parsed_metrics.push(count_toptrader_long_short_ratio);
//                                 }
//                                 if let Some(ratio) = metric.sum_toptrader_long_short_ratio {
//                                     let sum_toptrader_long_short_ratio = Arc::new(Metric::builder()
//                                         .event_time(metric.create_time)
//                                         .instrument(inst.clone())
//                                         .metric_type(MetricType::VolumeTopTraderLongShortRatio)
//                                         .value(ratio)
//                                         .build());
//                                     parsed_metrics.push(sum_toptrader_long_short_ratio);
//                                 }
//                                 if let Some(ratio) = metric.count_long_short_ratio {
//                                     let count_long_short_ratio = Arc::new(Metric::builder()
//                                         .event_time(metric.create_time)
//                                         .instrument(inst.clone())
//                                         .metric_type(MetricType::CountLongShortRatio)
//                                         .value(ratio)
//                                         .build());
//                                     parsed_metrics.push(count_long_short_ratio);
//                                 }
//                                 if let Some(ratio) = metric.sum_taker_long_short_vol_ratio {
//                                     let sum_taker_long_short_vol_ratio = Arc::new(Metric::builder()
//                                         .event_time(metric.create_time)
//                                         .instrument(inst.clone())
//                                         .metric_type(MetricType::VolumeTakerLongShortRatio)
//                                         .value(ratio)
//                                         .build());
//                                     parsed_metrics.push(sum_taker_long_short_vol_ratio);
//                                 }
//                                 parsed_metrics
//                             }).collect::<Vec<_>>();

//                             for metric in metrics {
//                               core_ctx.publish(Event::MetricUpdate(metric)).await;
//                             }
//                         }
//                         BinanceHistoricalData::MarkPriceKline(klines) => {
//                             info!("Downloaded {} futures mark price klines", klines.len());

//                             let metrics = klines.into_iter().map(|kline| {
//                                 Arc::new(Metric::builder()
//                                     .event_time(kline.open_time)
//                                     .instrument(inst.clone())
//                                     .metric_type(MetricType::MarkPrice)
//                                     .value(kline.open)
//                                     .build())
//                             }).collect::<Vec<_>>();

//                             for metric in metrics {
//                                 core_ctx.publish(Event::MetricUpdate(metric)).await;
//                             }
//                         }
//                         BinanceHistoricalData::IndexPriceKline(klines) => {
//                             info!("Downloaded {} futures index price klines", klines.len());

//                             let metrics = klines.into_iter().map(|kline| {
//                                 Arc::new(Metric::builder()
//                                     .event_time(kline.open_time)
//                                     .instrument(inst.clone())
//                                     .metric_type(MetricType::IndexPrice)
//                                     .value(kline.open)
//                                     .build())
//                             }).collect::<Vec<_>>();

//                             for metric in metrics {
//                                 core_ctx.publish(Event::MetricUpdate(metric)).await;
//                             }
//                         }
//                       }
//                     }
//                     Err(e) => error!("Stream error: {}", e),
//                 }
//             }
//         }
//     }
// }

// #[async_trait]
// impl Runnable for BinanceHistoricalIngestor {
//     async fn get_tasks(
//         self: Arc<Self>,
//         service_ctx: Arc<ServiceCtx>,
//         core_ctx: Arc<CoreCtx>,
//     ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
//         vec![Box::pin(download_task(self, core_ctx.clone(), service_ctx.clone()))]
//     }
// }

// // // Tests
// // #[cfg(test)]
// // mod tests {
// //     use futures::{stream, StreamExt};
// //     use time::macros::utc_datetime;

// //     use super::*;

// //     #[tokio::test]
// //     #[test_log::test]
// //     async fn test_binance_usdm_agg_trades() -> Result<()> {
// //         info!("Testing Binance Usdm AggTrade download");
// //         let venue = VenueName::BinanceUsdmFutures;
// //         let channel = Channel::AggTrades;
// //         let instruments = vec![
// //             "BTCUSDT".to_string(),
// //             "ETHUSDT".to_string(),
// //             "BNBUSDT".to_string(),
// //             "XRPUSDT".to_string(),
// //             "SOLUSDT".to_string(),
// //         ];

// //         let start = utc_datetime!(2025 - 10 - 01 00:00:00);
// //         let end = utc_datetime!(2025 - 10 - 01 00:00:00);

// //         let mut stream = stream::iter(construct_urls(venue, channel, &instruments, start, end))
// //             .map(|url| download(&venue, &&channel, url))
// //             .buffer_unordered(MAX_CONCURRENT);

// //         while let Some(result) = stream.next().await {
// //             match result {
// //                 Ok(data) => match data {
// //                     BinanceHistoricalData::AggTradeFutures(trades) => {
// //                         info!("Stream trade count: {}", trades.len());
// //                     }
// //                     _ => {
// //                         panic!("Unexpected data type received");
// //                     }
// //                 },
// //                 Err(e) => info!("Stream error: {}", e),
// //             }
// //         }

// //         Ok(())
// //     }

// //     #[tokio::test]
// //     #[test_log::test]
// //     async fn test_binance_usdm_metrics() -> Result<()> {
// //         info!("Testing Binance UsdM metrics download");
// //         let venue = VenueName::BinanceUsdmFutures;
// //         let channel = Channel::Metrics;
// //         let instruments = vec![
// //             "BTCUSDT".to_string(),
// //             "ETHUSDT".to_string(),
// //             "BNBUSDT".to_string(),
// //             "XRPUSDT".to_string(),
// //             "SOLUSDT".to_string(),
// //         ];

// //         let start = utc_datetime!(2025 - 10 - 01 00:00:00);
// //         let end = utc_datetime!(2025 - 10 - 01 00:00:00);

// //         let mut stream = stream::iter(construct_urls(venue, channel, &instruments, start, end))
// //             .map(|url| download(venue, channel, url))
// //             .buffer_unordered(MAX_CONCURRENT);

// //         while let Some(result) = stream.next().await {
// //             match result {
// //                 Ok(data) => match data {
// //                     BinanceHistoricalData::Metrics(metrics) => {
// //                         info!("Stream metrics count: {}", metrics.len());
// //                     }
// //                     _ => {
// //                         panic!("Unexpected data type received");
// //                     }
// //                 },
// //                 Err(e) => info!("Stream error: {}", e),
// //             }
// //         }

// //         Ok(())
// //     }

// //     #[tokio::test]
// //     #[test_log::test]
// //     async fn test_binance_usdm_mark_price_klines() -> Result<()> {
// //         info!("Testing Binance UsdM mark price klines download");
// //         let venue = VenueName::BinanceUsdmFutures;
// //         let channel = Channel::MarkPriceKlines;
// //         let instruments = vec![
// //             "BTCUSDT".to_string(),
// //             "ETHUSDT".to_string(),
// //             "BNBUSDT".to_string(),
// //             "XRPUSDT".to_string(),
// //             "SOLUSDT".to_string(),
// //         ];

// //         let start = utc_datetime!(2025 - 10 - 01 00:00:00);
// //         let end = utc_datetime!(2025 - 10 - 01 00:00:00);

// //         let mut stream = stream::iter(construct_urls(venue, channel, &instruments, start, end))
// //             .map(|url| download(venue, channel, url))
// //             .buffer_unordered(MAX_CONCURRENT);

// //         while let Some(result) = stream.next().await {
// //             match result {
// //                 Ok(data) => match data {
// //                     BinanceHistoricalData::MarkPriceKline(klines) => {
// //                         info!("Stream mark price klines count: {}", klines.len());
// //                     }
// //                     _ => {
// //                         panic!("Unexpected data type received");
// //                     }
// //                 },
// //                 Err(e) => info!("Stream error: {}", e),
// //             }
// //         }

// //         Ok(())
// //     }

// //     #[tokio::test]
// //     #[test_log::test]
// //     async fn test_binance_usdm_index_price_klines() -> Result<()> {
// //         info!("Testing Binance UsdM index price klines download");
// //         let venue = VenueName::BinanceUsdmFutures;
// //         let channel = Channel::IndexPriceKlines;
// //         let instruments = vec![
// //             "BTCUSDT".to_string(),
// //             "ETHUSDT".to_string(),
// //             "BNBUSDT".to_string(),
// //             "XRPUSDT".to_string(),
// //             "SOLUSDT".to_string(),
// //         ];

// //         let start = utc_datetime!(2025 - 10 - 01 00:00:00);
// //         let end = utc_datetime!(2025 - 10 - 01 00:00:00);

// //         let mut stream = stream::iter(construct_urls(venue, channel, &instruments, start, end))
// //             .map(|url| download(venue, channel, url))
// //             .buffer_unordered(MAX_CONCURRENT);

// //         while let Some(result) = stream.next().await {
// //             match result {
// //                 Ok(data) => match data {
// //                     BinanceHistoricalData::IndexPriceKline(klines) => {
// //                         info!("Stream index price klines count: {}", klines.len());
// //                     }
// //                     _ => {
// //                         panic!("Unexpected data type received");
// //                     }
// //                 },
// //                 Err(e) => info!("Stream error: {}", e),
// //             }
// //         }

// //         Ok(())
// //     }

// //     #[tokio::test]
// //     #[test_log::test]
// //     async fn test_binance_coinm_agg_trades() -> Result<()> {
// //         info!("Testing Binance CoinM AggTrade download");
// //         let venue = VenueName::BinanceCoinmFutures;
// //         let channel = Channel::AggTrades;
// //         let instruments = vec![
// //             "BTCUSD_PERP".to_string(),
// //             "ETHUSD_PERP".to_string(),
// //             "BNBUSD_PERP".to_string(),
// //             "XRPUSD_PERP".to_string(),
// //             "SOLUSD_PERP".to_string(),
// //         ];

// //         let start = utc_datetime!(2025 - 10 - 01 00:00:00);
// //         let end = utc_datetime!(2025 - 10 - 01 00:00:00);

// //         let mut stream = stream::iter(construct_urls(venue, channel, &instruments, start, end))
// //             .map(|url| download(venue, channel, url))
// //             .buffer_unordered(MAX_CONCURRENT);

// //         while let Some(result) = stream.next().await {
// //             match result {
// //                 Ok(data) => match data {
// //                     BinanceHistoricalData::AggTradeFutures(trades) => {
// //                         info!("Stream trade count: {}", trades.len());
// //                     }
// //                     _ => {
// //                         panic!("Unexpected data type received");
// //                     }
// //                 },
// //                 Err(e) => info!("Stream error: {}", e),
// //             }
// //         }

// //         Ok(())
// //     }

// //     #[tokio::test]
// //     #[test_log::test]
// //     async fn test_binance_coinm_metrics() -> Result<()> {
// //         info!("Testing Binance CoinM metrics download");
// //         let venue = VenueName::BinanceCoinmFutures;
// //         let channel = Channel::Metrics;
// //         let instruments = vec![
// //             "BTCUSD_PERP".to_string(),
// //             "ETHUSD_PERP".to_string(),
// //             "BNBUSD_PERP".to_string(),
// //             "XRPUSD_PERP".to_string(),
// //             "SOLUSD_PERP".to_string(),
// //         ];

// //         let start = utc_datetime!(2025 - 10 - 01 00:00:00);
// //         let end = utc_datetime!(2025 - 10 - 01 00:00:00);

// //         let mut stream = stream::iter(construct_urls(venue, channel, &instruments, start, end))
// //             .map(|url| download(venue, channel, url))
// //             .buffer_unordered(MAX_CONCURRENT);

// //         while let Some(result) = stream.next().await {
// //             match result {
// //                 Ok(data) => match data {
// //                     BinanceHistoricalData::Metrics(metrics) => {
// //                         info!("Stream metrics count: {}", metrics.len());
// //                     }
// //                     _ => {
// //                         panic!("Unexpected data type received");
// //                     }
// //                 },
// //                 Err(e) => info!("Stream error: {}", e),
// //             }
// //         }

// //         Ok(())
// //     }

// //     #[tokio::test]
// //     #[test_log::test]
// //     async fn test_binance_spot_agg_trades() -> Result<()> {
// //         info!("Testing Binance Spot AggTrade download");
// //         let venue = VenueName::BinanceSpot;
// //         let channel = Channel::AggTrades;
// //         let instruments = vec![
// //             "BTCUSDC".to_string(),
// //             "ETHUSDC".to_string(),
// //             "BNBUSDC".to_string(),
// //             "XRPUSDC".to_string(),
// //             "SOLUSDC".to_string(),
// //         ];

// //         let start = utc_datetime!(2025 - 10 - 01 00:00:00);
// //         let end = utc_datetime!(2025 - 10 - 01 00:00:00);

// //         let mut stream = stream::iter(construct_urls(venue, channel, &instruments, start, end))
// //             .map(|url| download(venue, channel, url))
// //             .buffer_unordered(MAX_CONCURRENT);

// //         while let Some(result) = stream.next().await {
// //             match result {
// //                 Ok(data) => match data {
// //                     BinanceHistoricalData::AggTradeSpot(trades) => {
// //                         info!("Stream trade count: {}", trades.len());
// //                     }
// //                     _ => {
// //                         panic!("Unexpected data type received");
// //                     }
// //                 },
// //                 Err(e) => info!("Stream error: {}", e),
// //             }
// //         }

// //         Ok(())
// //     }
// // }
