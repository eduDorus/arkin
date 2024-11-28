CREATE TYPE market_side AS ENUM ('buy', 'sell');
CREATE TYPE option_type AS ENUM ('call', 'put');
CREATE TYPE instrument_type AS ENUM ('spot', 'perpetual', 'future', 'option');
CREATE TYPE instrument_status AS ENUM ('trading', 'halted');
CREATE TYPE execution_order_type AS ENUM ('maker', 'taker', 'vwap');
CREATE TYPE execution_order_status AS ENUM (
    'new',
    'open',
    'partially_filled',
    'filled',
    'cancelled',
    'rejected'
);
CREATE TYPE venue_order_type AS ENUM ('market', 'limit');
CREATE TYPE venue_order_time_in_force AS ENUM ('gtc', 'ioc', 'fok', 'gtd');
CREATE TYPE venue_order_status AS ENUM (
    'new',
    'open',
    'partially_filled',
    'filled',
    'cancelled',
    'rejected',
    'expired'
);
CREATE TYPE position_side AS ENUM ('long', 'short');
CREATE TYPE position_status AS ENUM ('open', 'closed');


CREATE TABLE IF NOT EXISTS venues (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    name VARCHAR NOT NULL UNIQUE,
    venue_type VARCHAR NOT NULL
);


CREATE TABLE IF NOT EXISTS instruments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    venue_id uuid NOT NULL REFERENCES venues(id),
    symbol VARCHAR NOT NULL,
    venue_symbol VARCHAR NOT NULL,
    instrument_type instrument_type NOT NULL,
    base_asset VARCHAR NOT NULL,
    quote_asset VARCHAR NOT NULL,
    strike DECIMAL,
    maturity TIMESTAMPTZ,
    option_type option_type,
    contract_size NUMERIC NOT NULL,
    price_precision INTEGER NOT NULL,
    quantity_precision INTEGER NOT NULL,
    base_precision INTEGER NOT NULL,
    quote_precision INTEGER NOT NULL,
    lot_size DECIMAL NOT NULL,
    tick_size DECIMAL NOT NULL,
    status instrument_status NOT NULL
);


CREATE TABLE IF NOT EXISTS positions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    instrument_id uuid NOT NULL REFERENCES instruments(id),
    side position_side NOT NULL,
    open_price NUMERIC NOT NULL,
    open_quantity NUMERIC NOT NULL,
    close_price NUMERIC NOT NULL,
    close_quantity DECIMAL NOT NULL,
    realized_pnl DECIMAL NOT NULL,
    total_commission DECIMAL NOT NULL,
    status position_status NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL
);


CREATE TABLE IF NOT EXISTS execution_orders (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    instrument_id uuid NOT NULL REFERENCES instruments(id),
    side market_side NOT NULL,
    execution_type execution_order_type NOT NULL,
    fill_price NUMERIC NOT NULL,
    filled_quantity NUMERIC NOT NULL,
    total_commission NUMERIC NOT NULL,
    status execution_order_status NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL
);


CREATE TABLE IF NOT EXISTS venue_orders (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    execution_order_id uuid NOT NULL REFERENCES execution_orders(id),
    instrument_id uuid NOT NULL REFERENCES instruments(id),
    side market_side NOT NULL,
    order_type venue_order_type NOT NULL,
    time_in_force venue_order_time_in_force NOT NULL,
    price NUMERIC,
    quantity NUMERIC NOT NULL,
    fill_price NUMERIC NOT NULL,
    filled_quantity NUMERIC NOT NULL,
    total_commission NUMERIC NOT NULL,
    status venue_order_status NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL
);


CREATE TABLE IF NOT EXISTS insights (
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instrument_id uuid REFERENCES instruments(id),
    feature_id VARCHAR NOT NULL,
    value NUMERIC NOT NULL,
    PRIMARY KEY (instrument_id, feature_id, event_time)
);
SELECT create_hypertable('insights', by_range('event_time', interval '1 day'));
SELECT add_dimension('insights', by_hash('instrument_id', 4));



