use sqlx::PgPool;
use std::net::TcpListener;
use uuid::Uuid;

use crate::helpers::{configure_database, TRACING};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub struct IdHolder {
    pub customer: Option<Uuid>,
    pub cart: Option<Uuid>,
}

/// Just used to ensure environment variables that are required at runtime are
/// set to something
fn set_up_env_vars() {
    use std::env::set_var;
    set_var("SECRET_KEY", "TEST KEY");
    set_var("SALT", "TEST SALT");
}

pub async fn spawn_app() -> TestApp {
    lazy_static::initialize(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let mut configuration = bazaar::get_configuration().expect("failed to read configuration");
    set_up_env_vars();

    let database_name = Uuid::new_v4().to_string();
    configuration.set_database_name(database_name);

    let pool = configure_database(&configuration.database).await;

    let server = bazaar::build_app(listener, pool.clone()).expect("failed to bind address");

    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: pool,
    }
}
