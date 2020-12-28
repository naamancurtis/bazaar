CREATE TABLE auth(
  email TEXT NOT NULL,
  PRIMARY KEY (email),
  public_id uuid NOT NULL UNIQUE,
  id uuid NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  last_modified timestamptz NOT NULL DEFAULT NOW()
);

CREATE INDEX private_id_idx ON auth (id);
