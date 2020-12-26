use bazaar::{
    build_app, get_configuration,
    telemetry::{generate_subscriber, init_subscriber},
};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let subscriber = generate_subscriber(String::from("bazaar"), String::from("info"));
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("failed to read configuration");

    let connection = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with(configuration.database.with_db())
        .await
        .expect("failed to connect to database");

    let listener = TcpListener::bind(configuration.get_addr())?;

    build_app(listener, connection)?.await
}
