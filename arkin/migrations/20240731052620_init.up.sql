CREATE TABLE IF NOT EXISTS venue (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    venue_type VARCHAR NOT NULL,
);

CREATE TABLE IF NOT EXISTS accounts (
    id SERIAL PRIMARY KEY,
    venue INTEGER NOT NULL REFERENCES venue(id),
    name VARCHAR NOT NULL,
    balance DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS strategies (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    description TEXT
);

CREATE TYPE instrument_contract_type AS ENUM ('spot', 'perpetual', 'future', 'option');

CREATE TYPE instrument_option_type AS ENUM ('call', 'put');

CREATE TYPE instrument_status AS ENUM (
    'pending_trading',
    'trading',
    'halted',
    'pre_delivering',
    'delivering',
    'delivered',
    'pre_settling',
    'settling',
    'close'
);

CREATE TABLE IF NOT EXISTS instruments (
    id SERIAL PRIMARY KEY,
    venue INTEGER NOT NULL REFERENCES venue(id),
    symbol VARCHAR NOT NULL,
    venue_symbol VARCHAR NOT NULL,
    contract_type instrument_contract_type NOT NULL,
    base_asset VARCHAR NOT NULL,
    quote_asset VARCHAR NOT NULL,
    strike DECIMAL,
    maturity TIMESTAMPTZ,
    option_type instrument_option_type,
    contract_size NUMERIC NOT NULL,
    price_precision INTEGER NOT NULL,
    quantity_precision INTEGER NOT NULL,
    base_precision INTEGER NOT NULL,
    quote_precision INTEGER NOT NULL,
    lot_size DECIMAL NOT NULL,
    tick_size DECIMAL NOT NULL,
    status instrument_status NOT NULL
);

CREATE TABLE IF NOT EXISTS signals (
    id SERIAL PRIMARY KEY,
    instrument_id INTEGER NOT NULL REFERENCES instruments(id),
    strategy_id INTEGER NOT NULL REFERENCES strategies(id),
    signal NUMERIC NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL,
);

CREATE TYPE execution_order_side AS ENUM ('buy', 'sell');

CREATE TYPE execution_order_type AS ENUM ('maker', 'taker', 'vwap');

CREATE TYPE execution_order_status AS ENUM (
    'new',
    'open',
    'partially_filled',
    'filled',
    'cancelled',
    'rejected',
);

CREATE TABLE IF NOT EXISTS execution_orders (
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    instrument_id INTEGER NOT NULL REFERENCES instruments(id),
    strategy_id INTEGER NOT NULL REFERENCES strategies(id),
    signal_id INTEGER NOT NULL REFERENCES signals(id),
    side execution_order_side NOT NULL,
    execution_type execution_order_type NOT NULL,
    current_price NUMERIC NOT NULL,
    avg_fill_price NUMERIC NOT NULL,
    quantity NUMERIC NOT NULL,
    filled_quantity NUMERIC NOT NULL,
    total_commission NUMERIC NOT NULL,
    status execution_order_status NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL
);

CREATE TYPE venue_side AS ENUM ('buy', 'sell');

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

CREATE TABLE IF NOT EXISTS venue_orders (
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    instrument_id INTEGER NOT NULL REFERENCES instruments(id),
    strategy_id INTEGER NOT NULL REFERENCES strategies(id),
    execution_order_id INTEGER NOT NULL REFERENCES execution_orders(id),
    venue_order_id BIGINT,
    side venue_side NOT NULL,
    order_type exchange_order_type NOT NULL,
    time_in_force exchange_order_time_in_force NOT NULL,
    price NUMERIC NOT NULL,
    avg_fill_price NUMERIC NOT NULL,
    quantity NUMERIC NOT NULL,
    filled_quantity NUMERIC NOT NULL,
    total_commission NUMERIC NOT NULL,
    status venue_order_status NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL
);

CREATE TYPE fill_side AS ENUM ('buy', 'sell');

CREATE TABLE IF NOT EXISTS fills (
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    instrument_id INTEGER NOT NULL REFERENCES instruments(id),
    strategy_id INTEGER NOT NULL REFERENCES strategies(id),
    execution_order_id INTEGER NOT NULL REFERENCES execution_orders(id),
    venue_order_id BIGINT,
    side fill_side NOT NULL,
    price NUMERIC NOT NULL,
    quantity NUMERIC NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL
);

CREATE TYPE position_side AS ENUM ('long', 'short');

CREATE TYPE position_status AS ENUM ('open', 'closed');

CREATE TABLE IF NOT EXISTS positions (
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    instrument_id INTEGER NOT NULL REFERENCES instruments(id),
    strategy_id INTEGER NOT NULL REFERENCES strategies(id),
    side position_side NOT NULL,
    avg_open_price NUMERIC NOT NULL,
    avg_close_price NUMERIC,
    quantity DECIMAL NOT NULL,
    realized_pnl DECIMAL,
    commission DECIMAL NOT NULL,
    status position_status NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL
);

CREATE TABLE IF NOT EXISTS ticks (
    received_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instrument_id INTEGER NOT NULL REFERENCES instruments(id),
    tick_id BIGINT NOT NULL,
    bid_price NUMERIC(21, 9) NOT NULL,
    bid_quantity NUMERIC(21, 9) NOT NULL,
    ask_price NUMERIC(21, 9) NOT NULL,
    ask_quantity NUMERIC(21, 9) NOT NULL
);

CREATE TABLE IF NOT EXISTS trades (
    received_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instrument_id INTEGER NOT NULL REFERENCES instruments(id),
    trade_id BIGINT NOT NULL,
    price NUMERIC(21, 9) NOT NULL,
    quantity NUMERIC(21, 9) NOT NULL
);