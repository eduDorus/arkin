/// Parser Architecture & Design
///
/// # Overview
///
/// The parser system is designed to be:
/// - **Config-aware**: Uses `StreamConfig` to determine how to parse each data stream
/// - **Exchange-agnostic**: Produces uniform output types regardless of exchange
/// - **Extensible**: Easy to add new exchanges or stream types
/// - **Robust**: Handles parsing errors gracefully with detailed error reporting
///
/// # Core Concepts
///
/// ## Unified Event Types
///
/// All parsed data is normalized into three event types:
///
/// - `Trade`: Single trade execution with price, quantity, side, timestamp
/// - `Tick`: Market snapshot (best bid/ask, last price, 24h stats)
/// - `Metric`: Market metrics (funding rates, mark prices, liquidations, open interest)
///
/// Each event contains:
/// - `exchange`: Which exchange (binance, okx, bybit, coinbase)
/// - `market`: Market type (spot, perpetual, inverse_perpetual)
/// - `symbol`: Trading pair normalized to exchange format
/// - `timestamp`: Milliseconds since epoch
///
/// ## ExchangeParser Trait
///
/// The `ExchangeParser` trait is the core interface:
///
/// ```ignore
/// pub trait ExchangeParser: Send + Sync {
///     fn parse(&self, msg: &str, config: &StreamConfig) -> Result<Option<MarketEvent>, ParseError>;
/// }
/// ```
///
/// Each exchange implements this trait:
/// - `BinanceParser`
/// - `OkxParser`
/// - `BybitParser`
/// - `CoinbaseParser`
///
/// The parser takes:
/// 1. Raw JSON message from WebSocket
/// 2. `StreamConfig` describing the stream (exchange, market type, stream type)
///
/// Returns:
/// - `Ok(Some(event))` - Successfully parsed into a MarketEvent
/// - `Ok(None)` - Valid message but not the expected type (e.g., control message)
/// - `Err(ParseError)` - Parsing failed with a specific error
///
/// ## ParserFactory
///
/// Central factory for getting the right parser:
///
/// ```ignore
/// let parser = ParserFactory::get_parser("binance")?;
/// let event = parser.parse(json_message, &stream_config)?;
/// ```
///
/// # Design Patterns
///
/// ## Stream Type Dispatch
///
/// Parsers use the `StreamType` enum to dispatch to specialized parsing methods:
///
/// ```ignore
/// match config.stream_type {
///     StreamType::AggregateTrades | StreamType::Trades => self.parse_trade(data, config),
///     StreamType::TickerRealtime | StreamType::Ticker24h => self.parse_tick(data, config),
///     StreamType::MarkPrice | StreamType::FundingRate => self.parse_metric(data, config),
///     _ => Err(ParseError::UnknownStreamType(...)),
/// }
/// ```
///
/// ## Error Handling
///
/// The `ParseError` enum provides specific error information:
///
/// - `JsonParse(String)` - JSON parsing failed
/// - `MissingField(String)` - Required field not in JSON
/// - `InvalidValue(String)` - Field value has wrong type
/// - `UnknownExchange(String)` - Exchange not supported
/// - `UnknownStreamType(String)` - Stream type not supported
///
/// ## Helper Functions
///
/// Module-level helpers reduce boilerplate:
///
/// ```ignore
/// fn parse_number(value: &Value, field: &str) -> Result<f64, ParseError>
/// fn parse_string(value: &Value, field: &str) -> Result<String, ParseError>
/// fn parse_number_opt(value: &Value, field: &str) -> Option<f64>
/// fn parse_string_opt(value: &Value, field: &str) -> Option<String>
/// ```
///
/// # Implementation Example
///
/// See `parser/binance.rs` for a complete implementation example.
/// Each stream type (trades, tickers, metrics) has a dedicated parsing method.
///
/// # Adding a New Exchange
///
/// To add a new exchange (e.g., Kraken):
///
/// 1. Create `src/parser/kraken.rs` with `KrakenParser` struct
/// 2. Implement `ExchangeParser` trait
/// 3. Add match arm in `ParserFactory::get_parser()`
/// 4. Implement exchange-specific parsing logic for each stream type
/// 5. Add comprehensive tests
///
/// # Adding a New Stream Type
///
/// To add a new stream type:
///
/// 1. Add variant to `StreamType` enum in `market_config.rs`
/// 2. Update `MarketEvent` enum if a new event type is needed
/// 3. For each exchange, add parsing logic in their parser implementation
/// 4. Add tests for each exchange
///
/// # Performance Characteristics
///
/// - **Latency**: Parsing is <1ms per message (dominated by JSON parsing)
/// - **Memory**: ~50 bytes per MarketEvent (small, serializable)
/// - **Scalability**: Each stream can be parsed independently
///
/// Testing Strategy
///
/// Each parser has unit tests verifying:
/// - Correct extraction of all required fields
/// - Proper type conversions
/// - Error handling for malformed data
/// - End-to-end parsing of real exchange message formats
