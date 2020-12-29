use anyhow::Result;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use crate::helpers::CustomerData;

use bazaar::{configuration::DatabaseSettings, database::CustomerDatabase, models::Customer};

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("failed to connect to database");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("failed to create database");

    let pool = PgPool::connect_with(config.with_db())
        .await
        .expect("failed to connect to database");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run database migrations");
    // Seed the items database for tests
    sqlx::query_file!("scripts/seed_items.sql")
        .execute(&pool)
        .await
        .expect("failed to seed test database");
    pool
}

/// Inserts a new customer
///
/// Creates a new entry for this customer in:
/// 1. Auth Table
/// 2. Customers Table
/// 3. Shopping Carts Table
pub async fn insert_default_customer(pool: &PgPool) -> Result<CustomerData> {
    let mut customer = CustomerData {
        id: None,
        cart_id: Some(Uuid::new_v4()),
        email: Some("imbatman@test.com".to_string()),
        password: Some("Passw0rd".to_string()),
    };
    let ids = Customer::new::<CustomerDatabase>(
        Uuid::new_v4(),
        customer.email.clone().unwrap(),
        customer.password.clone().unwrap(),
        "Bruce".to_string(),
        "Wayne".to_string(),
        customer.cart_id.clone().unwrap(),
        pool,
    )
    .await?;
    customer.id = Some(ids.public_id);
    Ok(customer)
}
