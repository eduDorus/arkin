#![allow(dead_code)]
use core::fmt;
use std::{
    collections::{BTreeSet, HashMap},
    sync::{Arc, LazyLock},
    time::Duration,
};

use arkin_binance::prelude::*;
use async_trait::async_trait;
use dashmap::DashMap;
use flume::{Receiver, Sender};
use futures_util::StreamExt;
use mimalloc::MiMalloc;
use rust_decimal::prelude::*;
use strum::Display;
use time::OffsetDateTime;
use tokio::{signal, sync::Mutex};
use tokio_rustls::rustls::crypto::{aws_lc_rs, CryptoProvider};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info, instrument};

use arkin_core::prelude::*;
use uuid::Uuid;

pub static INSTRUMENT: LazyLock<Arc<Instrument>> = LazyLock::new(|| {
    Arc::new(Instrument {
        id: Uuid::from_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6").expect("Invalid UUID"),
        venue: Venue {
            id: Uuid::parse_str("48adfe42-29fb-4402-888a-0204bf417e32").expect("Invalid UUID"),
            name: "Binance".into(),
            venue_type: "exchange".into(),
        },
        symbol: "perp-btc-usdt@binance".into(),
        venue_symbol: "BTCUSDT".into(),
        instrument_type: InstrumentType::Perpetual,
        base_asset: "btc".into(),
        quote_asset: "usdt".into(),
        maturity: None,
        strike: None,
        option_type: None,
        contract_size: Decimal::from_f64(1.0).expect("Invalid decimal"),
        price_precision: 2,
        quantity_precision: 3,
        base_precision: 8,
        quote_precision: 8,
        tick_size: Decimal::from_f64(0.10).expect("Invalid decimal"),
        lot_size: Decimal::from_f64(0.001).expect("Invalid decimal"),
        status: InstrumentStatus::Trading,
    })
});

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");
    info!("Starting Arkin Order Manager 🚀");

    // let state = Arc::new(AppState::new());
    // let last_tick: Option<Tick> = state.read_last(&instrument).await;

    // Cancellation token
    let token = CancellationToken::new();
    let tracker = TaskTracker::new();

    let state = Arc::new(AppState::new());

    let ingestor = BinanceIngestor::new(
        "wss://fstream.binance.com/ws",
        vec![
            AggTradeStream::new("btcusdt").into(),
            BookTickerStream::from_symbol("btcusdt").into(),
        ],
        state.clone(),
    );

    ingestor.start(tracker.clone(), token.clone()).await;

    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received...");
            token.cancel();
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
            token.cancel();
        }
    }
    info!("Waiting for shutdown to complete...");
    tracker.close();
    tracker.wait().await;
    info!("Shutdown complete");

    let instrument = INSTRUMENT.clone();
    let res: Option<Trade> = state.read_last_trade(&instrument).await;
    match res {
        Some(trade) => {
            info!("Last trade: {}", trade);
        }
        None => {
            info!("No trades found");
        }
    }

    // // Create Portfolio
    // let portfolio = Arc::new(DefaultPortfolio::new());

    // // Create Order Manager
    // let (tx, rx) = flume::unbounded();
    // let executor = Arc::new(BinanceExecutor::new(tx));
    // let order_manager = Arc::new(DefaultOrderManager::new(portfolio.clone(), executor, rx));

    // // Create Allocation Optimizer
    // let allocation_optim = LimitedAllocation::new(order_manager.clone(), portfolio.clone());

    // order_manager.start().await;

    // allocation_optim.new_signal("SOLUSDT".to_string(), Decimal::from(1)).await;
    // allocation_optim.calculate_allocation().await;

    // tokio::time::sleep(Duration::from_secs(3)).await;

    // allocation_optim.new_signal("SOLUSDT".to_string(), Decimal::from(2)).await;
    // allocation_optim.calculate_allocation().await;

    // tokio::time::sleep(Duration::from_secs(5)).await;
    // order_manager.cancel_all_orders().await;
}

#[derive(Debug, Clone)]
pub struct ExecutionOrder {
    id: u16,
    instrument: String,
    side: MarketSide,
    price: Price,
    quantity: Quantity,
    status: ExecutionOrderStatus,
}