CREATE TABLE IF NOT EXISTS signals (
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instrument_id uuid NOT NULL REFERENCES instruments(id),
    strategy_id VARCHAR NOT NULL,
    signal NUMERIC NOT NULL,
    PRIMARY KEY (instrument_id, strategy_id, event_time)
);
SELECT create_hypertable('signals', by_range('event_time', interval '7 day'));
SELECT add_dimension('signals', by_hash('instrument_id', 4));



CREATE TABLE IF NOT EXISTS venue_order_fills (
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    venue_order_id uuid NOT NULL REFERENCES venue_orders(id),
    execution_order_id uuid NOT NULL REFERENCES execution_orders(id),
    instrument_id uuid NOT NULL REFERENCES instruments(id),
    side market_side NOT NULL,
    price NUMERIC NOT NULL,
    quantity NUMERIC NOT NULL,
    PRIMARY KEY (venue_order_id, execution_order_id, instrument_id, event_time)
);
SELECT create_hypertable('venue_order_fills', by_range('event_time', interval '7 day'));
SELECT add_dimension('venue_order_fills', by_hash('instrument_id', 4));



CREATE TABLE IF NOT EXISTS ticks (
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instrument_id uuid NOT NULL REFERENCES instruments(id),
    tick_id BIGINT NOT NULL,
    bid_price NUMERIC NOT NULL,
    bid_quantity NUMERIC NOT NULL,
    ask_price NUMERIC NOT NULL,
    ask_quantity NUMERIC NOT NULL,
    PRIMARY KEY (instrument_id, tick_id, event_time)
);
SELECT create_hypertable('ticks', by_range('event_time', interval '1 day'));
SELECT add_dimension('ticks', by_hash('instrument_id', 4));



CREATE TABLE IF NOT EXISTS trade (
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instrument_id uuid NOT NULL REFERENCES instruments(id),
    trade_id BIGINT NOT NULL,
    side market_side NOT NULL,
    price NUMERIC NOT NULL,
    quantity NUMERIC NOT NULL
    PRIMARY KEY (instrument_id, trade_id, event_time)
);
SELECT create_hypertable('trades', by_range('event_time', interval '1 day'));
SELECT add_dimension('trades', by_hash('instrument_id', 4));





-- INITIAL DATA
INSERT INTO
    venues (id, name, venue_type)
VALUES
    (
        '48adfe42-29fb-4402-888a-0204bf417e32',
        'binance',
        'exchange'
    );

INSERT INTO
    instruments (
        id,
        venue_id,
        symbol,
        venue_symbol,
        instrument_type,
        base_asset,
        quote_asset,
        strike,
        maturity,
        option_type,
        contract_size,
        price_precision,
        quantity_precision,
        base_precision,
        quote_precision,
        lot_size,
        tick_size,
        status
    )
VALUES
    (
        'f5dd7db6-89da-4c68-b62e-6f80b763bef6',
        '48adfe42-29fb-4402-888a-0204bf417e32',
        'perp-btc-usdt@binance',
        'BTCUSDT',
        'perpetual',
        'btc',
        'usdt',
        null,
        null,
        null,
        1,
        2,
        3,
        8,
        8,
        0.001,
        0.1,
        'trading'
    );

INSERT INTO
    instruments (
        id,
        venue_id,
        symbol,
        venue_symbol,
        instrument_type,
        base_asset,
        quote_asset,
        strike,
        maturity,
        option_type,
        contract_size,
        price_precision,
        quantity_precision,
        base_precision,
        quote_precision,
        lot_size,
        tick_size,
        status
    )
VALUES
    (
        '0a6400f4-abb5-4ff3-8720-cf2eeebef26e',
        '48adfe42-29fb-4402-888a-0204bf417e32',
        'perp-eth-usdt@binance',
        'ETHUSDT',
        'perpetual',
        'eth',
        'usdt',
        null,
        null,
        null,
        1,
        2,
        3,
        8,
        8,
        0.001,
        0.01,
        'trading'
    );