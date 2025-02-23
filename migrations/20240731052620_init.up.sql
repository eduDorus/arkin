
-- METADATA
-- INSTANCES
CREATE TYPE instance_type AS ENUM ( 'live', 'simulation', 'utility');
CREATE TABLE IF NOT EXISTS instances (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    instance_type instance_type NOT NULL
);


-- STRATEGIES
CREATE TABLE IF NOT EXISTS strategies (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    description TEXT
);
INSERT INTO public.strategies (id,"name",description) VALUES
	 ('1fce35ce-1583-4334-a410-bc0f71c7469b'::uuid,'test_strategy_1','This strategy is only for testing'),
	 ('a2d0951e-9bc6-47a4-b803-e4e0bb4e98a3'::uuid,'test_strategy_2','This strategy is only for testing');


-- VENUES
CREATE TYPE venue_type AS ENUM ('cex', 'dex', 'otc', 'user_funds');
CREATE TABLE IF NOT EXISTS venues (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    venue_type venue_type NOT NULL
);
INSERT INTO public.venues (id,"name","venue_type") VALUES
	 ('48adfe42-29fb-4402-888a-0204bf417e32'::uuid,'binance','cex'::public."venue_type"),
	 ('b8b9dcf2-77ea-4d24-964e-8243bb7298ea'::uuid,'personal','user_funds'::public."venue_type");



-- ASSETS
CREATE TYPE asset_type AS ENUM ('crypto', 'stock', 'fiat', 'commodity');
CREATE TABLE IF NOT EXISTS assets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    symbol TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    asset_type asset_type NOT NULL
);
INSERT INTO public.assets (id,symbol,"name","asset_type") VALUES
	 ('894ff9df-e76e-4b2e-aaec-49988de26a84'::uuid,'BTC','Bitcoin','crypto'::public."asset_type"),
	 ('3091ac12-64a7-4824-9ea5-e1c27e10af6f'::uuid,'ETH','Ethereum','crypto'::public."asset_type"),
	 ('97576805-9c3d-427f-82c4-09df0d796d44'::uuid,'SOL','Solana','crypto'::public."asset_type"),
	 ('5ba12a78-1f89-41b6-87c5-020afb7f680d'::uuid,'USDT','Tether','crypto'::public."asset_type"),
	 ('91e61c74-9e4c-4226-b848-8b96e1ec4941'::uuid,'BNB','Binance Coin','crypto'::public."asset_type");


-- PIPELINES
CREATE TABLE IF NOT EXISTS pipelines (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL
);
INSERT INTO public.pipelines (id,"name",description) VALUES
	 ('df5305b0-3e9b-4b7c-8a13-1406e93f5cc9'::uuid,'test_pipeline','This pipeline is only for testing');



-- INSTRUMENTS
CREATE TYPE instrument_type AS ENUM ('spot', 'perpetual', 'future', 'option');
CREATE TYPE instrument_status AS ENUM ('trading', 'halted', 'setteled');
CREATE TYPE instrument_option_type AS ENUM ('call', 'put');
CREATE TABLE IF NOT EXISTS instruments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    venue_id uuid NOT NULL REFERENCES venues(id),
    symbol TEXT NOT NULL,
    venue_symbol TEXT NOT NULL,
    instrument_type instrument_type NOT NULL,
    base_asset_id uuid NOT NULL REFERENCES assets(id),
    quote_asset_id uuid NOT NULL REFERENCES assets(id),
    margin_asset_id uuid NOT NULL REFERENCES assets(id),
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
INSERT INTO public.instruments (id,venue_id,symbol,venue_symbol,"instrument_type",base_asset_id,quote_asset_id,margin_asset_id,strike,maturity,"option_type",contract_size,price_precision,quantity_precision,base_precision,quote_precision,lot_size,tick_size,status) VALUES
	 ('f5dd7db6-89da-4c68-b62e-6f80b763bef6'::uuid,'48adfe42-29fb-4402-888a-0204bf417e32'::uuid,'perp-btc-usdt@binance','BTCUSDT','perpetual'::public."instrument_type",'894ff9df-e76e-4b2e-aaec-49988de26a84'::uuid,'5ba12a78-1f89-41b6-87c5-020afb7f680d'::uuid,'5ba12a78-1f89-41b6-87c5-020afb7f680d'::uuid,NULL,NULL,NULL,1,2,3,8,8,0.001,0.1,'trading'::public."instrument_status"),
	 ('0a6400f4-abb5-4ff3-8720-cf2eeebef26e'::uuid,'48adfe42-29fb-4402-888a-0204bf417e32'::uuid,'perp-eth-usdt@binance','ETHUSDT','perpetual'::public."instrument_type",'3091ac12-64a7-4824-9ea5-e1c27e10af6f'::uuid,'5ba12a78-1f89-41b6-87c5-020afb7f680d'::uuid,'5ba12a78-1f89-41b6-87c5-020afb7f680d'::uuid,NULL,NULL,NULL,1,2,3,8,8,0.001,0.01,'trading'::public."instrument_status"),
	 ('461c915c-de28-40af-ad5a-cc2a46e6473d'::uuid,'48adfe42-29fb-4402-888a-0204bf417e32'::uuid,'perp-sol-usdt@binance','SOLUSDT','perpetual'::public."instrument_type",'97576805-9c3d-427f-82c4-09df0d796d44'::uuid,'5ba12a78-1f89-41b6-87c5-020afb7f680d'::uuid,'5ba12a78-1f89-41b6-87c5-020afb7f680d'::uuid,NULL,NULL,NULL,1,4,0,8,8,1,0.0100,'trading'::public."instrument_status");



