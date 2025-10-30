use std::{fmt, sync::LazyLock};

use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum::{Display, EnumString};
use time::UtcDateTime;
use url::Url;
use uuid::Uuid;

use crate::subscriptions::{
    BinanceSubscription, BybitSubscription, CoinbaseSubscription, DeribitSubscription, HyperliquidSubscription,
    OkxSubscription, SubscriptionError, SubscriptionRequest,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, clap::ValueEnum, Serialize, Deserialize)] // For Clap auto-parsing
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VenueName {
    Personal,
    Index,
    Binance,
    Okx,
    Bybit,
    Coinbase,
    Deribit,
    Hyperliquid,
}

impl VenueName {
    /// Get the parent exchange name for grouping across venue variants.
    ///
    /// For example, BinanceSpot and BinanceUsdmFutures both return "binance".
    /// This is useful for creating exchange-level synthetic instruments that
    /// aggregate data across spot and derivatives markets.
    pub fn exchange_name(&self) -> &'static str {
        match self {
            VenueName::Personal => "personal",
            VenueName::Index => "index",
            VenueName::Binance => "binance",
            VenueName::Okx => "okx",
            VenueName::Bybit => "bybit",
            VenueName::Deribit => "deribit",
            VenueName::Coinbase => "coinbase",
            VenueName::Hyperliquid => "hyperliquid",
        }
    }
}

#[derive(Debug, Display, Copy, Clone, PartialEq, Eq, Hash, Type, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "venue_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MarketType {
    Spot,
    Perpetual,
    InversePerpetual,
    Futures,
    Options,
}

#[derive(Debug, Display, Copy, Clone, PartialEq, Eq, Hash, Type, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "venue_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VenueType {
    Cex,
    Dex,
    Otc,
    UserFunds,
    Virtual,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Venue {
    pub id: Uuid,
    pub name: VenueName,
    pub venue_type: VenueType,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl fmt::Display for Venue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name.to_string().to_lowercase())
    }
}

// Similarly for Channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, clap::ValueEnum, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum HttpChannel {
    Instrument,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, clap::ValueEnum, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum HttpMethod {
    Get,
    Post,
}