impl ExecutionOrder {
    pub fn new(
        instrument: String,
        side: MarketSide,
        price: Price,
        quantity: Quantity,
        status: ExecutionOrderStatus,
    ) -> Self {
        Self {
            id: rand::random::<u16>(),
            instrument,
            side,
            price,
            quantity,
            status,
        }
    }
}

impl fmt::Display for ExecutionOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Execution Order: instrument: {} side: {} price: {} quantity: {} status: {}",
            self.instrument, self.side, self.price, self.quantity, self.status
        )
    }
}

#[derive(Debug, Clone, Display)]
pub enum ExecutionOrderStatus {
    Executing,
    PartiallyFilled,
    PartiallyFilledCanceled,
    Filled,
    Canceled,
}

#[derive(Debug, Clone)]
pub struct Order {
    id: u16,
    instrument: String,
    side: MarketSide,
    price: Price,
    quantity: Quantity,
    status: OrderStatus,
}
impl Order {
    pub fn new(
        id: u16,
        instrument: String,
        side: MarketSide,
        price: Price,
        quantity: Quantity,
        status: OrderStatus,
    ) -> Self {
        Self {
            id,
            instrument,
            side,
            price,
            quantity,
            status,
        }
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Order: id: {} instrument: {} side: {} price: {} quantity: {} status: {}",
            self.id, self.instrument, self.side, self.price, self.quantity, self.status
        )
    }
}

impl From<ExecutionOrder> for Order {
    fn from(order: ExecutionOrder) -> Self {
        Self {
            id: order.id,
            instrument: order.instrument,
            side: order.side,
            price: order.price,
            quantity: order.quantity,
            status: OrderStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Display)]
pub enum OrderStatus {
    Pending,
    Placed,
    Rejected,
    PartiallyFilled,
    Filled,
    Canceled,
}

#[derive(Debug, Clone)]
pub struct Position {
    instrument: String,
    price: Price,
    quantity: Quantity,
}

#[async_trait]
pub trait Engine: Send + Sync {
    async fn start(&self);
}

pub struct DefaultEngine {
    ingestors: Vec<Arc<dyn Ingestor>>,
    portfolio: Arc<dyn Portfolio>,
    allocation_optim: Arc<dyn AllocationOptim>,
    order_manager: Arc<dyn OrderManager>,
}

impl DefaultEngine {
    pub fn new(
        ingestors: Vec<Arc<dyn Ingestor>>,
        portfolio: Arc<dyn Portfolio>,
        allocation_optim: Arc<dyn AllocationOptim>,
        order_manager: Arc<dyn OrderManager>,
    ) -> Self {
        Self {
            ingestors,
            portfolio,
            allocation_optim,
            order_manager,
        }
    }
}

#[async_trait]
impl Engine for DefaultEngine {
    async fn start(&self) {
        info!("Starting Engine");
        // Create task tracker and cancellation token
        let token = CancellationToken::new();
        let tracker = TaskTracker::new();

        for ingestor in &self.ingestors {
            ingestor.start(tracker.clone(), token.clone()).await;
        }

        // self.portfolio.start(tracker.clone(), token.clone()).await;
        // self.allocation_optim.start(tracker.clone(), token.clone()).await;
        self.order_manager.start(tracker.clone(), token.clone()).await;
    }
}

#[async_trait]
pub trait State: std::fmt::Debug + Send + Sync {
    // Trade
    async fn insert_trade(&self, trade: Trade);
    async fn read_last_trade(&self, instrument: &Arc<Instrument>) -> Option<Trade>;
    async fn read_range_trades(
        &self,
        instrument: &Arc<Instrument>,
        from: OffsetDateTime,
        lookback: Duration,
    ) -> BTreeSet<Trade>;

    // Tick
    async fn insert_tick(&self, ticker: Tick);
    async fn read_last_tick(&self, instrument: &Arc<Instrument>) -> Option<Tick>;
    async fn read_range_ticks(
        &self,
        instrument: &Arc<Instrument>,
        from: OffsetDateTime,
        lookback: Duration,
    ) -> BTreeSet<Tick>;
}

#[derive(Debug)]
pub struct AppState {
    trades: DashMap<Arc<Instrument>, BTreeSet<Trade>>,
    ticks: DashMap<Arc<Instrument>, BTreeSet<Tick>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            trades: DashMap::new(),
            ticks: DashMap::new(),
        }
    }
}

