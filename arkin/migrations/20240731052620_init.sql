-- Add migration script here
CREATE TABLE IF NOT EXISTS stocks_real_time (
  time TIMESTAMPTZ NOT NULL,
  symbol TEXT NOT NULL,
  price DOUBLE PRECISION NULL,
  day_volume INT NULL
);

SELECT create_hypertable('stocks_real_time', by_range('time'));
CREATE INDEX ix_symbol_time ON stocks_real_time (symbol, time DESC);
SELECT add_compression_policy('person', INTERVAL '1 day');