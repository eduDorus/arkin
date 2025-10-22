CREATE TABLE arkin.instruments_pg (
    id UUID,
    venue_id UUID,
    symbol String,
    venue_symbol String,
    instrument_type String,
    base_asset_id UUID,
    quote_asset_id UUID,
    margin_asset_id UUID,
    strike Decimal(18,8),
    maturity DateTime64(6, 'UTC'),    -- FIXED: 6 digits
    option_type String,
    contract_size Decimal(18,8),
    price_precision Int32,
    quantity_precision Int32,
    base_precision Int32,
    quote_precision Int32,
    lot_size Decimal(18,8),
    tick_size Decimal(18,8),
    status String,
    created DateTime64(6, 'UTC'),     -- FIXED: 6 digits
    updated DateTime64(6, 'UTC')      -- FIXED: 6 digits
) ENGINE = PostgreSQL(
    '192.168.100.100:5432',
    'arkin',
    'instruments',
    'arkin_admin',
    'test1234'
);


CREATE TABLE arkin.notional_imbalance_1min (
    window_start DateTime,
    instrument_id UUID,
    symbol String,
    buy_notional Decimal(18,2),
    sell_notional Decimal(18,2),
    total_notional Decimal(18,2),
    imbalance Float64
) ENGINE = MergeTree()
ORDER BY (window_start, instrument_id);


-- 2. MV: SIMPLE 1min notional imbalance
CREATE MATERIALIZED VIEW arkin.mv_notional_imbalance
TO arkin.notional_imbalance_1min
AS
SELECT
toStartOfMinute(t.event_time) AS window_start,
t.instrument_id ,
sumIf(t.price::Float64 * t.quantity::Float64, t.side = 1)::Decimal(18, 2) AS buy_notional,
sumIf(t.price::Float64 * t.quantity::Float64, t.side = -1)::Decimal(18, 2) AS sell_notional,
sum(t.price::Float64 * t.quantity::Float64)::Decimal(18, 2) AS total_notional,
((buy_notional::Float64 - sell_notional::Float64) / total_notional::Float64)::Float64 AS imbalance
FROM arkin.trades t
GROUP BY window_start, t.instrument_id;
