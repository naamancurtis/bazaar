use anyhow::Result;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use crate::helpers::IdHolder;

use bazaar::{
    configuration::DatabaseSettings,
    models::{Currency, Customer},
};

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
    pool
}

pub async fn insert_default_customer(pool: &PgPool) -> Result<IdHolder> {
    let email = format!("{}@test.com", Uuid::nil());
    let first_name = Uuid::nil().to_string();
    let last_name = Uuid::nil().to_string();

    let customer = Customer::new(email, first_name, last_name, pool).await;
    dbg!(&customer);
    let customer = customer.expect("failed to insert default customer");
    Ok(IdHolder {
        customer: Some(customer.id),
        cart: None,
    })
}

pub async fn insert_default_customer_with_cart(pool: &PgPool) -> Result<IdHolder> {
    let email = format!("{}@test.com", Uuid::nil());
    let first_name = Uuid::nil().to_string();
    let last_name = Uuid::nil().to_string();

    let customer = Customer::new(email, first_name, last_name, pool)
        .await
        .expect("failed to insert default customer");
    dbg!(&customer);
    let cart = Customer::add_new_cart(customer.id, Currency::GBP, pool)
        .await
        .expect("failed to attach cart to default customer");
    dbg!(&cart);
    Ok(IdHolder {
        customer: Some(customer.id),
        cart: Some(cart.id),
    })
}
