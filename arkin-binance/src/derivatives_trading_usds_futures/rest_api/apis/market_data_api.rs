#![allow(unused_imports)]
use async_trait::async_trait;
use reqwest;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use typed_builder::TypedBuilder;

use crate::common::{
    config::ConfigurationRestApi,
    models::{ParamBuildError, RestApiResponse},
    utils::send_request,
};
use crate::derivatives_trading_usds_futures::rest_api::models;

const HAS_TIME_UNIT: bool = false;

#[async_trait]
pub trait MarketDataApi: Send + Sync {
    async fn basis(&self, params: BasisParams) -> anyhow::Result<RestApiResponse<Vec<models::BasisResponseInner>>>;
    async fn check_server_time(&self) -> anyhow::Result<RestApiResponse<models::CheckServerTimeResponse>>;
    async fn composite_index_symbol_information(
        &self,
        params: CompositeIndexSymbolInformationParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::CompositeIndexSymbolInformationResponseInner>>>;
    async fn compressed_aggregate_trades_list(
        &self,
        params: CompressedAggregateTradesListParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::CompressedAggregateTradesListResponseInner>>>;
    async fn continuous_contract_kline_candlestick_data(
        &self,
        params: ContinuousContractKlineCandlestickDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::ContinuousContractKlineCandlestickDataResponseItemInner>>>>;
    async fn exchange_information(&self) -> anyhow::Result<RestApiResponse<models::ExchangeInformationResponse>>;
    async fn get_funding_rate_history(
        &self,
        params: GetFundingRateHistoryParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetFundingRateHistoryResponseInner>>>;
    async fn get_funding_rate_info(
        &self,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetFundingRateInfoResponseInner>>>;
    async fn index_price_kline_candlestick_data(
        &self,
        params: IndexPriceKlineCandlestickDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::IndexPriceKlineCandlestickDataResponseItemInner>>>>;
    async fn kline_candlestick_data(
        &self,
        params: KlineCandlestickDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::KlineCandlestickDataResponseItemInner>>>>;
    async fn long_short_ratio(
        &self,
        params: LongShortRatioParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::LongShortRatioResponseInner>>>;
    async fn mark_price(&self, params: MarkPriceParams) -> anyhow::Result<RestApiResponse<models::MarkPriceResponse>>;
    async fn mark_price_kline_candlestick_data(
        &self,
        params: MarkPriceKlineCandlestickDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::MarkPriceKlineCandlestickDataResponseItemInner>>>>;
    async fn multi_assets_mode_asset_index(
        &self,
        params: MultiAssetsModeAssetIndexParams,
    ) -> anyhow::Result<RestApiResponse<models::MultiAssetsModeAssetIndexResponse>>;
    async fn old_trades_lookup(
        &self,
        params: OldTradesLookupParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::OldTradesLookupResponseInner>>>;
    async fn open_interest(
        &self,
        params: OpenInterestParams,
    ) -> anyhow::Result<RestApiResponse<models::OpenInterestResponse>>;
    async fn open_interest_statistics(
        &self,
        params: OpenInterestStatisticsParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::OpenInterestStatisticsResponseInner>>>;
    async fn order_book(&self, params: OrderBookParams) -> anyhow::Result<RestApiResponse<models::OrderBookResponse>>;
    async fn premium_index_kline_data(
        &self,
        params: PremiumIndexKlineDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::PremiumIndexKlineDataResponseItemInner>>>>;
    async fn quarterly_contract_settlement_price(
        &self,
        params: QuarterlyContractSettlementPriceParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::QuarterlyContractSettlementPriceResponseInner>>>;
    async fn query_index_price_constituents(
        &self,
        params: QueryIndexPriceConstituentsParams,
    ) -> anyhow::Result<RestApiResponse<models::QueryIndexPriceConstituentsResponse>>;
    async fn query_insurance_fund_balance_snapshot(
        &self,
        params: QueryInsuranceFundBalanceSnapshotParams,
    ) -> anyhow::Result<RestApiResponse<models::QueryInsuranceFundBalanceSnapshotResponse>>;
    async fn recent_trades_list(
        &self,
        params: RecentTradesListParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::RecentTradesListResponseInner>>>;
    async fn symbol_order_book_ticker(
        &self,
        params: SymbolOrderBookTickerParams,
    ) -> anyhow::Result<RestApiResponse<models::SymbolOrderBookTickerResponse>>;
    async fn symbol_price_ticker(
        &self,
        params: SymbolPriceTickerParams,
    ) -> anyhow::Result<RestApiResponse<models::SymbolPriceTickerResponse>>;
    async fn symbol_price_ticker_v2(
        &self,
        params: SymbolPriceTickerV2Params,
    ) -> anyhow::Result<RestApiResponse<models::SymbolPriceTickerV2Response>>;
    async fn taker_buy_sell_volume(
        &self,
        params: TakerBuySellVolumeParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::TakerBuySellVolumeResponseInner>>>;
    async fn test_connectivity(&self) -> anyhow::Result<RestApiResponse<Value>>;
    async fn ticker24hr_price_change_statistics(
        &self,
        params: Ticker24hrPriceChangeStatisticsParams,
    ) -> anyhow::Result<RestApiResponse<models::Ticker24hrPriceChangeStatisticsResponse>>;
    async fn top_trader_long_short_ratio_accounts(
        &self,
        params: TopTraderLongShortRatioAccountsParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::TopTraderLongShortRatioAccountsResponseInner>>>;
    async fn top_trader_long_short_ratio_positions(
        &self,
        params: TopTraderLongShortRatioPositionsParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::TopTraderLongShortRatioPositionsResponseInner>>>;
}

#[derive(Debug, Clone)]
pub struct MarketDataApiClient {
    configuration: ConfigurationRestApi,
}

impl MarketDataApiClient {
    pub fn new(configuration: ConfigurationRestApi) -> Self {
        Self { configuration }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BasisContractTypeEnum {
    #[serde(rename = "PERPETUAL")]
    Perpetual,
    #[serde(rename = "CURRENT_MONTH")]
    CurrentMonth,
    #[serde(rename = "NEXT_MONTH")]
    NextMonth,
    #[serde(rename = "CURRENT_QUARTER")]
    CurrentQuarter,
    #[serde(rename = "NEXT_QUARTER")]
    NextQuarter,
    #[serde(rename = "PERPETUAL_DELIVERING")]
    PerpetualDelivering,
}

impl BasisContractTypeEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Perpetual => "PERPETUAL",
            Self::CurrentMonth => "CURRENT_MONTH",
            Self::NextMonth => "NEXT_MONTH",
            Self::CurrentQuarter => "CURRENT_QUARTER",
            Self::NextQuarter => "NEXT_QUARTER",
            Self::PerpetualDelivering => "PERPETUAL_DELIVERING",
        }
    }
}

impl std::str::FromStr for BasisContractTypeEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PERPETUAL" => Ok(Self::Perpetual),
            "CURRENT_MONTH" => Ok(Self::CurrentMonth),
            "NEXT_MONTH" => Ok(Self::NextMonth),
            "CURRENT_QUARTER" => Ok(Self::CurrentQuarter),
            "NEXT_QUARTER" => Ok(Self::NextQuarter),
            "PERPETUAL_DELIVERING" => Ok(Self::PerpetualDelivering),
            other => Err(format!("invalid BasisContractTypeEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BasisPeriodEnum {
    #[serde(rename = "5m")]
    Period5m,
    #[serde(rename = "15m")]
    Period15m,
    #[serde(rename = "30m")]
    Period30m,
    #[serde(rename = "1h")]
    Period1h,
    #[serde(rename = "2h")]
    Period2h,
    #[serde(rename = "4h")]
    Period4h,
    #[serde(rename = "6h")]
    Period6h,
    #[serde(rename = "12h")]
    Period12h,
    #[serde(rename = "1d")]
    Period1d,
}

impl BasisPeriodEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Period5m => "5m",
            Self::Period15m => "15m",
            Self::Period30m => "30m",
            Self::Period1h => "1h",
            Self::Period2h => "2h",
            Self::Period4h => "4h",
            Self::Period6h => "6h",
            Self::Period12h => "12h",
            Self::Period1d => "1d",
        }
    }
}

impl std::str::FromStr for BasisPeriodEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "5m" => Ok(Self::Period5m),
            "15m" => Ok(Self::Period15m),
            "30m" => Ok(Self::Period30m),
            "1h" => Ok(Self::Period1h),
            "2h" => Ok(Self::Period2h),
            "4h" => Ok(Self::Period4h),
            "6h" => Ok(Self::Period6h),
            "12h" => Ok(Self::Period12h),
            "1d" => Ok(Self::Period1d),
            other => Err(format!("invalid BasisPeriodEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContinuousContractKlineCandlestickDataContractTypeEnum {
    #[serde(rename = "PERPETUAL")]
    Perpetual,
    #[serde(rename = "CURRENT_MONTH")]
    CurrentMonth,
    #[serde(rename = "NEXT_MONTH")]
    NextMonth,
    #[serde(rename = "CURRENT_QUARTER")]
    CurrentQuarter,
    #[serde(rename = "NEXT_QUARTER")]
    NextQuarter,
    #[serde(rename = "PERPETUAL_DELIVERING")]
    PerpetualDelivering,
}

impl ContinuousContractKlineCandlestickDataContractTypeEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Perpetual => "PERPETUAL",
            Self::CurrentMonth => "CURRENT_MONTH",
            Self::NextMonth => "NEXT_MONTH",
            Self::CurrentQuarter => "CURRENT_QUARTER",
            Self::NextQuarter => "NEXT_QUARTER",
            Self::PerpetualDelivering => "PERPETUAL_DELIVERING",
        }
    }
}

impl std::str::FromStr for ContinuousContractKlineCandlestickDataContractTypeEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PERPETUAL" => Ok(Self::Perpetual),
            "CURRENT_MONTH" => Ok(Self::CurrentMonth),
            "NEXT_MONTH" => Ok(Self::NextMonth),
            "CURRENT_QUARTER" => Ok(Self::CurrentQuarter),
            "NEXT_QUARTER" => Ok(Self::NextQuarter),
            "PERPETUAL_DELIVERING" => Ok(Self::PerpetualDelivering),
            other => Err(format!("invalid ContinuousContractKlineCandlestickDataContractTypeEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContinuousContractKlineCandlestickDataIntervalEnum {
    #[serde(rename = "1m")]
    Interval1m,
    #[serde(rename = "3m")]
    Interval3m,
    #[serde(rename = "5m")]
    Interval5m,
    #[serde(rename = "15m")]
    Interval15m,
    #[serde(rename = "30m")]
    Interval30m,
    #[serde(rename = "1h")]
    Interval1h,
    #[serde(rename = "2h")]
    Interval2h,
    #[serde(rename = "4h")]
    Interval4h,
    #[serde(rename = "6h")]
    Interval6h,
    #[serde(rename = "8h")]
    Interval8h,
    #[serde(rename = "12h")]
    Interval12h,
    #[serde(rename = "1d")]
    Interval1d,
    #[serde(rename = "3d")]
    Interval3d,
    #[serde(rename = "1w")]
    Interval1w,
    #[serde(rename = "1M")]
    Interval1M,
}

impl ContinuousContractKlineCandlestickDataIntervalEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Interval1m => "1m",
            Self::Interval3m => "3m",
            Self::Interval5m => "5m",
            Self::Interval15m => "15m",
            Self::Interval30m => "30m",
            Self::Interval1h => "1h",
            Self::Interval2h => "2h",
            Self::Interval4h => "4h",
            Self::Interval6h => "6h",
            Self::Interval8h => "8h",
            Self::Interval12h => "12h",
            Self::Interval1d => "1d",
            Self::Interval3d => "3d",
            Self::Interval1w => "1w",
            Self::Interval1M => "1M",
        }
    }
}

impl std::str::FromStr for ContinuousContractKlineCandlestickDataIntervalEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1m" => Ok(Self::Interval1m),
            "3m" => Ok(Self::Interval3m),
            "5m" => Ok(Self::Interval5m),
            "15m" => Ok(Self::Interval15m),
            "30m" => Ok(Self::Interval30m),
            "1h" => Ok(Self::Interval1h),
            "2h" => Ok(Self::Interval2h),
            "4h" => Ok(Self::Interval4h),
            "6h" => Ok(Self::Interval6h),
            "8h" => Ok(Self::Interval8h),
            "12h" => Ok(Self::Interval12h),
            "1d" => Ok(Self::Interval1d),
            "3d" => Ok(Self::Interval3d),
            "1w" => Ok(Self::Interval1w),
            "1M" => Ok(Self::Interval1M),
            other => Err(format!("invalid ContinuousContractKlineCandlestickDataIntervalEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexPriceKlineCandlestickDataIntervalEnum {
    #[serde(rename = "1m")]
    Interval1m,
    #[serde(rename = "3m")]
    Interval3m,
    #[serde(rename = "5m")]
    Interval5m,
    #[serde(rename = "15m")]
    Interval15m,
    #[serde(rename = "30m")]
    Interval30m,
    #[serde(rename = "1h")]
    Interval1h,
    #[serde(rename = "2h")]
    Interval2h,
    #[serde(rename = "4h")]
    Interval4h,
    #[serde(rename = "6h")]
    Interval6h,
    #[serde(rename = "8h")]
    Interval8h,
    #[serde(rename = "12h")]
    Interval12h,
    #[serde(rename = "1d")]
    Interval1d,
    #[serde(rename = "3d")]
    Interval3d,
    #[serde(rename = "1w")]
    Interval1w,
    #[serde(rename = "1M")]
    Interval1M,
}

impl IndexPriceKlineCandlestickDataIntervalEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Interval1m => "1m",
            Self::Interval3m => "3m",
            Self::Interval5m => "5m",
            Self::Interval15m => "15m",
            Self::Interval30m => "30m",
            Self::Interval1h => "1h",
            Self::Interval2h => "2h",
            Self::Interval4h => "4h",
            Self::Interval6h => "6h",
            Self::Interval8h => "8h",
            Self::Interval12h => "12h",
            Self::Interval1d => "1d",
            Self::Interval3d => "3d",
            Self::Interval1w => "1w",
            Self::Interval1M => "1M",
        }
    }
}

impl std::str::FromStr for IndexPriceKlineCandlestickDataIntervalEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1m" => Ok(Self::Interval1m),
            "3m" => Ok(Self::Interval3m),
            "5m" => Ok(Self::Interval5m),
            "15m" => Ok(Self::Interval15m),
            "30m" => Ok(Self::Interval30m),
            "1h" => Ok(Self::Interval1h),
            "2h" => Ok(Self::Interval2h),
            "4h" => Ok(Self::Interval4h),
            "6h" => Ok(Self::Interval6h),
            "8h" => Ok(Self::Interval8h),
            "12h" => Ok(Self::Interval12h),
            "1d" => Ok(Self::Interval1d),
            "3d" => Ok(Self::Interval3d),
            "1w" => Ok(Self::Interval1w),
            "1M" => Ok(Self::Interval1M),
            other => Err(format!("invalid IndexPriceKlineCandlestickDataIntervalEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KlineCandlestickDataIntervalEnum {
    #[serde(rename = "1m")]
    Interval1m,
    #[serde(rename = "3m")]
    Interval3m,
    #[serde(rename = "5m")]
    Interval5m,
    #[serde(rename = "15m")]
    Interval15m,
    #[serde(rename = "30m")]
    Interval30m,
    #[serde(rename = "1h")]
    Interval1h,
    #[serde(rename = "2h")]
    Interval2h,
    #[serde(rename = "4h")]
    Interval4h,
    #[serde(rename = "6h")]
    Interval6h,
    #[serde(rename = "8h")]
    Interval8h,
    #[serde(rename = "12h")]
    Interval12h,
    #[serde(rename = "1d")]
    Interval1d,
    #[serde(rename = "3d")]
    Interval3d,
    #[serde(rename = "1w")]
    Interval1w,
    #[serde(rename = "1M")]
    Interval1M,
}

impl KlineCandlestickDataIntervalEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Interval1m => "1m",
            Self::Interval3m => "3m",
            Self::Interval5m => "5m",
            Self::Interval15m => "15m",
            Self::Interval30m => "30m",
            Self::Interval1h => "1h",
            Self::Interval2h => "2h",
            Self::Interval4h => "4h",
            Self::Interval6h => "6h",
            Self::Interval8h => "8h",
            Self::Interval12h => "12h",
            Self::Interval1d => "1d",
            Self::Interval3d => "3d",
            Self::Interval1w => "1w",
            Self::Interval1M => "1M",
        }
    }
}

impl std::str::FromStr for KlineCandlestickDataIntervalEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1m" => Ok(Self::Interval1m),
            "3m" => Ok(Self::Interval3m),
            "5m" => Ok(Self::Interval5m),
            "15m" => Ok(Self::Interval15m),
            "30m" => Ok(Self::Interval30m),
            "1h" => Ok(Self::Interval1h),
            "2h" => Ok(Self::Interval2h),
            "4h" => Ok(Self::Interval4h),
            "6h" => Ok(Self::Interval6h),
            "8h" => Ok(Self::Interval8h),
            "12h" => Ok(Self::Interval12h),
            "1d" => Ok(Self::Interval1d),
            "3d" => Ok(Self::Interval3d),
            "1w" => Ok(Self::Interval1w),
            "1M" => Ok(Self::Interval1M),
            other => Err(format!("invalid KlineCandlestickDataIntervalEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LongShortRatioPeriodEnum {
    #[serde(rename = "5m")]
    Period5m,
    #[serde(rename = "15m")]
    Period15m,
    #[serde(rename = "30m")]
    Period30m,
    #[serde(rename = "1h")]
    Period1h,
    #[serde(rename = "2h")]
    Period2h,
    #[serde(rename = "4h")]
    Period4h,
    #[serde(rename = "6h")]
    Period6h,
    #[serde(rename = "12h")]
    Period12h,
    #[serde(rename = "1d")]
    Period1d,
}

impl LongShortRatioPeriodEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Period5m => "5m",
            Self::Period15m => "15m",
            Self::Period30m => "30m",
            Self::Period1h => "1h",
            Self::Period2h => "2h",
            Self::Period4h => "4h",
            Self::Period6h => "6h",
            Self::Period12h => "12h",
            Self::Period1d => "1d",
        }
    }
}

impl std::str::FromStr for LongShortRatioPeriodEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "5m" => Ok(Self::Period5m),
            "15m" => Ok(Self::Period15m),
            "30m" => Ok(Self::Period30m),
            "1h" => Ok(Self::Period1h),
            "2h" => Ok(Self::Period2h),
            "4h" => Ok(Self::Period4h),
            "6h" => Ok(Self::Period6h),
            "12h" => Ok(Self::Period12h),
            "1d" => Ok(Self::Period1d),
            other => Err(format!("invalid LongShortRatioPeriodEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarkPriceKlineCandlestickDataIntervalEnum {
    #[serde(rename = "1m")]
    Interval1m,
    #[serde(rename = "3m")]
    Interval3m,
    #[serde(rename = "5m")]
    Interval5m,
    #[serde(rename = "15m")]
    Interval15m,
    #[serde(rename = "30m")]
    Interval30m,
    #[serde(rename = "1h")]
    Interval1h,
    #[serde(rename = "2h")]
    Interval2h,
    #[serde(rename = "4h")]
    Interval4h,
    #[serde(rename = "6h")]
    Interval6h,
    #[serde(rename = "8h")]
    Interval8h,
    #[serde(rename = "12h")]
    Interval12h,
    #[serde(rename = "1d")]
    Interval1d,
    #[serde(rename = "3d")]
    Interval3d,
    #[serde(rename = "1w")]
    Interval1w,
    #[serde(rename = "1M")]
    Interval1M,
}

impl MarkPriceKlineCandlestickDataIntervalEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Interval1m => "1m",
            Self::Interval3m => "3m",
            Self::Interval5m => "5m",
            Self::Interval15m => "15m",
            Self::Interval30m => "30m",
            Self::Interval1h => "1h",
            Self::Interval2h => "2h",
            Self::Interval4h => "4h",
            Self::Interval6h => "6h",
            Self::Interval8h => "8h",
            Self::Interval12h => "12h",
            Self::Interval1d => "1d",
            Self::Interval3d => "3d",
            Self::Interval1w => "1w",
            Self::Interval1M => "1M",
        }
    }
}

impl std::str::FromStr for MarkPriceKlineCandlestickDataIntervalEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1m" => Ok(Self::Interval1m),
            "3m" => Ok(Self::Interval3m),
            "5m" => Ok(Self::Interval5m),
            "15m" => Ok(Self::Interval15m),
            "30m" => Ok(Self::Interval30m),
            "1h" => Ok(Self::Interval1h),
            "2h" => Ok(Self::Interval2h),
            "4h" => Ok(Self::Interval4h),
            "6h" => Ok(Self::Interval6h),
            "8h" => Ok(Self::Interval8h),
            "12h" => Ok(Self::Interval12h),
            "1d" => Ok(Self::Interval1d),
            "3d" => Ok(Self::Interval3d),
            "1w" => Ok(Self::Interval1w),
            "1M" => Ok(Self::Interval1M),
            other => Err(format!("invalid MarkPriceKlineCandlestickDataIntervalEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpenInterestStatisticsPeriodEnum {
    #[serde(rename = "5m")]
    Period5m,
    #[serde(rename = "15m")]
    Period15m,
    #[serde(rename = "30m")]
    Period30m,
    #[serde(rename = "1h")]
    Period1h,
    #[serde(rename = "2h")]
    Period2h,
    #[serde(rename = "4h")]
    Period4h,
    #[serde(rename = "6h")]
    Period6h,
    #[serde(rename = "12h")]
    Period12h,
    #[serde(rename = "1d")]
    Period1d,
}

impl OpenInterestStatisticsPeriodEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Period5m => "5m",
            Self::Period15m => "15m",
            Self::Period30m => "30m",
            Self::Period1h => "1h",
            Self::Period2h => "2h",
            Self::Period4h => "4h",
            Self::Period6h => "6h",
            Self::Period12h => "12h",
            Self::Period1d => "1d",
        }
    }
}

