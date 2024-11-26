use thiserror::Error;

#[derive(Error, Debug)]
pub enum PortfolioError {
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
}
