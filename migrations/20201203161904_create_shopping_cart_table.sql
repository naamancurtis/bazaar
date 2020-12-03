CREATE TABLE shopping_carts(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  items VARCHAR[] DEFAULT ARRAY[]::VARCHAR[],
  cart_type SMALLINT NOT NULL,
  price_before_discounts DOUBLE PRECISION NOT NULL DEFAULT 0,
  price_after_discounts DOUBLE PRECISION NOT NULL DEFAULT 0,
  discounts uuid[] DEFAULT ARRAY[]::uuid[],
  currency TEXT NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  last_modified timestamptz NOT NULL DEFAULT NOW()
);

CREATE TRIGGER trigger_last_modified_shopping_carts
  BEFORE UPDATE ON shopping_carts
  FOR EACH ROW
  EXECUTE PROCEDURE update_last_modified_column();
