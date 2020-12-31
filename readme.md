# Bazaar [WIP]

[![CircleCI Build Status](https://circleci.com/gh/naamancurtis/bazaar.svg?style=shield)](https://app.circleci.com/pipelines/github/naamancurtis/bazaar)

## Docker

```sh
docker build --tag bazaar --file Dockerfile .
docker run -p 8000:8000 bazaar
```

## CI

The CI pipeline includes checks on `sqlx-data.json`, if
it detects that there have been changes without updating this file it will fail CI. See [preparing for SQLx offline below](#preparing_for_sqlx_offline) for more details.

## GraphQL Schema

_Although it's not pretty_ there's a small binary in the workspace which can be
used to generate the graphql schema for the application and write it to
`schema.graphql`. To do so just run `cargo run --bin schema`. The easiest way to
keep it up to date is to create a basic `pre-commit` hook to run it for you.

## Tooling

| Name                                                                 | Purpose                                                                   | Installation                                    |
| -------------------------------------------------------------------- | ------------------------------------------------------------------------- | ----------------------------------------------- |
| [SQLx CLI](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli) | Database Migrations                                                       | `cargo install --version=0.2.0 sqlx-cli`        |
| [PSQL](https://formulae.brew.sh/formula/libpq)                       | Used predominately for the utilities of `psql`                            | `brew install libpq && brew link --force libpq` |
| [direnv](https://github.com/direnv/direnv)                           | This is just a nice way of managing environment variables within projects | `brew install direnv`                           |

## Environment Variables

Most environment variables are managed through configuration files found within
the `configuration` directory, however some that are not included in there (and
need to be set up manually in order to run the application) can be found below

| Name                          | Key                         | Description                                               | Example                                 |
| ----------------------------- | --------------------------- | --------------------------------------------------------- | --------------------------------------- |
| Authentication Secret Key     | `SECRET_KEY`                | Holds the secret key used while hashing passwords         | `@TODO`                                 |
| Authentication Salt           | `SALT`                      | Holds the salt key used while hashing passwords           | `@TODO`                                 |
| Private key for Refresh Token | `REFRESH_TOKEN_PRIVATE_KEY` | Holds the private key for signing the refresh token JWTs  | Typical RSA Private Key (`.pem` format) |
| Public key for Refresh Token  | `REFRESH_TOKEN_PUBLIC_KEY`  | Holds the public key for verifying the refresh token JWTs | Typical RSA Public Key (`.pem` format)  |
| Private key for Access Token  | `ACCESS_TOKEN_PRIVATE_KEY`  | Holds the private key for signing the access token JWTs   | Typical RSA Private Key (`.pem` format) |
| Public key for Access Token   | `ACCESS_TOKEN_PUBLIC_KEY`   | Holds the public key for verifying the access token JWTs  | Typical RSA Public Key (`.pem` format)  |

## Generating RSA Keys

Using OpenSSL to generate key pairs: _replace what's in the `<>`_

Private & Public Key Pair

```sh
openssl genpkey -out <name>.pem -algorithm RSA -pkeyopt rsa_keygen_bits:<len>
```

Extracting Public key

```sh
openssl rsa -in <name>.pem -pubout > <name>.pub
```

## Database Migrations

### Creating Migrations

```sh
# Ensure $DATABASE_URL is correctly set
sqlx migrate add <migration name>
```

### Running Migrations

These are set up to run manually with `./scripts/init_db.sh`, optionally you can
pass a skip variable if a docker instance is already running `SKIP_DOCKER=true ./scripts/init_db.sh`

If you need to run them manually, you can do so with the sqlx CLI:

```sh
sqlx migrate run
```

### Preparing for SQLX offline

CI will fail if the offline SQLX schema hasn't been updated when it should have
been. Again it's probably useful to add this as a `pre-commit` hook on the
project

```sh
cargo sqlx prepare -- --lib
```
