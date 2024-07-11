use crate::{
    features::FeatureEvent,
    models::{BookUpdate, Instrument, Tick, Trade},
};
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use tracing::info;

#[derive(Default)]
#[allow(unused)]
pub struct MarketState {
    book: RwLock<HashMap<Instrument, VecDeque<BookUpdate>>>,
    quotes: RwLock<HashMap<Instrument, VecDeque<Tick>>>,
    trades: RwLock<HashMap<Instrument, VecDeque<Trade>>>,
    agg_trades: RwLock<HashMap<Instrument, VecDeque<Trade>>>,
    features: RwLock<HashMap<Instrument, VecDeque<FeatureEvent>>>,
}

impl MarketState {
    pub fn handle_book_update(&self, book_update: &BookUpdate) {
        let instrument = book_update.instrument.clone();
        let mut locked_book = self.book.write();
        let book = locked_book.entry(instrument).or_default();
        book.push_back(book_update.to_owned());
    }

    pub fn handle_tick_update(&self, tick: &Tick) {
        let instrument = tick.instrument.clone();
        let mut locked_quotes = self.quotes.write();
        let quotes = locked_quotes.entry(instrument).or_default();
        quotes.push_back(tick.to_owned());
    }

    pub fn handle_trade_update(&self, trade: &Trade) {
        let instrument = trade.instrument.clone();
        let mut locked_trades = self.trades.write();
        let trades = locked_trades.entry(instrument).or_default();
        trades.push_back(trade.to_owned());
    }

    pub fn handle_agg_trade_update(&self, trade: &Trade) {
        let instrument = trade.instrument.clone();
        let mut locked_agg_trades = self.agg_trades.write();
        let agg_trades = locked_agg_trades.entry(instrument).or_default();
        agg_trades.push_back(trade.to_owned());
    }

    pub fn handle_feature_update(&self, feature_event: &FeatureEvent) {
        info!("MarketState received feature event: {}", feature_event);
    }
}
