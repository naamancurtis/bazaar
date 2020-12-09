CREATE TABLE items(
  sku VARCHAR NOT NULL UNIQUE,
  PRIMARY KEY (sku),
  price DOUBLE PRECISION NOT NULL,
  name VARCHAR NOT NULL,
  description VARCHAR NOT NULL DEFAULT '',
  img_src VARCHAR NOT NULL DEFAULT '',
  tags VARCHAR[] NOT NULL DEFAULT ARRAY[]::VARCHAR[]
);