impl std::str::FromStr for OpenInterestStatisticsPeriodEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "5m" => Ok(Self::Period5m),
            "15m" => Ok(Self::Period15m),
            "30m" => Ok(Self::Period30m),
            "1h" => Ok(Self::Period1h),
            "2h" => Ok(Self::Period2h),
            "4h" => Ok(Self::Period4h),
            "6h" => Ok(Self::Period6h),
            "12h" => Ok(Self::Period12h),
            "1d" => Ok(Self::Period1d),
            other => Err(format!("invalid OpenInterestStatisticsPeriodEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PremiumIndexKlineDataIntervalEnum {
    #[serde(rename = "1m")]
    Interval1m,
    #[serde(rename = "3m")]
    Interval3m,
    #[serde(rename = "5m")]
    Interval5m,
    #[serde(rename = "15m")]
    Interval15m,
    #[serde(rename = "30m")]
    Interval30m,
    #[serde(rename = "1h")]
    Interval1h,
    #[serde(rename = "2h")]
    Interval2h,
    #[serde(rename = "4h")]
    Interval4h,
    #[serde(rename = "6h")]
    Interval6h,
    #[serde(rename = "8h")]
    Interval8h,
    #[serde(rename = "12h")]
    Interval12h,
    #[serde(rename = "1d")]
    Interval1d,
    #[serde(rename = "3d")]
    Interval3d,
    #[serde(rename = "1w")]
    Interval1w,
    #[serde(rename = "1M")]
    Interval1M,
}

impl PremiumIndexKlineDataIntervalEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Interval1m => "1m",
            Self::Interval3m => "3m",
            Self::Interval5m => "5m",
            Self::Interval15m => "15m",
            Self::Interval30m => "30m",
            Self::Interval1h => "1h",
            Self::Interval2h => "2h",
            Self::Interval4h => "4h",
            Self::Interval6h => "6h",
            Self::Interval8h => "8h",
            Self::Interval12h => "12h",
            Self::Interval1d => "1d",
            Self::Interval3d => "3d",
            Self::Interval1w => "1w",
            Self::Interval1M => "1M",
        }
    }
}

impl std::str::FromStr for PremiumIndexKlineDataIntervalEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1m" => Ok(Self::Interval1m),
            "3m" => Ok(Self::Interval3m),
            "5m" => Ok(Self::Interval5m),
            "15m" => Ok(Self::Interval15m),
            "30m" => Ok(Self::Interval30m),
            "1h" => Ok(Self::Interval1h),
            "2h" => Ok(Self::Interval2h),
            "4h" => Ok(Self::Interval4h),
            "6h" => Ok(Self::Interval6h),
            "8h" => Ok(Self::Interval8h),
            "12h" => Ok(Self::Interval12h),
            "1d" => Ok(Self::Interval1d),
            "3d" => Ok(Self::Interval3d),
            "1w" => Ok(Self::Interval1w),
            "1M" => Ok(Self::Interval1M),
            other => Err(format!("invalid PremiumIndexKlineDataIntervalEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TakerBuySellVolumePeriodEnum {
    #[serde(rename = "5m")]
    Period5m,
    #[serde(rename = "15m")]
    Period15m,
    #[serde(rename = "30m")]
    Period30m,
    #[serde(rename = "1h")]
    Period1h,
    #[serde(rename = "2h")]
    Period2h,
    #[serde(rename = "4h")]
    Period4h,
    #[serde(rename = "6h")]
    Period6h,
    #[serde(rename = "12h")]
    Period12h,
    #[serde(rename = "1d")]
    Period1d,
}

impl TakerBuySellVolumePeriodEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Period5m => "5m",
            Self::Period15m => "15m",
            Self::Period30m => "30m",
            Self::Period1h => "1h",
            Self::Period2h => "2h",
            Self::Period4h => "4h",
            Self::Period6h => "6h",
            Self::Period12h => "12h",
            Self::Period1d => "1d",
        }
    }
}

impl std::str::FromStr for TakerBuySellVolumePeriodEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "5m" => Ok(Self::Period5m),
            "15m" => Ok(Self::Period15m),
            "30m" => Ok(Self::Period30m),
            "1h" => Ok(Self::Period1h),
            "2h" => Ok(Self::Period2h),
            "4h" => Ok(Self::Period4h),
            "6h" => Ok(Self::Period6h),
            "12h" => Ok(Self::Period12h),
            "1d" => Ok(Self::Period1d),
            other => Err(format!("invalid TakerBuySellVolumePeriodEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopTraderLongShortRatioAccountsPeriodEnum {
    #[serde(rename = "5m")]
    Period5m,
    #[serde(rename = "15m")]
    Period15m,
    #[serde(rename = "30m")]
    Period30m,
    #[serde(rename = "1h")]
    Period1h,
    #[serde(rename = "2h")]
    Period2h,
    #[serde(rename = "4h")]
    Period4h,
    #[serde(rename = "6h")]
    Period6h,
    #[serde(rename = "12h")]
    Period12h,
    #[serde(rename = "1d")]
    Period1d,
}

impl TopTraderLongShortRatioAccountsPeriodEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Period5m => "5m",
            Self::Period15m => "15m",
            Self::Period30m => "30m",
            Self::Period1h => "1h",
            Self::Period2h => "2h",
            Self::Period4h => "4h",
            Self::Period6h => "6h",
            Self::Period12h => "12h",
            Self::Period1d => "1d",
        }
    }
}

impl std::str::FromStr for TopTraderLongShortRatioAccountsPeriodEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "5m" => Ok(Self::Period5m),
            "15m" => Ok(Self::Period15m),
            "30m" => Ok(Self::Period30m),
            "1h" => Ok(Self::Period1h),
            "2h" => Ok(Self::Period2h),
            "4h" => Ok(Self::Period4h),
            "6h" => Ok(Self::Period6h),
            "12h" => Ok(Self::Period12h),
            "1d" => Ok(Self::Period1d),
            other => Err(format!("invalid TopTraderLongShortRatioAccountsPeriodEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopTraderLongShortRatioPositionsPeriodEnum {
    #[serde(rename = "5m")]
    Period5m,
    #[serde(rename = "15m")]
    Period15m,
    #[serde(rename = "30m")]
    Period30m,
    #[serde(rename = "1h")]
    Period1h,
    #[serde(rename = "2h")]
    Period2h,
    #[serde(rename = "4h")]
    Period4h,
    #[serde(rename = "6h")]
    Period6h,
    #[serde(rename = "12h")]
    Period12h,
    #[serde(rename = "1d")]
    Period1d,
}

impl TopTraderLongShortRatioPositionsPeriodEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Period5m => "5m",
            Self::Period15m => "15m",
            Self::Period30m => "30m",
            Self::Period1h => "1h",
            Self::Period2h => "2h",
            Self::Period4h => "4h",
            Self::Period6h => "6h",
            Self::Period12h => "12h",
            Self::Period1d => "1d",
        }
    }
}

