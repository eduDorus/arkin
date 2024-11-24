mod single_strategy;

use std::sync::Arc;

pub use single_strategy::SingleStrategyPortfolio;
pub use single_strategy::SingleStrategyPortfolioBuilder;

use arkin_core::prelude::*;
use tokio_util::sync::CancellationToken;
use tracing::info;
use tracing::instrument;
use tracing::warn;

use crate::Portfolio;
use crate::PortfolioError;

#[instrument(skip(portfolio, pubsub, shutdown))]
pub async fn start_portfolio(
    portfolio: Arc<dyn Portfolio>,
    pubsub: Arc<PubSub>,
    shutdown: CancellationToken,
) -> Result<(), PortfolioError> {
    let mut balance_updates = pubsub.subscribe::<Holding>();
    let mut position_updates = pubsub.subscribe::<Position>();
    let mut fill_updates = pubsub.subscribe::<Fill>();
    loop {
        tokio::select! {
            Ok(balance) = balance_updates.recv() => {
                match portfolio.balance_update(balance).await {
                    Ok(_) => info!("Balance update processed"),
                    Err(e) => warn!("Failed to process balance update: {}", e),
                }
            }
            Ok(position) = position_updates.recv() => {
                match portfolio.position_update(position).await {
                    Ok(_) => info!("Position update processed"),
                    Err(e) => warn!("Failed to process position update: {}", e),
                }
            }
            Ok(fill) = fill_updates.recv() => {
                match portfolio.fill_update(fill).await {
                    Ok(_) => info!("Fill update processed"),
                    Err(e) => warn!("Failed to process fill update: {}", e),
                }
            }
            _ = shutdown.cancelled() => {
                break;
            }
        }
    }
    Ok(())
}
