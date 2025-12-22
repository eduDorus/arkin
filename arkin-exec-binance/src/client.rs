use arkin_core::prelude::*;

use crate::config::BinanceExecutionServiceConfig;
use crate::margin::BinanceMarginClient;
use crate::spot::BinanceSpotClient;
use crate::types::{BinanceCancelResponse, BinanceMarketType, BinanceOrderResponse};
use crate::usdm::BinanceUsdmClient;
use crate::utils::instrument_to_market_type;

#[derive(Debug, Clone)]
pub struct BinanceClient {
    spot_client: Option<BinanceSpotClient>,
    margin_client: Option<BinanceMarginClient>,
    usdm_client: Option<BinanceUsdmClient>,
}

impl BinanceClient {
    pub fn from_config() -> Self {
        let service_config = load::<BinanceExecutionServiceConfig>();
        let config = service_config.binance_execution;
        Self::new(config)
    }

    pub fn new(config: crate::config::BinanceExecutionConfig) -> Self {
        let spot_client = config.spot.filter(|c| c.enabled).map(|c| BinanceSpotClient::new(c));
        let margin_client = config.margin.filter(|c| c.enabled).map(|c| BinanceMarginClient::new(c));
        let usdm_client = config.usdm.filter(|c| c.enabled).map(|c| BinanceUsdmClient::new(c));

        Self {
            spot_client,
            margin_client,
            usdm_client,
        }
    }

    pub async fn place_order(
        &self,
        order: &VenueOrder,
    ) -> Result<BinanceOrderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let market_type = instrument_to_market_type(&order.instrument);
        let params = crate::utils::venue_order_to_params(order, &market_type);

        match market_type {
            BinanceMarketType::Spot => {
                if let Some(client) = &self.spot_client {
                    client.place_order(params).await
                } else {
                    Err("Spot client not configured".into())
                }
            }
            BinanceMarketType::Margin => {
                if let Some(client) = &self.margin_client {
                    client.place_order(params).await
                } else {
                    Err("Margin client not configured".into())
                }
            }
            BinanceMarketType::Usdm => {
                if let Some(client) = &self.usdm_client {
                    client.place_order(params).await
                } else {
                    Err("USDM client not configured".into())
                }
            }
        }
    }

    pub async fn cancel_order(
        &self,
        order: &VenueOrder,
    ) -> Result<BinanceCancelResponse, Box<dyn std::error::Error + Send + Sync>> {
        let market_type = instrument_to_market_type(&order.instrument);

        match market_type {
            BinanceMarketType::Spot => {
                if let Some(client) = &self.spot_client {
                    let client_order_id_str = order.id.to_string();
                    client
                        .cancel_order(
                            &order.instrument.venue_symbol,
                            None,
                            Some(client_order_id_str.as_str()), // Use internal order ID as client_order_id
                        )
                        .await
                } else {
                    Err("Spot client not configured".into())
                }
            }
            BinanceMarketType::Margin => {
                if let Some(client) = &self.margin_client {
                    let client_order_id_str = order.id.to_string();
                    client
                        .cancel_order(
                            &order.instrument.venue_symbol,
                            None,
                            Some(client_order_id_str.as_str()), // Use internal order ID as client_order_id
                        )
                        .await
                } else {
                    Err("Margin client not configured".into())
                }
            }
            BinanceMarketType::Usdm => {
                if let Some(client) = &self.usdm_client {
                    let client_order_id_str = order.id.to_string();
                    client
                        .cancel_order(
                            &order.instrument.venue_symbol,
                            None,
                            Some(client_order_id_str.as_str()), // Use internal order ID as client_order_id
                        )
                        .await
                } else {
                    Err("USDM client not configured".into())
                }
            }
        }
    }

    pub async fn cancel_all_orders(
        &self,
        symbol: Option<&str>,
        market_type: BinanceMarketType,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match market_type {
            BinanceMarketType::Spot => {
                if let Some(client) = &self.spot_client {
                    client.cancel_all_orders(symbol).await
                } else {
                    Err("Spot client not configured".into())
                }
            }
            BinanceMarketType::Margin => {
                if let Some(client) = &self.margin_client {
                    client.cancel_all_orders(symbol).await
                } else {
                    Err("Margin client not configured".into())
                }
            }
            BinanceMarketType::Usdm => {
                if let Some(client) = &self.usdm_client {
                    client.cancel_all_orders(symbol).await
                } else {
                    Err("USDM client not configured".into())
                }
            }
        }
    }
}
