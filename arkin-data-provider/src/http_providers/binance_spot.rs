#![allow(unused)]
use std::{
    collections::BTreeMap,
    fmt,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use anyhow::Result;
use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT_ENCODING},
    Method, Request,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, info};
use typed_builder::TypedBuilder;
use url::Url;

use arkin_core::prelude::*;

use crate::{
    errors::ProviderError,
    http::{HttpRequest, HttpRequestContext},
    traits::HttpProvider,
};

#[derive(TypedBuilder)]
pub struct BinanceSpotHttpProvider {
    base_url: Url,
    #[builder(default)]
    api_key: Option<String>,
    #[builder(default)]
    api_secret: Option<String>,
    #[builder(default = "unknown".to_string())]
    user_agent: String,
    #[builder(default = Some(TimeUnit::Millisecond))]
    time_unit: Option<TimeUnit>,
    #[builder(default = 3)]
    retries: u32,
    #[builder(default = 1000)]
    backoff_secs: u64,
}

#[async_trait]
impl HttpProvider for BinanceSpotHttpProvider {
    fn get_endpoints(&self) -> Vec<HttpRequest> {
        vec![
            HttpRequest::new_oneshot(HttpRequestContext {
                channel: Channel::Instruments,
                method: Method::GET,
                params: BTreeMap::new(),
                is_signed: false,
                last_fetched: Arc::new(AtomicU64::new(0)),
                endpoint: "/api/v3/exchangeInfo".to_string(),
                custom_headers: None,
            }),
            HttpRequest::new_polling(
                HttpRequestContext {
                    channel: Channel::Ping,
                    method: Method::GET,
                    params: BTreeMap::new(),
                    is_signed: false,
                    last_fetched: Arc::new(AtomicU64::new(0)),
                    endpoint: "/api/v3/ping".to_string(),
                    custom_headers: None,
                },
                Duration::from_secs(1),
            ),
        ]
    }

    fn build_request(&self, endpoint: &HttpRequestContext) -> Result<Request, ProviderError> {
        let full_url = Url::parse(self.base_url.as_str())
            .and_then(|u| u.join(&endpoint.endpoint))
            .map_err(|e| ProviderError::RequestBuildError(format!("Failed to join base URL and endpoint: {e}")))?
            .to_string();

        // let signature = endpoint
        //     .is_signed
        //     .then(|| {
        //         let timestamp = get_timestamp();
        //         params.insert("timestamp".to_string(), json!(timestamp));
        //         configuration.signature_gen.get_signature(&params)
        //     })
        //     .transpose()?;
        // let signature = None;

        let mut url = Url::parse(&full_url)
            .map_err(|e| ProviderError::RequestBuildError(format!("Failed to parse the url {e}")))?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in &endpoint.params {
                let val_str = match value {
                    Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                pairs.append_pair(key, &val_str);
            }
            // if let Some(signature) = signature {
            //     pairs.append_pair("signature", &signature);
            // }
        }

        let mut headers = HeaderMap::new();

        let forbidden = ["host", "authorization", "cookie", ":method", ":path"]
            .into_iter()
            .map(str::to_ascii_lowercase)
            .collect::<std::collections::HashSet<_>>();

        if let Some(custom) = &endpoint.custom_headers {
            for (raw_name, raw_val) in custom {
                let name = raw_name.trim();
                if forbidden.contains(&name.to_ascii_lowercase()) {
                    continue;
                }
                if let (Ok(header_name), Ok(header_val)) =
                    (name.parse::<reqwest::header::HeaderName>(), HeaderValue::from_str(raw_val))
                {
                    headers.append(header_name, header_val);
                }
            }
        }

        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("User-Agent", self.user_agent.parse().unwrap());
        if let Some(api_key) = &self.api_key {
            headers.insert(
                "X-MBX-APIKEY",
                HeaderValue::from_str(api_key).map_err(|e| {
                    ProviderError::RequestBuildError(format!("Failed to parse API key header value {e}"))
                })?,
            );
        }

        // if configuration.compression {
        headers.insert(
            ACCEPT_ENCODING,
            "gzip, deflate, br"
                .parse()
                .map_err(|e| ProviderError::RequestBuildError(format!("Failed to parse encoding header value: {e}")))?,
        );
        // }

        let time_unit_to_apply = self.time_unit;
        if let Some(time_unit) = time_unit_to_apply {
            headers.insert(
                "X-MBX-TIME-UNIT",
                time_unit
                    .as_upper_str()
                    .parse()
                    .map_err(|e| ProviderError::RequestBuildError(format!("Failed to parse header value: {e}")))?,
            );
        }

        let method = endpoint.method.clone();
        let mut req = Request::new(method, url);
        *req.headers_mut() = headers;
        debug!("Built request: {:?}", req);
        Ok(req)
    }

    async fn parse(&self, headers: &HeaderMap, body: &str, channel: &Channel) -> Option<Event> {
        match channel {
            Channel::Instruments => match serde_json::from_str::<BinanceSpotExchangeInfo>(body) {
                Ok(info) => {
                    info!("Parsed exchange info with {} symbols", info.symbols.len());
                    None
                }
                Err(e) => {
                    info!("Failed to parse exchange info: {}", e);
                    None
                }
            },
            Channel::Ping => {
                debug!("Received ping response");
                None
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Millisecond,
    Microsecond,
}

impl fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeUnit::Millisecond => write!(f, "millisecond"),
            TimeUnit::Microsecond => write!(f, "microsecond"),
        }
    }
}

