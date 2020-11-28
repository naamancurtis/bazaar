use bazaar::{build_app, get_configuration};
use sqlx::PgPool;
use std::net::TcpListener;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("failed to read configuration");

    let connection = PgPool::connect(&configuration.database.generate_connection_string())
        .await
        .expect("failed to connect to database");

    let addr = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(addr)?;

    build_app(listener, connection)?.await
}
