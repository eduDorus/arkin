use derive_builder::Builder;

use crate::stores::{AssetStore, InstrumentStore, TickStore, TradeStore, VenueStore};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct StateService {
    pub venue_store: VenueStore,
    pub asset_store: AssetStore,
    pub instrument_store: InstrumentStore,
    pub tick_store: TickStore,
    pub trade_store: TradeStore,
}