#[async_trait]
impl State for AppState {
    #[instrument(skip(self))]
    async fn insert_trade(&self, trade: Trade) {
        let instrument = trade.instrument.clone();
        let mut trades = self.trades.entry(instrument).or_insert_with(BTreeSet::new);
        info!("Inserted trade: {}", trade);
        trades.insert(trade);
    }

    #[instrument(skip(self))]
    async fn read_last_trade(&self, instrument: &Arc<Instrument>) -> Option<Trade> {
        let trades = self.trades.get(instrument).unwrap();
        trades.iter().next_back().cloned()
    }

    #[instrument(skip(self))]
    async fn read_range_trades(
        &self,
        instrument: &Arc<Instrument>,
        from: OffsetDateTime,
        lookback: Duration,
    ) -> BTreeSet<Trade> {
        let trades = self.trades.get(instrument).unwrap();
        let end = from - lookback;
        trades
            .iter()
            .filter(|trade| trade.event_time >= end && trade.event_time < from)
            .cloned()
            .collect()
    }

    #[instrument(skip(self))]
    async fn insert_tick(&self, tick: Tick) {
        let instrument = tick.instrument.clone();
        let mut ticks = self.ticks.entry(instrument).or_insert_with(BTreeSet::new);
        info!("Inserted tick: {}", tick);
        ticks.insert(tick);
    }

    #[instrument(skip(self))]
    async fn read_last_tick(&self, instrument: &Arc<Instrument>) -> Option<Tick> {
        let ticks = self.ticks.get(instrument).unwrap();
        ticks.iter().next_back().cloned()
    }

    #[instrument(skip(self))]
    async fn read_range_ticks(
        &self,
        instrument: &Arc<Instrument>,
        from: OffsetDateTime,
        lookback: Duration,
    ) -> BTreeSet<Tick> {
        let ticks = self.ticks.get(instrument).unwrap();
        let end = from - lookback;
        ticks
            .iter()
            .filter(|tick| tick.event_time >= end && tick.event_time < from)
            .cloned()
            .collect()
    }
}

#[async_trait]
pub trait Ingestor: std::fmt::Debug + Send + Sync {
    async fn start(&self, tracker: TaskTracker, shutdown: CancellationToken);
}

#[derive(Debug)]
pub struct BinanceIngestor {
    url: String,
    streams: Vec<Stream>,
    state: Arc<dyn State>,
}

impl BinanceIngestor {
    #[instrument]
    pub fn new(url: &str, streams: Vec<Stream>, state: Arc<dyn State>) -> Self {
        Self {
            url: url.to_string(),
            streams,
            state,
        }
    }
}

#[async_trait]
impl Ingestor for BinanceIngestor {
    #[instrument(skip(self))]
    async fn start(&self, tracker: TaskTracker, shutdown: CancellationToken) {
        info!("Starting Binance Ingestor");
        let url = self.url.clone();
        let streams = self.streams.clone();
        let state = self.state.clone();

        tracker.spawn(async move {
            let (mut conn, _) = BinanceWebSocketClient::connect_async(&url).await.unwrap();

            conn.subscribe(streams.iter()).await;

            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => {
                        info!("Disconnecting from Binance...");
                        conn.close().await.expect("Failed to disconnect");
                        info!("Disconnected from Binance");
                        break;
                    }
                    message = conn.as_mut().next() => {
                        match message {
                            Some(Ok(message)) => {
                                let data = message.into_data();
                                let res = serde_json::from_slice::<BinanceSwapEvent>(&data);
                                match res {
                                    Ok(e) => {
                                        match e {
                                            BinanceSwapEvent::AggTrade(e) => {
                                                let side = if e.maker {
                                                    MarketSide::Sell
                                                } else {
                                                    MarketSide::Buy
                                                };
                                                let instrument = INSTRUMENT.clone();
                                                let trade = Trade::new(e.event_time, instrument, e.agg_trade_id, side, e.price, e.quantity, );
                                                state.insert_trade(trade).await;
                                            }
                                            BinanceSwapEvent::Tick(e) => {
                                                let instrument = INSTRUMENT.clone();
                                                let tick = Tick::new(e.event_time, instrument, e.update_id, e.bid_price, e.bid_quantity, e.ask_price, e.ask_quantity);
                                                state.insert_tick(tick).await;
                                            }
                                            _ => {}
                                        }
                                    }
                                    Err(e) => {
                                        error!("Error: {:?}", e);
                                        error!("Invalid JSON: {}", String::from_utf8_lossy(&data));
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                error!("Error: {:?}", e);
                            },
                            None => {
                                info!("Connection closed... reconnecting...");
                                let (new_conn, _) = BinanceWebSocketClient::connect_async(&url).await.unwrap();
                                conn = new_conn;
                                conn.subscribe(streams.iter()).await;
                            },
                        }
                    }
                }
            }
        });
    }
}

