use typed_builder::TypedBuilder;

use crate::stores::{AssetStore, InstrumentStore, TickStore, TradeStore, VenueStore};

#[derive(Debug, Clone, TypedBuilder)]
pub struct StateService {
    pub venue_store: VenueStore,
    pub asset_store: AssetStore,
    pub instrument_store: InstrumentStore,
    pub tick_store: TickStore,
    pub trade_store: TradeStore,
}
