# Bazaar [WIP]

[![CircleCI Build Status](https://circleci.com/gh/naamancurtis/bazaar.svg?style=shield)](https://app.circleci.com/pipelines/github/naamancurtis/bazaar)

## What is this?

Bazaar is a fully async GraphQL Server written in Rust using `Actix Web`,
`Async-GraphQL` and `SQLx`. It implements the basic functionality you would
expect to find on an **e-commerce platform** [(see below for more details)](#functionality).

It's my hope that this project will be a useful example for others to see how to compose a production ready Rust
application from some of the awesome crates in the ecosystem, along with how various
things you would typically see in enterprise applications _eg. Testing,
Observability, CI etc._ can be implemented.

Although this project is intended to be fully functional, certain parts of it
will be mocked/stubbed/out-of-scope to keep the bounds on what this side project does reasonable.

I'm still very much learning, so if you spot any issues or have any improvements
or feedback on anything within the app, please feel free to drop me a message or
raise an issue on the repo.

#### Out of Scope - _at least for now_

- Stock & Stock Management - Will be included via some stub functionality
- Checkout & Payments - This might be included in the future

## Functionality

### Cart Management

As a customer, I want to be able to add items to my cart so that I can purchase
them.

As a customer, I want to be able to remove items from my cart so that I can purchase
them.

As a customer, I want to be able to view all the items in my cart and see how
much they would cost me, so that I can decide to purchase them or not

### Customer Management

As a customer, I want to be able to log in at any point and have the items I
have in my cart maintained, so that I don't have to go back and re-add them.

As a customer, I want to be able to log out of my account so that I know the
device I am on no longer has access to my account.

As a logged in customer, I want to be able to view my personal details so that I
can verify they're correct.

As a logged in customer, I want to be able to edit my personal details so that I
can keep them up to date.

### Authentication

_Breaking the User Story type requirements to explain what's going on here._

Authentication in the application has been implemented with Access & Refresh **JSON Web Tokens**.
Where **Access tokens** are short lived and irrevocable and **Refresh tokens** are longer lived and
revocable.

## Running the App

### Docker

```sh
docker build --tag bazaar --file Dockerfile .
docker run -p 8000:8000 bazaar
```

### CI

The CI pipeline includes checks on `sqlx-data.json`, if
it detects that there have been changes without updating this file it will fail CI. See [preparing for SQLx offline below](#preparing_for_sqlx_offline) for more details.

### GraphQL Schema

_Although it's not pretty_ there's a small binary in the workspace which can be
used to generate the graphql schema for the application and write it to
`schema.graphql`. To do so just run `cargo run --bin schema`. The easiest way to
keep it up to date is to create a basic `pre-commit` hook to run it for you.

### Tooling

| Name                                                                 | Purpose                                                                   | Installation                                    |
| -------------------------------------------------------------------- | ------------------------------------------------------------------------- | ----------------------------------------------- |
| [SQLx CLI](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli) | Database Migrations                                                       | `cargo install --version=0.2.0 sqlx-cli`        |
| [PSQL](https://formulae.brew.sh/formula/libpq)                       | Used predominately for the utilities of `psql`                            | `brew install libpq && brew link --force libpq` |
| [direnv](https://github.com/direnv/direnv)                           | This is just a nice way of managing environment variables within projects | `brew install direnv`                           |

### Environment Variables

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

### Generating RSA Keys

Using OpenSSL to generate key pairs: _replace what's in the `<>`_

Private & Public Key Pair

```sh
openssl genpkey -out <name>.pem -algorithm RSA -pkeyopt rsa_keygen_bits:<len>
```

Extracting Public key

```sh
openssl rsa -in <name>.pem -pubout > <name>.pub
```

### Database Migrations

#### Creating Migrations

```sh
# Ensure $DATABASE_URL is correctly set
sqlx migrate add <migration name>
```

#### Running Migrations

These are set up to run manually with `./scripts/init_db.sh`, optionally you can
pass a skip variable if a docker instance is already running `SKIP_DOCKER=true ./scripts/init_db.sh`

If you need to run them manually, you can do so with the sqlx CLI:

```sh
sqlx migrate run
```

#### Preparing for SQLX offline

CI will fail if the offline SQLX schema hasn't been updated when it should have
been. Again it's probably useful to add this as a `pre-commit` hook on the
project

```sh
cargo sqlx prepare -- --lib
```

## Useful resources if you want to build something similar

- [Luca Palmieri's](https://github.com/LukeMathWalker) - [Zero to Production In Rust](https://www.zero2prod.com/): This is a
  great resource that I would highly recommend if you're interested in Rust
  at all, but particularly if you're interested in building Web
  Applications/APIs