// Similarly for Channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, clap::ValueEnum, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum WsChannel {
    TopOfBook,
    OrderBook,
    Trades,
    AggTrades,
    OpenInterest,
    FundingRate,
    LongShortRatio,
    Metrics,
    MarkPriceKlines,
    IndexPriceKlines,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryLookup {
    pub ingestors: Vec<MappingEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingEntry {
    pub venue: VenueName,
    pub market_type: MarketType,
    pub ws_url: Url,
    pub ws_channels: Vec<WsChannelInfo>,
    pub http_url: Url,
    pub http_channels: Vec<HttpChannelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpChannelInfo {
    pub channel: HttpChannel,
    pub method: HttpMethod,
    pub endpoint: String,
    pub interval_s: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsChannelInfo {
    pub channel: WsChannel,
    pub prefix: String,
    pub suffix: String,
    pub has_instrument: bool,
}

// Static mapping (HashMap for fast lookup; add instruments_endpoint with params where static)
pub static MAPPINGS: LazyLock<Vec<MappingEntry>> = LazyLock::new(|| {
    vec![
        // Binance Spot
        MappingEntry {
            venue: VenueName::Binance,
            market_type: MarketType::Spot,
            ws_url: Url::parse("wss://stream.binance.com:9443/ws")
                .expect("Invalid WS URL for Binance Spot: wss://stream.binance.com:9443/ws"),
            http_url: Url::parse("https://api.binance.com")
                .expect("Invalid HTTP URL for Binance Spot: https://api.binance.com"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/api/v3/exchangeInfo".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::AggTrades,
                    prefix: "".to_string(),
                    suffix: "@aggTrade".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "".to_string(),
                    suffix: "@bookTicker".to_string(),
                    has_instrument: true,
                },
            ],
        },
        // Binance Perpetual (USDM)
        MappingEntry {
            venue: VenueName::Binance,
            market_type: MarketType::Perpetual,
            ws_url: Url::parse("wss://fstream.binance.com/ws")
                .expect("Invalid WS URL for Binance Perpetual: wss://fstream.binance.com/ws"),
            http_url: Url::parse("https://fapi.binance.com")
                .expect("Invalid HTTP URL for Binance Perpetual: https://fapi.binance.com"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/fapi/v1/exchangeInfo".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::AggTrades,
                    prefix: "".to_string(),
                    suffix: "@aggTrade".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "".to_string(),
                    suffix: "@bookTicker".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "".to_string(),
                    suffix: "@depth".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::FundingRate,
                    prefix: "".to_string(),
                    suffix: "@forceOrder".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "".to_string(),
                    suffix: "@markPrice_kline".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::IndexPriceKlines,
                    prefix: "".to_string(),
                    suffix: "@indexPrice_kline".to_string(),
                    has_instrument: true,
                },
            ],
        },
        // Binance Inverse Perpetual (CoinM)
        MappingEntry {
            venue: VenueName::Binance,
            market_type: MarketType::InversePerpetual,
            ws_url: Url::parse("wss://dstream.binance.com/ws")
                .expect("Invalid WS URL for Binance Inverse Perpetual: wss://dstream.binance.com/ws"),
            http_url: Url::parse("https://dapi.binance.com")
                .expect("Invalid HTTP URL for Binance Inverse Perpetual: https://dapi.binance.com"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/dapi/v1/exchangeInfo".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::AggTrades,
                    prefix: "".to_string(),
                    suffix: "@aggTrade".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "".to_string(),
                    suffix: "@bookTicker".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "".to_string(),
                    suffix: "@depth".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::FundingRate,
                    prefix: "".to_string(),
                    suffix: "@forceOrder".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "".to_string(),
                    suffix: "@markPrice_kline".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::IndexPriceKlines,
                    prefix: "".to_string(),
                    suffix: "@indexPrice_kline".to_string(),
                    has_instrument: true,
                },
            ],
        },
        // OKX Spot
        MappingEntry {
            venue: VenueName::Okx,
            market_type: MarketType::Spot,
            ws_url: Url::parse("wss://ws.okx.com:8443/ws/v5/public")
                .expect("Invalid WS URL for OKX Spot: wss://ws.okx.com:8443/ws/v5/public"),
            http_url: Url::parse("https://www.okx.com/api/v5")
                .expect("Invalid HTTP URL for OKX Spot: https://www.okx.com/api/v5"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/api/v5/public/instruments?instType=SPOT".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "trades".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "tickers".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "books".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
            ],
        },
        // OKX Perpetual
        MappingEntry {
            venue: VenueName::Okx,
            market_type: MarketType::Perpetual,
            ws_url: Url::parse("wss://ws.okx.com:8443/ws/v5/public").expect("Invalid URL"),
            http_url: Url::parse("https://www.okx.com").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/api/v5/public/instruments?instType=SWAP".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "trades".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "tickers".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "books".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::FundingRate,
                    prefix: "funding-rate".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::IndexPriceKlines,
                    prefix: "index-tickers".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "mark-price".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::OpenInterest,
                    prefix: "open-interest".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
            ],
        },
        // OKX Futures
        MappingEntry {
            venue: VenueName::Okx,
            market_type: MarketType::Futures,
            ws_url: Url::parse("wss://ws.okx.com:8443/ws/v5/public").expect("Invalid URL"),
            http_url: Url::parse("https://www.okx.com").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/api/v5/public/instruments?instType=FUTURES".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "trades".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "tickers".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "books".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::IndexPriceKlines,
                    prefix: "index-tickers".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "mark-price".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::OpenInterest,
                    prefix: "open-interest".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
            ],
        },
        // OKX Options
        MappingEntry {
            venue: VenueName::Okx,
            market_type: MarketType::Options,
            ws_url: Url::parse("wss://ws.okx.com:8443/ws/v5/public").expect("Invalid URL"),
            http_url: Url::parse("https://www.okx.com").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/api/v5/public/instruments?instType=OPTION".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "trades".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "tickers".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "books".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::IndexPriceKlines,
                    prefix: "index-tickers".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "mark-price".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::OpenInterest,
                    prefix: "open-interest".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
            ],
        },
        // Bybit Spot
        MappingEntry {
            venue: VenueName::Bybit,
            market_type: MarketType::Spot,
            ws_url: Url::parse("wss://stream.bybit.com/v5/public/spot").expect("Invalid URL"),
            http_url: Url::parse("https://api.bybit.com").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/v5/market/instruments-info?category=spot".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "publicTrade.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "tickers.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "orderbook.1.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "kline.1.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
            ],
        },
        // Bybit Perpetual (linear/USDT)
        MappingEntry {
            venue: VenueName::Bybit,
            market_type: MarketType::Perpetual,
            ws_url: Url::parse("wss://stream.bybit.com/v5/public/linear").expect("Invalid URL"),
            http_url: Url::parse("https://api.bybit.com").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/v5/market/instruments-info?category=linear".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "publicTrade.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "tickers.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "orderbook.1.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "kline.1.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OpenInterest,
                    prefix: "openInterest.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::FundingRate,
                    prefix: "funding.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
            ],
        },
        // Bybit Inverse Perpetual
        MappingEntry {
            venue: VenueName::Bybit,
            market_type: MarketType::InversePerpetual,
            ws_url: Url::parse("wss://stream.bybit.com/v5/public/inverse").expect("Invalid URL"),
            http_url: Url::parse("https://api.bybit.com").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/v5/market/instruments-info?category=inverse".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "publicTrade.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "tickers.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "orderbook.1.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "kline.1.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OpenInterest,
                    prefix: "openInterest.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::FundingRate,
                    prefix: "funding.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
            ],
        },
        // Bybit Options
        MappingEntry {
            venue: VenueName::Bybit,
            market_type: MarketType::Options,
            ws_url: Url::parse("wss://stream.bybit.com/v5/public/option").expect("Invalid URL"),
            http_url: Url::parse("https://api.bybit.com").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/v5/market/instruments-info?category=option".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "publicTrade.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "tickers.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "orderbook.1.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "kline.1.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
            ],
        },
        // Coinbase Spot
        MappingEntry {
            venue: VenueName::Coinbase,
            market_type: MarketType::Spot,
            ws_url: Url::parse("wss://advanced-trade-ws.coinbase.com").expect("Invalid URL"),
            http_url: Url::parse("https://api.coinbase.com").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/api/v3/brokerage/products".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "matches".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "ticker".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "level2".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
            ],
        },
        // Deribit Perpetual
        MappingEntry {
            venue: VenueName::Deribit,
            market_type: MarketType::Perpetual,
            ws_url: Url::parse("wss://www.deribit.com/ws/api/v2").expect("Invalid URL"),
            http_url: Url::parse("https://www.deribit.com").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/api/v2/public/get_instruments?kind=perpetual".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "trades.perpetual.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "ticker.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "book.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::IndexPriceKlines,
                    prefix: "deribit_price_index.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "mark_price.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::FundingRate,
                    prefix: "funding_rate.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OpenInterest,
                    prefix: "open_interest.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
            ],
        },
        // Deribit Inverse Perpetual (treat as future)
        MappingEntry {
            venue: VenueName::Deribit,
            market_type: MarketType::InversePerpetual,
            ws_url: Url::parse("wss://www.deribit.com/ws/api/v2").expect("Invalid URL"),
            http_url: Url::parse("https://www.deribit.com").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/api/v2/public/get_instruments?kind=future".to_string().to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "trades.future.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "ticker.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "book.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::IndexPriceKlines,
                    prefix: "deribit_price_index.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "mark_price.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::FundingRate,
                    prefix: "funding_rate.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OpenInterest,
                    prefix: "open_interest.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
            ],
        },
        // Deribit Futures
        MappingEntry {
            venue: VenueName::Deribit,
            market_type: MarketType::Futures,
            ws_url: Url::parse("wss://www.deribit.com/ws/api/v2").expect("Invalid URL"),
            http_url: Url::parse("https://www.deribit.com").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/api/v2/public/get_instruments?kind=future".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "trades.future.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "ticker.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "book.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::IndexPriceKlines,
                    prefix: "deribit_price_index.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "mark_price.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OpenInterest,
                    prefix: "open_interest.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
            ],
        },
        // Deribit Options
        MappingEntry {
            venue: VenueName::Deribit,
            market_type: MarketType::Options,
            ws_url: Url::parse("wss://www.deribit.com/ws/api/v2").expect("Invalid URL"),
            http_url: Url::parse("https://www.deribit.com").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/api/v2/public/get_instruments?kind=option".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "trades.option.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::TopOfBook,
                    prefix: "ticker.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "book.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::IndexPriceKlines,
                    prefix: "deribit_price_index.".to_string(),
                    suffix: "".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "mark_price.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
                WsChannelInfo {
                    channel: WsChannel::OpenInterest,
                    prefix: "open_interest.".to_string(),
                    suffix: ".raw".to_string(),
                    has_instrument: true,
                },
            ],
        },
        // Hyperliquid Perpetual
        MappingEntry {
            venue: VenueName::Hyperliquid,
            market_type: MarketType::Perpetual,
            ws_url: Url::parse("wss://api.hyperliquid.xyz/ws").expect("Invalid URL"),
            http_url: Url::parse("https://api.hyperliquid.xyz").expect("Invalid URL"),
            http_channels: vec![HttpChannelInfo {
                channel: HttpChannel::Instrument,
                method: HttpMethod::Get,
                endpoint: "/info".to_string(),
                interval_s: 3600,
            }],
            ws_channels: vec![
                WsChannelInfo {
                    channel: WsChannel::Trades,
                    prefix: "trades".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::OrderBook,
                    prefix: "l2Book".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::FundingRate,
                    prefix: "funding".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
                WsChannelInfo {
                    channel: WsChannel::MarkPriceKlines,
                    prefix: "premiumIndex".to_string(),
                    suffix: "".to_string(),
                    has_instrument: false,
                },
            ],
        },
    ]
});

