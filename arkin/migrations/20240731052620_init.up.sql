CREATE TABLE IF NOT EXISTS instruments (
    instrument_id SERIAL PRIMARY KEY,
    instrument_type TEXT NOT NULL,
    venue TEXT NOT NULL,
    base TEXT NOT NULL,
    quote TEXT NOT NULL,
    maturity TIMESTAMP(3) WITH TIME ZONE, -- Nullable
    strike NUMERIC(21, 9), -- Nullable
    option_type TEXT -- Nullable
);
-- Add a partial unique index to handle NULL values
CREATE UNIQUE INDEX IF NOT EXISTS unique_instruments_idx 
ON instruments (instrument_type, venue, base, quote, maturity, strike, option_type) NULLS NOT DISTINCT;


CREATE TABLE IF NOT EXISTS ticks (
    received_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instrument_id INTEGER NOT NULL REFERENCES instruments,
    tick_id BIGINT NOT NULL,
    bid_price NUMERIC(21, 9) NOT NULL,
    bid_quantity NUMERIC(21, 9) NOT NULL,
    ask_price NUMERIC(21, 9) NOT NULL,
    ask_quantity NUMERIC(21, 9) NOT NULL,
    source TEXT NOT NULL,
    PRIMARY KEY (source, instrument_id, tick_id, event_time)
);
-- Convert the table to a hypertable
SELECT create_hypertable('ticks', 'event_time');
-- Create index
-- CREATE INDEX ix_ticks_instrument_type_venue_base_quote_time ON ticks (instrument_type, venue, base, quote, event_time DESC);
-- Add compression policy
-- SELECT add_compression_policy('ticks', INTERVAL '1 day');



CREATE TABLE IF NOT EXISTS trades (
    received_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instrument_id INTEGER NOT NULL REFERENCES instruments,
    trade_id BIGINT NOT NULL,
    price NUMERIC(21, 9) NOT NULL,
    quantity NUMERIC(21, 9) NOT NULL,
    source TEXT NOT NULL,
    PRIMARY KEY (source, instrument_id, trade_id, event_time)
);

-- Convert the table to a hypertable
SELECT create_hypertable('trades', 'event_time');
-- Create index
-- CREATE INDEX ix_trades_instrument_type_venue_base_quote_time ON trades (instrument_type, venue, base, quote, event_time DESC);
-- Add compression policy
-- SELECT add_compression_policy('trades', INTERVAL '1 day');
