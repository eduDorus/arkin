CREATE TABLE IF NOT EXISTS venues (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS accounts (
    id       SERIAL PRIMARY KEY,
    venue_id INTEGER NOT NULL REFERENCES venues (id),
    name     VARCHAR NOT NULL,
    balance    DECIMAL NOT NULL,
    api_key  VARCHAR,
    api_secret VARCHAR
);

CREATE TABLE IF NOT EXISTS assets (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR NOT NULL
);

CREATE TYPE contract_type AS ENUM ('spot', 'perpetual', 'future', 'option');
CREATE TYPE option_type AS ENUM ('call', 'put');
CREATE TABLE IF NOT EXISTS instruments (
    id SERIAL PRIMARY KEY,
    venue_id INTEGER NOT NULL REFERENCES venues(id),
    symbol VARCHAR NOT NULL,
    venue_symbol VARCHAR NOT NULL,
    contract_type contract_type NOT NULL,
    base_asset_id INTEGER NOT NULL REFERENCES assets(id),
    quote_asset_id INTEGER NOT NULL REFERENCES assets(id),
    strike DECIMAL,
    maturity TIMESTAMPTZ,
    option_type option_type,
    price_precision INTEGER NOT NULL,
    quantity_precision INTEGER NOT NULL,
    base_precision INTEGER NOT NULL,
    quote_precision INTEGER NOT NULL,
    lot_size DECIMAL NOT NULL,
    tick_size DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS strategies (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS signals (
    id SERIAL PRIMARY KEY,
    strategy_id INTEGER NOT NULL REFERENCES strategies(id),
    instrument_id INTEGER NOT NULL REFERENCES instruments(id),
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    signal NUMERIC(3, 2) NOT NULL CHECK (signal >= -1 AND signal <= 1)
);

CREATE TYPE execution_order_type AS ENUM (
    'market',
    'limit'
);
CREATE TABLE IF NOT EXISTS execution_orders (
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    strategy_id INTEGER NOT NULL REFERENCES strategies(id),
    instrument_id INTEGER NOT NULL REFERENCES instruments(id),
    signal_id INTEGER NOT NULL REFERENCES signals(id),
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    order_id BIGINT NOT NULL,
    order_type execution_order_type NOT NULL,
    price NUMERIC,
    quantity DECIMAL NOT NULL,
    status VARCHAR NOT NULL
);

CREATE TYPE exchange_order_type AS ENUM (
    'market',
    'limit'
);
CREATE TYPE time_in_force AS ENUM (
    'gtc',
    'ioc',
    'fok',
    'gtd'
);
CREATE TABLE IF NOT EXISTS exchange_orders (
   id SERIAL PRIMARY KEY,
   account_id INTEGER NOT NULL REFERENCES accounts(id),
   strategy_id INTEGER NOT NULL REFERENCES strategies(id),
   instrument_id INTEGER NOT NULL REFERENCES instruments(id),
   execution_order_id INTEGER NOT NULL REFERENCES execution_orders(id),
   event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
   exchange_order_id BIGINT NOT NULL,
   order_type exchange_order_type NOT NULL,
   time_in_force time_in_force NOT NULL,
   price NUMERIC,
   take_profit NUMERIC,
   stop_loss NUMERIC,
   quantity DECIMAL NOT NULL,
   status VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS fills (
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    strategy_id INTEGER NOT NULL REFERENCES strategies(id),
    instrument_id INTEGER NOT NULL REFERENCES instruments(id),
    order_id INTEGER NOT NULL REFERENCES execution_orders(id),
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    venue_order_id BIGINT, -- Reference to order / can be null for reconsiliation
    price NUMERIC NOT NULL,
    quantity NUMERIC NOT NULL,
    commission NUMERIC NOT NULL
);


CREATE TYPE position_side AS ENUM ('long', 'short');
CREATE TYPE position_status AS ENUM ('open', 'closed');
CREATE TABLE IF NOT EXISTS positions (
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    strategy_id INTEGER NOT NULL REFERENCES strategies(id),
    instrument_id INTEGER NOT NULL REFERENCES instruments(id),
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

