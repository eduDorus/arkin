use crate::{
    features::FeatureEvent,
    models::{BookUpdate, Instrument, Tick, Trade},
};
use dashmap::DashMap;
use std::collections::BTreeMap;
use time::{Duration, OffsetDateTime};
use tracing::{info, warn};

#[derive(Default)]
#[allow(unused)]
pub struct MarketState {
    book: DashMap<Instrument, BTreeMap<OffsetDateTime, BookUpdate>>,
    quotes: DashMap<Instrument, BTreeMap<OffsetDateTime, Tick>>,
    trades: DashMap<Instrument, BTreeMap<OffsetDateTime, Trade>>,
    agg_trades: DashMap<Instrument, BTreeMap<OffsetDateTime, Trade>>,
    features: DashMap<Instrument, BTreeMap<OffsetDateTime, FeatureEvent>>,
}

impl MarketState {
    pub fn handle_book_update(&self, book_update: &BookUpdate) {
        let instrument = book_update.instrument.clone();
        let mut book = self.book.entry(instrument).or_default();
        book.insert(book_update.event_time, book_update.to_owned());
    }

    pub fn handle_tick_update(&self, tick: &Tick) {
        let instrument = tick.instrument.clone();
        let mut quotes = self.quotes.entry(instrument).or_default();
        quotes.insert(tick.event_time, tick.to_owned());
    }

    pub fn handle_trade_update(&self, trade: &Trade) {
        let instrument = trade.instrument.clone();
        let mut trades = self.trades.entry(instrument).or_default();
        trades.insert(trade.event_time, trade.to_owned());
    }

    pub fn handle_agg_trade_update(&self, trade: &Trade) {
        info!("MarketState received agg trade: {}", trade);
        let instrument = trade.instrument.clone();
        {
            let mut agg_trades = self.agg_trades.entry(instrument.to_owned()).or_default();
            agg_trades.insert(trade.event_time, trade.to_owned());
        }

        // Print current trade history
        let trades = self.get_agg_trades(&instrument, &trade.event_time, &Duration::seconds(5));
        for trade in trades {
            info!("- Trade: {}", trade);
        }
    }

    pub fn handle_feature_update(&self, feature_event: &FeatureEvent) {
        info!("MarketState received feature event: {}", feature_event);
    }

    pub fn get_agg_trades(
        &self,
        instrument: &Instrument,
        start_time: &OffsetDateTime,
        window: &Duration,
    ) -> Vec<Trade> {
        let end_time = *start_time - *window;
        info!(
            "Getting trades for instrument: {} from: {} till: {}",
            instrument, start_time, end_time
        );
        if let Some(trades_map) = self.agg_trades.get(instrument) {
            trades_map
                .range(end_time..=*start_time)
                .map(|(_, trade)| trade)
                .cloned()
                .collect()
        } else {
            warn!("No trades found for instrument: {}", instrument);
            Vec::new()
        }
    }
}
