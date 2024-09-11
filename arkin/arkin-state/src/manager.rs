use arkin_core::prelude::*;

pub struct SharedState {
    db_pool: PgPool,
    strategy_cache: Mutex<TwoWayCache<StrategyId, i32>>,
    instrument_cache: Mutex<TwoWayCache<Instrument, i32>>,
    account_cache: Mutex<TwoWayCache<Account, i32>>,
    order_id_counter: Mutex<i64>,
}
