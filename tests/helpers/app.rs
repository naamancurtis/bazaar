use sqlx::PgPool;
use std::net::TcpListener;
use std::sync::Arc;
use uuid::Uuid;

use crate::helpers::{configure_database, set_env_vars_for_tests, TRACING};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub struct IdHolder {
    pub customer: Option<Uuid>,
    pub cart: Option<Uuid>,
}

pub async fn spawn_app() -> TestApp {
    lazy_static::initialize(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let mut configuration = bazaar::get_configuration().expect("failed to read configuration");
    set_env_vars_for_tests();

    let database_name = Uuid::new_v4().to_string();
    configuration.set_database_name(database_name);

    let pool = configure_database(&configuration.database).await;

    let server = bazaar::build_app(listener, pool.clone(), Arc::new(configuration))
        .expect("failed to bind address");

    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: pool,
    }
}
