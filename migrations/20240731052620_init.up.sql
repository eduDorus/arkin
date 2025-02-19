CREATE TYPE market_side AS ENUM ('buy', 'sell');


-- CREATE INSTANCE TABLE
CREATE TYPE instance_type AS ENUM ( 'live', 'simulation', 'other');
CREATE TABLE IF NOT EXISTS instances (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    instance_type instance_type NOT NULL
);


CREATE TABLE IF NOT EXISTS strategies (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL
);

CREATE TYPE venue_type AS ENUM ('cex', 'dex', 'otc');
CREATE TABLE IF NOT EXISTS venues (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    venue_type venue_type NOT NULL,
);


CREATE TYPE asset_type AS ENUM ('crypto', 'stock', 'forex', 'commodity');
CREATE TABLE IF NOT EXISTS assets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    symbol TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    asset_type asset_type NOT NULL
);

CREATE TABLE IF NOT EXISTS pipelines (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
);


CREATE TYPE instrument_type AS ENUM ('spot', 'perpetual', 'future', 'option');
CREATE TYPE instrument_status AS ENUM ('trading', 'halted');
CREATE TYPE instrument_option_type AS ENUM ('call', 'put');
CREATE TABLE IF NOT EXISTS instruments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    venue_id uuid NOT NULL REFERENCES venues(id),
    symbol TEXT NOT NULL,
    venue_symbol TEXT NOT NULL,
    instrument_type instrument_type NOT NULL,
    base_asset uuid NOT NULL REFERENCES assets(id),
    quote_asset uuid NOT NULL REFERENCES assets(id),
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
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3)
);

CREATE TABLE IF NOT EXISTS portfolios (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3)
);

CREATE TYPE transaction_type AS ENUM (
    'deposit',
    'withdrawal',
    'buy',
    'sell',
    'collateral',
    'dividend',
    'fee',
    'settlement',
    'interest',
    'funding',
    'liquidation',
    'transfer'
    'rebate',
    'adjustment',
    'other'
);
-- Transactions Table (Enhanced)
CREATE TABLE IF NOT EXISTS transactions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    group_id uuid NOT NULL,
    portfolio_id uuid NOT NULL REFERENCES portfolios(id),
    asset_id uuid REFERENCES assets(id),
    instrument_id uuid REFERENCES instruments(id),
    transaction_type transaction_type NOT NULL,
    price NUMERIC,
    quantity NUMERIC NOT NULL,
    total_value NUMERIC
);

-- CREATE TABLE IF NOT EXISTS holdings (
--     id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
--     instance_id uuid NOT NULL REFERENCES instances(id), -- instance_id
--     asset_id uuid NOT NULL REFERENCES assets(id),
--     quantity NUMERIC NOT NULL,
--     created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
--     updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3)
-- );

-- CREATE TYPE position_side AS ENUM ('long', 'short');
-- CREATE TYPE position_status AS ENUM ('open', 'closed');
-- CREATE TABLE IF NOT EXISTS positions (
--     id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
--     instance_id uuid NOT NULL REFERENCES instances(id), -- instance_id
--     strategy_id uuid NOT NULL REFERENCES strategies(id), -- strategy_id
--     instrument_id uuid NOT NULL REFERENCES instruments(id),
--     side position_side NOT NULL,
--     open_price NUMERIC NOT NULL,
--     open_quantity NUMERIC NOT NULL,
--     close_price NUMERIC NOT NULL,
--     close_quantity DECIMAL NOT NULL,
--     realized_pnl DECIMAL NOT NULL,
--     total_commission DECIMAL NOT NULL,
--     status position_status NOT NULL,
--     created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
--     updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3)
-- );

