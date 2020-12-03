CREATE TABLE customers(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  email TEXT NOT NULL UNIQUE,
  first_name TEXT NOT NULL,
  last_name TEXT NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  last_modified timestamptz NOT NULL DEFAULT NOW(),
  cart_id uuid
);

CREATE INDEX email_address_idx ON customers (email);

CREATE OR REPLACE FUNCTION update_last_modified_column() RETURNS TRIGGER
  LANGUAGE plpgsql AS $$
BEGIN
    NEW.last_modified = NOW();
    RETURN NEW; 
END;    
$$;

CREATE TRIGGER trigger_last_modified
  BEFORE UPDATE ON customers
  FOR EACH ROW
  EXECUTE PROCEDURE update_last_modified_column();
