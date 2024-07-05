use tracing::info;

use crate::config::DataProviderFactoryConfig;

use super::{binance::BinanceDataProvider, DataProviderType};

pub struct DataProviderFactory {
    config: DataProviderFactoryConfig,
}

impl DataProviderFactory {
    pub fn new(config: &DataProviderFactoryConfig) -> DataProviderFactory {
        DataProviderFactory {
            config: config.to_owned(),
        }
    }

    pub fn create_data_providers(&self) -> Vec<DataProviderType> {
        let mut data_providers = Vec::new();

        for (name, config) in &self.config.binance {
            if config.enabled {
                info!("Creating Binance data provider: {}", name);
                data_providers.push(DataProviderType::Binance(BinanceDataProvider::new(config)));
            }
        }

        data_providers
    }
}
