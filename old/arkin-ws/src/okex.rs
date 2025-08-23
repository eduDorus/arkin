// Okx structs (from docs, simplified for tickers/trades)
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OkxEvent {
    SubscribeSuccess(OkxSubResp),
    Error(OkxError),
    Tickers(OkxData<OkxTicker>),
    Trades(OkxData<OkxTrade>),
}

#[derive(Debug, Deserialize)]
pub struct OkxSubResp {
    event: String,
    arg: OkxArg,
    #[serde(rename = "connId")]
    conn_id: String,
}

#[derive(Debug, Deserialize)]
pub struct OkxError {
    event: String,
    code: String,
    msg: String,
    #[serde(rename = "connId")]
    conn_id: String,
}

#[derive(Debug, Deserialize)]
pub struct OkxData<T> {
    arg: OkxArg,
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct OkxArg {
    channel: String,
    #[serde(rename = "instId")]
    inst_id: String,
}

#[derive(Debug, Deserialize)]
pub struct OkxTicker {
    #[serde(rename = "instType")]
    inst_type: String,
    #[serde(rename = "instId")]
    inst_id: String,
    last: Decimal,
    #[serde(rename = "lastSz")]
    last_sz: Decimal,
    #[serde(rename = "askPx")]
    ask_px: Decimal,
    #[serde(rename = "askSz")]
    ask_sz: Decimal,
    #[serde(rename = "bidPx")]
    bid_px: Decimal,
    #[serde(rename = "bidSz")]
    bid_sz: Decimal,
    #[serde(rename = "open24h")]
    open_24h: Decimal,
    #[serde(rename = "high24h")]
    high_24h: Decimal,
    #[serde(rename = "low24h")]
    low_24h: Decimal,
    #[serde(rename = "volCcy24h")]
    vol_ccy_24h: Decimal,
    #[serde(rename = "vol24h")]
    vol_24h: Decimal,
    ts: u64, // Millis, convert to UtcDateTime if needed
}

#[derive(Debug, Deserialize)]
pub struct OkxTrade {
    #[serde(rename = "instId")]
    inst_id: String,
    #[serde(rename = "tradeId")]
    trade_id: String,
    px: Decimal,
    sz: Decimal,
    side: String, // "buy"/"sell"
    ts: u64,
}

// Okx config
pub struct OkxMarketData {
    channels: Vec<(String, String)>, // (channel, instId) e.g., ("tickers", "BTC-USDT")
}

#[derive(Serialize)]
struct OkxSubscription {
    op: String,
    args: Vec<OkxSubArg>,
}

#[derive(Serialize)]
struct OkxSubArg {
    channel: String,
    #[serde(rename = "instId")]
    inst_id: String,
}

impl WSConfig for OkxMarketData {
    type Inbound = OkxEvent;

    fn url(&self) -> Url {
        Url::parse("wss://ws.okx.com:8443/ws/v5/public").expect("Invalid base URL")
    }

    fn auth_strategy(&self) -> AuthStrategy {
        AuthStrategy::None
    }

    fn initial_subscribe(&self) -> Option<Message> {
        let args: Vec<OkxSubArg> = self
            .channels
            .iter()
            .map(|(ch, inst)| OkxSubArg {
                channel: ch.clone(),
                inst_id: inst.clone(),
            })
            .collect();
        let sub = OkxSubscription {
            op: "subscribe".to_string(),
            args,
        };
        Some(Message::Text(serde_json::to_string(&sub).expect("Serialize fail")))
    }

    fn ping_interval(&self) -> Duration {
        Duration::ZERO // No proactive, respond to server pings
    }

    fn format_ping(&self) -> Message {
        Message::Ping(Vec::new().into()) // Unused since interval=0
    }

    fn parse_inbound(&self, msg: &str) -> Result<Self::Inbound, WebsocketError> {
        debug!(target: "ws", "received msg from ws: {}", msg);
        serde_json::from_str(msg).map_err(|e| WebsocketError::UnexpectedError(e.to_string()))
    }
}

impl ExchangeMarketData for OkxMarketData {}

// For Binance: Adjust ping_interval to ZERO, format_ping to Pong(Vec::new()) if server sends Ping("ping")—check docs, Binance sends {"id":null,"method":"ping"} or raw Ping?

// In tests: Add similar for Okx
#[tokio::test]
async fn subscribe_okx_tickers() {
    let config = OkxMarketData {
        channels: vec![("tickers".to_string(), "BTC-USDT".to_string())],
    };
    let mut manager = MarketDataWSManager::new(config, 1, 100);
    let (inbound_tx, inbound_rx) = kanal::unbounded_async::<OkxEvent>();
    let shutdown = CancellationToken::new();

    let shutdown_token = shutdown.clone();
    let handle = tokio::spawn(async move {
        manager.run(inbound_tx, shutdown_token.clone()).await.unwrap();
    });

    while let Ok(Ok(event)) = timeout(Duration::from_secs(5), inbound_rx.recv()).await {
        match event {
            OkxEvent::SubscribeSuccess(_) => info!(target: "ws", "received subscribe confirmation"),
            OkxEvent::Tickers(data) => {
                info!(target: "ws", "Received tickers: {:?}", data);
                assert_eq!(data.arg.channel, "tickers");
                break;
            }
            e => panic!("unexpected: {:?}", e),
        }
    }

    shutdown.cancel();
    handle.await.unwrap();
}
