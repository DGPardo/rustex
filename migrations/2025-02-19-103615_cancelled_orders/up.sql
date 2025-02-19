CREATE TABLE cancelled_orders
(
    order_id bigserial NOT NULL,
    exchange ExchangeMarket NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),

    PRIMARY KEY ("order_id", "exchange")  -- Composite primary key
);
