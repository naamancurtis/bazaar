CREATE TYPE user_cart_type AS ENUM ('ANONYMOUS', 'KNOWN');

CREATE TYPE currency_type AS ENUM ('GBP', 'USD');

CREATE TABLE shopping_carts(
  id uuid NOT NULL,
  customer_id uuid DEFAULT NULL,
  PRIMARY KEY (id),
  items JSONB DEFAULT '[]'::JSONB,
  cart_type user_cart_type NOT NULL,
  currency currency_type NOT NULL,
  discounts uuid[] DEFAULT ARRAY[]::uuid[],
  price_before_discounts DOUBLE PRECISION NOT NULL DEFAULT 0,
  price_after_discounts DOUBLE PRECISION NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  last_modified timestamptz NOT NULL DEFAULT NOW(),
  CONSTRAINT fk_customer
    FOREIGN KEY(customer_id)
    REFERENCES customers(id)
    ON DELETE CASCADE
);

CREATE TRIGGER trigger_last_modified_shopping_carts
  BEFORE UPDATE ON shopping_carts
  FOR EACH ROW
  EXECUTE PROCEDURE update_last_modified_column();
