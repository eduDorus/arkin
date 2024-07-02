use std::collections::{HashMap, VecDeque};

use crate::models::{Instrument, Tick, Trade};

#[derive(Default)]
pub struct MarketState {
    quotes: HashMap<Instrument, VecDeque<Tick>>,
    trades: HashMap<Instrument, VecDeque<Trade>>,
    agg_trades: HashMap<Instrument, VecDeque<Trade>>,
}

impl MarketState {
    pub fn handle_tick_update(&mut self, tick: &Tick) {
        let instrument = tick.instrument.clone();
        let quotes = self.quotes.entry(instrument).or_insert_with(VecDeque::new);
        quotes.push_back(tick.to_owned());
    }

    pub fn handle_trade_update(&mut self, trade: &Trade) {
        let instrument = trade.instrument.clone();
        let trades = self.trades.entry(instrument).or_insert_with(VecDeque::new);
        trades.push_back(trade.to_owned());
    }

    pub fn handle_agg_trade_update(&mut self, trade: &Trade) {
        let instrument = trade.instrument.clone();
        let agg_trades = self.agg_trades.entry(instrument).or_insert_with(VecDeque::new);
        agg_trades.push_back(trade.to_owned());
    }
}
