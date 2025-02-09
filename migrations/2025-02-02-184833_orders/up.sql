CREATE TYPE OrderType AS ENUM ('buy', 'sell');

CREATE TABLE orders
(
    order_id bigserial NOT NULL,
    user_id bigserial NOT NULL,
    price bigserial NOT NULL,
    quantity double precision NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),
    order_type OrderType NOT NULL,
    PRIMARY KEY ("order_id", "user_id")
);