impl MappingEntry {
    /// Build a subscription JSON message for this venue
    ///
    /// # Arguments
    /// * `channels` - Channels to subscribe to
    /// * `instruments` - Instruments/symbols to subscribe to
    ///
    /// # Returns
    /// JSON-serialized subscription message as a string
    pub fn build_subscription_json(
        &self,
        channels: &[WsChannel],
        instruments: &[String],
    ) -> Result<String, SubscriptionError> {
        match self.venue {
            VenueName::Binance => {
                let sub = BinanceSubscription::build(channels, instruments)?;
                serde_json::to_string(&sub).map_err(|e| SubscriptionError::SerializationFailed(e.to_string()))
            }
            VenueName::Okx => {
                let sub = OkxSubscription::build(channels, instruments)?;
                serde_json::to_string(&sub).map_err(|e| SubscriptionError::SerializationFailed(e.to_string()))
            }
            VenueName::Bybit => {
                let sub = BybitSubscription::build(channels, instruments)?;
                serde_json::to_string(&sub).map_err(|e| SubscriptionError::SerializationFailed(e.to_string()))
            }
            VenueName::Coinbase => {
                let sub = CoinbaseSubscription::build(channels, instruments)?;
                serde_json::to_string(&sub).map_err(|e| SubscriptionError::SerializationFailed(e.to_string()))
            }
            VenueName::Deribit => {
                let sub = DeribitSubscription::build(channels, instruments)?;
                serde_json::to_string(&sub).map_err(|e| SubscriptionError::SerializationFailed(e.to_string()))
            }
            VenueName::Hyperliquid => {
                let sub = HyperliquidSubscription::build(channels, instruments)?;
                serde_json::to_string(&sub).map_err(|e| SubscriptionError::SerializationFailed(e.to_string()))
            }
            _ => Err(SubscriptionError::InvalidChannelForVenue(format!(
                "Subscriptions not yet implemented for venue: {:?}",
                self.venue
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use arkin_core::prelude::load;
    use tracing::info;

    use super::*;

    #[test]
    fn test_create_yaml() {
        let map = MAPPINGS.clone();
        let yaml_str = serde_yaml::to_string(&map).unwrap();
        println!("{}", yaml_str);
    }

    #[test]
    #[test_log::test]
    fn test_load_config() {
        let config = load::<RegistryLookup>();
        info!("{:#?}", config);
    }
}
