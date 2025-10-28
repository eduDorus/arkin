use std::sync::Arc;

use arkin_core::prelude::*;

/// Defines the scope of computation for a feature calculation.
/// Maps input instruments (data sources) to an output instrument (result target).
///
/// # Examples
///
/// **Instrument features** (1:1 mapping - read and write to same instrument):
/// ```ignore
/// InstrumentScope {
///     inputs: vec![btc_usdt],
///     output: btc_usdt,
/// }
/// ```
///
/// **Grouped features** (N:1 mapping - aggregate multiple inputs into one synthetic):
/// ```ignore
/// InstrumentScope {
///     inputs: vec![spot_btc_usdt, spot_btc_usdc, perp_btc_usdt, perp_btc_usdc],
///     output: syn_btc_usd_at_binance,
/// }
/// ```
///
/// **Index features** (N:1 mapping - aggregate multiple synthetics into one index):
/// ```ignore
/// InstrumentScope {
///     inputs: vec![syn_btc_usd, syn_eth_usd, syn_bnb_usd],
///     output: index_global,
/// }
/// ```
#[derive(Debug, Clone)]
pub struct InstrumentScope {
    /// Instruments to read data from (for aggregation across multiple sources)
    pub inputs: Vec<Arc<Instrument>>,
    /// Instrument to write results to (the target of the computation)
    pub output: Arc<Instrument>,
}

impl InstrumentScope {
    /// Create a new instrument scope
    pub fn new(inputs: Vec<Arc<Instrument>>, output: Arc<Instrument>) -> Self {
        Self { inputs, output }
    }

    /// Create a simple 1:1 scope where input == output (for instrument-level features)
    pub fn single(instrument: Arc<Instrument>) -> Self {
        Self {
            inputs: vec![Arc::clone(&instrument)],
            output: instrument,
        }
    }
}
