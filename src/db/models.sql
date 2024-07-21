CREATE TABLE IF NOT EXISTS ticks
(
    received_time   DateTime64(3, 'UTC') CODEC (Delta, ZSTD(1)),
    event_time      DateTime64(3, 'UTC') CODEC (Delta, ZSTD(1)),
    instrument      LowCardinality(String) CODEC (ZSTD(1)),
    bid_price       Decimal64(9) CODEC (ZSTD(1)),
    bid_quantity    Decimal64(9) CODEC (ZSTD(1)),
    ask_price       Decimal64(9) CODEC (ZSTD(1)),
    ask_quantity    Decimal64(9) CODEC (ZSTD(1)),
    source          LowCardinality(String) CODEC (ZSTD(1))
)
engine = ReplacingMergeTree()
PRIMARY KEY (source, instrument, event_time)
ORDER BY (source, instrument, event_time);

CREATE TABLE IF NOT EXISTS trades
(
    received_time   DateTime64(3, 'UTC') CODEC (Delta, ZSTD(1)),
    event_time      DateTime64(3, 'UTC') CODEC (Delta, ZSTD(1)),
    instrument      LowCardinality(String) CODEC (ZSTD(1)),
    trade_id        UInt64 CODEC (Delta, ZSTD(1)),
    price           Decimal64(9) CODEC (ZSTD(1)),
    quantity        Decimal64(9) CODEC (ZSTD(1)),
    source          LowCardinality(String) CODEC (ZSTD(1))
)
engine = ReplacingMergeTree()
PRIMARY KEY (source, instrument, trade_id)
ORDER BY (source, instrument, trade_id, event_time);

CREATE TABLE IF NOT EXISTS book_updates
(
    received_time   DateTime64(3, 'UTC') CODEC (Delta, ZSTD(1)),
    event_time       DateTime64(3, 'UTC') CODEC (Delta, ZSTD(1)),
    instrument       LowCardinality(String) CODEC (ZSTD(1)),
    update_id        UInt64 CODEC (Delta, ZSTD(1)),
    bids             Array(Tuple(Decimal64(9), Decimal64(9))) CODEC (ZSTD(1)),
    asks             Array(Tuple(Decimal64(9), Decimal64(9))) CODEC (ZSTD(1)),
    source           LowCardinality(String) CODEC (ZSTD(1))
) ENGINE = ReplacingMergeTree()
PRIMARY KEY (source, instrument, update_id, event_time)
ORDER BY (source, instrument, update_id, event_time);