-- CREATE TABLE IF NOT EXISTS venue_order_fills (
--     event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
--     instance_id uuid NOT NULL REFERENCES instances(id), -- instance_id
--     venue_order_id uuid NOT NULL REFERENCES venue_orders(id),
--     instrument_id uuid NOT NULL REFERENCES instruments(id),
--     side market_side NOT NULL,
--     price NUMERIC NOT NULL,
--     quantity NUMERIC NOT NULL,
--     commission NUMERIC NOT NULL,
--     PRIMARY KEY (venue_order_id, instrument_id, instance_id, event_time)
-- );
-- SELECT create_hypertable('venue_order_fills', by_range('event_time', interval '1 day'));
-- SELECT add_dimension('venue_order_fills', by_hash('instrument_id', 4));

CREATE TYPE execution_order_type AS ENUM ('maker', 'taker', 'vwap', 'twap', 'algo');
CREATE TYPE execution_order_status AS ENUM (
    'new',
    'in_progress',
    'partially_filled',
    'partially_filled_cancelling',
    'partially_filled_cancelled',
    'filled',
    'cancelling',
    'cancelled',
);
CREATE TABLE IF NOT EXISTS execution_orders (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    instance_id uuid NOT NULL REFERENCES instances(id) ON DELETE CASCADE,
    strategy_id uuid REFERENCES strategies(id),
    instrument_id uuid NOT NULL REFERENCES instruments(id),
    order_type execution_order_type NOT NULL,
    side market_side NOT NULL,
    price NUMERIC,
    quantity NUMERIC NOT NULL,
    fill_price NUMERIC NOT NULL,
    filled_quantity NUMERIC NOT NULL,
    total_commission NUMERIC NOT NULL,
    status execution_order_status NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL
);


CREATE TYPE venue_order_type AS ENUM (
    'market', 
    'limit'
    'stop',
    'stop_market',
    'take_profit',
    'take_profit_market',
    'trailing_stop_market'
);
CREATE TYPE venue_order_time_in_force AS ENUM (
    'gtc', 
    'ioc', 
    'fok', 
    'gtx',
    'gtd'
);
CREATE TYPE venue_order_status AS ENUM (
    'new',
    'placed',
    'partially_filled',
    'partially_filled_cancelled',
    'partially_filled_expired',
    'filled',
    'cancelled',
    'rejected',
    'expired'
);
CREATE TABLE IF NOT EXISTS venue_orders (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    portfolio_id uuid NOT NULL REFERENCES portfolios(id),
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
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3)
);


CREATE TABLE IF NOT EXISTS insights (
    id uuid DEFAULT gen_random_uuid (),
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    pipeline_id uuid NOT NULL REFERENCES pipelines(id),
    instrument_id uuid REFERENCES instruments(id),
    feature_id TEXT NOT NULL,
    value NUMERIC NOT NULL,
    PRIMARY KEY (pipeline_id, instrument_id, feature_id, event_time)
);
SELECT create_hypertable('insights', by_range('event_time', interval '1 day'));
SELECT add_dimension('insights', by_hash('pipeline_id', 4));



CREATE TABLE IF NOT EXISTS signals (
    id uuid DEFAULT gen_random_uuid (),
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instance_id uuid NOT NULL REFERENCES instances(id) ON DELETE CASCADE,
    strategy_id uuid NOT NULL REFERENCES strategies(id),
    instrument_id uuid NOT NULL REFERENCES instruments(id),
    weight NUMERIC NOT NULL,
    PRIMARY KEY (instance_id, instrument_id, strategy_id, event_time)
);

CREATE TABLE IF NOT EXISTS allocations (
    id uuid DEFAULT gen_random_uuid (),
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    group_id uuid NOT NULL,
    portfolio_id uuid NOT NULL REFERENCES portfolios(id),
    strategy_id uuid NOT NULL REFERENCES strategies(id), 
    instrument_id uuid NOT NULL REFERENCES instruments(id),
    signal_id uuid NOT NULL, -- Can't reference hypertable table
    allocation_weight NUMERIC NOT NULL,
    weight NUMERIC NOT NULL
    PRIMARY KEY (instrument_id, strategy_id, group_id, event_time)
)
SELECT create_hypertable('allocations', by_range('event_time', interval '1 day'));


CREATE TABLE IF NOT EXISTS ticks (
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
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
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
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