-- ACCOUNTING
-- ACCOUNTS
CREATE TYPE account_owner AS ENUM ('user', 'venue');
CREATE TYPE account_type AS ENUM ('spot', 'margin', 'instrument');
CREATE TABLE IF NOT EXISTS accounts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    instance_id uuid NOT NULL REFERENCES instances(id) ON DELETE CASCADE,
    asset_id uuid NOT,
    venue_id uuid NOT NULL REFERENCES venues(id),
    owner account_owner NOT null,
    account_type account_type NOT NULL
);

-- TRANSFERS
CREATE TYPE transfer_type AS ENUM ('deposit', 'withdrawal', 'trade', 'pnl', 'exchange', 'margin', 'commission', 'interest', 'funding', 'settlement', 'liquidation', 'rebate', 'adjustment');
CREATE TABLE IF NOT EXISTS transfers (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instance_id uuid NOT NULL REFERENCES instances(id) ON DELETE CASCADE,
    transfer_group_id uuid NOT NULL,
    debit_account_id uuid NOT NULL REFERENCES accounts(id),
    credit_account_id uuid NOT NULL REFERENCES accounts(id),
    asset_id uuid NOT NULL,
    amount NUMERIC NOT NULL,
    unit_price NUMERIC NOT NULL,
    transfer_type transfer_type NOT null,
    strategy_id uuid,
    instrument_id uuid
);

-- EVENTS
-- SIGNALS
CREATE TABLE IF NOT EXISTS signals (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instance_id uuid NOT NULL REFERENCES instances(id) ON DELETE CASCADE,
    strategy_id uuid NOT NULL REFERENCES strategies(id),
    instrument_id uuid NOT NULL REFERENCES instruments(id),
    weight NUMERIC NOT NULL
);
CREATE INDEX signals_idx ON signals (instance_id, instrument_id, strategy_id, event_time);


-- EXECUTION ORDERS
CREATE TYPE market_side AS ENUM ('buy', 'sell');
CREATE TYPE execution_order_type AS ENUM ('maker', 'taker', 'vwap', 'twap', 'algo');
CREATE TYPE execution_order_status AS ENUM (
    'new',
    'in_progress',
    'partially_filled',
    'partially_filled_cancelling',
    'partially_filled_cancelled',
    'filled',
    'cancelling',
    'cancelled'
);
CREATE TABLE IF NOT EXISTS execution_orders (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
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
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL
);
CREATE INDEX execution_orders_idx ON execution_orders (instance_id, instrument_id, strategy_id, event_time);



-- VENUE ORDERS
CREATE TYPE venue_order_type AS ENUM (
    'market', 
    'limit',
    'stop_market',
    'stop_limit',
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
    event_time TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    instance_id uuid REFERENCES instances(id) ON DELETE CASCADE,
    strategy_id uuid REFERENCES strategies(id),
    instrument_id uuid NOT NULL REFERENCES instruments(id),
    side market_side NOT NULL,
    order_type venue_order_type NOT NULL,
    time_in_force venue_order_time_in_force NOT NULL,
    price NUMERIC,
    quantity NUMERIC NOT NULL,
    last_fill_price NUMERIC NOT NULL,
    last_fill_quantity NUMERIC NOT NULL,
    last_fill_commission NUMERIC NOT NULL,
    filled_price NUMERIC NOT NULL,
    filled_quantity NUMERIC NOT NULL,
    commission_asset_id uuid REFERENCES assets(id),
    commission NUMERIC NOT NULL,
    status venue_order_status NOT NULL,
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL
);
CREATE INDEX venue_orders_idx ON venue_orders (instance_id, instrument_id, strategy_id, event_time);
