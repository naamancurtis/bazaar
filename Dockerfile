# === Generate a recipe for all of our dependencies ===
FROM rust:1.48 AS planner
WORKDIR app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# === Cache the compilation of our dependencies ===
FROM rust:1.48 AS cacher
WORKDIR app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# === Build our application using the cached dependencies ===
FROM rust:1.48 AS builder
WORKDIR app
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin app

# === Generate a lean runtime for the binary ===
FROM debian:buster-slim AS runtime
WORKDIR app

RUN apt-get update -y \
	&& apt-get install -y --no-install-recommends openssl \
	# Clean up
	&& apt-get autoremove -y \
	&& apt-get clean -y \
	&& rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/app application
COPY configuration configuration
ENV APP_ENVIRONMENT production

ENTRYPOINT ["./application"]
