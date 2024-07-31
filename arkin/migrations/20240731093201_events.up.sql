CREATE TABLE IF NOT EXISTS orders (
    received_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instrument_type TEXT NOT NULL,
    venue TEXT NOT NULL,
    base TEXT NOT NULL,
    quote TEXT NOT NULL,
    maturity TIMESTAMP(3) WITH TIME ZONE, -- Nullable
    strike NUMERIC(21, 9), -- Nullable
    option_type TEXT, -- Nullable
    order_id BIGINT NOT NULL,
    strategy_id TEXT NOT NULL, -- Reference to strategy
    order_type TEXT NOT NULL, -- e.g., market, limit
    side TEXT NOT NULL, -- e.g., buy, sell
    price NUMERIC(21, 9), -- Nullable for market orders
    avg_fill_price NUMERIC(21, 9), -- Nullable for new orders
    quantity NUMERIC(21, 9) NOT NULL,
    quantity_filled NUMERIC(21, 9) NOT NULL,
    status TEXT NOT NULL, -- e.g., pending, filled, cancelled
    PRIMARY KEY (venue, instrument_type, base, quote, strategy_id, order_id, event_time)
);
-- Convert the table to a hypertable
SELECT create_hypertable('orders', 'event_time');
-- Create index
-- CREATE INDEX ix_orders_instrument_type_venue_base_quote_time ON orders (instrument_type, venue, base, quote, event_time DESC);



CREATE TABLE IF NOT EXISTS fills (
    received_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instrument_type TEXT NOT NULL,
    venue TEXT NOT NULL,
    base TEXT NOT NULL,
    quote TEXT NOT NULL,
    maturity TIMESTAMP(3) WITH TIME ZONE, -- Nullable
    strike NUMERIC(21, 9), -- Nullable
    option_type TEXT, -- Nullable
    order_id BIGINT NOT NULL, -- Reference to order
    strategy_id TEXT NOT NULL, -- Reference to strategy
    price NUMERIC(21, 9) NOT NULL,
    quantity NUMERIC(21, 9) NOT NULL,
    commission NUMERIC(21, 9) NOT NULL, -- Nullable, if applicable
    PRIMARY KEY (venue, instrument_type, base, quote, strategy_id, order_id, event_time)
);
-- Convert the table to a hypertable
SELECT create_hypertable('fills', 'event_time');
-- Create index
-- CREATE INDEX ix_fills_instrument_type_venue_base_quote_time ON fills (instrument_type, venue, base, quote, event_time DESC);



CREATE TABLE IF NOT EXISTS strategy_signals (
    received_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instrument_type TEXT NOT NULL,
    venue TEXT NOT NULL,
    base TEXT NOT NULL,
    quote TEXT NOT NULL,
    maturity TIMESTAMP(3) WITH TIME ZONE, -- Nullable
    strike NUMERIC(21, 9), -- Nullable
    option_type TEXT, -- Nullable
    strategy_id TEXT NOT NULL,
    signal_weight NUMERIC(3, 2) NOT NULL CHECK (signal_weight >= -1 AND signal_weight <= 1),
    PRIMARY KEY (venue, instrument_type, base, quote, strategy_id, event_time)
);
-- Convert the table to a hypertable
SELECT create_hypertable('strategy_signals', 'event_time');
-- Create index
-- CREATE INDEX ix_strategy_signals_instrument_type_venue_base_quote_time ON strategy_signals (instrument_type, venue, base, quote, event_time DESC);



CREATE TABLE IF NOT EXISTS allocations (
    received_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instrument_type TEXT NOT NULL,
    venue TEXT NOT NULL,
    base TEXT NOT NULL,
    quote TEXT NOT NULL,
    maturity TIMESTAMP(3) WITH TIME ZONE, -- Nullable
    strike NUMERIC(21, 9), -- Nullable
    option_type TEXT, -- Nullable
    strategy_id TEXT NOT NULL,
    allocated_notional NUMERIC(21, 9) NOT NULL,
    PRIMARY KEY (venue, instrument_type, base, quote, strategy_id, event_time)
);
-- Convert the table to a hypertable
SELECT create_hypertable('allocations', 'event_time');
-- Create index
-- CREATE INDEX ix_allocations_instrument_type_venue_base_quote_time ON allocations (instrument_type, venue, base, quote, event_time DESC);
