use std::sync::Arc;

use anyhow::Result;
use arkin_core::{
    AccountListQuery, AssetListQuery, AssetQuery, AssetType, FeatureListQuery, FeatureQuery, Instance,
    InstanceListQuery, InstanceQuery, InstanceType, InstrumentListQuery, InstrumentQuery, InstrumentType, PersistenceReader,
    PipelineListQuery, StrategyListQuery, VenueName,
};
use arkin_persistence::Persistence;

async fn init_test_persistence() -> Arc<dyn PersistenceReader> {
    let instance = Instance::builder()
        .id(uuid::Uuid::new_v4())
        .name("integration-test".to_string())
        .instance_type(InstanceType::Test)
        .created(time::OffsetDateTime::now_utc().into())
        .updated(time::OffsetDateTime::now_utc().into())
        .build();
    let persistence = Persistence::from_config(instance, false, false, true);
    persistence.refresh().await.expect("Failed to refresh persistence");
    persistence
}

#[tokio::test]
#[test_log::test]
async fn test_persistence_asset_queries() -> Result<()> {
    let persistence = init_test_persistence().await;

    // Test get_asset with AssetQuery
    let btc_query = AssetQuery::builder().symbol("BTC").asset_type(AssetType::Crypto).build();
    let asset = persistence.get_asset(&btc_query).await?;
    assert_eq!(asset.symbol, "BTC");

    // Test list_assets with AssetListQuery
    let crypto_query = AssetListQuery::builder().asset_types(vec![AssetType::Crypto]).build();
    let assets = persistence.list_assets(&crypto_query).await?;
    for asset in &assets {
        assert_eq!(asset.asset_type, AssetType::Crypto);
    }

    let query = AssetListQuery::builder()
        .symbols(vec!["ETH".to_string(), "SOL".to_string(), "USDT".to_string()])
        .build();
    let assets = persistence.list_assets(&query).await?;
    assert_eq!(assets.len(), 3);

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_persistence_single_queries() -> Result<()> {
    let persistence = init_test_persistence().await;

    // Test get_asset
    let btc_query = AssetQuery::builder().symbol("BTC").asset_type(AssetType::Crypto).build();
    let asset = persistence.get_asset(&btc_query).await?;
    assert_eq!(asset.symbol, "BTC");

    // Test get_venue
    // Assuming some venue exists, skip for now
    // let venue_query = VenueQuery::builder().name(VenueName::BinanceSpot).build();
    // let venue = persistence.get_venue(&venue_query).await?;
    // assert_eq!(venue.name, VenueName::BinanceSpot);

    // Test get_instrument
    let instrument_query = InstrumentQuery::builder()
        .venue_symbol("BTCUSDT")
        .build();
    let instrument = persistence.get_instrument(&instrument_query).await?;
    assert_eq!(instrument.venue_symbol, "BTCUSDT");

    // Test get_instance
    // Assuming test instance exists
    let instance_query = InstanceQuery::builder().name("integration-test").build();
    let instance = persistence.get_instance(&instance_query).await?;
    assert_eq!(instance.name, "integration-test");

    // Test get_pipeline
    // Assuming some pipeline exists, or skip if not
    // For now, skip

    // Test get_feature
    // Assuming "trade_price" exists
    let feature = persistence
        .get_feature(&FeatureQuery {
            id: "trade_price".to_string(),
        })
        .await;
    assert!(!feature.is_empty());

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_persistence_list_queries() -> Result<()> {
    let persistence = init_test_persistence().await;

    // Test list_assets with asset_types
    let crypto_query = AssetListQuery::builder().asset_types(vec![AssetType::Crypto]).build();
    let assets = persistence.list_assets(&crypto_query).await?;
    for asset in &assets {
        assert_eq!(asset.asset_type, AssetType::Crypto);
    }

    // Test list_assets with symbols
    let symbol_query = AssetListQuery::builder()
        .symbols(vec!["ETH".to_string(), "SOL".to_string(), "USDT".to_string()])
        .build();
    let assets = persistence.list_assets(&symbol_query).await?;
    assert_eq!(assets.len(), 3);

    // Test list_venues with venue_types
    // let exchange_query = VenueListQuery::builder().venue_types(vec![VenueType::Exchange]).build();
    // let venues = persistence.list_venues(&exchange_query).await?;
    // for venue in &venues {
    //     assert_eq!(venue.venue_type, VenueType::Exchange);
    // }

    // Test list_instruments with instrument_types
    let spot_query = InstrumentListQuery::builder()
        .instrument_types(vec![InstrumentType::Spot])
        .build();
    let instruments = persistence.list_instruments(&spot_query).await?;
    for instrument in &instruments {
        assert_eq!(instrument.instrument_type, InstrumentType::Spot);
    }

    // Test total instrument count
    let all_instruments = persistence.list_instruments(&InstrumentListQuery::default()).await?;
    assert_eq!(all_instruments.len(), 285, "Should have exactly 285 instruments loaded");

    // Test list_instances
    let instances = persistence.list_instances(&InstanceListQuery::default()).await?;
    assert!(instances.len() > 0, "Should have at least one instance");

    // Test list_pipelines
    let pipelines = persistence.list_pipelines(&PipelineListQuery::default()).await?;
    assert!(pipelines.len() > 0, "Should have at least one pipeline");

    // Test list_features
    let _features = persistence.list_features(&FeatureListQuery::default()).await?;
    // Note: features might be empty if no features have been cached yet
    // assert!(features.len() > 0, "Should have at least one feature");

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_persistence_account_strategy_queries() -> Result<()> {
    let persistence = init_test_persistence().await;

    // Test list_accounts with default query
    let _accounts = persistence.list_accounts(&AccountListQuery::default()).await?;
    // Note: test database might not have accounts, so don't assert count

    // Test list_strategies with default query
    let strategies = persistence.list_strategies(&StrategyListQuery::default()).await?;
    assert!(strategies.len() > 0, "Should have at least one strategy");

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_persistence_instrument_counts_and_filtering() -> Result<()> {
    let persistence = init_test_persistence().await;

    // Test total instrument count
    let all_instruments = persistence.list_instruments(&InstrumentListQuery::default()).await?;
    assert_eq!(all_instruments.len(), 285, "Should have exactly 285 instruments loaded");

    // Test filtering by venue (Binance)
    let binance_query = InstrumentListQuery::builder()
        .venues(vec![VenueName::Binance])
        .build();
    let binance_instruments = persistence.list_instruments(&binance_query).await?;

    // Manually filter all instruments for comparison
    let manual_binance_count = all_instruments.iter()
        .filter(|i| i.venue.name == VenueName::Binance)
        .count();
    assert_eq!(binance_instruments.len(), manual_binance_count,
        "Reader filtering should match manual filtering for Binance instruments");

    // Test filtering by instrument type (Spot)
    let spot_query = InstrumentListQuery::builder()
        .instrument_types(vec![InstrumentType::Spot])
        .build();
    let spot_instruments = persistence.list_instruments(&spot_query).await?;

    let manual_spot_count = all_instruments.iter()
        .filter(|i| i.instrument_type == InstrumentType::Spot)
        .count();
    assert_eq!(spot_instruments.len(), manual_spot_count,
        "Reader filtering should match manual filtering for Spot instruments");

    // Test combined filtering (Binance + Spot)
    let binance_spot_query = InstrumentListQuery::builder()
        .venues(vec![VenueName::Binance])
        .instrument_types(vec![InstrumentType::Spot])
        .build();
    let binance_spot_instruments = persistence.list_instruments(&binance_spot_query).await?;

    let manual_binance_spot_count = all_instruments.iter()
        .filter(|i| i.venue.name == VenueName::Binance && i.instrument_type == InstrumentType::Spot)
        .count();
    assert_eq!(binance_spot_instruments.len(), manual_binance_spot_count,
        "Reader filtering should match manual filtering for Binance Spot instruments");

    // Verify that combined filter is subset of individual filters
    assert!(binance_spot_instruments.len() <= binance_instruments.len(),
        "Combined filter should return subset of venue filter");
    assert!(binance_spot_instruments.len() <= spot_instruments.len(),
        "Combined filter should return subset of type filter");

    // Test that all filtered instruments match the criteria
    for instrument in &binance_spot_instruments {
        assert_eq!(instrument.venue.name, VenueName::Binance, "All instruments should be from Binance");
        assert_eq!(instrument.instrument_type, InstrumentType::Spot, "All instruments should be Spot");
    }

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_persistence_comprehensive_instrument_queries() -> Result<()> {
    let persistence = init_test_persistence().await;
    let all_instruments = persistence.list_instruments(&InstrumentListQuery::default()).await?;
    assert!(!all_instruments.is_empty(), "Should have instruments loaded");

    // 1. Dynamic Base Asset Testing
    // Find the most common base asset symbol
    let mut base_counts = std::collections::HashMap::new();
    for inst in &all_instruments {
        *base_counts.entry(inst.base_asset.symbol.clone()).or_insert(0) += 1;
    }
    let (common_base, expected_count) = base_counts.iter().max_by_key(|&(_, count)| count).unwrap();
    
    let base_query = InstrumentListQuery::builder()
        .base_asset_symbols(vec![common_base.clone()])
        .build();
    let base_instruments = persistence.list_instruments(&base_query).await?;
    assert_eq!(base_instruments.len(), *expected_count, 
        "Should find exactly {} instruments with base asset {}", expected_count, common_base);

    // 2. Dynamic Quote Asset Testing
    // Find the most common quote asset symbol
    let mut quote_counts = std::collections::HashMap::new();
    for inst in &all_instruments {
        *quote_counts.entry(inst.quote_asset.symbol.clone()).or_insert(0) += 1;
    }
    let (common_quote, expected_quote_count) = quote_counts.iter().max_by_key(|&(_, count)| count).unwrap();

    let quote_query = InstrumentListQuery::builder()
        .quote_asset_symbols(vec![common_quote.clone()])
        .build();
    let quote_instruments = persistence.list_instruments(&quote_query).await?;
    assert_eq!(quote_instruments.len(), *expected_quote_count, 
        "Should find exactly {} instruments with quote asset {}", expected_quote_count, common_quote);

    // 3. Specific Instrument Targeting
    // Pick the first instrument and try to find it with a very specific query
    let target = &all_instruments[0];
    let specific_query = InstrumentListQuery::builder()
        .venues(vec![target.venue.name.clone()])
        .base_asset_symbols(vec![target.base_asset.symbol.clone()])
        .quote_asset_symbols(vec![target.quote_asset.symbol.clone()])
        .instrument_types(vec![target.instrument_type.clone()])
        .build();
    
    let specific_results = persistence.list_instruments(&specific_query).await?;
    assert!(specific_results.iter().any(|i| i.id == target.id), 
        "Should find the specific target instrument {:?} with specific query", target.symbol);

    // 4. Synthetic Filter Testing
    let synthetic_count = all_instruments.iter().filter(|i| i.synthetic).count();
    let synthetic_query = InstrumentListQuery::builder().synthetic(Some(true)).build();
    let synthetic_results = persistence.list_instruments(&synthetic_query).await?;
    assert_eq!(synthetic_results.len(), synthetic_count, 
        "Should find exactly {} synthetic instruments", synthetic_count);

    let real_count = all_instruments.iter().filter(|i| !i.synthetic).count();
    let real_query = InstrumentListQuery::builder().synthetic(Some(false)).build();
    let real_results = persistence.list_instruments(&real_query).await?;
    assert_eq!(real_results.len(), real_count, 
        "Should find exactly {} real instruments", real_count);

    Ok(())
}