impl std::str::FromStr for TopTraderLongShortRatioPositionsPeriodEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "5m" => Ok(Self::Period5m),
            "15m" => Ok(Self::Period15m),
            "30m" => Ok(Self::Period30m),
            "1h" => Ok(Self::Period1h),
            "2h" => Ok(Self::Period2h),
            "4h" => Ok(Self::Period4h),
            "6h" => Ok(Self::Period6h),
            "12h" => Ok(Self::Period12h),
            "1d" => Ok(Self::Period1d),
            other => Err(format!("invalid TopTraderLongShortRatioPositionsPeriodEnum: {}", other).into()),
        }
    }
}

/// Request parameters for the [`basis`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`basis`](#method.basis).
#[derive(Clone, Debug, TypedBuilder)]
pub struct BasisParams {
    ///
    /// The `pair` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub pair: String,
    ///
    /// The `contract_type` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub contract_type: BasisContractTypeEnum,
    /// "5m","15m","30m","1h","2h","4h","6h","12h","1d"
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub period: BasisPeriodEnum,
    /// Default 30,Max 500
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub limit: i64,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
}

/// Request parameters for the [`composite_index_symbol_information`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`composite_index_symbol_information`](#method.composite_index_symbol_information).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct CompositeIndexSymbolInformationParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
}

/// Request parameters for the [`compressed_aggregate_trades_list`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`compressed_aggregate_trades_list`](#method.compressed_aggregate_trades_list).
#[derive(Clone, Debug, TypedBuilder)]
pub struct CompressedAggregateTradesListParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// ID to get aggregate trades from INCLUSIVE.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub from_id: Option<i64>,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
}

/// Request parameters for the [`continuous_contract_kline_candlestick_data`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`continuous_contract_kline_candlestick_data`](#method.continuous_contract_kline_candlestick_data).
#[derive(Clone, Debug, TypedBuilder)]
pub struct ContinuousContractKlineCandlestickDataParams {
    ///
    /// The `pair` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub pair: String,
    ///
    /// The `contract_type` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub contract_type: ContinuousContractKlineCandlestickDataContractTypeEnum,
    ///
    /// The `interval` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub interval: ContinuousContractKlineCandlestickDataIntervalEnum,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
}

/// Request parameters for the [`get_funding_rate_history`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_funding_rate_history`](#method.get_funding_rate_history).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct GetFundingRateHistoryParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
}

/// Request parameters for the [`index_price_kline_candlestick_data`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`index_price_kline_candlestick_data`](#method.index_price_kline_candlestick_data).
#[derive(Clone, Debug, TypedBuilder)]
pub struct IndexPriceKlineCandlestickDataParams {
    ///
    /// The `pair` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub pair: String,
    ///
    /// The `interval` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub interval: IndexPriceKlineCandlestickDataIntervalEnum,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
}

/// Request parameters for the [`kline_candlestick_data`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`kline_candlestick_data`](#method.kline_candlestick_data).
#[derive(Clone, Debug, TypedBuilder)]
pub struct KlineCandlestickDataParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    ///
    /// The `interval` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub interval: KlineCandlestickDataIntervalEnum,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
}

/// Request parameters for the [`long_short_ratio`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`long_short_ratio`](#method.long_short_ratio).
#[derive(Clone, Debug, TypedBuilder)]
pub struct LongShortRatioParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// "5m","15m","30m","1h","2h","4h","6h","12h","1d"
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub period: LongShortRatioPeriodEnum,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
}

/// Request parameters for the [`mark_price`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`mark_price`](#method.mark_price).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct MarkPriceParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
}

/// Request parameters for the [`mark_price_kline_candlestick_data`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`mark_price_kline_candlestick_data`](#method.mark_price_kline_candlestick_data).
#[derive(Clone, Debug, TypedBuilder)]
pub struct MarkPriceKlineCandlestickDataParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    ///
    /// The `interval` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub interval: MarkPriceKlineCandlestickDataIntervalEnum,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
}

/// Request parameters for the [`multi_assets_mode_asset_index`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`multi_assets_mode_asset_index`](#method.multi_assets_mode_asset_index).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct MultiAssetsModeAssetIndexParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
}

/// Request parameters for the [`old_trades_lookup`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`old_trades_lookup`](#method.old_trades_lookup).
#[derive(Clone, Debug, TypedBuilder)]
pub struct OldTradesLookupParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
    /// ID to get aggregate trades from INCLUSIVE.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub from_id: Option<i64>,
}

/// Request parameters for the [`open_interest`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`open_interest`](#method.open_interest).
#[derive(Clone, Debug, TypedBuilder)]
pub struct OpenInterestParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
}

/// Request parameters for the [`open_interest_statistics`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`open_interest_statistics`](#method.open_interest_statistics).
#[derive(Clone, Debug, TypedBuilder)]
pub struct OpenInterestStatisticsParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// "5m","15m","30m","1h","2h","4h","6h","12h","1d"
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub period: OpenInterestStatisticsPeriodEnum,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
}

/// Request parameters for the [`order_book`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`order_book`](#method.order_book).
#[derive(Clone, Debug, TypedBuilder)]
pub struct OrderBookParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
}

/// Request parameters for the [`premium_index_kline_data`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`premium_index_kline_data`](#method.premium_index_kline_data).
#[derive(Clone, Debug, TypedBuilder)]
pub struct PremiumIndexKlineDataParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    ///
    /// The `interval` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub interval: PremiumIndexKlineDataIntervalEnum,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
}

/// Request parameters for the [`quarterly_contract_settlement_price`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`quarterly_contract_settlement_price`](#method.quarterly_contract_settlement_price).
#[derive(Clone, Debug, TypedBuilder)]
pub struct QuarterlyContractSettlementPriceParams {
    ///
    /// The `pair` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub pair: String,
}

/// Request parameters for the [`query_index_price_constituents`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`query_index_price_constituents`](#method.query_index_price_constituents).
#[derive(Clone, Debug, TypedBuilder)]
pub struct QueryIndexPriceConstituentsParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
}

/// Request parameters for the [`query_insurance_fund_balance_snapshot`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`query_insurance_fund_balance_snapshot`](#method.query_insurance_fund_balance_snapshot).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct QueryInsuranceFundBalanceSnapshotParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
}

/// Request parameters for the [`recent_trades_list`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`recent_trades_list`](#method.recent_trades_list).
#[derive(Clone, Debug, TypedBuilder)]
pub struct RecentTradesListParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
}

/// Request parameters for the [`symbol_order_book_ticker`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`symbol_order_book_ticker`](#method.symbol_order_book_ticker).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct SymbolOrderBookTickerParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
}

/// Request parameters for the [`symbol_price_ticker`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`symbol_price_ticker`](#method.symbol_price_ticker).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct SymbolPriceTickerParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
}

/// Request parameters for the [`symbol_price_ticker_v2`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`symbol_price_ticker_v2`](#method.symbol_price_ticker_v2).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct SymbolPriceTickerV2Params {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
}

/// Request parameters for the [`taker_buy_sell_volume`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`taker_buy_sell_volume`](#method.taker_buy_sell_volume).
#[derive(Clone, Debug, TypedBuilder)]
pub struct TakerBuySellVolumeParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// "5m","15m","30m","1h","2h","4h","6h","12h","1d"
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub period: TakerBuySellVolumePeriodEnum,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
}

/// Request parameters for the [`ticker24hr_price_change_statistics`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`ticker24hr_price_change_statistics`](#method.ticker24hr_price_change_statistics).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct Ticker24hrPriceChangeStatisticsParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
}

/// Request parameters for the [`top_trader_long_short_ratio_accounts`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`top_trader_long_short_ratio_accounts`](#method.top_trader_long_short_ratio_accounts).
#[derive(Clone, Debug, TypedBuilder)]
pub struct TopTraderLongShortRatioAccountsParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// "5m","15m","30m","1h","2h","4h","6h","12h","1d"
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub period: TopTraderLongShortRatioAccountsPeriodEnum,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
}

/// Request parameters for the [`top_trader_long_short_ratio_positions`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`top_trader_long_short_ratio_positions`](#method.top_trader_long_short_ratio_positions).
#[derive(Clone, Debug, TypedBuilder)]
pub struct TopTraderLongShortRatioPositionsParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// "5m","15m","30m","1h","2h","4h","6h","12h","1d"
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub period: TopTraderLongShortRatioPositionsPeriodEnum,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
}

