#![allow(unused_imports)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use typed_builder::TypedBuilder;

use crate::common::{
    models::ParamBuildError,
    utils::replace_websocket_streams_placeholders,
    websocket::{create_stream_handler, WebsocketBase, WebsocketStream, WebsocketStreams},
};
use crate::derivatives_trading_usds_futures::websocket_streams::models;

#[async_trait]
pub trait WebsocketMarketStreamsApi: Send + Sync {
    async fn aggregate_trade_streams(
        &self,
        params: AggregateTradeStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::AggregateTradeStreamsResponse>>>;
    async fn all_book_tickers_stream(
        &self,
        params: AllBookTickersStreamParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::AllBookTickersStreamResponse>>>;
    async fn all_market_liquidation_order_streams(
        &self,
        params: AllMarketLiquidationOrderStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::AllMarketLiquidationOrderStreamsResponse>>>;
    async fn all_market_mini_tickers_stream(
        &self,
        params: AllMarketMiniTickersStreamParams,
    ) -> anyhow::Result<Arc<WebsocketStream<Vec<models::AllMarketMiniTickersStreamResponseInner>>>>;
    async fn all_market_tickers_streams(
        &self,
        params: AllMarketTickersStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<Vec<models::AllMarketTickersStreamsResponseInner>>>>;
    async fn composite_index_symbol_information_streams(
        &self,
        params: CompositeIndexSymbolInformationStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::CompositeIndexSymbolInformationStreamsResponse>>>;
    async fn continuous_contract_kline_candlestick_streams(
        &self,
        params: ContinuousContractKlineCandlestickStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::ContinuousContractKlineCandlestickStreamsResponse>>>;
    async fn contract_info_stream(
        &self,
        params: ContractInfoStreamParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::ContractInfoStreamResponse>>>;
    async fn diff_book_depth_streams(
        &self,
        params: DiffBookDepthStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::DiffBookDepthStreamsResponse>>>;
    async fn individual_symbol_book_ticker_streams(
        &self,
        params: IndividualSymbolBookTickerStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::IndividualSymbolBookTickerStreamsResponse>>>;
    async fn individual_symbol_mini_ticker_stream(
        &self,
        params: IndividualSymbolMiniTickerStreamParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::IndividualSymbolMiniTickerStreamResponse>>>;
    async fn individual_symbol_ticker_streams(
        &self,
        params: IndividualSymbolTickerStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::IndividualSymbolTickerStreamsResponse>>>;
    async fn kline_candlestick_streams(
        &self,
        params: KlineCandlestickStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::KlineCandlestickStreamsResponse>>>;
    async fn liquidation_order_streams(
        &self,
        params: LiquidationOrderStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::LiquidationOrderStreamsResponse>>>;
    async fn mark_price_stream(
        &self,
        params: MarkPriceStreamParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::MarkPriceStreamResponse>>>;
    async fn mark_price_stream_for_all_market(
        &self,
        params: MarkPriceStreamForAllMarketParams,
    ) -> anyhow::Result<Arc<WebsocketStream<Vec<models::MarkPriceStreamForAllMarketResponseInner>>>>;
    async fn multi_assets_mode_asset_index(
        &self,
        params: MultiAssetsModeAssetIndexParams,
    ) -> anyhow::Result<Arc<WebsocketStream<Vec<models::MultiAssetsModeAssetIndexResponseInner>>>>;
    async fn partial_book_depth_streams(
        &self,
        params: PartialBookDepthStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::PartialBookDepthStreamsResponse>>>;
}

pub struct WebsocketMarketStreamsApiClient {
    websocket_streams_base: Arc<WebsocketStreams>,
}

impl WebsocketMarketStreamsApiClient {
    pub fn new(websocket_streams_base: Arc<WebsocketStreams>) -> Self {
        Self {
            websocket_streams_base,
        }
    }
}

/// Request parameters for the [`aggregate_trade_streams`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`aggregate_trade_streams`](#method.aggregate_trade_streams).
#[derive(Clone, Debug, TypedBuilder)]
pub struct AggregateTradeStreamsParams {
    /// The symbol parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`all_book_tickers_stream`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`all_book_tickers_stream`](#method.all_book_tickers_stream).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct AllBookTickersStreamParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`all_market_liquidation_order_streams`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`all_market_liquidation_order_streams`](#method.all_market_liquidation_order_streams).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct AllMarketLiquidationOrderStreamsParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`all_market_mini_tickers_stream`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`all_market_mini_tickers_stream`](#method.all_market_mini_tickers_stream).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct AllMarketMiniTickersStreamParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`all_market_tickers_streams`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`all_market_tickers_streams`](#method.all_market_tickers_streams).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct AllMarketTickersStreamsParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`composite_index_symbol_information_streams`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`composite_index_symbol_information_streams`](#method.composite_index_symbol_information_streams).
#[derive(Clone, Debug, TypedBuilder)]
pub struct CompositeIndexSymbolInformationStreamsParams {
    /// The symbol parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`continuous_contract_kline_candlestick_streams`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`continuous_contract_kline_candlestick_streams`](#method.continuous_contract_kline_candlestick_streams).
#[derive(Clone, Debug, TypedBuilder)]
pub struct ContinuousContractKlineCandlestickStreamsParams {
    /// The pair parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub pair: String,
    /// The contractType parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub contract_type: String,
    /// The interval parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub interval: String,
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`contract_info_stream`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`contract_info_stream`](#method.contract_info_stream).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct ContractInfoStreamParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`diff_book_depth_streams`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`diff_book_depth_streams`](#method.diff_book_depth_streams).
#[derive(Clone, Debug, TypedBuilder)]
pub struct DiffBookDepthStreamsParams {
    /// The symbol parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
    /// WebSocket stream update speed
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub update_speed: Option<String>,
}

/// Request parameters for the [`individual_symbol_book_ticker_streams`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`individual_symbol_book_ticker_streams`](#method.individual_symbol_book_ticker_streams).
#[derive(Clone, Debug, TypedBuilder)]
pub struct IndividualSymbolBookTickerStreamsParams {
    /// The symbol parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`individual_symbol_mini_ticker_stream`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`individual_symbol_mini_ticker_stream`](#method.individual_symbol_mini_ticker_stream).
#[derive(Clone, Debug, TypedBuilder)]
pub struct IndividualSymbolMiniTickerStreamParams {
    /// The symbol parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`individual_symbol_ticker_streams`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`individual_symbol_ticker_streams`](#method.individual_symbol_ticker_streams).
#[derive(Clone, Debug, TypedBuilder)]
pub struct IndividualSymbolTickerStreamsParams {
    /// The symbol parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`kline_candlestick_streams`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`kline_candlestick_streams`](#method.kline_candlestick_streams).
#[derive(Clone, Debug, TypedBuilder)]
pub struct KlineCandlestickStreamsParams {
    /// The symbol parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// The interval parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub interval: String,
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`liquidation_order_streams`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`liquidation_order_streams`](#method.liquidation_order_streams).
#[derive(Clone, Debug, TypedBuilder)]
pub struct LiquidationOrderStreamsParams {
    /// The symbol parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`mark_price_stream`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`mark_price_stream`](#method.mark_price_stream).
#[derive(Clone, Debug, TypedBuilder)]
pub struct MarkPriceStreamParams {
    /// The symbol parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
    /// WebSocket stream update speed
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub update_speed: Option<String>,
}

/// Request parameters for the [`mark_price_stream_for_all_market`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`mark_price_stream_for_all_market`](#method.mark_price_stream_for_all_market).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct MarkPriceStreamForAllMarketParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
    /// WebSocket stream update speed
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub update_speed: Option<String>,
}

/// Request parameters for the [`multi_assets_mode_asset_index`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`multi_assets_mode_asset_index`](#method.multi_assets_mode_asset_index).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct MultiAssetsModeAssetIndexParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`partial_book_depth_streams`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`partial_book_depth_streams`](#method.partial_book_depth_streams).
#[derive(Clone, Debug, TypedBuilder)]
pub struct PartialBookDepthStreamsParams {
    /// The symbol parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// The levels parameter
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub levels: i64,
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
    /// WebSocket stream update speed
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub update_speed: Option<String>,
}

#[async_trait]
impl WebsocketMarketStreamsApi for WebsocketMarketStreamsApiClient {
    async fn aggregate_trade_streams(
        &self,
        params: AggregateTradeStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::AggregateTradeStreamsResponse>>> {
        let AggregateTradeStreamsParams { symbol, id } = params;

        let pairs: &[(&str, Option<String>)] = &[("symbol", Some(symbol.clone())), ("id", id.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/<symbol>@aggTrade", &vars);

        Ok(create_stream_handler::<models::AggregateTradeStreamsResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn all_book_tickers_stream(
        &self,
        params: AllBookTickersStreamParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::AllBookTickersStreamResponse>>> {
        let AllBookTickersStreamParams { id } = params;

        let pairs: &[(&str, Option<String>)] = &[("id", id.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/!bookTicker", &vars);

        Ok(create_stream_handler::<models::AllBookTickersStreamResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn all_market_liquidation_order_streams(
        &self,
        params: AllMarketLiquidationOrderStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::AllMarketLiquidationOrderStreamsResponse>>> {
        let AllMarketLiquidationOrderStreamsParams { id } = params;

        let pairs: &[(&str, Option<String>)] = &[("id", id.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/!forceOrder@arr", &vars);

        Ok(create_stream_handler::<models::AllMarketLiquidationOrderStreamsResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn all_market_mini_tickers_stream(
        &self,
        params: AllMarketMiniTickersStreamParams,
    ) -> anyhow::Result<Arc<WebsocketStream<Vec<models::AllMarketMiniTickersStreamResponseInner>>>> {
        let AllMarketMiniTickersStreamParams { id } = params;

        let pairs: &[(&str, Option<String>)] = &[("id", id.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/!miniTicker@arr", &vars);

        Ok(create_stream_handler::<Vec<models::AllMarketMiniTickersStreamResponseInner>>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn all_market_tickers_streams(
        &self,
        params: AllMarketTickersStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<Vec<models::AllMarketTickersStreamsResponseInner>>>> {
        let AllMarketTickersStreamsParams { id } = params;

        let pairs: &[(&str, Option<String>)] = &[("id", id.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/!ticker@arr", &vars);

        Ok(create_stream_handler::<Vec<models::AllMarketTickersStreamsResponseInner>>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn composite_index_symbol_information_streams(
        &self,
        params: CompositeIndexSymbolInformationStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::CompositeIndexSymbolInformationStreamsResponse>>> {
        let CompositeIndexSymbolInformationStreamsParams { symbol, id } = params;

        let pairs: &[(&str, Option<String>)] = &[("symbol", Some(symbol.clone())), ("id", id.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/<symbol>@compositeIndex", &vars);

        Ok(create_stream_handler::<models::CompositeIndexSymbolInformationStreamsResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn continuous_contract_kline_candlestick_streams(
        &self,
        params: ContinuousContractKlineCandlestickStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::ContinuousContractKlineCandlestickStreamsResponse>>> {
        let ContinuousContractKlineCandlestickStreamsParams {
            pair,
            contract_type,
            interval,
            id,
        } = params;

        let pairs: &[(&str, Option<String>)] = &[
            ("pair", Some(pair.clone())),
            ("contractType", Some(contract_type.clone())),
            ("interval", Some(interval.clone())),
            ("id", id.clone()),
        ];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/<pair>_<contractType>@continuousKline_<interval>", &vars);

        Ok(
            create_stream_handler::<models::ContinuousContractKlineCandlestickStreamsResponse>(
                WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
                stream,
                id_opt,
            )
            .await,
        )
    }

    async fn contract_info_stream(
        &self,
        params: ContractInfoStreamParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::ContractInfoStreamResponse>>> {
        let ContractInfoStreamParams { id } = params;

        let pairs: &[(&str, Option<String>)] = &[("id", id.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/!contractInfo", &vars);

        Ok(create_stream_handler::<models::ContractInfoStreamResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn diff_book_depth_streams(
        &self,
        params: DiffBookDepthStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::DiffBookDepthStreamsResponse>>> {
        let DiffBookDepthStreamsParams {
            symbol,
            id,
            update_speed,
        } = params;

        let pairs: &[(&str, Option<String>)] = &[
            ("symbol", Some(symbol.clone())),
            ("id", id.clone()),
            ("updateSpeed", update_speed.clone()),
        ];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/<symbol>@depth@<updateSpeed>", &vars);

        Ok(create_stream_handler::<models::DiffBookDepthStreamsResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn individual_symbol_book_ticker_streams(
        &self,
        params: IndividualSymbolBookTickerStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::IndividualSymbolBookTickerStreamsResponse>>> {
        let IndividualSymbolBookTickerStreamsParams { symbol, id } = params;

        let pairs: &[(&str, Option<String>)] = &[("symbol", Some(symbol.clone())), ("id", id.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/<symbol>@bookTicker", &vars);

        Ok(create_stream_handler::<models::IndividualSymbolBookTickerStreamsResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn individual_symbol_mini_ticker_stream(
        &self,
        params: IndividualSymbolMiniTickerStreamParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::IndividualSymbolMiniTickerStreamResponse>>> {
        let IndividualSymbolMiniTickerStreamParams { symbol, id } = params;

        let pairs: &[(&str, Option<String>)] = &[("symbol", Some(symbol.clone())), ("id", id.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/<symbol>@miniTicker", &vars);

        Ok(create_stream_handler::<models::IndividualSymbolMiniTickerStreamResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn individual_symbol_ticker_streams(
        &self,
        params: IndividualSymbolTickerStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::IndividualSymbolTickerStreamsResponse>>> {
        let IndividualSymbolTickerStreamsParams { symbol, id } = params;

        let pairs: &[(&str, Option<String>)] = &[("symbol", Some(symbol.clone())), ("id", id.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/<symbol>@ticker", &vars);

        Ok(create_stream_handler::<models::IndividualSymbolTickerStreamsResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn kline_candlestick_streams(
        &self,
        params: KlineCandlestickStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::KlineCandlestickStreamsResponse>>> {
        let KlineCandlestickStreamsParams {
            symbol,
            interval,
            id,
        } = params;

        let pairs: &[(&str, Option<String>)] = &[
            ("symbol", Some(symbol.clone())),
            ("interval", Some(interval.clone())),
            ("id", id.clone()),
        ];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/<symbol>@kline_<interval>", &vars);

        Ok(create_stream_handler::<models::KlineCandlestickStreamsResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn liquidation_order_streams(
        &self,
        params: LiquidationOrderStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::LiquidationOrderStreamsResponse>>> {
        let LiquidationOrderStreamsParams { symbol, id } = params;

        let pairs: &[(&str, Option<String>)] = &[("symbol", Some(symbol.clone())), ("id", id.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/<symbol>@forceOrder", &vars);

        Ok(create_stream_handler::<models::LiquidationOrderStreamsResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn mark_price_stream(
        &self,
        params: MarkPriceStreamParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::MarkPriceStreamResponse>>> {
        let MarkPriceStreamParams {
            symbol,
            id,
            update_speed,
        } = params;

        let pairs: &[(&str, Option<String>)] = &[
            ("symbol", Some(symbol.clone())),
            ("id", id.clone()),
            ("updateSpeed", update_speed.clone()),
        ];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/<symbol>@markPrice@<updateSpeed>", &vars);

        Ok(create_stream_handler::<models::MarkPriceStreamResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn mark_price_stream_for_all_market(
        &self,
        params: MarkPriceStreamForAllMarketParams,
    ) -> anyhow::Result<Arc<WebsocketStream<Vec<models::MarkPriceStreamForAllMarketResponseInner>>>> {
        let MarkPriceStreamForAllMarketParams { id, update_speed } = params;

        let pairs: &[(&str, Option<String>)] = &[("id", id.clone()), ("updateSpeed", update_speed.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/!markPrice@arr@<updateSpeed>", &vars);

        Ok(create_stream_handler::<Vec<models::MarkPriceStreamForAllMarketResponseInner>>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn multi_assets_mode_asset_index(
        &self,
        params: MultiAssetsModeAssetIndexParams,
    ) -> anyhow::Result<Arc<WebsocketStream<Vec<models::MultiAssetsModeAssetIndexResponseInner>>>> {
        let MultiAssetsModeAssetIndexParams { id } = params;

        let pairs: &[(&str, Option<String>)] = &[("id", id.clone())];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/!assetIndex@arr", &vars);

        Ok(create_stream_handler::<Vec<models::MultiAssetsModeAssetIndexResponseInner>>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }

    async fn partial_book_depth_streams(
        &self,
        params: PartialBookDepthStreamsParams,
    ) -> anyhow::Result<Arc<WebsocketStream<models::PartialBookDepthStreamsResponse>>> {
        let PartialBookDepthStreamsParams {
            symbol,
            levels,
            id,
            update_speed,
        } = params;

        let pairs: &[(&str, Option<String>)] = &[
            ("symbol", Some(symbol.clone())),
            ("levels", Some(levels.to_string())),
            ("id", id.clone()),
            ("updateSpeed", update_speed.clone()),
        ];

        let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();

        let id_opt: Option<String> = vars.get("id").map(std::string::ToString::to_string);

        let stream = replace_websocket_streams_placeholders("/<symbol>@depth<levels>@<updateSpeed>", &vars);

        Ok(create_stream_handler::<models::PartialBookDepthStreamsResponse>(
            WebsocketBase::WebsocketStreams(Arc::clone(&self.websocket_streams_base)),
            stream,
            id_opt,
        )
        .await)
    }
}

// #[cfg(all(test, feature = "derivatives_trading_usds_futures"))]
// mod tests {
//     use super::*;
//     use crate::TOKIO_SHARED_RT;
//     use crate::{
//         common::websocket::{WebsocketConnection, WebsocketHandler},
//         config::ConfigurationWebsocketStreams,
//     };
//     use serde_json::json;
//     use std::sync::atomic::{AtomicBool, Ordering};
//     use tokio::task::yield_now;

//     async fn make_streams_base() -> (Arc<WebsocketStreams>, Arc<WebsocketConnection>) {
//         let conn = WebsocketConnection::new("test");
//         let config = ConfigurationWebsocketStreams::builder()
//             .build()
//             .expect("Failed to build configuration");
//         let streams_base = WebsocketStreams::new(config, vec![conn.clone()]);
//         conn.set_handler(streams_base.clone() as Arc<dyn WebsocketHandler>).await;
//         (streams_base, conn)
//     }

//     #[test]
//     fn aggregate_trade_streams_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AggregateTradeStreamsParams::builder("btcusdt".to_string())
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let AggregateTradeStreamsParams { symbol, id } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("symbol", Some(symbol.clone())), ("id", id.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@aggTrade", &vars);
//             let ws_stream = api
//                 .aggregate_trade_streams(params)
//                 .await
//                 .expect("aggregate_trade_streams should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn aggregate_trade_streams_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AggregateTradeStreamsParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let AggregateTradeStreamsParams {
//                 symbol,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@aggTrade", &vars);

//             let ws_stream = api.aggregate_trade_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::AggregateTradeStreamsResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"aggTrade","E":123456789,"s":"BTCUSDT","a":5933014,"p":"0.001","q":"100","f":100,"l":105,"T":123456785,"m":true}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn aggregate_trade_streams_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AggregateTradeStreamsParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let AggregateTradeStreamsParams {
//                 symbol,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@aggTrade", &vars);

//             let ws_stream = api.aggregate_trade_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::AggregateTradeStreamsResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"aggTrade","E":123456789,"s":"BTCUSDT","a":5933014,"p":"0.001","q":"100","f":100,"l":105,"T":123456785,"m":true}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn all_book_tickers_stream_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AllBookTickersStreamParams::builder().id(Some(id.clone())).build().unwrap();

//             let AllBookTickersStreamParams { id } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("id", id.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/!bookTicker", &vars);
//             let ws_stream = api
//                 .all_book_tickers_stream(params)
//                 .await
//                 .expect("all_book_tickers_stream should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn all_book_tickers_stream_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AllBookTickersStreamParams::builder().id(Some(id.clone())).build().unwrap();

//             let AllBookTickersStreamParams {
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!bookTicker", &vars);

//             let ws_stream = api.all_book_tickers_stream(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::AllBookTickersStreamResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"bookTicker","u":400900217,"E":1568014460893,"T":1568014460891,"s":"BNBUSDT","b":"25.35190000","B":"31.21000000","a":"25.36520000","A":"40.66000000"}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn all_book_tickers_stream_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AllBookTickersStreamParams::builder().id(Some(id.clone())).build().unwrap();

//             let AllBookTickersStreamParams {
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!bookTicker", &vars);

//             let ws_stream = api.all_book_tickers_stream(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::AllBookTickersStreamResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"bookTicker","u":400900217,"E":1568014460893,"T":1568014460891,"s":"BNBUSDT","b":"25.35190000","B":"31.21000000","a":"25.36520000","A":"40.66000000"}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn all_market_liquidation_order_streams_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AllMarketLiquidationOrderStreamsParams::builder()
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let AllMarketLiquidationOrderStreamsParams { id } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("id", id.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/!forceOrder@arr", &vars);
//             let ws_stream = api
//                 .all_market_liquidation_order_streams(params)
//                 .await
//                 .expect("all_market_liquidation_order_streams should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn all_market_liquidation_order_streams_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AllMarketLiquidationOrderStreamsParams::builder().id(Some(id.clone())).build().unwrap();

//             let AllMarketLiquidationOrderStreamsParams {
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!forceOrder@arr", &vars);

//             let ws_stream = api.all_market_liquidation_order_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::AllMarketLiquidationOrderStreamsResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"forceOrder","E":1568014460893,"o":{"s":"BTCUSDT","S":"SELL","o":"LIMIT","f":"IOC","q":"0.014","p":"9910","ap":"9910","X":"FILLED","l":"0.014","z":"0.014","T":1568014460893}}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn all_market_liquidation_order_streams_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AllMarketLiquidationOrderStreamsParams::builder().id(Some(id.clone())).build().unwrap();

//             let AllMarketLiquidationOrderStreamsParams {
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!forceOrder@arr", &vars);

//             let ws_stream = api.all_market_liquidation_order_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::AllMarketLiquidationOrderStreamsResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"forceOrder","E":1568014460893,"o":{"s":"BTCUSDT","S":"SELL","o":"LIMIT","f":"IOC","q":"0.014","p":"9910","ap":"9910","X":"FILLED","l":"0.014","z":"0.014","T":1568014460893}}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn all_market_mini_tickers_stream_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AllMarketMiniTickersStreamParams::builder()
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let AllMarketMiniTickersStreamParams { id } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("id", id.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/!miniTicker@arr", &vars);
//             let ws_stream = api
//                 .all_market_mini_tickers_stream(params)
//                 .await
//                 .expect("all_market_mini_tickers_stream should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn all_market_mini_tickers_stream_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AllMarketMiniTickersStreamParams::builder().id(Some(id.clone())).build().unwrap();

//             let AllMarketMiniTickersStreamParams {
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!miniTicker@arr", &vars);

//             let ws_stream = api.all_market_mini_tickers_stream(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: Vec<models::AllMarketMiniTickersStreamResponseInner>| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"[{"e":"24hrMiniTicker","E":123456789,"s":"BTCUSDT","c":"0.0025","o":"0.0010","h":"0.0025","l":"0.0010","v":"10000","q":"18"}]"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn all_market_mini_tickers_stream_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AllMarketMiniTickersStreamParams::builder().id(Some(id.clone())).build().unwrap();

//             let AllMarketMiniTickersStreamParams {
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!miniTicker@arr", &vars);

//             let ws_stream = api.all_market_mini_tickers_stream(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: Vec<models::AllMarketMiniTickersStreamResponseInner>| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"[{"e":"24hrMiniTicker","E":123456789,"s":"BTCUSDT","c":"0.0025","o":"0.0010","h":"0.0025","l":"0.0010","v":"10000","q":"18"}]"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn all_market_tickers_streams_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AllMarketTickersStreamsParams::builder().id(Some(id.clone())).build().unwrap();

//             let AllMarketTickersStreamsParams { id } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("id", id.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/!ticker@arr", &vars);
//             let ws_stream = api
//                 .all_market_tickers_streams(params)
//                 .await
//                 .expect("all_market_tickers_streams should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn all_market_tickers_streams_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AllMarketTickersStreamsParams::builder().id(Some(id.clone())).build().unwrap();

//             let AllMarketTickersStreamsParams {
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!ticker@arr", &vars);

//             let ws_stream = api.all_market_tickers_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: Vec<models::AllMarketTickersStreamsResponseInner>| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"[{"e":"24hrTicker","E":123456789,"s":"BTCUSDT","p":"0.0015","P":"250.00","w":"0.0018","c":"0.0025","Q":"10","o":"0.0010","h":"0.0025","l":"0.0010","v":"10000","q":"18","O":0,"C":86400000,"F":0,"L":18150,"n":18151}]"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn all_market_tickers_streams_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = AllMarketTickersStreamsParams::builder().id(Some(id.clone())).build().unwrap();

//             let AllMarketTickersStreamsParams {
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!ticker@arr", &vars);

//             let ws_stream = api.all_market_tickers_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: Vec<models::AllMarketTickersStreamsResponseInner>| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"[{"e":"24hrTicker","E":123456789,"s":"BTCUSDT","p":"0.0015","P":"250.00","w":"0.0018","c":"0.0025","Q":"10","o":"0.0010","h":"0.0025","l":"0.0010","v":"10000","q":"18","O":0,"C":86400000,"F":0,"L":18150,"n":18151}]"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn composite_index_symbol_information_streams_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = CompositeIndexSymbolInformationStreamsParams::builder("btcusdt".to_string())
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let CompositeIndexSymbolInformationStreamsParams { symbol, id } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("symbol", Some(symbol.clone())), ("id", id.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@compositeIndex", &vars);
//             let ws_stream = api
//                 .composite_index_symbol_information_streams(params)
//                 .await
//                 .expect("composite_index_symbol_information_streams should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn composite_index_symbol_information_streams_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = CompositeIndexSymbolInformationStreamsParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let CompositeIndexSymbolInformationStreamsParams {
//                 symbol,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@compositeIndex", &vars);

//             let ws_stream = api.composite_index_symbol_information_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::CompositeIndexSymbolInformationStreamsResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"compositeIndex","E":1602310596000,"s":"DEFIUSDT","p":"554.41604065","C":"baseAsset","c":[{"b":"BAL","q":"USDT","w":"1.04884844","W":"0.01457800","i":"24.33521021"},{"b":"BAND","q":"USDT","w":"3.53782729","W":"0.03935200","i":"7.26420084"}]}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn composite_index_symbol_information_streams_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = CompositeIndexSymbolInformationStreamsParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let CompositeIndexSymbolInformationStreamsParams {
//                 symbol,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@compositeIndex", &vars);

//             let ws_stream = api.composite_index_symbol_information_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::CompositeIndexSymbolInformationStreamsResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"compositeIndex","E":1602310596000,"s":"DEFIUSDT","p":"554.41604065","C":"baseAsset","c":[{"b":"BAL","q":"USDT","w":"1.04884844","W":"0.01457800","i":"24.33521021"},{"b":"BAND","q":"USDT","w":"3.53782729","W":"0.03935200","i":"7.26420084"}]}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn continuous_contract_kline_candlestick_streams_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = ContinuousContractKlineCandlestickStreamsParams::builder(
//                 "btcusdt".to_string(),
//                 "next_quarter".to_string(),
//                 "1m".to_string(),
//             )
//             .id(Some(id.clone()))
//             .build()
//             .unwrap();

//             let ContinuousContractKlineCandlestickStreamsParams {
//                 pair,
//                 contract_type,
//                 interval,
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("pair", Some(pair.clone())),
//                 ("contractType", Some(contract_type.clone())),
//                 ("interval", Some(interval.clone())),
//                 ("id", id.clone()),
//             ];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream =
//                 replace_websocket_streams_placeholders("/<pair>_<contractType>@continuousKline_<interval>", &vars);
//             let ws_stream = api
//                 .continuous_contract_kline_candlestick_streams(params)
//                 .await
//                 .expect("continuous_contract_kline_candlestick_streams should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn continuous_contract_kline_candlestick_streams_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = ContinuousContractKlineCandlestickStreamsParams::builder("btcusdt".to_string(),"next_quarter".to_string(),"1m".to_string(),).id(Some(id.clone())).build().unwrap();

//             let ContinuousContractKlineCandlestickStreamsParams {
//                 pair,contract_type,interval,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("pair",
//                         Some(pair.clone())
//                 ),
//                 ("contractType",
//                         Some(contract_type.clone())
//                 ),
//                 ("interval",
//                         Some(interval.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<pair>_<contractType>@continuousKline_<interval>", &vars);

//             let ws_stream = api.continuous_contract_kline_candlestick_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::ContinuousContractKlineCandlestickStreamsResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"continuous_kline","E":1607443058651,"ps":"BTCUSDT","ct":"PERPETUAL","k":{"t":1607443020000,"T":1607443079999,"i":"1m","f":116467658886,"L":116468012423,"o":"18787.00","c":"18804.04","h":"18804.04","l":"18786.54","v":"197.664","n":543,"x":false,"q":"3715253.19494","V":"184.769","Q":"3472925.84746","B":"0"}}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn continuous_contract_kline_candlestick_streams_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = ContinuousContractKlineCandlestickStreamsParams::builder("btcusdt".to_string(),"next_quarter".to_string(),"1m".to_string(),).id(Some(id.clone())).build().unwrap();

//             let ContinuousContractKlineCandlestickStreamsParams {
//                 pair,contract_type,interval,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("pair",
//                         Some(pair.clone())
//                 ),
//                 ("contractType",
//                         Some(contract_type.clone())
//                 ),
//                 ("interval",
//                         Some(interval.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<pair>_<contractType>@continuousKline_<interval>", &vars);

//             let ws_stream = api.continuous_contract_kline_candlestick_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::ContinuousContractKlineCandlestickStreamsResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"continuous_kline","E":1607443058651,"ps":"BTCUSDT","ct":"PERPETUAL","k":{"t":1607443020000,"T":1607443079999,"i":"1m","f":116467658886,"L":116468012423,"o":"18787.00","c":"18804.04","h":"18804.04","l":"18786.54","v":"197.664","n":543,"x":false,"q":"3715253.19494","V":"184.769","Q":"3472925.84746","B":"0"}}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn contract_info_stream_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = ContractInfoStreamParams::builder().id(Some(id.clone())).build().unwrap();

//             let ContractInfoStreamParams { id } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("id", id.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/!contractInfo", &vars);
//             let ws_stream = api
//                 .contract_info_stream(params)
//                 .await
//                 .expect("contract_info_stream should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn contract_info_stream_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = ContractInfoStreamParams::builder().id(Some(id.clone())).build().unwrap();

//             let ContractInfoStreamParams {
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!contractInfo", &vars);

//             let ws_stream = api.contract_info_stream(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::ContractInfoStreamResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"contractInfo","E":1669356423908,"s":"IOTAUSDT","ps":"IOTAUSDT","ct":"PERPETUAL","dt":4133404800000,"ot":1569398400000,"cs":"TRADING","bks":[{"bs":1,"bnf":0,"bnc":5000,"mmr":0.01,"cf":0,"mi":21,"ma":50},{"bs":2,"bnf":5000,"bnc":25000,"mmr":0.025,"cf":75,"mi":11,"ma":20}]}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn contract_info_stream_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = ContractInfoStreamParams::builder().id(Some(id.clone())).build().unwrap();

//             let ContractInfoStreamParams {
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!contractInfo", &vars);

//             let ws_stream = api.contract_info_stream(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::ContractInfoStreamResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"contractInfo","E":1669356423908,"s":"IOTAUSDT","ps":"IOTAUSDT","ct":"PERPETUAL","dt":4133404800000,"ot":1569398400000,"cs":"TRADING","bks":[{"bs":1,"bnf":0,"bnc":5000,"mmr":0.01,"cf":0,"mi":21,"ma":50},{"bs":2,"bnf":5000,"bnc":25000,"mmr":0.025,"cf":75,"mi":11,"ma":20}]}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn diff_book_depth_streams_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = DiffBookDepthStreamsParams::builder("btcusdt".to_string())
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let DiffBookDepthStreamsParams {
//                 symbol,
//                 id,
//                 update_speed,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol", Some(symbol.clone())),
//                 ("id", id.clone()),
//                 ("updateSpeed", update_speed.clone()),
//             ];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@depth@<updateSpeed>", &vars);
//             let ws_stream = api
//                 .diff_book_depth_streams(params)
//                 .await
//                 .expect("diff_book_depth_streams should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn diff_book_depth_streams_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = DiffBookDepthStreamsParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let DiffBookDepthStreamsParams {
//                 symbol,id,update_speed,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//                 ("updateSpeed",
//                         update_speed.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@depth@<updateSpeed>", &vars);

//             let ws_stream = api.diff_book_depth_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::DiffBookDepthStreamsResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"depthUpdate","E":123456789,"T":123456788,"s":"BTCUSDT","U":157,"u":160,"pu":149,"b":[["0.0024","10"]],"a":[["0.0026","100"]]}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn diff_book_depth_streams_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = DiffBookDepthStreamsParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let DiffBookDepthStreamsParams {
//                 symbol,id,update_speed,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//                 ("updateSpeed",
//                         update_speed.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@depth@<updateSpeed>", &vars);

//             let ws_stream = api.diff_book_depth_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::DiffBookDepthStreamsResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"depthUpdate","E":123456789,"T":123456788,"s":"BTCUSDT","U":157,"u":160,"pu":149,"b":[["0.0024","10"]],"a":[["0.0026","100"]]}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn individual_symbol_book_ticker_streams_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = IndividualSymbolBookTickerStreamsParams::builder("btcusdt".to_string())
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let IndividualSymbolBookTickerStreamsParams { symbol, id } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("symbol", Some(symbol.clone())), ("id", id.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@bookTicker", &vars);
//             let ws_stream = api
//                 .individual_symbol_book_ticker_streams(params)
//                 .await
//                 .expect("individual_symbol_book_ticker_streams should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn individual_symbol_book_ticker_streams_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = IndividualSymbolBookTickerStreamsParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let IndividualSymbolBookTickerStreamsParams {
//                 symbol,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@bookTicker", &vars);

//             let ws_stream = api.individual_symbol_book_ticker_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::IndividualSymbolBookTickerStreamsResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"bookTicker","u":400900217,"E":1568014460893,"T":1568014460891,"s":"BNBUSDT","b":"25.35190000","B":"31.21000000","a":"25.36520000","A":"40.66000000"}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn individual_symbol_book_ticker_streams_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = IndividualSymbolBookTickerStreamsParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let IndividualSymbolBookTickerStreamsParams {
//                 symbol,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@bookTicker", &vars);

//             let ws_stream = api.individual_symbol_book_ticker_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::IndividualSymbolBookTickerStreamsResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"bookTicker","u":400900217,"E":1568014460893,"T":1568014460891,"s":"BNBUSDT","b":"25.35190000","B":"31.21000000","a":"25.36520000","A":"40.66000000"}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn individual_symbol_mini_ticker_stream_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = IndividualSymbolMiniTickerStreamParams::builder("btcusdt".to_string())
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let IndividualSymbolMiniTickerStreamParams { symbol, id } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("symbol", Some(symbol.clone())), ("id", id.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@miniTicker", &vars);
//             let ws_stream = api
//                 .individual_symbol_mini_ticker_stream(params)
//                 .await
//                 .expect("individual_symbol_mini_ticker_stream should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn individual_symbol_mini_ticker_stream_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = IndividualSymbolMiniTickerStreamParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let IndividualSymbolMiniTickerStreamParams {
//                 symbol,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@miniTicker", &vars);

//             let ws_stream = api.individual_symbol_mini_ticker_stream(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::IndividualSymbolMiniTickerStreamResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"24hrMiniTicker","E":123456789,"s":"BTCUSDT","c":"0.0025","o":"0.0010","h":"0.0025","l":"0.0010","v":"10000","q":"18"}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn individual_symbol_mini_ticker_stream_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = IndividualSymbolMiniTickerStreamParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let IndividualSymbolMiniTickerStreamParams {
//                 symbol,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@miniTicker", &vars);

//             let ws_stream = api.individual_symbol_mini_ticker_stream(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::IndividualSymbolMiniTickerStreamResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"24hrMiniTicker","E":123456789,"s":"BTCUSDT","c":"0.0025","o":"0.0010","h":"0.0025","l":"0.0010","v":"10000","q":"18"}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn individual_symbol_ticker_streams_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = IndividualSymbolTickerStreamsParams::builder("btcusdt".to_string())
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let IndividualSymbolTickerStreamsParams { symbol, id } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("symbol", Some(symbol.clone())), ("id", id.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@ticker", &vars);
//             let ws_stream = api
//                 .individual_symbol_ticker_streams(params)
//                 .await
//                 .expect("individual_symbol_ticker_streams should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn individual_symbol_ticker_streams_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = IndividualSymbolTickerStreamsParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let IndividualSymbolTickerStreamsParams {
//                 symbol,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@ticker", &vars);

//             let ws_stream = api.individual_symbol_ticker_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::IndividualSymbolTickerStreamsResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"24hrTicker","E":123456789,"s":"BTCUSDT","p":"0.0015","P":"250.00","w":"0.0018","c":"0.0025","Q":"10","o":"0.0010","h":"0.0025","l":"0.0010","v":"10000","q":"18","O":0,"C":86400000,"F":0,"L":18150,"n":18151}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn individual_symbol_ticker_streams_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = IndividualSymbolTickerStreamsParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let IndividualSymbolTickerStreamsParams {
//                 symbol,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@ticker", &vars);

//             let ws_stream = api.individual_symbol_ticker_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::IndividualSymbolTickerStreamsResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"24hrTicker","E":123456789,"s":"BTCUSDT","p":"0.0015","P":"250.00","w":"0.0018","c":"0.0025","Q":"10","o":"0.0010","h":"0.0025","l":"0.0010","v":"10000","q":"18","O":0,"C":86400000,"F":0,"L":18150,"n":18151}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn kline_candlestick_streams_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = KlineCandlestickStreamsParams::builder("btcusdt".to_string(), "1m".to_string())
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let KlineCandlestickStreamsParams {
//                 symbol,
//                 interval,
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol", Some(symbol.clone())),
//                 ("interval", Some(interval.clone())),
//                 ("id", id.clone()),
//             ];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@kline_<interval>", &vars);
//             let ws_stream = api
//                 .kline_candlestick_streams(params)
//                 .await
//                 .expect("kline_candlestick_streams should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn kline_candlestick_streams_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = KlineCandlestickStreamsParams::builder("btcusdt".to_string(),"1m".to_string(),).id(Some(id.clone())).build().unwrap();

//             let KlineCandlestickStreamsParams {
//                 symbol,interval,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("interval",
//                         Some(interval.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@kline_<interval>", &vars);

//             let ws_stream = api.kline_candlestick_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::KlineCandlestickStreamsResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"kline","E":1638747660000,"s":"BTCUSDT","k":{"t":1638747660000,"T":1638747719999,"s":"BTCUSDT","i":"1m","f":100,"L":200,"o":"0.0010","c":"0.0020","h":"0.0025","l":"0.0015","v":"1000","n":100,"x":false,"q":"1.0000","V":"500","Q":"0.500","B":"123456"}}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn kline_candlestick_streams_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = KlineCandlestickStreamsParams::builder("btcusdt".to_string(),"1m".to_string(),).id(Some(id.clone())).build().unwrap();

//             let KlineCandlestickStreamsParams {
//                 symbol,interval,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("interval",
//                         Some(interval.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@kline_<interval>", &vars);

//             let ws_stream = api.kline_candlestick_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::KlineCandlestickStreamsResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"kline","E":1638747660000,"s":"BTCUSDT","k":{"t":1638747660000,"T":1638747719999,"s":"BTCUSDT","i":"1m","f":100,"L":200,"o":"0.0010","c":"0.0020","h":"0.0025","l":"0.0015","v":"1000","n":100,"x":false,"q":"1.0000","V":"500","Q":"0.500","B":"123456"}}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn liquidation_order_streams_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = LiquidationOrderStreamsParams::builder("btcusdt".to_string())
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let LiquidationOrderStreamsParams { symbol, id } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("symbol", Some(symbol.clone())), ("id", id.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@forceOrder", &vars);
//             let ws_stream = api
//                 .liquidation_order_streams(params)
//                 .await
//                 .expect("liquidation_order_streams should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn liquidation_order_streams_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = LiquidationOrderStreamsParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let LiquidationOrderStreamsParams {
//                 symbol,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@forceOrder", &vars);

//             let ws_stream = api.liquidation_order_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::LiquidationOrderStreamsResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"forceOrder","E":1568014460893,"o":{"s":"BTCUSDT","S":"SELL","o":"LIMIT","f":"IOC","q":"0.014","p":"9910","ap":"9910","X":"FILLED","l":"0.014","z":"0.014","T":1568014460893}}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn liquidation_order_streams_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = LiquidationOrderStreamsParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let LiquidationOrderStreamsParams {
//                 symbol,id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@forceOrder", &vars);

//             let ws_stream = api.liquidation_order_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::LiquidationOrderStreamsResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"forceOrder","E":1568014460893,"o":{"s":"BTCUSDT","S":"SELL","o":"LIMIT","f":"IOC","q":"0.014","p":"9910","ap":"9910","X":"FILLED","l":"0.014","z":"0.014","T":1568014460893}}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn mark_price_stream_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = MarkPriceStreamParams::builder("btcusdt".to_string())
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let MarkPriceStreamParams {
//                 symbol,
//                 id,
//                 update_speed,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol", Some(symbol.clone())),
//                 ("id", id.clone()),
//                 ("updateSpeed", update_speed.clone()),
//             ];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@markPrice@<updateSpeed>", &vars);
//             let ws_stream = api
//                 .mark_price_stream(params)
//                 .await
//                 .expect("mark_price_stream should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn mark_price_stream_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = MarkPriceStreamParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let MarkPriceStreamParams {
//                 symbol,id,update_speed,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//                 ("updateSpeed",
//                         update_speed.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@markPrice@<updateSpeed>", &vars);

//             let ws_stream = api.mark_price_stream(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::MarkPriceStreamResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"markPriceUpdate","E":1562305380000,"s":"BTCUSDT","p":"11794.15000000","i":"11784.62659091","P":"11784.25641265","r":"0.00038167","T":1562306400000}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn mark_price_stream_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = MarkPriceStreamParams::builder("btcusdt".to_string(),).id(Some(id.clone())).build().unwrap();

//             let MarkPriceStreamParams {
//                 symbol,id,update_speed,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//                 ("updateSpeed",
//                         update_speed.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@markPrice@<updateSpeed>", &vars);

//             let ws_stream = api.mark_price_stream(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::MarkPriceStreamResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"markPriceUpdate","E":1562305380000,"s":"BTCUSDT","p":"11794.15000000","i":"11784.62659091","P":"11784.25641265","r":"0.00038167","T":1562306400000}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn mark_price_stream_for_all_market_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = MarkPriceStreamForAllMarketParams::builder()
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let MarkPriceStreamForAllMarketParams { id, update_speed } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("id", id.clone()), ("updateSpeed", update_speed.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/!markPrice@arr@<updateSpeed>", &vars);
//             let ws_stream = api
//                 .mark_price_stream_for_all_market(params)
//                 .await
//                 .expect("mark_price_stream_for_all_market should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn mark_price_stream_for_all_market_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = MarkPriceStreamForAllMarketParams::builder().id(Some(id.clone())).build().unwrap();

//             let MarkPriceStreamForAllMarketParams {
//                 id,update_speed,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//                 ("updateSpeed",
//                         update_speed.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!markPrice@arr@<updateSpeed>", &vars);

//             let ws_stream = api.mark_price_stream_for_all_market(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: Vec<models::MarkPriceStreamForAllMarketResponseInner>| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"[{"e":"markPriceUpdate","E":1562305380000,"s":"BTCUSDT","p":"11185.87786614","i":"11784.62659091","P":"11784.25641265","r":"0.00030000","T":1562306400000}]"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn mark_price_stream_for_all_market_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = MarkPriceStreamForAllMarketParams::builder().id(Some(id.clone())).build().unwrap();

//             let MarkPriceStreamForAllMarketParams {
//                 id,update_speed,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//                 ("updateSpeed",
//                         update_speed.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!markPrice@arr@<updateSpeed>", &vars);

//             let ws_stream = api.mark_price_stream_for_all_market(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: Vec<models::MarkPriceStreamForAllMarketResponseInner>| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"[{"e":"markPriceUpdate","E":1562305380000,"s":"BTCUSDT","p":"11185.87786614","i":"11784.62659091","P":"11784.25641265","r":"0.00030000","T":1562306400000}]"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn multi_assets_mode_asset_index_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = MultiAssetsModeAssetIndexParams::builder().id(Some(id.clone())).build().unwrap();

//             let MultiAssetsModeAssetIndexParams { id } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[("id", id.clone())];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/!assetIndex@arr", &vars);
//             let ws_stream = api
//                 .multi_assets_mode_asset_index(params)
//                 .await
//                 .expect("multi_assets_mode_asset_index should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn multi_assets_mode_asset_index_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = MultiAssetsModeAssetIndexParams::builder().id(Some(id.clone())).build().unwrap();

//             let MultiAssetsModeAssetIndexParams {
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!assetIndex@arr", &vars);

//             let ws_stream = api.multi_assets_mode_asset_index(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: Vec<models::MultiAssetsModeAssetIndexResponseInner>| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"[{"e":"assetIndexUpdate","E":1686749230000,"s":"ADAUSD","i":"0.27462452","b":"0.10000000","a":"0.10000000","B":"0.24716207","A":"0.30208698","q":"0.05000000","g":"0.05000000","Q":"0.26089330","G":"0.28835575"},{"e":"assetIndexUpdate","E":1686749230000,"s":"USDTUSD","i":"0.99987691","b":"0.00010000","a":"0.00010000","B":"0.99977692","A":"0.99997689","q":"0.00010000","g":"0.00010000","Q":"0.99977692","G":"0.99997689"}]"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn multi_assets_mode_asset_index_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = MultiAssetsModeAssetIndexParams::builder().id(Some(id.clone())).build().unwrap();

//             let MultiAssetsModeAssetIndexParams {
//                 id,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("id",
//                         id.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/!assetIndex@arr", &vars);

//             let ws_stream = api.multi_assets_mode_asset_index(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: Vec<models::MultiAssetsModeAssetIndexResponseInner>| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"[{"e":"assetIndexUpdate","E":1686749230000,"s":"ADAUSD","i":"0.27462452","b":"0.10000000","a":"0.10000000","B":"0.24716207","A":"0.30208698","q":"0.05000000","g":"0.05000000","Q":"0.26089330","G":"0.28835575"},{"e":"assetIndexUpdate","E":1686749230000,"s":"USDTUSD","i":"0.99987691","b":"0.00010000","a":"0.00010000","B":"0.99977692","A":"0.99997689","q":"0.00010000","g":"0.00010000","Q":"0.99977692","G":"0.99997689"}]"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }

//     #[test]
//     fn partial_book_depth_streams_should_execute_successfully() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, _) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = PartialBookDepthStreamsParams::builder("btcusdt".to_string(), 10)
//                 .id(Some(id.clone()))
//                 .build()
//                 .unwrap();

//             let PartialBookDepthStreamsParams {
//                 symbol,
//                 levels,
//                 id,
//                 update_speed,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol", Some(symbol.clone())),
//                 ("levels", Some(levels.to_string())),
//                 ("id", id.clone()),
//                 ("updateSpeed", update_speed.clone()),
//             ];

//             let vars: HashMap<_, _> = pairs.iter().filter_map(|&(k, ref v)| v.clone().map(|v| (k, v))).collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@depth<levels>@<updateSpeed>", &vars);
//             let ws_stream = api
//                 .partial_book_depth_streams(params)
//                 .await
//                 .expect("partial_book_depth_streams should return a WebsocketStream");

//             assert!(
//                 streams_base.is_subscribed(&stream).await,
//                 "expected stream '{stream}' to be subscribed"
//             );
//             assert_eq!(ws_stream.id.as_deref(), Some("test-id-123"));
//         });
//     }

//     #[test]
//     fn partial_book_depth_streams_should_handle_incoming_message() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = PartialBookDepthStreamsParams::builder("btcusdt".to_string(),10,).id(Some(id.clone())).build().unwrap();

//             let PartialBookDepthStreamsParams {
//                 symbol,levels,id,update_speed,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("levels",
//                         Some(levels.to_string())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//                 ("updateSpeed",
//                         update_speed.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@depth<levels>@<updateSpeed>", &vars);

//             let ws_stream = api.partial_book_depth_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_with_message = called.clone();
//             ws_stream.on_message(move |_payload: models::PartialBookDepthStreamsResponse| {
//                 called_with_message.store(true, Ordering::SeqCst);
//             });

//             let payload: Value = serde_json::from_str(r#"{"e":"depthUpdate","E":1571889248277,"T":1571889248276,"s":"BTCUSDT","U":390497796,"u":390497878,"pu":390497794,"b":[["7403.89","0.002"],["7403.90","3.906"],["7404.00","1.428"],["7404.85","5.239"],["7405.43","2.562"]],"a":[["7405.96","3.340"],["7406.63","4.525"],["7407.08","2.475"],["7407.15","4.800"],["7407.20","0.175"]]}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;
//             yield_now().await;

//             assert!(called.load(Ordering::SeqCst), "expected our callback to have been invoked");
//         });
//     }

//     #[test]
//     fn partial_book_depth_streams_should_not_fire_after_unsubscribe() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (streams_base, conn) = make_streams_base().await;
//             let api = WebsocketMarketStreamsApiClient::new(streams_base.clone());

//             let id = "test-id-123".to_string();

//             let params = PartialBookDepthStreamsParams::builder("btcusdt".to_string(),10,).id(Some(id.clone())).build().unwrap();

//             let PartialBookDepthStreamsParams {
//                 symbol,levels,id,update_speed,
//             } = params.clone();

//             let pairs: &[(&str, Option<String>)] = &[
//                 ("symbol",
//                         Some(symbol.clone())
//                 ),
//                 ("levels",
//                         Some(levels.to_string())
//                 ),
//                 ("id",
//                         id.clone()
//                 ),
//                 ("updateSpeed",
//                         update_speed.clone()
//                 ),
//             ];

//             let vars: HashMap<_, _> = pairs
//                 .iter()
//                 .filter_map(|&(k, ref v)| v.clone().map(|v| (k, v)))
//                 .collect();
//             let stream = replace_websocket_streams_placeholders("/<symbol>@depth<levels>@<updateSpeed>", &vars);

//             let ws_stream = api.partial_book_depth_streams(params).await.unwrap();

//             let called = Arc::new(AtomicBool::new(false));
//             let called_clone = called.clone();
//             ws_stream.on_message(move |_payload: models::PartialBookDepthStreamsResponse| {
//                 called_clone.store(true, Ordering::SeqCst);
//             });

//             assert!(streams_base.is_subscribed(&stream).await, "should be subscribed before unsubscribe");

//             ws_stream.unsubscribe().await;

//             let payload: Value = serde_json::from_str(r#"{"e":"depthUpdate","E":1571889248277,"T":1571889248276,"s":"BTCUSDT","U":390497796,"u":390497878,"pu":390497794,"b":[["7403.89","0.002"],["7403.90","3.906"],["7404.00","1.428"],["7404.85","5.239"],["7405.43","2.562"]],"a":[["7405.96","3.340"],["7406.63","4.525"],["7407.08","2.475"],["7407.15","4.800"],["7407.20","0.175"]]}"#).unwrap();
//             let msg = json!({
//                 "stream": stream,
//                 "data": payload,
//             });

//             streams_base.on_message(msg.to_string(), conn.clone()).await;

//             yield_now().await;

//             assert!(!called.load(Ordering::SeqCst), "callback should not be invoked after unsubscribe");
//         });
//     }
// }
