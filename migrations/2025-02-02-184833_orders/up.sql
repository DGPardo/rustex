CREATE TYPE OrderType AS ENUM ('buy', 'sell');
CREATE TYPE ExchangeMarket AS ENUM ('btc_usd', 'btc_gbp', 'btc_eur');

CREATE TABLE orders
(
    order_id bigserial NOT NULL,
    user_id bigserial NOT NULL,
    price bigserial NOT NULL,
    quantity double precision NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),
    order_type OrderType NOT NULL,
    exchange ExchangeMarket NOT NULL,

    PRIMARY KEY ("order_id", "exchange")  -- Composite primary key
);