#[async_trait]
pub trait Portfolio: Send + Sync {
    async fn positions(&self) -> HashMap<String, Position>;
    async fn position_instrument(&self, instrument: String) -> Option<Position>;
    async fn update_position(&self, instrument: String, price: Price, quantity: Quantity);
    async fn reconsile_positions(&self, instrument: String, price: Price, quantity: Quantity);
}

#[derive(Debug)]
pub struct DefaultPortfolio {
    positions: Arc<Mutex<HashMap<String, Position>>>,
}

impl DefaultPortfolio {
    #[instrument]
    pub fn new() -> Self {
        info!("Initializing Default Portfolio");
        Self {
            positions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Portfolio for DefaultPortfolio {
    #[instrument(skip(self))]
    async fn positions(&self) -> HashMap<String, Position> {
        info!("Fetching positions");
        let lock = self.positions.lock().await;
        lock.clone()
    }

    #[instrument(skip(self))]
    async fn position_instrument(&self, instrument: String) -> Option<Position> {
        info!("Fetching position for instrument: {}", instrument);
        let lock = self.positions.lock().await;
        lock.get(&instrument).cloned()
    }

    #[instrument(skip(self))]
    async fn update_position(&self, instrument: String, price: Price, quantity: Quantity) {
        info!(
            "Updating position for instrument: {} price: {} quantity: {}",
            instrument, price, quantity
        );
        let mut lock = self.positions.lock().await;
        if let Some(position) = lock.get_mut(&instrument) {
            let pos_price = position.price;
            let pos_quantity = position.quantity;

            position.price = (pos_price * pos_quantity + price * quantity) / (pos_quantity + quantity);
            position.quantity = pos_quantity + quantity;
            info!(
                "Updated position for instrument: {} price: {} quantity: {}",
                instrument, position.price, position.quantity
            );
        } else {
            lock.insert(
                instrument.clone(),
                Position {
                    instrument,
                    price,
                    quantity,
                },
            );
        }
    }

    #[instrument(skip(self))]
    async fn reconsile_positions(&self, instrument: String, price: Price, quantity: Quantity) {
        info!(
            "Reconciling positions for instrument: {} price: {} quantity: {}",
            instrument, price, quantity
        );
    }
}

#[async_trait]
pub trait AllocationOptim: Send + Sync {
    async fn new_signal(&self, instrument: String, signal: Decimal);
    async fn signals(&self) -> HashMap<String, Decimal>;
    async fn calculate_allocation(&self);
}

pub struct LimitedAllocation {
    order_manager: Arc<dyn OrderManager>,
    portfolio: Arc<dyn Portfolio>,
    signals: Arc<Mutex<HashMap<String, Decimal>>>,
}

impl LimitedAllocation {
    pub fn new(order_manager: Arc<dyn OrderManager>, portfolio: Arc<dyn Portfolio>) -> Self {
        info!("Initializing Limited Allocation Optimizer");
        Self {
            order_manager,
            portfolio,
            signals: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl AllocationOptim for LimitedAllocation {
    async fn new_signal(&self, instrument: String, signal: Decimal) {
        info!("Received new signal for instrument: {} weight: {}", instrument, signal);
        let mut signals = self.signals.lock().await;
        signals.insert(instrument, signal);
    }

    async fn signals(&self) -> HashMap<String, Decimal> {
        info!("Fetching signals");
        let lock = self.signals.lock().await;
        lock.clone()
    }

    async fn calculate_allocation(&self) {
        info!("Calculating allocation");
        // Get the current positions from the portfolio
        let positions = self.portfolio.positions().await;

        // Calculate the allocation based on the signals
        let signals = self.signals().await;

        // Calculate the difference between signals and positions
        let diff = signals
            .iter()
            .map(|(instrument, signal)| {
                if let Some(position) = positions.get(instrument) {
                    (instrument.clone(), signal - position.quantity)
                } else {
                    (instrument.clone(), signal.clone())
                }
            })
            .collect::<HashMap<_, _>>();

        // Create Execution Orders
        let orders = diff
            .into_iter()
            .filter_map(|(instrument, mut quantity)| {
                quantity.rescale(2);

                match quantity {
                    q if q > Decimal::ZERO => Some(ExecutionOrder::new(
                        instrument,
                        MarketSide::Buy,
                        Decimal::new(2000000, 4),
                        q,
                        ExecutionOrderStatus::Executing,
                    )),
                    q if q < Decimal::ZERO => Some(ExecutionOrder::new(
                        instrument,
                        MarketSide::Sell,
                        Decimal::new(2000000, 4),
                        q.abs(),
                        ExecutionOrderStatus::Executing,
                    )),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        // Send the orders to the order manager
        for order in orders {
            self.order_manager.new_order(order).await;
        }
    }
}

#[async_trait]
pub trait OrderManager: Send + Sync {
    async fn start(&self, tracker: TaskTracker, shutdown: CancellationToken);
    async fn new_order(&self, order: ExecutionOrder);
    async fn cancel_order(&self, order: ExecutionOrder);
    async fn cancel_all_orders(&self);
}

pub struct DefaultOrderManager {
    portfolio: Arc<dyn Portfolio>,
    executor: Arc<dyn Executor>,
    notify_rx: Receiver<ExecutorMessage>,
    orders: Arc<Mutex<HashMap<String, ExecutionOrder>>>,
}

impl DefaultOrderManager {
    pub fn new(portfolio: Arc<dyn Portfolio>, executor: Arc<dyn Executor>, rx: Receiver<ExecutorMessage>) -> Self {
        info!("Initializing Default Order Manager");
        Self {
            portfolio,
            executor,
            notify_rx: rx,
            orders: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl OrderManager for DefaultOrderManager {
    async fn start(&self, tracker: TaskTracker, shutdown: CancellationToken) {
        // Connect the executor and pass the sender side of the channel
        let executor = self.executor.clone();
        let portfolio = self.portfolio.clone();
        // let orders = self.orders.clone();

        // Start the executor
        executor.start(tracker.clone(), shutdown.clone()).await;

        // Spawn a task to listen for completed orders
        let rx = self.notify_rx.clone();
        tracker.spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => {
                        info!("Shutdown OrderManager...");
                        break;
                    }
                    Ok(completed_order) = rx.recv_async() => {
                        match completed_order {
                            ExecutorMessage::OrderPlaced(order) => {
                                info!("Order placed: {}", order);
                            }
                            ExecutorMessage::OrderCanceled(order) => {
                                info!("Order canceled: {}", order);
                            }
                            ExecutorMessage::OrderPartiallyFilled(order) => {
                                info!("Order partially filled: {}", order);
                                // portfolio.update_position(order, Decimal::from(1), Decimal::from(1)).await;
                            }
                            ExecutorMessage::OrderFilled(order) => {
                                info!("Order filled: {}", order);
                                portfolio.update_position(order.instrument, order.price, order.quantity).await;
                            }
                            ExecutorMessage::AllOrdersCanceled => {
                                info!("All orders canceled");
                            }
                            ExecutorMessage::Error(order) => {
                                info!("Error on order: {}", order);
                            }
                        }
                    }

                }
            }
        });
    }

    async fn new_order(&self, order: ExecutionOrder) {
        info!("New order: {}", order);
        {
            // First check if order exists
            let queue_order = self.orders.lock().await.get(&order.instrument).cloned();
            if let Some(order) = queue_order {
                info!("Order already exists: {}", order);
                // cancel previous order
                self.cancel_order(order).await;
            }
            self.orders.lock().await.insert(order.instrument.clone(), order.clone());
        }
        let order = Order::from(order);
        self.executor.place_order(order).await;
    }

    async fn cancel_order(&self, order: ExecutionOrder) {
        info!("Canceling order for instrument: {}", order.instrument);
        {
            let mut orders = self.orders.lock().await;
            orders.remove(&order.instrument);
        }
        self.executor.cancel_order(order.into()).await;
    }

    async fn cancel_all_orders(&self) {
        info!("Canceling all orders");
        {
            let mut orders = self.orders.lock().await;
            orders.clear();
        }
        self.executor.cancel_all_orders().await;
    }
}

#[async_trait]
pub trait Executor: Send + Sync {
    async fn start(&self, tracker: TaskTracker, shutdown: CancellationToken);
    async fn place_order(&self, order: Order);
    async fn cancel_order(&self, order: Order);
    async fn cancel_all_orders(&self);
}

#[derive(Debug, Clone, Display)]
pub enum ExecutorMessage {
    OrderPlaced(u16),
    OrderCanceled(u16),
    OrderPartiallyFilled(u16),
    OrderFilled(Order),
    AllOrdersCanceled,
    Error(u16),
}

pub struct SimExecutor {
    response_tx: Sender<ExecutorMessage>,
    open_orders: Arc<Mutex<HashMap<u16, Order>>>,
}

impl SimExecutor {
    pub fn new(tx: Sender<ExecutorMessage>) -> Self {
        Self {
            response_tx: tx,
            open_orders: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Executor for SimExecutor {
    async fn start(&self, tracker: TaskTracker, shutdown: CancellationToken) {
        info!("Connected to simulation executor");
        let open_orders = self.open_orders.clone();
        let tx = self.response_tx.clone();

        tracker.spawn(async move {
            loop {
                if shutdown.is_cancelled() {
                    info!("Stopping SimExecutor...");
                    break;
                }
                // Generate random number between 0-3s
                let delay = rand::random::<u64>() % 4;
                tokio::time::sleep(Duration::from_secs(delay)).await;
                // info!("Executing orders");

                // Drain open_orders
                let mut open_orders_lock = open_orders.lock().await;
                let finished_orders = open_orders_lock.drain().collect::<Vec<_>>();
                drop(open_orders_lock); // Release the lock early

                // Send completed orders through the channel
                for (_id, order) in finished_orders {
                    tx.send_async(ExecutorMessage::OrderFilled(order)).await.unwrap();
                }
            }
        });
    }

    async fn place_order(&self, order: Order) {
        info!("Placed order on exchange: {}", order);
        let id = order.id;

        // Insert in open orders tracking
        let mut open_orders = self.open_orders.lock().await;
        open_orders.insert(id, order);

        // Send respons
        self.response_tx.send_async(ExecutorMessage::OrderPlaced(id)).await.unwrap();
    }

    async fn cancel_order(&self, order: Order) {
        info!("Canceled order: {}", order.id);

        // Remove from open orders tracking
        let mut open_orders = self.open_orders.lock().await;
        open_orders.remove(&order.id);

        // Send response
        self.response_tx
            .send_async(ExecutorMessage::OrderCanceled(order.id))
            .await
            .unwrap();
    }

    async fn cancel_all_orders(&self) {
        info!("Canceled all orders");
        let mut open_orders = self.open_orders.lock().await;
        open_orders.clear();
        self.response_tx.send_async(ExecutorMessage::AllOrdersCanceled).await.unwrap();
    }
}

pub struct BinanceExecutor {
    response_tx: Sender<ExecutorMessage>,
    client: BinanceHttpClient,
    orders: Arc<Mutex<HashMap<u16, Order>>>,
}

impl BinanceExecutor {
    pub fn new(tx: Sender<ExecutorMessage>) -> Self {
        let api_key = "01f41656acbd934ced98ad2c2ecc0b3c41243a82234affc808c1aa88d2c51ed2";
        let api_secret = "d13f2251d6906457ca55e6af92edeb9077a94c059aceebb02794fe19cc7e985c";
        let url = "https://testnet.binancefuture.com";
        let credentials = Credentials::from_hmac(api_key, api_secret);
        let client = BinanceHttpClient::with_url(url).credentials(credentials);
        Self {
            response_tx: tx,
            client,
            orders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn insert_order(&self, order: Order) {
        info!("Inserting order: {}", order);
        let id = order.id;

        let mut orders = self.orders.lock().await;
        orders.insert(id, order);
        info!("Binance executor has orders: {}", orders.len());
    }

    async fn remove_order(&self, order: u16) {
        let mut orders = self.orders.lock().await;
        orders.remove(&order);
        info!("Binance executor has orders: {}", orders.len());
    }

    async fn clear_orders(&self) {
        let mut orders = self.orders.lock().await;
        orders.clear();
        info!("Binance executor has orders: {:?}", orders.len());
    }
}

#[async_trait]
impl Executor for BinanceExecutor {
    async fn start(&self, _tracker: TaskTracker, shutdown: CancellationToken) {
        info!("Connected to Binance executor");

        // On shutdown we want to send a cancel all orders
        shutdown.cancelled().await;
        info!("Stopping BinanceExecutor...");
        info!("Cancelling all open orders...");
        self.cancel_all_orders().await;
        info!("All orders cancelled");
    }

    async fn place_order(&self, order: Order) {
        info!("Placing order on binance exchange: {}", order);

        // Send request
        let req: Request = NewOrder::new(
            &order.instrument,
            match order.side {
                MarketSide::Buy => Side::Buy,
                MarketSide::Sell => Side::Sell,
            },
            "LIMIT",
        )
        .new_client_order_id(&order.id.to_string())
        .price(order.price)
        .quantity(order.quantity)
        .time_in_force(TimeInForce::Gtc)
        .into();

        let res = self.client.send(req).await;
        match res {
            Ok(res) => {
                info!("Response: {:?}", res);
                self.insert_order(order.clone()).await;
                self.response_tx
                    .send_async(ExecutorMessage::OrderPlaced(order.id))
                    .await
                    .unwrap();
            }
            Err(e) => {
                error!("Error: {:?}", e);
                self.response_tx.send_async(ExecutorMessage::Error(order.id)).await.unwrap();
            }
        }
        // Implement the place_order method for BinanceExecutor
    }

    async fn cancel_order(&self, order: Order) {
        info!("Canceling order: {}", order.id);

        let queue_order = {
            let orders = self.orders.lock().await;
            orders.get(&order.id).cloned()
        };

        // Implement the cancel_order method for BinanceExecutor
        if let Some(order) = queue_order {
            let req: Request = CancelOrder::new(&order.instrument)
                .orig_client_order_id(&order.id.to_string())
                .into();
            let res = self.client.send(req).await;
            match res {
                Ok(res) => {
                    info!("Response: {:?}", res);
                    self.remove_order(order.id).await;
                    self.response_tx
                        .send_async(ExecutorMessage::OrderCanceled(order.id))
                        .await
                        .unwrap();
                }
                Err(e) => {
                    error!("Error: {:?}", e);
                    self.response_tx.send_async(ExecutorMessage::Error(order.id)).await.unwrap();
                }
            }
        } else {
            error!("Order not found: {}", order.id);
            self.response_tx.send_async(ExecutorMessage::Error(order.id)).await.unwrap();
        }
    }

    async fn cancel_all_orders(&self) {
        let req: Request = CancelOpenOrders::new("SOLUSDT").into();
        let res = self.client.send(req).await;
        match res {
            Ok(res) => {
                info!("Response: {:?}", res);
                self.clear_orders().await;
                self.response_tx.send_async(ExecutorMessage::AllOrdersCanceled).await.unwrap();
            }
            Err(e) => {
                error!("Error: {:?}", e);
                self.response_tx.send_async(ExecutorMessage::Error(0)).await.unwrap();
            }
        }
    }
}
