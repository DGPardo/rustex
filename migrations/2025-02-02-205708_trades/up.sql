CREATE TABLE trades
(
    trade_id bigserial NOT NULL,
    exchange ExchangeMarket NOT NULL,
    buy_order bigserial NOT NULL,
    sell_order bigserial NOT NULL,
    price bigserial NOT NULL,
    quantity double precision NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),

    PRIMARY KEY ("trade_id", "exchange")  -- Composite primary key
);