impl TimeUnit {
    #[must_use]
    pub fn as_upper_str(&self) -> &'static str {
        match self {
            TimeUnit::Millisecond => "MILLISECOND",
            TimeUnit::Microsecond => "MICROSECOND",
        }
    }
    #[must_use]
    pub fn as_lower_str(&self) -> &'static str {
        match self {
            TimeUnit::Millisecond => "millisecond",
            TimeUnit::Microsecond => "microsecond",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceSpotExchangeInfo {
    pub timezone: String,
    pub server_time: i64,
    pub rate_limits: Vec<RateLimit>,
    pub exchange_filters: Vec<Value>,
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: String,
    pub interval: String,
    pub interval_num: i64,
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub symbol: String,
    pub status: String,
    pub base_asset: String,
    pub base_asset_precision: u8,
    pub quote_asset: String,
    pub quote_precision: u8,
    pub quote_asset_precision: u8,
    pub base_commission_precision: u8,
    pub quote_commission_precision: u8,
    pub order_types: Vec<String>,
    pub iceberg_allowed: bool,
    pub oco_allowed: bool,
    pub oto_allowed: bool,
    pub quote_order_qty_market_allowed: bool,
    pub allow_trailing_stop: bool,
    pub cancel_replace_allowed: bool,
    pub amend_allowed: bool,
    pub peg_instructions_allowed: bool,
    pub is_spot_trading_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub filters: Vec<Filter>,
    pub permissions: Vec<Value>,
    pub permission_sets: Vec<Vec<String>>,
    pub default_self_trade_prevention_mode: String,
    pub allowed_self_trade_prevention_modes: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "filterType")]
pub enum Filter {
    #[serde(rename = "PRICE_FILTER")]
    PriceFilter {
        #[serde(rename = "minPrice")]
        min_price: Decimal,
        #[serde(rename = "maxPrice")]
        max_price: Decimal,
        #[serde(rename = "tickSize")]
        tick_size: Decimal,
    },
    #[serde(rename = "LOT_SIZE")]
    LotSize {
        #[serde(rename = "minQty")]
        min_qty: Decimal,
        #[serde(rename = "maxQty")]
        max_qty: Decimal,
        #[serde(rename = "stepSize")]
        step_size: Decimal,
    },
    #[serde(rename = "ICEBERG_PARTS")]
    IcebergParts { limit: u32 },
    #[serde(rename = "MARKET_LOT_SIZE")]
    MarketLotSize {
        #[serde(rename = "minQty")]
        min_qty: Decimal,
        #[serde(rename = "maxQty")]
        max_qty: Decimal,
        #[serde(rename = "stepSize")]
        step_size: Decimal,
    },
    #[serde(rename = "TRAILING_DELTA")]
    TrailingDelta {
        #[serde(rename = "minTrailingAboveDelta")]
        min_trailing_above_delta: u32,
        #[serde(rename = "maxTrailingAboveDelta")]
        max_trailing_above_delta: u32,
        #[serde(rename = "minTrailingBelowDelta")]
        min_trailing_below_delta: u32,
        #[serde(rename = "maxTrailingBelowDelta")]
        max_trailing_below_delta: u32,
    },
    #[serde(rename = "PERCENT_PRICE_BY_SIDE")]
    PercentPriceBySide {
        #[serde(rename = "bidMultiplierUp")]
        bid_multiplier_up: Decimal,
        #[serde(rename = "bidMultiplierDown")]
        bid_multiplier_down: Decimal,
        #[serde(rename = "askMultiplierUp")]
        ask_multiplier_up: Decimal,
        #[serde(rename = "askMultiplierDown")]
        ask_multiplier_down: Decimal,
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: u32,
    },
    #[serde(rename = "NOTIONAL")]
    Notional {
        #[serde(rename = "minNotional")]
        min_notional: Decimal,
        #[serde(rename = "applyMinToMarket")]
        apply_min_to_market: bool,
        #[serde(rename = "maxNotional")]
        max_notional: Decimal,
        #[serde(rename = "applyMaxToMarket")]
        apply_max_to_market: bool,
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: u32,
    },
    #[serde(rename = "MAX_NUM_ORDERS")]
    MaxNumOrders {
        #[serde(rename = "maxNumOrders")]
        max_num_orders: u32,
    },
    #[serde(rename = "MAX_NUM_ORDER_LISTS")]
    MaxNumOrderLists {
        #[serde(rename = "maxNumOrderLists")]
        max_num_order_lists: u32,
    },
    #[serde(rename = "MAX_NUM_ALGO_ORDERS")]
    MaxNumAlgoOrders {
        #[serde(rename = "maxNumAlgoOrders")]
        max_num_algo_orders: u32,
    },
    #[serde(rename = "MAX_NUM_ORDER_AMENDS")]
    MaxNumOrderAmends {
        #[serde(rename = "maxNumOrderAmends")]
        max_num_order_amends: u32,
    },
    #[serde(rename = "MAX_POSITION")]
    MaxPosition {
        #[serde(rename = "maxPosition")]
        max_position: Decimal,
    },
    #[serde(other)]
    Unknown,
}