#[async_trait]
impl MarketDataApi for MarketDataApiClient {
    async fn basis(&self, params: BasisParams) -> anyhow::Result<RestApiResponse<Vec<models::BasisResponseInner>>> {
        let BasisParams {
            pair,
            contract_type,
            period,
            limit,
            start_time,
            end_time,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("pair".to_string(), json!(pair));

        query_params.insert("contractType".to_string(), json!(contract_type));

        query_params.insert("period".to_string(), json!(period));

        query_params.insert("limit".to_string(), json!(limit));

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        send_request::<Vec<models::BasisResponseInner>>(
            &self.configuration,
            "/futures/data/basis",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn check_server_time(&self) -> anyhow::Result<RestApiResponse<models::CheckServerTimeResponse>> {
        let query_params = BTreeMap::new();

        send_request::<models::CheckServerTimeResponse>(
            &self.configuration,
            "/fapi/v1/time",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn composite_index_symbol_information(
        &self,
        params: CompositeIndexSymbolInformationParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::CompositeIndexSymbolInformationResponseInner>>> {
        let CompositeIndexSymbolInformationParams { symbol } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        send_request::<Vec<models::CompositeIndexSymbolInformationResponseInner>>(
            &self.configuration,
            "/fapi/v1/indexInfo",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn compressed_aggregate_trades_list(
        &self,
        params: CompressedAggregateTradesListParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::CompressedAggregateTradesListResponseInner>>> {
        let CompressedAggregateTradesListParams {
            symbol,
            from_id,
            start_time,
            end_time,
            limit,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = from_id {
            query_params.insert("fromId".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        send_request::<Vec<models::CompressedAggregateTradesListResponseInner>>(
            &self.configuration,
            "/fapi/v1/aggTrades",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn continuous_contract_kline_candlestick_data(
        &self,
        params: ContinuousContractKlineCandlestickDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::ContinuousContractKlineCandlestickDataResponseItemInner>>>>
    {
        let ContinuousContractKlineCandlestickDataParams {
            pair,
            contract_type,
            interval,
            start_time,
            end_time,
            limit,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("pair".to_string(), json!(pair));

        query_params.insert("contractType".to_string(), json!(contract_type));

        query_params.insert("interval".to_string(), json!(interval));

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        send_request::<Vec<Vec<models::ContinuousContractKlineCandlestickDataResponseItemInner>>>(
            &self.configuration,
            "/fapi/v1/continuousKlines",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn exchange_information(&self) -> anyhow::Result<RestApiResponse<models::ExchangeInformationResponse>> {
        let query_params = BTreeMap::new();

        send_request::<models::ExchangeInformationResponse>(
            &self.configuration,
            "/fapi/v1/exchangeInfo",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn get_funding_rate_history(
        &self,
        params: GetFundingRateHistoryParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetFundingRateHistoryResponseInner>>> {
        let GetFundingRateHistoryParams {
            symbol,
            start_time,
            end_time,
            limit,
        } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        send_request::<Vec<models::GetFundingRateHistoryResponseInner>>(
            &self.configuration,
            "/fapi/v1/fundingRate",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn get_funding_rate_info(
        &self,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetFundingRateInfoResponseInner>>> {
        let query_params = BTreeMap::new();

        send_request::<Vec<models::GetFundingRateInfoResponseInner>>(
            &self.configuration,
            "/fapi/v1/fundingInfo",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn index_price_kline_candlestick_data(
        &self,
        params: IndexPriceKlineCandlestickDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::IndexPriceKlineCandlestickDataResponseItemInner>>>> {
        let IndexPriceKlineCandlestickDataParams {
            pair,
            interval,
            start_time,
            end_time,
            limit,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("pair".to_string(), json!(pair));

        query_params.insert("interval".to_string(), json!(interval));

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        send_request::<Vec<Vec<models::IndexPriceKlineCandlestickDataResponseItemInner>>>(
            &self.configuration,
            "/fapi/v1/indexPriceKlines",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn kline_candlestick_data(
        &self,
        params: KlineCandlestickDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::KlineCandlestickDataResponseItemInner>>>> {
        let KlineCandlestickDataParams {
            symbol,
            interval,
            start_time,
            end_time,
            limit,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("interval".to_string(), json!(interval));

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        send_request::<Vec<Vec<models::KlineCandlestickDataResponseItemInner>>>(
            &self.configuration,
            "/fapi/v1/klines",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn long_short_ratio(
        &self,
        params: LongShortRatioParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::LongShortRatioResponseInner>>> {
        let LongShortRatioParams {
            symbol,
            period,
            limit,
            start_time,
            end_time,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("period".to_string(), json!(period));

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        send_request::<Vec<models::LongShortRatioResponseInner>>(
            &self.configuration,
            "/futures/data/globalLongShortAccountRatio",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn mark_price(&self, params: MarkPriceParams) -> anyhow::Result<RestApiResponse<models::MarkPriceResponse>> {
        let MarkPriceParams { symbol } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        send_request::<models::MarkPriceResponse>(
            &self.configuration,
            "/fapi/v1/premiumIndex",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn mark_price_kline_candlestick_data(
        &self,
        params: MarkPriceKlineCandlestickDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::MarkPriceKlineCandlestickDataResponseItemInner>>>> {
        let MarkPriceKlineCandlestickDataParams {
            symbol,
            interval,
            start_time,
            end_time,
            limit,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("interval".to_string(), json!(interval));

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        send_request::<Vec<Vec<models::MarkPriceKlineCandlestickDataResponseItemInner>>>(
            &self.configuration,
            "/fapi/v1/markPriceKlines",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn multi_assets_mode_asset_index(
        &self,
        params: MultiAssetsModeAssetIndexParams,
    ) -> anyhow::Result<RestApiResponse<models::MultiAssetsModeAssetIndexResponse>> {
        let MultiAssetsModeAssetIndexParams { symbol } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        send_request::<models::MultiAssetsModeAssetIndexResponse>(
            &self.configuration,
            "/fapi/v1/assetIndex",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn old_trades_lookup(
        &self,
        params: OldTradesLookupParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::OldTradesLookupResponseInner>>> {
        let OldTradesLookupParams {
            symbol,
            limit,
            from_id,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        if let Some(rw) = from_id {
            query_params.insert("fromId".to_string(), json!(rw));
        }

        send_request::<Vec<models::OldTradesLookupResponseInner>>(
            &self.configuration,
            "/fapi/v1/historicalTrades",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn open_interest(
        &self,
        params: OpenInterestParams,
    ) -> anyhow::Result<RestApiResponse<models::OpenInterestResponse>> {
        let OpenInterestParams { symbol } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        send_request::<models::OpenInterestResponse>(
            &self.configuration,
            "/fapi/v1/openInterest",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn open_interest_statistics(
        &self,
        params: OpenInterestStatisticsParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::OpenInterestStatisticsResponseInner>>> {
        let OpenInterestStatisticsParams {
            symbol,
            period,
            limit,
            start_time,
            end_time,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("period".to_string(), json!(period));

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        send_request::<Vec<models::OpenInterestStatisticsResponseInner>>(
            &self.configuration,
            "/futures/data/openInterestHist",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn order_book(&self, params: OrderBookParams) -> anyhow::Result<RestApiResponse<models::OrderBookResponse>> {
        let OrderBookParams { symbol, limit } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        send_request::<models::OrderBookResponse>(
            &self.configuration,
            "/fapi/v1/depth",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn premium_index_kline_data(
        &self,
        params: PremiumIndexKlineDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::PremiumIndexKlineDataResponseItemInner>>>> {
        let PremiumIndexKlineDataParams {
            symbol,
            interval,
            start_time,
            end_time,
            limit,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("interval".to_string(), json!(interval));

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        send_request::<Vec<Vec<models::PremiumIndexKlineDataResponseItemInner>>>(
            &self.configuration,
            "/fapi/v1/premiumIndexKlines",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn quarterly_contract_settlement_price(
        &self,
        params: QuarterlyContractSettlementPriceParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::QuarterlyContractSettlementPriceResponseInner>>> {
        let QuarterlyContractSettlementPriceParams { pair } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("pair".to_string(), json!(pair));

        send_request::<Vec<models::QuarterlyContractSettlementPriceResponseInner>>(
            &self.configuration,
            "/futures/data/delivery-price",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn query_index_price_constituents(
        &self,
        params: QueryIndexPriceConstituentsParams,
    ) -> anyhow::Result<RestApiResponse<models::QueryIndexPriceConstituentsResponse>> {
        let QueryIndexPriceConstituentsParams { symbol } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        send_request::<models::QueryIndexPriceConstituentsResponse>(
            &self.configuration,
            "/fapi/v1/constituents",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn query_insurance_fund_balance_snapshot(
        &self,
        params: QueryInsuranceFundBalanceSnapshotParams,
    ) -> anyhow::Result<RestApiResponse<models::QueryInsuranceFundBalanceSnapshotResponse>> {
        let QueryInsuranceFundBalanceSnapshotParams { symbol } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        send_request::<models::QueryInsuranceFundBalanceSnapshotResponse>(
            &self.configuration,
            "/fapi/v1/insuranceBalance",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn recent_trades_list(
        &self,
        params: RecentTradesListParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::RecentTradesListResponseInner>>> {
        let RecentTradesListParams { symbol, limit } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        send_request::<Vec<models::RecentTradesListResponseInner>>(
            &self.configuration,
            "/fapi/v1/trades",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn symbol_order_book_ticker(
        &self,
        params: SymbolOrderBookTickerParams,
    ) -> anyhow::Result<RestApiResponse<models::SymbolOrderBookTickerResponse>> {
        let SymbolOrderBookTickerParams { symbol } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        send_request::<models::SymbolOrderBookTickerResponse>(
            &self.configuration,
            "/fapi/v1/ticker/bookTicker",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn symbol_price_ticker(
        &self,
        params: SymbolPriceTickerParams,
    ) -> anyhow::Result<RestApiResponse<models::SymbolPriceTickerResponse>> {
        let SymbolPriceTickerParams { symbol } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        send_request::<models::SymbolPriceTickerResponse>(
            &self.configuration,
            "/fapi/v1/ticker/price",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn symbol_price_ticker_v2(
        &self,
        params: SymbolPriceTickerV2Params,
    ) -> anyhow::Result<RestApiResponse<models::SymbolPriceTickerV2Response>> {
        let SymbolPriceTickerV2Params { symbol } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        send_request::<models::SymbolPriceTickerV2Response>(
            &self.configuration,
            "/fapi/v2/ticker/price",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn taker_buy_sell_volume(
        &self,
        params: TakerBuySellVolumeParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::TakerBuySellVolumeResponseInner>>> {
        let TakerBuySellVolumeParams {
            symbol,
            period,
            limit,
            start_time,
            end_time,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("period".to_string(), json!(period));

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        send_request::<Vec<models::TakerBuySellVolumeResponseInner>>(
            &self.configuration,
            "/futures/data/takerlongshortRatio",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn test_connectivity(&self) -> anyhow::Result<RestApiResponse<Value>> {
        let query_params = BTreeMap::new();

        send_request::<Value>(
            &self.configuration,
            "/fapi/v1/ping",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn ticker24hr_price_change_statistics(
        &self,
        params: Ticker24hrPriceChangeStatisticsParams,
    ) -> anyhow::Result<RestApiResponse<models::Ticker24hrPriceChangeStatisticsResponse>> {
        let Ticker24hrPriceChangeStatisticsParams { symbol } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        send_request::<models::Ticker24hrPriceChangeStatisticsResponse>(
            &self.configuration,
            "/fapi/v1/ticker/24hr",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn top_trader_long_short_ratio_accounts(
        &self,
        params: TopTraderLongShortRatioAccountsParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::TopTraderLongShortRatioAccountsResponseInner>>> {
        let TopTraderLongShortRatioAccountsParams {
            symbol,
            period,
            limit,
            start_time,
            end_time,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("period".to_string(), json!(period));

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        send_request::<Vec<models::TopTraderLongShortRatioAccountsResponseInner>>(
            &self.configuration,
            "/futures/data/topLongShortAccountRatio",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn top_trader_long_short_ratio_positions(
        &self,
        params: TopTraderLongShortRatioPositionsParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::TopTraderLongShortRatioPositionsResponseInner>>> {
        let TopTraderLongShortRatioPositionsParams {
            symbol,
            period,
            limit,
            start_time,
            end_time,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("period".to_string(), json!(period));

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        send_request::<Vec<models::TopTraderLongShortRatioPositionsResponseInner>>(
            &self.configuration,
            "/futures/data/topLongShortPositionRatio",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }
}

// #[cfg(all(test, feature = "derivatives_trading_usds_futures"))]
// mod tests {
//     use super::*;
//     use crate::TOKIO_SHARED_RT;
//     use crate::{errors::ConnectorError, models::DataFuture, models::RestApiRateLimit};
//     use async_trait::async_trait;
//     use std::collections::HashMap;

//     struct DummyRestApiResponse<T> {
//         inner: Box<dyn FnOnce() -> DataFuture<Result<T, ConnectorError>> + Send + Sync>,
//         status: u16,
//         headers: HashMap<String, String>,
//         rate_limits: Option<Vec<RestApiRateLimit>>,
//     }

//     impl<T> From<DummyRestApiResponse<T>> for RestApiResponse<T> {
//         fn from(dummy: DummyRestApiResponse<T>) -> Self {
//             Self {
//                 data_fn: dummy.inner,
//                 status: dummy.status,
//                 headers: dummy.headers,
//                 rate_limits: dummy.rate_limits,
//             }
//         }
//     }

//     struct MockMarketDataApiClient {
//         force_error: bool,
//     }

//     #[async_trait]
//     impl MarketDataApi for MockMarketDataApiClient {
//         async fn basis(
//             &self,
//             _params: BasisParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::BasisResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"indexPrice":"34400.15945055","contractType":"PERPETUAL","basisRate":"0.0004","futuresPrice":"34414.10","annualizedBasisRate":"","basis":"13.94054945","pair":"BTCUSDT","timestamp":1698742800000}]"#).unwrap();
//             let dummy_response: Vec<models::BasisResponseInner> =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::BasisResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn check_server_time(&self) -> anyhow::Result<RestApiResponse<models::CheckServerTimeResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"serverTime":1499827319559}"#).unwrap();
//             let dummy_response: models::CheckServerTimeResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::CheckServerTimeResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn composite_index_symbol_information(
//             &self,
//             _params: CompositeIndexSymbolInformationParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::CompositeIndexSymbolInformationResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"DEFIUSDT","time":1589437530011,"component":"baseAsset","baseAssetList":[{"baseAsset":"BAL","quoteAsset":"USDT","weightInQuantity":"1.04406228","weightInPercentage":"0.02783900"},{"baseAsset":"BAND","quoteAsset":"USDT","weightInQuantity":"3.53782729","weightInPercentage":"0.03935200"}]}]"#).unwrap();
//             let dummy_response: Vec<models::CompositeIndexSymbolInformationResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::CompositeIndexSymbolInformationResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn compressed_aggregate_trades_list(
//             &self,
//             _params: CompressedAggregateTradesListParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::CompressedAggregateTradesListResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(
//                 r#"[{"a":26129,"p":"0.01633102","q":"4.70443515","f":27781,"l":27781,"T":1498793709153,"m":true}]"#,
//             )
//             .unwrap();
//             let dummy_response: Vec<models::CompressedAggregateTradesListResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::CompressedAggregateTradesListResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn continuous_contract_kline_candlestick_data(
//             &self,
//             _params: ContinuousContractKlineCandlestickDataParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::ContinuousContractKlineCandlestickDataResponseItemInner>>>>
//         {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[[1607444700000,"18879.99","18900.00","18878.98","18896.13","492.363",1607444759999,"9302145.66080",1874,"385.983","7292402.33267","0"]]"#).unwrap();
//             let dummy_response: Vec<Vec<models::ContinuousContractKlineCandlestickDataResponseItemInner>> =
//                 serde_json::from_value(resp_json.clone()).expect(
//                     "should parse into Vec<Vec<models::ContinuousContractKlineCandlestickDataResponseItemInner>>",
//                 );

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn exchange_information(&self) -> anyhow::Result<RestApiResponse<models::ExchangeInformationResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"exchangeFilters":[],"rateLimits":[{"interval":"MINUTE","intervalNum":1,"limit":2400,"rateLimitType":"REQUEST_WEIGHT"},{"interval":"MINUTE","intervalNum":1,"limit":1200,"rateLimitType":"ORDERS"}],"serverTime":1565613908500,"assets":[{"asset":"BTC","marginAvailable":true,"autoAssetExchange":"-0.10"},{"asset":"USDT","marginAvailable":true,"autoAssetExchange":"0"},{"asset":"BNB","marginAvailable":false,"autoAssetExchange":null}],"symbols":[{"symbol":"BLZUSDT","pair":"BLZUSDT","contractType":"PERPETUAL","deliveryDate":4133404800000,"onboardDate":1598252400000,"status":"TRADING","maintMarginPercent":"2.5000","requiredMarginPercent":"5.0000","baseAsset":"BLZ","quoteAsset":"USDT","marginAsset":"USDT","pricePrecision":5,"quantityPrecision":0,"baseAssetPrecision":8,"quotePrecision":8,"underlyingType":"COIN","underlyingSubType":["STORAGE"],"settlePlan":0,"triggerProtect":"0.15","filters":[{"filterType":"PRICE_FILTER","maxPrice":"300","minPrice":"0.0001","tickSize":"0.0001"},{"filterType":"LOT_SIZE","maxQty":"10000000","minQty":"1","stepSize":"1"},{"filterType":"MARKET_LOT_SIZE","maxQty":"590119","minQty":"1","stepSize":"1"},{"filterType":"MAX_NUM_ORDERS","limit":200},{"filterType":"MAX_NUM_ALGO_ORDERS","limit":10},{"filterType":"MIN_NOTIONAL","notional":"5.0"},{"filterType":"PERCENT_PRICE","multiplierUp":"1.1500","multiplierDown":"0.8500","multiplierDecimal":"4"}],"OrderType":["LIMIT","MARKET","STOP","STOP_MARKET","TAKE_PROFIT","TAKE_PROFIT_MARKET","TRAILING_STOP_MARKET"],"timeInForce":["GTC","IOC","FOK","GTX"],"liquidationFee":"0.010000","marketTakeBound":"0.30"}],"timezone":"UTC"}"#).unwrap();
//             let dummy_response: models::ExchangeInformationResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::ExchangeInformationResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_funding_rate_history(
//             &self,
//             _params: GetFundingRateHistoryParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::GetFundingRateHistoryResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","fundingRate":"-0.03750000","fundingTime":1570608000000,"markPrice":"34287.54619963"},{"symbol":"BTCUSDT","fundingRate":"0.00010000","fundingTime":1570636800000,"markPrice":"34287.54619963"}]"#).unwrap();
//             let dummy_response: Vec<models::GetFundingRateHistoryResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::GetFundingRateHistoryResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_funding_rate_info(
//             &self,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::GetFundingRateInfoResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BLZUSDT","adjustedFundingRateCap":"0.02500000","adjustedFundingRateFloor":"-0.02500000","fundingIntervalHours":8,"disclaimer":false}]"#).unwrap();
//             let dummy_response: Vec<models::GetFundingRateInfoResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::GetFundingRateInfoResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn index_price_kline_candlestick_data(
//             &self,
//             _params: IndexPriceKlineCandlestickDataParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::IndexPriceKlineCandlestickDataResponseItemInner>>>>
//         {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[[1591256400000,"9653.69440000","9653.69640000","9651.38600000","9651.55200000","0",1591256459999,"0",60,"0","0","0"]]"#).unwrap();
//             let dummy_response: Vec<Vec<models::IndexPriceKlineCandlestickDataResponseItemInner>> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<Vec<models::IndexPriceKlineCandlestickDataResponseItemInner>>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn kline_candlestick_data(
//             &self,
//             _params: KlineCandlestickDataParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::KlineCandlestickDataResponseItemInner>>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[[1499040000000,"0.01634790","0.80000000","0.01575800","0.01577100","148976.11427815",1499644799999,"2434.19055334",308,"1756.87402397","28.46694368","17928899.62484339"]]"#).unwrap();
//             let dummy_response: Vec<Vec<models::KlineCandlestickDataResponseItemInner>> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<Vec<models::KlineCandlestickDataResponseItemInner>>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn long_short_ratio(
//             &self,
//             _params: LongShortRatioParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::LongShortRatioResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","longShortRatio":"0.1960","longAccount":"0.6622","shortAccount":"0.3378","timestamp":"1583139600000"},{"symbol":"BTCUSDT","longShortRatio":"1.9559","longAccount":"0.6617","shortAccount":"0.3382","timestamp":"1583139900000"}]"#).unwrap();
//             let dummy_response: Vec<models::LongShortRatioResponseInner> = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into Vec<models::LongShortRatioResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn mark_price(
//             &self,
//             _params: MarkPriceParams,
//         ) -> anyhow::Result<RestApiResponse<models::MarkPriceResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","markPrice":"11793.63104562","indexPrice":"11781.80495970","estimatedSettlePrice":"11781.16138815","lastFundingRate":"0.00038246","interestRate":"0.00010000","nextFundingTime":1597392000000,"time":1597370495002}"#).unwrap();
//             let dummy_response: models::MarkPriceResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::MarkPriceResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn mark_price_kline_candlestick_data(
//             &self,
//             _params: MarkPriceKlineCandlestickDataParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::MarkPriceKlineCandlestickDataResponseItemInner>>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[[1591256460000,"9653.29201333","9654.56401333","9653.07367333","9653.07367333","0",1591256519999,"0",60,"0","0","0"]]"#).unwrap();
//             let dummy_response: Vec<Vec<models::MarkPriceKlineCandlestickDataResponseItemInner>> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<Vec<models::MarkPriceKlineCandlestickDataResponseItemInner>>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn multi_assets_mode_asset_index(
//             &self,
//             _params: MultiAssetsModeAssetIndexParams,
//         ) -> anyhow::Result<RestApiResponse<models::MultiAssetsModeAssetIndexResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"ADAUSD","time":1635740268004,"index":"1.92957370","bidBuffer":"0.10000000","askBuffer":"0.10000000","bidRate":"1.73661633","askRate":"2.12253107","autoExchangeBidBuffer":"0.05000000","autoExchangeAskBuffer":"0.05000000","autoExchangeBidRate":"1.83309501","autoExchangeAskRate":"2.02605238"}"#).unwrap();
//             let dummy_response: models::MultiAssetsModeAssetIndexResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::MultiAssetsModeAssetIndexResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn old_trades_lookup(
//             &self,
//             _params: OldTradesLookupParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::OldTradesLookupResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"id":28457,"price":"4.00000100","qty":"12.00000000","quoteQty":"8000.00","time":1499865549590,"isBuyerMaker":true}]"#).unwrap();
//             let dummy_response: Vec<models::OldTradesLookupResponseInner> = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into Vec<models::OldTradesLookupResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn open_interest(
//             &self,
//             _params: OpenInterestParams,
//         ) -> anyhow::Result<RestApiResponse<models::OpenInterestResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"openInterest":"10659.509","symbol":"BTCUSDT","time":1589437530011}"#)
//                     .unwrap();
//             let dummy_response: models::OpenInterestResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::OpenInterestResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn open_interest_statistics(
//             &self,
//             _params: OpenInterestStatisticsParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::OpenInterestStatisticsResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","sumOpenInterest":"20403.63700000","sumOpenInterestValue":"150570784.07809979","CMCCirculatingSupply":"165880.538","timestamp":"1583127900000"},{"symbol":"BTCUSDT","sumOpenInterest":"20401.36700000","sumOpenInterestValue":"149940752.14464448","CMCCirculatingSupply":"165900.14853","timestamp":"1583128200000"}]"#).unwrap();
//             let dummy_response: Vec<models::OpenInterestStatisticsResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::OpenInterestStatisticsResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn order_book(
//             &self,
//             _params: OrderBookParams,
//         ) -> anyhow::Result<RestApiResponse<models::OrderBookResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"lastUpdateId":1027024,"E":1589436922972,"T":1589436922959,"bids":[["4.00000000","431.00000000"]],"asks":[["4.00000200","12.00000000"]]}"#).unwrap();
//             let dummy_response: models::OrderBookResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::OrderBookResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn premium_index_kline_data(
//             &self,
//             _params: PremiumIndexKlineDataParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::PremiumIndexKlineDataResponseItemInner>>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[[1691603820000,"-0.00042931","-0.00023641","-0.00059406","-0.00043659","0",1691603879999,"0",12,"0","0","0"]]"#).unwrap();
//             let dummy_response: Vec<Vec<models::PremiumIndexKlineDataResponseItemInner>> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<Vec<models::PremiumIndexKlineDataResponseItemInner>>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn quarterly_contract_settlement_price(
//             &self,
//             _params: QuarterlyContractSettlementPriceParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::QuarterlyContractSettlementPriceResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"deliveryTime":1695945600000,"deliveryPrice":27103},{"deliveryTime":1688083200000,"deliveryPrice":30733.6},{"deliveryTime":1680220800000,"deliveryPrice":27814.2},{"deliveryTime":1648166400000,"deliveryPrice":44066.3}]"#).unwrap();
//             let dummy_response: Vec<models::QuarterlyContractSettlementPriceResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::QuarterlyContractSettlementPriceResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn query_index_price_constituents(
//             &self,
//             _params: QueryIndexPriceConstituentsParams,
//         ) -> anyhow::Result<RestApiResponse<models::QueryIndexPriceConstituentsResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","time":1745401553408,"constituents":[{"exchange":"binance","symbol":"BTCUSDT","price":"94057.03000000","weight":"0.51282051"},{"exchange":"coinbase","symbol":"BTC-USDT","price":"94140.58000000","weight":"0.15384615"},{"exchange":"gateio","symbol":"BTC_USDT","price":"94060.10000000","weight":"0.02564103"},{"exchange":"kucoin","symbol":"BTC-USDT","price":"94096.70000000","weight":"0.07692308"},{"exchange":"mxc","symbol":"BTCUSDT","price":"94057.02000000","weight":"0.07692308"},{"exchange":"bitget","symbol":"BTCUSDT","price":"94064.03000000","weight":"0.07692308"},{"exchange":"bybit","symbol":"BTCUSDT","price":"94067.90000000","weight":"0.07692308"}]}"#).unwrap();
//             let dummy_response: models::QueryIndexPriceConstituentsResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::QueryIndexPriceConstituentsResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn query_insurance_fund_balance_snapshot(
//             &self,
//             _params: QueryInsuranceFundBalanceSnapshotParams,
//         ) -> anyhow::Result<RestApiResponse<models::QueryInsuranceFundBalanceSnapshotResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"symbols":["BNBUSDT","BTCUSDT","BTCUSDT_250627","BTCUSDT_250926","ETHBTC","ETHUSDT","ETHUSDT_250627","ETHUSDT_250926"],"assets":[{"asset":"USDC","marginBalance":"299999998.6497832","updateTime":1745366402000},{"asset":"USDT","marginBalance":"793930579.315848","updateTime":1745366402000},{"asset":"BTC","marginBalance":"61.73143554","updateTime":1745366402000},{"asset":"BNFCR","marginBalance":"633223.99396922","updateTime":1745366402000}]}"#).unwrap();
//             let dummy_response: models::QueryInsuranceFundBalanceSnapshotResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::QueryInsuranceFundBalanceSnapshotResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn recent_trades_list(
//             &self,
//             _params: RecentTradesListParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::RecentTradesListResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"id":28457,"price":"4.00000100","qty":"12.00000000","quoteQty":"48.00","time":1499865549590,"isBuyerMaker":true}]"#).unwrap();
//             let dummy_response: Vec<models::RecentTradesListResponseInner> = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into Vec<models::RecentTradesListResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn symbol_order_book_ticker(
//             &self,
//             _params: SymbolOrderBookTickerParams,
//         ) -> anyhow::Result<RestApiResponse<models::SymbolOrderBookTickerResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","bidPrice":"4.00000000","bidQty":"431.00000000","askPrice":"4.00000200","askQty":"9.00000000","time":1589437530011}"#).unwrap();
//             let dummy_response: models::SymbolOrderBookTickerResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::SymbolOrderBookTickerResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn symbol_price_ticker(
//             &self,
//             _params: SymbolPriceTickerParams,
//         ) -> anyhow::Result<RestApiResponse<models::SymbolPriceTickerResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"symbol":"BTCUSDT","price":"6000.01","time":1589437530011}"#).unwrap();
//             let dummy_response: models::SymbolPriceTickerResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::SymbolPriceTickerResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn symbol_price_ticker_v2(
//             &self,
//             _params: SymbolPriceTickerV2Params,
//         ) -> anyhow::Result<RestApiResponse<models::SymbolPriceTickerV2Response>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"symbol":"BTCUSDT","price":"6000.01","time":1589437530011}"#).unwrap();
//             let dummy_response: models::SymbolPriceTickerV2Response = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::SymbolPriceTickerV2Response");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn taker_buy_sell_volume(
//             &self,
//             _params: TakerBuySellVolumeParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::TakerBuySellVolumeResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"buySellRatio":"1.5586","buyVol":"387.3300","sellVol":"248.5030","timestamp":"1585614900000"},{"buySellRatio":"1.3104","buyVol":"343.9290","sellVol":"248.5030","timestamp":"1583139900000"}]"#).unwrap();
//             let dummy_response: Vec<models::TakerBuySellVolumeResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::TakerBuySellVolumeResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn test_connectivity(&self) -> anyhow::Result<RestApiResponse<Value>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let dummy_response = Value::Null;

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn ticker24hr_price_change_statistics(
//             &self,
//             _params: Ticker24hrPriceChangeStatisticsParams,
//         ) -> anyhow::Result<RestApiResponse<models::Ticker24hrPriceChangeStatisticsResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","priceChange":"-94.99999800","priceChangePercent":"-95.960","weightedAvgPrice":"0.29628482","lastPrice":"4.00000200","lastQty":"200.00000000","openPrice":"99.00000000","highPrice":"100.00000000","lowPrice":"0.10000000","volume":"8913.30000000","quoteVolume":"15.30000000","openTime":1499783499040,"closeTime":1499869899040,"firstId":28385,"lastId":28460,"count":76}"#).unwrap();
//             let dummy_response: models::Ticker24hrPriceChangeStatisticsResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::Ticker24hrPriceChangeStatisticsResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn top_trader_long_short_ratio_accounts(
//             &self,
//             _params: TopTraderLongShortRatioAccountsParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::TopTraderLongShortRatioAccountsResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","longShortRatio":"1.8105","longAccount":"0.6442","shortAccount":"0.3558","timestamp":"1583139600000"},{"symbol":"BTCUSDT","longShortRatio":"0.5576","longAccount":"0.3580","shortAccount":"0.6420","timestamp":"1583139900000"}]"#).unwrap();
//             let dummy_response: Vec<models::TopTraderLongShortRatioAccountsResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::TopTraderLongShortRatioAccountsResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn top_trader_long_short_ratio_positions(
//             &self,
//             _params: TopTraderLongShortRatioPositionsParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::TopTraderLongShortRatioPositionsResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","longShortRatio":"1.4342","longAccount":"0.5891","shortAccount":"0.4108","timestamp":"1583139600000"},{"symbol":"BTCUSDT","longShortRatio":"1.4337","longAccount":"0.3583","shortAccount":"0.6417","timestamp":"1583139900000"}]"#).unwrap();
//             let dummy_response: Vec<models::TopTraderLongShortRatioPositionsResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::TopTraderLongShortRatioPositionsResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }
//     }

//     #[test]
//     fn basis_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = BasisParams::builder("pair_example".to_string(),BasisContractTypeEnum::Perpetual,BasisPeriodEnum::Period5m,30,).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"indexPrice":"34400.15945055","contractType":"PERPETUAL","basisRate":"0.0004","futuresPrice":"34414.10","annualizedBasisRate":"","basis":"13.94054945","pair":"BTCUSDT","timestamp":1698742800000}]"#).unwrap();
//             let expected_response : Vec<models::BasisResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::BasisResponseInner>");

//             let resp = client.basis(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn basis_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = BasisParams::builder("pair_example".to_string(),BasisContractTypeEnum::Perpetual,BasisPeriodEnum::Period5m,30,).start_time(1623319461670).end_time(1641782889000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"indexPrice":"34400.15945055","contractType":"PERPETUAL","basisRate":"0.0004","futuresPrice":"34414.10","annualizedBasisRate":"","basis":"13.94054945","pair":"BTCUSDT","timestamp":1698742800000}]"#).unwrap();
//             let expected_response : Vec<models::BasisResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::BasisResponseInner>");

//             let resp = client.basis(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn basis_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = BasisParams::builder(
//                 "pair_example".to_string(),
//                 BasisContractTypeEnum::Perpetual,
//                 BasisPeriodEnum::Period5m,
//                 30,
//             )
//             .build()
//             .unwrap();

//             match client.basis(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn check_server_time_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let resp_json: Value = serde_json::from_str(r#"{"serverTime":1499827319559}"#).unwrap();
//             let expected_response: models::CheckServerTimeResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::CheckServerTimeResponse");

//             let resp = client.check_server_time().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn check_server_time_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let resp_json: Value = serde_json::from_str(r#"{"serverTime":1499827319559}"#).unwrap();
//             let expected_response: models::CheckServerTimeResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::CheckServerTimeResponse");

//             let resp = client.check_server_time().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn check_server_time_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             match client.check_server_time().await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn composite_index_symbol_information_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = CompositeIndexSymbolInformationParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"DEFIUSDT","time":1589437530011,"component":"baseAsset","baseAssetList":[{"baseAsset":"BAL","quoteAsset":"USDT","weightInQuantity":"1.04406228","weightInPercentage":"0.02783900"},{"baseAsset":"BAND","quoteAsset":"USDT","weightInQuantity":"3.53782729","weightInPercentage":"0.03935200"}]}]"#).unwrap();
//             let expected_response : Vec<models::CompositeIndexSymbolInformationResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::CompositeIndexSymbolInformationResponseInner>");

//             let resp = client.composite_index_symbol_information(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn composite_index_symbol_information_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = CompositeIndexSymbolInformationParams::builder().symbol("symbol_example".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"DEFIUSDT","time":1589437530011,"component":"baseAsset","baseAssetList":[{"baseAsset":"BAL","quoteAsset":"USDT","weightInQuantity":"1.04406228","weightInPercentage":"0.02783900"},{"baseAsset":"BAND","quoteAsset":"USDT","weightInQuantity":"3.53782729","weightInPercentage":"0.03935200"}]}]"#).unwrap();
//             let expected_response : Vec<models::CompositeIndexSymbolInformationResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::CompositeIndexSymbolInformationResponseInner>");

//             let resp = client.composite_index_symbol_information(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn composite_index_symbol_information_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = CompositeIndexSymbolInformationParams::builder().build().unwrap();

//             match client.composite_index_symbol_information(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn compressed_aggregate_trades_list_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = CompressedAggregateTradesListParams::builder("symbol_example".to_string())
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(
//                 r#"[{"a":26129,"p":"0.01633102","q":"4.70443515","f":27781,"l":27781,"T":1498793709153,"m":true}]"#,
//             )
//             .unwrap();
//             let expected_response: Vec<models::CompressedAggregateTradesListResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::CompressedAggregateTradesListResponseInner>");

//             let resp = client
//                 .compressed_aggregate_trades_list(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn compressed_aggregate_trades_list_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = CompressedAggregateTradesListParams::builder("symbol_example".to_string())
//                 .from_id(1)
//                 .start_time(1623319461670)
//                 .end_time(1641782889000)
//                 .limit(100)
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(
//                 r#"[{"a":26129,"p":"0.01633102","q":"4.70443515","f":27781,"l":27781,"T":1498793709153,"m":true}]"#,
//             )
//             .unwrap();
//             let expected_response: Vec<models::CompressedAggregateTradesListResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::CompressedAggregateTradesListResponseInner>");

//             let resp = client
//                 .compressed_aggregate_trades_list(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn compressed_aggregate_trades_list_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = CompressedAggregateTradesListParams::builder("symbol_example".to_string())
//                 .build()
//                 .unwrap();

//             match client.compressed_aggregate_trades_list(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn continuous_contract_kline_candlestick_data_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = ContinuousContractKlineCandlestickDataParams::builder("pair_example".to_string(),ContinuousContractKlineCandlestickDataContractTypeEnum::Perpetual,ContinuousContractKlineCandlestickDataIntervalEnum::Interval1m,).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[[1607444700000,"18879.99","18900.00","18878.98","18896.13","492.363",1607444759999,"9302145.66080",1874,"385.983","7292402.33267","0"]]"#).unwrap();
//             let expected_response : Vec<Vec<models::ContinuousContractKlineCandlestickDataResponseItemInner>> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<Vec<models::ContinuousContractKlineCandlestickDataResponseItemInner>>");

//             let resp = client.continuous_contract_kline_candlestick_data(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn continuous_contract_kline_candlestick_data_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = ContinuousContractKlineCandlestickDataParams::builder("pair_example".to_string(),ContinuousContractKlineCandlestickDataContractTypeEnum::Perpetual,ContinuousContractKlineCandlestickDataIntervalEnum::Interval1m,).start_time(1623319461670).end_time(1641782889000).limit(100).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[[1607444700000,"18879.99","18900.00","18878.98","18896.13","492.363",1607444759999,"9302145.66080",1874,"385.983","7292402.33267","0"]]"#).unwrap();
//             let expected_response : Vec<Vec<models::ContinuousContractKlineCandlestickDataResponseItemInner>> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<Vec<models::ContinuousContractKlineCandlestickDataResponseItemInner>>");

//             let resp = client.continuous_contract_kline_candlestick_data(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn continuous_contract_kline_candlestick_data_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = ContinuousContractKlineCandlestickDataParams::builder(
//                 "pair_example".to_string(),
//                 ContinuousContractKlineCandlestickDataContractTypeEnum::Perpetual,
//                 ContinuousContractKlineCandlestickDataIntervalEnum::Interval1m,
//             )
//             .build()
//             .unwrap();

//             match client.continuous_contract_kline_candlestick_data(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn exchange_information_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let resp_json: Value = serde_json::from_str(r#"{"exchangeFilters":[],"rateLimits":[{"interval":"MINUTE","intervalNum":1,"limit":2400,"rateLimitType":"REQUEST_WEIGHT"},{"interval":"MINUTE","intervalNum":1,"limit":1200,"rateLimitType":"ORDERS"}],"serverTime":1565613908500,"assets":[{"asset":"BTC","marginAvailable":true,"autoAssetExchange":"-0.10"},{"asset":"USDT","marginAvailable":true,"autoAssetExchange":"0"},{"asset":"BNB","marginAvailable":false,"autoAssetExchange":null}],"symbols":[{"symbol":"BLZUSDT","pair":"BLZUSDT","contractType":"PERPETUAL","deliveryDate":4133404800000,"onboardDate":1598252400000,"status":"TRADING","maintMarginPercent":"2.5000","requiredMarginPercent":"5.0000","baseAsset":"BLZ","quoteAsset":"USDT","marginAsset":"USDT","pricePrecision":5,"quantityPrecision":0,"baseAssetPrecision":8,"quotePrecision":8,"underlyingType":"COIN","underlyingSubType":["STORAGE"],"settlePlan":0,"triggerProtect":"0.15","filters":[{"filterType":"PRICE_FILTER","maxPrice":"300","minPrice":"0.0001","tickSize":"0.0001"},{"filterType":"LOT_SIZE","maxQty":"10000000","minQty":"1","stepSize":"1"},{"filterType":"MARKET_LOT_SIZE","maxQty":"590119","minQty":"1","stepSize":"1"},{"filterType":"MAX_NUM_ORDERS","limit":200},{"filterType":"MAX_NUM_ALGO_ORDERS","limit":10},{"filterType":"MIN_NOTIONAL","notional":"5.0"},{"filterType":"PERCENT_PRICE","multiplierUp":"1.1500","multiplierDown":"0.8500","multiplierDecimal":"4"}],"OrderType":["LIMIT","MARKET","STOP","STOP_MARKET","TAKE_PROFIT","TAKE_PROFIT_MARKET","TRAILING_STOP_MARKET"],"timeInForce":["GTC","IOC","FOK","GTX"],"liquidationFee":"0.010000","marketTakeBound":"0.30"}],"timezone":"UTC"}"#).unwrap();
//             let expected_response : models::ExchangeInformationResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::ExchangeInformationResponse");

//             let resp = client.exchange_information().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn exchange_information_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let resp_json: Value = serde_json::from_str(r#"{"exchangeFilters":[],"rateLimits":[{"interval":"MINUTE","intervalNum":1,"limit":2400,"rateLimitType":"REQUEST_WEIGHT"},{"interval":"MINUTE","intervalNum":1,"limit":1200,"rateLimitType":"ORDERS"}],"serverTime":1565613908500,"assets":[{"asset":"BTC","marginAvailable":true,"autoAssetExchange":"-0.10"},{"asset":"USDT","marginAvailable":true,"autoAssetExchange":"0"},{"asset":"BNB","marginAvailable":false,"autoAssetExchange":null}],"symbols":[{"symbol":"BLZUSDT","pair":"BLZUSDT","contractType":"PERPETUAL","deliveryDate":4133404800000,"onboardDate":1598252400000,"status":"TRADING","maintMarginPercent":"2.5000","requiredMarginPercent":"5.0000","baseAsset":"BLZ","quoteAsset":"USDT","marginAsset":"USDT","pricePrecision":5,"quantityPrecision":0,"baseAssetPrecision":8,"quotePrecision":8,"underlyingType":"COIN","underlyingSubType":["STORAGE"],"settlePlan":0,"triggerProtect":"0.15","filters":[{"filterType":"PRICE_FILTER","maxPrice":"300","minPrice":"0.0001","tickSize":"0.0001"},{"filterType":"LOT_SIZE","maxQty":"10000000","minQty":"1","stepSize":"1"},{"filterType":"MARKET_LOT_SIZE","maxQty":"590119","minQty":"1","stepSize":"1"},{"filterType":"MAX_NUM_ORDERS","limit":200},{"filterType":"MAX_NUM_ALGO_ORDERS","limit":10},{"filterType":"MIN_NOTIONAL","notional":"5.0"},{"filterType":"PERCENT_PRICE","multiplierUp":"1.1500","multiplierDown":"0.8500","multiplierDecimal":"4"}],"OrderType":["LIMIT","MARKET","STOP","STOP_MARKET","TAKE_PROFIT","TAKE_PROFIT_MARKET","TRAILING_STOP_MARKET"],"timeInForce":["GTC","IOC","FOK","GTX"],"liquidationFee":"0.010000","marketTakeBound":"0.30"}],"timezone":"UTC"}"#).unwrap();
//             let expected_response : models::ExchangeInformationResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::ExchangeInformationResponse");

//             let resp = client.exchange_information().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn exchange_information_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             match client.exchange_information().await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_funding_rate_history_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = GetFundingRateHistoryParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","fundingRate":"-0.03750000","fundingTime":1570608000000,"markPrice":"34287.54619963"},{"symbol":"BTCUSDT","fundingRate":"0.00010000","fundingTime":1570636800000,"markPrice":"34287.54619963"}]"#).unwrap();
//             let expected_response : Vec<models::GetFundingRateHistoryResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::GetFundingRateHistoryResponseInner>");

//             let resp = client.get_funding_rate_history(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_funding_rate_history_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = GetFundingRateHistoryParams::builder().symbol("symbol_example".to_string()).start_time(1623319461670).end_time(1641782889000).limit(100).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","fundingRate":"-0.03750000","fundingTime":1570608000000,"markPrice":"34287.54619963"},{"symbol":"BTCUSDT","fundingRate":"0.00010000","fundingTime":1570636800000,"markPrice":"34287.54619963"}]"#).unwrap();
//             let expected_response : Vec<models::GetFundingRateHistoryResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::GetFundingRateHistoryResponseInner>");

//             let resp = client.get_funding_rate_history(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_funding_rate_history_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = GetFundingRateHistoryParams::builder().build().unwrap();

//             match client.get_funding_rate_history(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_funding_rate_info_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BLZUSDT","adjustedFundingRateCap":"0.02500000","adjustedFundingRateFloor":"-0.02500000","fundingIntervalHours":8,"disclaimer":false}]"#).unwrap();
//             let expected_response : Vec<models::GetFundingRateInfoResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::GetFundingRateInfoResponseInner>");

//             let resp = client.get_funding_rate_info().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_funding_rate_info_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BLZUSDT","adjustedFundingRateCap":"0.02500000","adjustedFundingRateFloor":"-0.02500000","fundingIntervalHours":8,"disclaimer":false}]"#).unwrap();
//             let expected_response : Vec<models::GetFundingRateInfoResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::GetFundingRateInfoResponseInner>");

//             let resp = client.get_funding_rate_info().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_funding_rate_info_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             match client.get_funding_rate_info().await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn index_price_kline_candlestick_data_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = IndexPriceKlineCandlestickDataParams::builder("pair_example".to_string(),IndexPriceKlineCandlestickDataIntervalEnum::Interval1m,).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[[1591256400000,"9653.69440000","9653.69640000","9651.38600000","9651.55200000","0",1591256459999,"0",60,"0","0","0"]]"#).unwrap();
//             let expected_response : Vec<Vec<models::IndexPriceKlineCandlestickDataResponseItemInner>> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<Vec<models::IndexPriceKlineCandlestickDataResponseItemInner>>");

//             let resp = client.index_price_kline_candlestick_data(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn index_price_kline_candlestick_data_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = IndexPriceKlineCandlestickDataParams::builder("pair_example".to_string(),IndexPriceKlineCandlestickDataIntervalEnum::Interval1m,).start_time(1623319461670).end_time(1641782889000).limit(100).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[[1591256400000,"9653.69440000","9653.69640000","9651.38600000","9651.55200000","0",1591256459999,"0",60,"0","0","0"]]"#).unwrap();
//             let expected_response : Vec<Vec<models::IndexPriceKlineCandlestickDataResponseItemInner>> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<Vec<models::IndexPriceKlineCandlestickDataResponseItemInner>>");

//             let resp = client.index_price_kline_candlestick_data(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn index_price_kline_candlestick_data_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = IndexPriceKlineCandlestickDataParams::builder(
//                 "pair_example".to_string(),
//                 IndexPriceKlineCandlestickDataIntervalEnum::Interval1m,
//             )
//             .build()
//             .unwrap();

//             match client.index_price_kline_candlestick_data(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn kline_candlestick_data_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = KlineCandlestickDataParams::builder("symbol_example".to_string(),KlineCandlestickDataIntervalEnum::Interval1m,).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[[1499040000000,"0.01634790","0.80000000","0.01575800","0.01577100","148976.11427815",1499644799999,"2434.19055334",308,"1756.87402397","28.46694368","17928899.62484339"]]"#).unwrap();
//             let expected_response : Vec<Vec<models::KlineCandlestickDataResponseItemInner>> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<Vec<models::KlineCandlestickDataResponseItemInner>>");

//             let resp = client.kline_candlestick_data(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn kline_candlestick_data_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = KlineCandlestickDataParams::builder("symbol_example".to_string(),KlineCandlestickDataIntervalEnum::Interval1m,).start_time(1623319461670).end_time(1641782889000).limit(100).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[[1499040000000,"0.01634790","0.80000000","0.01575800","0.01577100","148976.11427815",1499644799999,"2434.19055334",308,"1756.87402397","28.46694368","17928899.62484339"]]"#).unwrap();
//             let expected_response : Vec<Vec<models::KlineCandlestickDataResponseItemInner>> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<Vec<models::KlineCandlestickDataResponseItemInner>>");

//             let resp = client.kline_candlestick_data(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn kline_candlestick_data_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = KlineCandlestickDataParams::builder(
//                 "symbol_example".to_string(),
//                 KlineCandlestickDataIntervalEnum::Interval1m,
//             )
//             .build()
//             .unwrap();

//             match client.kline_candlestick_data(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn long_short_ratio_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = LongShortRatioParams::builder("symbol_example".to_string(),LongShortRatioPeriodEnum::Period5m,).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","longShortRatio":"0.1960","longAccount":"0.6622","shortAccount":"0.3378","timestamp":"1583139600000"},{"symbol":"BTCUSDT","longShortRatio":"1.9559","longAccount":"0.6617","shortAccount":"0.3382","timestamp":"1583139900000"}]"#).unwrap();
//             let expected_response : Vec<models::LongShortRatioResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::LongShortRatioResponseInner>");

//             let resp = client.long_short_ratio(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn long_short_ratio_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = LongShortRatioParams::builder("symbol_example".to_string(),LongShortRatioPeriodEnum::Period5m,).limit(100).start_time(1623319461670).end_time(1641782889000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","longShortRatio":"0.1960","longAccount":"0.6622","shortAccount":"0.3378","timestamp":"1583139600000"},{"symbol":"BTCUSDT","longShortRatio":"1.9559","longAccount":"0.6617","shortAccount":"0.3382","timestamp":"1583139900000"}]"#).unwrap();
//             let expected_response : Vec<models::LongShortRatioResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::LongShortRatioResponseInner>");

//             let resp = client.long_short_ratio(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn long_short_ratio_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params =
//                 LongShortRatioParams::builder("symbol_example".to_string(), LongShortRatioPeriodEnum::Period5m)
//                     .build()
//                     .unwrap();

//             match client.long_short_ratio(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn mark_price_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = MarkPriceParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","markPrice":"11793.63104562","indexPrice":"11781.80495970","estimatedSettlePrice":"11781.16138815","lastFundingRate":"0.00038246","interestRate":"0.00010000","nextFundingTime":1597392000000,"time":1597370495002}"#).unwrap();
//             let expected_response : models::MarkPriceResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::MarkPriceResponse");

//             let resp = client.mark_price(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn mark_price_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = MarkPriceParams::builder().symbol("symbol_example".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","markPrice":"11793.63104562","indexPrice":"11781.80495970","estimatedSettlePrice":"11781.16138815","lastFundingRate":"0.00038246","interestRate":"0.00010000","nextFundingTime":1597392000000,"time":1597370495002}"#).unwrap();
//             let expected_response : models::MarkPriceResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::MarkPriceResponse");

//             let resp = client.mark_price(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn mark_price_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = MarkPriceParams::builder().build().unwrap();

//             match client.mark_price(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn mark_price_kline_candlestick_data_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = MarkPriceKlineCandlestickDataParams::builder("symbol_example".to_string(),MarkPriceKlineCandlestickDataIntervalEnum::Interval1m,).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[[1591256460000,"9653.29201333","9654.56401333","9653.07367333","9653.07367333","0",1591256519999,"0",60,"0","0","0"]]"#).unwrap();
//             let expected_response : Vec<Vec<models::MarkPriceKlineCandlestickDataResponseItemInner>> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<Vec<models::MarkPriceKlineCandlestickDataResponseItemInner>>");

//             let resp = client.mark_price_kline_candlestick_data(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn mark_price_kline_candlestick_data_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = MarkPriceKlineCandlestickDataParams::builder("symbol_example".to_string(),MarkPriceKlineCandlestickDataIntervalEnum::Interval1m,).start_time(1623319461670).end_time(1641782889000).limit(100).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[[1591256460000,"9653.29201333","9654.56401333","9653.07367333","9653.07367333","0",1591256519999,"0",60,"0","0","0"]]"#).unwrap();
//             let expected_response : Vec<Vec<models::MarkPriceKlineCandlestickDataResponseItemInner>> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<Vec<models::MarkPriceKlineCandlestickDataResponseItemInner>>");

//             let resp = client.mark_price_kline_candlestick_data(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn mark_price_kline_candlestick_data_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = MarkPriceKlineCandlestickDataParams::builder(
//                 "symbol_example".to_string(),
//                 MarkPriceKlineCandlestickDataIntervalEnum::Interval1m,
//             )
//             .build()
//             .unwrap();

//             match client.mark_price_kline_candlestick_data(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn multi_assets_mode_asset_index_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = MultiAssetsModeAssetIndexParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"ADAUSD","time":1635740268004,"index":"1.92957370","bidBuffer":"0.10000000","askBuffer":"0.10000000","bidRate":"1.73661633","askRate":"2.12253107","autoExchangeBidBuffer":"0.05000000","autoExchangeAskBuffer":"0.05000000","autoExchangeBidRate":"1.83309501","autoExchangeAskRate":"2.02605238"}"#).unwrap();
//             let expected_response : models::MultiAssetsModeAssetIndexResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::MultiAssetsModeAssetIndexResponse");

//             let resp = client.multi_assets_mode_asset_index(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn multi_assets_mode_asset_index_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = MultiAssetsModeAssetIndexParams::builder().symbol("symbol_example".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"ADAUSD","time":1635740268004,"index":"1.92957370","bidBuffer":"0.10000000","askBuffer":"0.10000000","bidRate":"1.73661633","askRate":"2.12253107","autoExchangeBidBuffer":"0.05000000","autoExchangeAskBuffer":"0.05000000","autoExchangeBidRate":"1.83309501","autoExchangeAskRate":"2.02605238"}"#).unwrap();
//             let expected_response : models::MultiAssetsModeAssetIndexResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::MultiAssetsModeAssetIndexResponse");

//             let resp = client.multi_assets_mode_asset_index(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn multi_assets_mode_asset_index_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = MultiAssetsModeAssetIndexParams::builder().build().unwrap();

//             match client.multi_assets_mode_asset_index(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn old_trades_lookup_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = OldTradesLookupParams::builder("symbol_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"id":28457,"price":"4.00000100","qty":"12.00000000","quoteQty":"8000.00","time":1499865549590,"isBuyerMaker":true}]"#).unwrap();
//             let expected_response : Vec<models::OldTradesLookupResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::OldTradesLookupResponseInner>");

//             let resp = client.old_trades_lookup(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn old_trades_lookup_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = OldTradesLookupParams::builder("symbol_example".to_string(),).limit(100).from_id(1).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"id":28457,"price":"4.00000100","qty":"12.00000000","quoteQty":"8000.00","time":1499865549590,"isBuyerMaker":true}]"#).unwrap();
//             let expected_response : Vec<models::OldTradesLookupResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::OldTradesLookupResponseInner>");

//             let resp = client.old_trades_lookup(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn old_trades_lookup_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = OldTradesLookupParams::builder("symbol_example".to_string()).build().unwrap();

//             match client.old_trades_lookup(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn open_interest_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = OpenInterestParams::builder("symbol_example".to_string()).build().unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"openInterest":"10659.509","symbol":"BTCUSDT","time":1589437530011}"#)
//                     .unwrap();
//             let expected_response: models::OpenInterestResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::OpenInterestResponse");

//             let resp = client.open_interest(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn open_interest_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = OpenInterestParams::builder("symbol_example".to_string()).build().unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"openInterest":"10659.509","symbol":"BTCUSDT","time":1589437530011}"#)
//                     .unwrap();
//             let expected_response: models::OpenInterestResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::OpenInterestResponse");

//             let resp = client.open_interest(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn open_interest_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = OpenInterestParams::builder("symbol_example".to_string()).build().unwrap();

//             match client.open_interest(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn open_interest_statistics_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = OpenInterestStatisticsParams::builder("symbol_example".to_string(),OpenInterestStatisticsPeriodEnum::Period5m,).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","sumOpenInterest":"20403.63700000","sumOpenInterestValue":"150570784.07809979","CMCCirculatingSupply":"165880.538","timestamp":"1583127900000"},{"symbol":"BTCUSDT","sumOpenInterest":"20401.36700000","sumOpenInterestValue":"149940752.14464448","CMCCirculatingSupply":"165900.14853","timestamp":"1583128200000"}]"#).unwrap();
//             let expected_response : Vec<models::OpenInterestStatisticsResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::OpenInterestStatisticsResponseInner>");

//             let resp = client.open_interest_statistics(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn open_interest_statistics_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = OpenInterestStatisticsParams::builder("symbol_example".to_string(),OpenInterestStatisticsPeriodEnum::Period5m,).limit(100).start_time(1623319461670).end_time(1641782889000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","sumOpenInterest":"20403.63700000","sumOpenInterestValue":"150570784.07809979","CMCCirculatingSupply":"165880.538","timestamp":"1583127900000"},{"symbol":"BTCUSDT","sumOpenInterest":"20401.36700000","sumOpenInterestValue":"149940752.14464448","CMCCirculatingSupply":"165900.14853","timestamp":"1583128200000"}]"#).unwrap();
//             let expected_response : Vec<models::OpenInterestStatisticsResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::OpenInterestStatisticsResponseInner>");

//             let resp = client.open_interest_statistics(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn open_interest_statistics_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = OpenInterestStatisticsParams::builder(
//                 "symbol_example".to_string(),
//                 OpenInterestStatisticsPeriodEnum::Period5m,
//             )
//             .build()
//             .unwrap();

//             match client.open_interest_statistics(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn order_book_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = OrderBookParams::builder("symbol_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"lastUpdateId":1027024,"E":1589436922972,"T":1589436922959,"bids":[["4.00000000","431.00000000"]],"asks":[["4.00000200","12.00000000"]]}"#).unwrap();
//             let expected_response : models::OrderBookResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::OrderBookResponse");

//             let resp = client.order_book(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn order_book_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = OrderBookParams::builder("symbol_example".to_string(),).limit(100).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"lastUpdateId":1027024,"E":1589436922972,"T":1589436922959,"bids":[["4.00000000","431.00000000"]],"asks":[["4.00000200","12.00000000"]]}"#).unwrap();
//             let expected_response : models::OrderBookResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::OrderBookResponse");

//             let resp = client.order_book(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn order_book_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = OrderBookParams::builder("symbol_example".to_string()).build().unwrap();

//             match client.order_book(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn premium_index_kline_data_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = PremiumIndexKlineDataParams::builder("symbol_example".to_string(),PremiumIndexKlineDataIntervalEnum::Interval1m,).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[[1691603820000,"-0.00042931","-0.00023641","-0.00059406","-0.00043659","0",1691603879999,"0",12,"0","0","0"]]"#).unwrap();
//             let expected_response : Vec<Vec<models::PremiumIndexKlineDataResponseItemInner>> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<Vec<models::PremiumIndexKlineDataResponseItemInner>>");

//             let resp = client.premium_index_kline_data(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn premium_index_kline_data_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = PremiumIndexKlineDataParams::builder("symbol_example".to_string(),PremiumIndexKlineDataIntervalEnum::Interval1m,).start_time(1623319461670).end_time(1641782889000).limit(100).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[[1691603820000,"-0.00042931","-0.00023641","-0.00059406","-0.00043659","0",1691603879999,"0",12,"0","0","0"]]"#).unwrap();
//             let expected_response : Vec<Vec<models::PremiumIndexKlineDataResponseItemInner>> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<Vec<models::PremiumIndexKlineDataResponseItemInner>>");

//             let resp = client.premium_index_kline_data(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn premium_index_kline_data_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = PremiumIndexKlineDataParams::builder(
//                 "symbol_example".to_string(),
//                 PremiumIndexKlineDataIntervalEnum::Interval1m,
//             )
//             .build()
//             .unwrap();

//             match client.premium_index_kline_data(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn quarterly_contract_settlement_price_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = QuarterlyContractSettlementPriceParams::builder("pair_example".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"deliveryTime":1695945600000,"deliveryPrice":27103},{"deliveryTime":1688083200000,"deliveryPrice":30733.6},{"deliveryTime":1680220800000,"deliveryPrice":27814.2},{"deliveryTime":1648166400000,"deliveryPrice":44066.3}]"#).unwrap();
//             let expected_response : Vec<models::QuarterlyContractSettlementPriceResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::QuarterlyContractSettlementPriceResponseInner>");

//             let resp = client.quarterly_contract_settlement_price(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn quarterly_contract_settlement_price_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = QuarterlyContractSettlementPriceParams::builder("pair_example".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"deliveryTime":1695945600000,"deliveryPrice":27103},{"deliveryTime":1688083200000,"deliveryPrice":30733.6},{"deliveryTime":1680220800000,"deliveryPrice":27814.2},{"deliveryTime":1648166400000,"deliveryPrice":44066.3}]"#).unwrap();
//             let expected_response : Vec<models::QuarterlyContractSettlementPriceResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::QuarterlyContractSettlementPriceResponseInner>");

//             let resp = client.quarterly_contract_settlement_price(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn quarterly_contract_settlement_price_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = QuarterlyContractSettlementPriceParams::builder("pair_example".to_string())
//                 .build()
//                 .unwrap();

//             match client.quarterly_contract_settlement_price(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn query_index_price_constituents_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = QueryIndexPriceConstituentsParams::builder("symbol_example".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","time":1745401553408,"constituents":[{"exchange":"binance","symbol":"BTCUSDT","price":"94057.03000000","weight":"0.51282051"},{"exchange":"coinbase","symbol":"BTC-USDT","price":"94140.58000000","weight":"0.15384615"},{"exchange":"gateio","symbol":"BTC_USDT","price":"94060.10000000","weight":"0.02564103"},{"exchange":"kucoin","symbol":"BTC-USDT","price":"94096.70000000","weight":"0.07692308"},{"exchange":"mxc","symbol":"BTCUSDT","price":"94057.02000000","weight":"0.07692308"},{"exchange":"bitget","symbol":"BTCUSDT","price":"94064.03000000","weight":"0.07692308"},{"exchange":"bybit","symbol":"BTCUSDT","price":"94067.90000000","weight":"0.07692308"}]}"#).unwrap();
//             let expected_response : models::QueryIndexPriceConstituentsResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::QueryIndexPriceConstituentsResponse");

//             let resp = client.query_index_price_constituents(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn query_index_price_constituents_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = QueryIndexPriceConstituentsParams::builder("symbol_example".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","time":1745401553408,"constituents":[{"exchange":"binance","symbol":"BTCUSDT","price":"94057.03000000","weight":"0.51282051"},{"exchange":"coinbase","symbol":"BTC-USDT","price":"94140.58000000","weight":"0.15384615"},{"exchange":"gateio","symbol":"BTC_USDT","price":"94060.10000000","weight":"0.02564103"},{"exchange":"kucoin","symbol":"BTC-USDT","price":"94096.70000000","weight":"0.07692308"},{"exchange":"mxc","symbol":"BTCUSDT","price":"94057.02000000","weight":"0.07692308"},{"exchange":"bitget","symbol":"BTCUSDT","price":"94064.03000000","weight":"0.07692308"},{"exchange":"bybit","symbol":"BTCUSDT","price":"94067.90000000","weight":"0.07692308"}]}"#).unwrap();
//             let expected_response : models::QueryIndexPriceConstituentsResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::QueryIndexPriceConstituentsResponse");

//             let resp = client.query_index_price_constituents(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn query_index_price_constituents_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = QueryIndexPriceConstituentsParams::builder("symbol_example".to_string())
//                 .build()
//                 .unwrap();

//             match client.query_index_price_constituents(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn query_insurance_fund_balance_snapshot_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = QueryInsuranceFundBalanceSnapshotParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbols":["BNBUSDT","BTCUSDT","BTCUSDT_250627","BTCUSDT_250926","ETHBTC","ETHUSDT","ETHUSDT_250627","ETHUSDT_250926"],"assets":[{"asset":"USDC","marginBalance":"299999998.6497832","updateTime":1745366402000},{"asset":"USDT","marginBalance":"793930579.315848","updateTime":1745366402000},{"asset":"BTC","marginBalance":"61.73143554","updateTime":1745366402000},{"asset":"BNFCR","marginBalance":"633223.99396922","updateTime":1745366402000}]}"#).unwrap();
//             let expected_response : models::QueryInsuranceFundBalanceSnapshotResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::QueryInsuranceFundBalanceSnapshotResponse");

//             let resp = client.query_insurance_fund_balance_snapshot(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn query_insurance_fund_balance_snapshot_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = QueryInsuranceFundBalanceSnapshotParams::builder().symbol("symbol_example".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbols":["BNBUSDT","BTCUSDT","BTCUSDT_250627","BTCUSDT_250926","ETHBTC","ETHUSDT","ETHUSDT_250627","ETHUSDT_250926"],"assets":[{"asset":"USDC","marginBalance":"299999998.6497832","updateTime":1745366402000},{"asset":"USDT","marginBalance":"793930579.315848","updateTime":1745366402000},{"asset":"BTC","marginBalance":"61.73143554","updateTime":1745366402000},{"asset":"BNFCR","marginBalance":"633223.99396922","updateTime":1745366402000}]}"#).unwrap();
//             let expected_response : models::QueryInsuranceFundBalanceSnapshotResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::QueryInsuranceFundBalanceSnapshotResponse");

//             let resp = client.query_insurance_fund_balance_snapshot(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn query_insurance_fund_balance_snapshot_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = QueryInsuranceFundBalanceSnapshotParams::builder().build().unwrap();

//             match client.query_insurance_fund_balance_snapshot(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn recent_trades_list_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = RecentTradesListParams::builder("symbol_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"id":28457,"price":"4.00000100","qty":"12.00000000","quoteQty":"48.00","time":1499865549590,"isBuyerMaker":true}]"#).unwrap();
//             let expected_response : Vec<models::RecentTradesListResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::RecentTradesListResponseInner>");

//             let resp = client.recent_trades_list(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn recent_trades_list_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = RecentTradesListParams::builder("symbol_example".to_string(),).limit(100).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"id":28457,"price":"4.00000100","qty":"12.00000000","quoteQty":"48.00","time":1499865549590,"isBuyerMaker":true}]"#).unwrap();
//             let expected_response : Vec<models::RecentTradesListResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::RecentTradesListResponseInner>");

//             let resp = client.recent_trades_list(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn recent_trades_list_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = RecentTradesListParams::builder("symbol_example".to_string()).build().unwrap();

//             match client.recent_trades_list(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn symbol_order_book_ticker_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = SymbolOrderBookTickerParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","bidPrice":"4.00000000","bidQty":"431.00000000","askPrice":"4.00000200","askQty":"9.00000000","time":1589437530011}"#).unwrap();
//             let expected_response : models::SymbolOrderBookTickerResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::SymbolOrderBookTickerResponse");

//             let resp = client.symbol_order_book_ticker(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn symbol_order_book_ticker_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = SymbolOrderBookTickerParams::builder().symbol("symbol_example".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","bidPrice":"4.00000000","bidQty":"431.00000000","askPrice":"4.00000200","askQty":"9.00000000","time":1589437530011}"#).unwrap();
//             let expected_response : models::SymbolOrderBookTickerResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::SymbolOrderBookTickerResponse");

//             let resp = client.symbol_order_book_ticker(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn symbol_order_book_ticker_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = SymbolOrderBookTickerParams::builder().build().unwrap();

//             match client.symbol_order_book_ticker(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn symbol_price_ticker_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = SymbolPriceTickerParams::builder().build().unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"symbol":"BTCUSDT","price":"6000.01","time":1589437530011}"#).unwrap();
//             let expected_response: models::SymbolPriceTickerResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::SymbolPriceTickerResponse");

//             let resp = client.symbol_price_ticker(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn symbol_price_ticker_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = SymbolPriceTickerParams::builder()
//                 .symbol("symbol_example".to_string())
//                 .build()
//                 .unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"symbol":"BTCUSDT","price":"6000.01","time":1589437530011}"#).unwrap();
//             let expected_response: models::SymbolPriceTickerResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::SymbolPriceTickerResponse");

//             let resp = client.symbol_price_ticker(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn symbol_price_ticker_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = SymbolPriceTickerParams::builder().build().unwrap();

//             match client.symbol_price_ticker(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn symbol_price_ticker_v2_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = SymbolPriceTickerV2Params::builder().build().unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"symbol":"BTCUSDT","price":"6000.01","time":1589437530011}"#).unwrap();
//             let expected_response: models::SymbolPriceTickerV2Response = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::SymbolPriceTickerV2Response");

//             let resp = client.symbol_price_ticker_v2(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn symbol_price_ticker_v2_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = SymbolPriceTickerV2Params::builder()
//                 .symbol("symbol_example".to_string())
//                 .build()
//                 .unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"symbol":"BTCUSDT","price":"6000.01","time":1589437530011}"#).unwrap();
//             let expected_response: models::SymbolPriceTickerV2Response = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::SymbolPriceTickerV2Response");

//             let resp = client.symbol_price_ticker_v2(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn symbol_price_ticker_v2_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = SymbolPriceTickerV2Params::builder().build().unwrap();

//             match client.symbol_price_ticker_v2(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn taker_buy_sell_volume_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = TakerBuySellVolumeParams::builder("symbol_example".to_string(),TakerBuySellVolumePeriodEnum::Period5m,).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"buySellRatio":"1.5586","buyVol":"387.3300","sellVol":"248.5030","timestamp":"1585614900000"},{"buySellRatio":"1.3104","buyVol":"343.9290","sellVol":"248.5030","timestamp":"1583139900000"}]"#).unwrap();
//             let expected_response : Vec<models::TakerBuySellVolumeResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::TakerBuySellVolumeResponseInner>");

//             let resp = client.taker_buy_sell_volume(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn taker_buy_sell_volume_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = TakerBuySellVolumeParams::builder("symbol_example".to_string(),TakerBuySellVolumePeriodEnum::Period5m,).limit(100).start_time(1623319461670).end_time(1641782889000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"buySellRatio":"1.5586","buyVol":"387.3300","sellVol":"248.5030","timestamp":"1585614900000"},{"buySellRatio":"1.3104","buyVol":"343.9290","sellVol":"248.5030","timestamp":"1583139900000"}]"#).unwrap();
//             let expected_response : Vec<models::TakerBuySellVolumeResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::TakerBuySellVolumeResponseInner>");

//             let resp = client.taker_buy_sell_volume(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn taker_buy_sell_volume_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params =
//                 TakerBuySellVolumeParams::builder("symbol_example".to_string(), TakerBuySellVolumePeriodEnum::Period5m)
//                     .build()
//                     .unwrap();

//             match client.taker_buy_sell_volume(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn test_connectivity_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let expected_response = Value::Null;

//             let resp = client.test_connectivity().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn test_connectivity_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let expected_response = Value::Null;

//             let resp = client.test_connectivity().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn test_connectivity_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             match client.test_connectivity().await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn ticker24hr_price_change_statistics_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = Ticker24hrPriceChangeStatisticsParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","priceChange":"-94.99999800","priceChangePercent":"-95.960","weightedAvgPrice":"0.29628482","lastPrice":"4.00000200","lastQty":"200.00000000","openPrice":"99.00000000","highPrice":"100.00000000","lowPrice":"0.10000000","volume":"8913.30000000","quoteVolume":"15.30000000","openTime":1499783499040,"closeTime":1499869899040,"firstId":28385,"lastId":28460,"count":76}"#).unwrap();
//             let expected_response : models::Ticker24hrPriceChangeStatisticsResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::Ticker24hrPriceChangeStatisticsResponse");

//             let resp = client.ticker24hr_price_change_statistics(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn ticker24hr_price_change_statistics_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = Ticker24hrPriceChangeStatisticsParams::builder().symbol("symbol_example".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","priceChange":"-94.99999800","priceChangePercent":"-95.960","weightedAvgPrice":"0.29628482","lastPrice":"4.00000200","lastQty":"200.00000000","openPrice":"99.00000000","highPrice":"100.00000000","lowPrice":"0.10000000","volume":"8913.30000000","quoteVolume":"15.30000000","openTime":1499783499040,"closeTime":1499869899040,"firstId":28385,"lastId":28460,"count":76}"#).unwrap();
//             let expected_response : models::Ticker24hrPriceChangeStatisticsResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::Ticker24hrPriceChangeStatisticsResponse");

//             let resp = client.ticker24hr_price_change_statistics(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn ticker24hr_price_change_statistics_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = Ticker24hrPriceChangeStatisticsParams::builder().build().unwrap();

//             match client.ticker24hr_price_change_statistics(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn top_trader_long_short_ratio_accounts_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = TopTraderLongShortRatioAccountsParams::builder("symbol_example".to_string(),TopTraderLongShortRatioAccountsPeriodEnum::Period5m,).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","longShortRatio":"1.8105","longAccount":"0.6442","shortAccount":"0.3558","timestamp":"1583139600000"},{"symbol":"BTCUSDT","longShortRatio":"0.5576","longAccount":"0.3580","shortAccount":"0.6420","timestamp":"1583139900000"}]"#).unwrap();
//             let expected_response : Vec<models::TopTraderLongShortRatioAccountsResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::TopTraderLongShortRatioAccountsResponseInner>");

//             let resp = client.top_trader_long_short_ratio_accounts(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn top_trader_long_short_ratio_accounts_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = TopTraderLongShortRatioAccountsParams::builder("symbol_example".to_string(),TopTraderLongShortRatioAccountsPeriodEnum::Period5m,).limit(100).start_time(1623319461670).end_time(1641782889000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","longShortRatio":"1.8105","longAccount":"0.6442","shortAccount":"0.3558","timestamp":"1583139600000"},{"symbol":"BTCUSDT","longShortRatio":"0.5576","longAccount":"0.3580","shortAccount":"0.6420","timestamp":"1583139900000"}]"#).unwrap();
//             let expected_response : Vec<models::TopTraderLongShortRatioAccountsResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::TopTraderLongShortRatioAccountsResponseInner>");

//             let resp = client.top_trader_long_short_ratio_accounts(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn top_trader_long_short_ratio_accounts_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = TopTraderLongShortRatioAccountsParams::builder(
//                 "symbol_example".to_string(),
//                 TopTraderLongShortRatioAccountsPeriodEnum::Period5m,
//             )
//             .build()
//             .unwrap();

//             match client.top_trader_long_short_ratio_accounts(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn top_trader_long_short_ratio_positions_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = TopTraderLongShortRatioPositionsParams::builder("symbol_example".to_string(),TopTraderLongShortRatioPositionsPeriodEnum::Period5m,).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","longShortRatio":"1.4342","longAccount":"0.5891","shortAccount":"0.4108","timestamp":"1583139600000"},{"symbol":"BTCUSDT","longShortRatio":"1.4337","longAccount":"0.3583","shortAccount":"0.6417","timestamp":"1583139900000"}]"#).unwrap();
//             let expected_response : Vec<models::TopTraderLongShortRatioPositionsResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::TopTraderLongShortRatioPositionsResponseInner>");

//             let resp = client.top_trader_long_short_ratio_positions(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn top_trader_long_short_ratio_positions_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: false };

//             let params = TopTraderLongShortRatioPositionsParams::builder("symbol_example".to_string(),TopTraderLongShortRatioPositionsPeriodEnum::Period5m,).limit(100).start_time(1623319461670).end_time(1641782889000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","longShortRatio":"1.4342","longAccount":"0.5891","shortAccount":"0.4108","timestamp":"1583139600000"},{"symbol":"BTCUSDT","longShortRatio":"1.4337","longAccount":"0.3583","shortAccount":"0.6417","timestamp":"1583139900000"}]"#).unwrap();
//             let expected_response : Vec<models::TopTraderLongShortRatioPositionsResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::TopTraderLongShortRatioPositionsResponseInner>");

//             let resp = client.top_trader_long_short_ratio_positions(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn top_trader_long_short_ratio_positions_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockMarketDataApiClient { force_error: true };

//             let params = TopTraderLongShortRatioPositionsParams::builder(
//                 "symbol_example".to_string(),
//                 TopTraderLongShortRatioPositionsPeriodEnum::Period5m,
//             )
//             .build()
//             .unwrap();

//             match client.top_trader_long_short_ratio_positions(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }
// }
