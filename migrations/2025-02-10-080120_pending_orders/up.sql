CREATE TABLE pending_orders
(
    order_id bigserial NOT NULL,
    exchange ExchangeMarket NOT NULL,

    PRIMARY KEY ("order_id", "exchange")  -- Composite primary key
);
