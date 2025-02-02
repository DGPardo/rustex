-- Your SQL goes here
CREATE TABLE orders
(
    order_id uuid NOT NULL,
    user_id bigserial NOT NULL,
    price bigserial NOT NULL,
    quantity double precision NOT NULL,
    utc_epoch bigserial NOT NULL,
    buy_order boolean NOT NULL,
    PRIMARY KEY ("order_id", "user_id")
);