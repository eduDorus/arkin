-- arkin.metrics definition

CREATE TABLE arkin.metrics
(
    `event_time` DateTime64(3, 'UTC') CODEC(Delta(8), ZSTD(3)),
    `instrument_id` LowCardinality(UUID) CODEC(ZSTD(3)),
    `metric_type` LowCardinality(String) CODEC(ZSTD(3)),
    `value` Decimal(18, 8) CODEC(ZSTD(3))
)
ENGINE = ReplacingMergeTree
PARTITION BY toYYYYMM(event_time)
ORDER BY (instrument_id, metric_type, event_time)
SETTINGS index_granularity = 8192;


-- arkin.insights definition

CREATE TABLE arkin.insights
(
    `event_time` DateTime64(3, 'UTC') CODEC(Delta(8), ZSTD(3)),
    `pipeline_id` UUID CODEC(ZSTD(3)),
    `instrument_id` UUID CODEC(ZSTD(3)),
    `feature_id` LowCardinality(String) CODEC(ZSTD(3)),
    `value` Float64 CODEC(ZSTD(3)),
    `insight_type` LowCardinality(String) CODEC(ZSTD(3))
)
ENGINE = ReplacingMergeTree
PARTITION BY toYYYYMM(event_time)
ORDER BY (pipeline_id, instrument_id, feature_id, insight_type, event_time)
SETTINGS index_granularity = 8192;


-- arkin.ticks definition

CREATE TABLE arkin.ticks
(
    `event_time` DateTime64(3, 'UTC') CODEC(Delta(8), ZSTD(3)),
    `instrument_id` UUID CODEC(ZSTD(3)),
    `tick_id` UInt64 CODEC(Delta(8), ZSTD(3)),
    `bid_price` Decimal(18, 8) CODEC(GCD, ZSTD(3)),
    `bid_quantity` Decimal(18, 8) CODEC(GCD, ZSTD(3)),
    `ask_price` Decimal(18, 8) CODEC(GCD, ZSTD(3)),
    `ask_quantity` Decimal(18, 8) CODEC(GCD, ZSTD(3))
)
ENGINE = ReplacingMergeTree
PARTITION BY toYYYYMM(event_time)
ORDER BY (instrument_id, event_time, tick_id)
SETTINGS index_granularity = 8192;


-- arkin.trades definition

CREATE TABLE arkin.trades
(
    `event_time` DateTime64(3, 'UTC') CODEC(Delta(8), ZSTD(3)),
    `instrument_id` UUID CODEC(ZSTD(3)),
    `trade_id` UInt64 CODEC(Delta(8), ZSTD(3)),
    `side` Int8 CODEC(ZSTD(3)),
    `price` Decimal(18, 8) CODEC(GCD, ZSTD(3)),
    `quantity` Decimal(18, 8) CODEC(GCD, ZSTD(3))
)
ENGINE = ReplacingMergeTree
PARTITION BY toYYYYMM(event_time)
ORDER BY (instrument_id, event_time, trade_id)
SETTINGS index_granularity = 8192;


-- arkin.agg_trades definition

CREATE TABLE arkin.agg_trades
(
    `event_time` DateTime64(3, 'UTC') CODEC(Delta(8), ZSTD(3)),
    `instrument_id` LowCardinality(UUID) CODEC(ZSTD(3)),
    `trade_id` UInt64 CODEC(Delta(8), ZSTD(3)),
    `side` Int8 CODEC(ZSTD(3)),
    `price` Decimal(18, 8) CODEC(GCD, ZSTD(3)),
    `quantity` Decimal(18, 8) CODEC(GCD, ZSTD(3))
)
ENGINE = ReplacingMergeTree
PARTITION BY toYYYY(event_time)
ORDER BY (instrument_id, event_time, trade_id)
SETTINGS index_granularity = 8192;