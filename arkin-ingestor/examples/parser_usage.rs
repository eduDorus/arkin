use arkin_ingestor::{
    events::MarketEvent,
    market_config::{Exchange, MarketType, StreamConfig, StreamType},
    parser::ParserFactory,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Parse Binance SPOT trade
    println!("=== Example 1: Binance Spot Trade ===");
    let parser = ParserFactory::get_parser("binance")?;
    let config = StreamConfig {
        exchange: Exchange::Binance,
        market_type: MarketType::Spot,
        stream_type: StreamType::AggregateTrades,
        url: "wss://stream.binance.com:9443/ws/btcusdt@aggTrade".to_string(),
        subscription_message: "{}".to_string(),
        update_frequency_ms: None,
        description: "Binance SPOT aggregate trades".to_string(),
        params: HashMap::new(),
    };

    let binance_trade_json =
        r#"{"e":"aggTrade","E":1234567890000,"s":"BTCUSDT","a":12345,"p":45000.50,"q":1.5,"m":false}"#;

    match parser.parse(binance_trade_json, &config)? {
        Some(MarketEvent::Trade(trade)) => {
            println!("  Exchange: {}", trade.exchange);
            println!("  Symbol: {}", trade.symbol);
            println!("  Price: {}", trade.price);
            println!("  Quantity: {}", trade.quantity);
            println!("  Side: {}", trade.side);
            println!("  Timestamp: {}", trade.timestamp);
        }
        _ => println!("  Unexpected event type"),
    }

    // Example 2: Parse Binance PERPETUAL mark price
    println!("\n=== Example 2: Binance Perpetual Mark Price ===");
    let config_perpetual = StreamConfig {
        exchange: Exchange::Binance,
        market_type: MarketType::Perpetual,
        stream_type: StreamType::MarkPrice,
        url: "wss://fstream.binance.com/ws/btcusdt@markPrice".to_string(),
        subscription_message: "{}".to_string(),
        update_frequency_ms: Some(1000),
        description: "Binance USDT-M mark price".to_string(),
        params: HashMap::new(),
    };

    let binance_metric_json = r#"{"e":"markPriceUpdate","E":1234567890000,"s":"BTCUSDT","p":45100.25}"#;

    match parser.parse(binance_metric_json, &config_perpetual)? {
        Some(MarketEvent::Metric(metric)) => {
            println!("  Exchange: {}", metric.exchange);
            println!("  Metric Type: {}", metric.metric_type);
            println!("  Symbol: {}", metric.symbol);
            println!("  Value: {}", metric.value);
            println!("  Timestamp: {}", metric.timestamp);
        }
        _ => println!("  Unexpected event type"),
    }

    // Example 3: Parse ticker
    println!("\n=== Example 3: Binance Ticker ===");
    let config_ticker = StreamConfig {
        exchange: Exchange::Binance,
        market_type: MarketType::Spot,
        stream_type: StreamType::Ticker24h,
        url: "wss://stream.binance.com:9443/ws/btcusdt@ticker".to_string(),
        subscription_message: "{}".to_string(),
        update_frequency_ms: Some(1000),
        description: "Binance 24h ticker".to_string(),
        params: HashMap::new(),
    };

    let binance_ticker_json = r#"{
        "e":"24hrTicker",
        "E":1234567890000,
        "s":"BTCUSDT",
        "c":45000.00,
        "b":44999.50,
        "a":45000.50,
        "B":5.0,
        "A":10.0,
        "h":46000.00,
        "l":44000.00,
        "v":1000000.0,
        "q":45000000000.0
    }"#;

    match parser.parse(binance_ticker_json, &config_ticker)? {
        Some(MarketEvent::Tick(tick)) => {
            println!("  Exchange: {}", tick.exchange);
            println!("  Symbol: {}", tick.symbol);
            println!("  Bid: {:?}", tick.bid);
            println!("  Ask: {:?}", tick.ask);
            println!("  Last Price: {:?}", tick.last_price);
            println!("  24h High: {:?}", tick.high_24h);
            println!("  24h Low: {:?}", tick.low_24h);
            println!("  24h Volume: {:?}", tick.volume_24h);
        }
        _ => println!("  Unexpected event type"),
    }

    // Example 4: Error handling
    println!("\n=== Example 4: Error Handling ===");
    let invalid_json = "not valid json";
    match parser.parse(invalid_json, &config) {
        Ok(Some(_)) => println!("  Parsed successfully"),
        Ok(None) => println!("  Message was not the expected type"),
        Err(e) => println!("  Error: {}", e),
    }

    // Example 5: Using ParserFactory for different exchanges
    println!("\n=== Example 5: ParserFactory ===");
    for exchange in &["binance", "okx", "bybit", "coinbase"] {
        match ParserFactory::get_parser(exchange) {
            Ok(_) => println!("  {} parser available ✓", exchange),
            Err(_) => println!("  {} parser not available ✗", exchange),
        }
    }

    Ok(())
}
