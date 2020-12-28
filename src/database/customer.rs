use async_trait::async_trait;
use sqlx::{query, query_as, PgPool};
use tracing::error;
use uuid::Uuid;

use crate::{
    database::ShoppingCartDatabase,
    models::{shopping_cart::CartType, Currency, Customer, CustomerUpdate, ShoppingCart},
    Result,
};

#[async_trait]
pub trait CustomerRepository {
    async fn create_new_user(
        public_id: Uuid,
        id: Uuid,
        email: &str,
        password_hash: &str,
        first_name: &str,
        last_name: &str,
        currency: Currency,
        pool: &PgPool,
    ) -> Result<()>;
    async fn find_all(pool: &PgPool) -> Result<Vec<Customer>>;
    async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<Customer>;
    async fn find_by_email(email: String, pool: &PgPool) -> Result<Customer>;
    async fn check_cart(id: Uuid, pool: &PgPool) -> Option<Uuid>;
    async fn update(id: Uuid, update: Vec<CustomerUpdate>, pool: &PgPool) -> Result<()>;
    async fn add_new_cart(
        customer_id: Uuid,
        cart_id: Uuid,
        currency: Currency,
        pool: &PgPool,
    ) -> Result<ShoppingCart>;
}

pub struct CustomerDatabase;

#[async_trait]
impl CustomerRepository for CustomerDatabase {
    #[tracing::instrument(skip(pool), fields(repository = "customer"))]
    async fn find_all(pool: &PgPool) -> Result<Vec<Customer>> {
        let customer = query_as!(
            Customer,
            r#"
            SELECT * FROM customers
            "#
        )
        .fetch_all(pool)
        .await?;
        Ok(customer)
    }

    #[tracing::instrument(skip(pool), fields(repository = "customer"))]
    async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<Customer> {
        let customer = query_as!(
            Customer,
            r#"
            SELECT * FROM customers WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(customer)
    }

    #[tracing::instrument(skip(pool), fields(repository = "customer"))]
    async fn find_by_email(email: String, pool: &PgPool) -> Result<Customer> {
        let customer = query_as!(
            Customer,
            r#"
            SELECT * FROM customers WHERE email = $1;
            "#,
            email
        )
        .fetch_one(pool)
        .await?;
        Ok(customer)
    }

    #[tracing::instrument(skip(pool, password_hash), fields(repository = "customer"))]
    async fn create_new_user(
        public_id: Uuid,
        id: Uuid,
        email: &str,
        password_hash: &str,
        first_name: &str,
        last_name: &str,
        currency: Currency,
        pool: &PgPool,
    ) -> Result<()> {
        let cart_id = Uuid::new_v4();

        let mut tx = pool.begin().await?;

        query!(
            r#"
            INSERT INTO auth (public_id, id, password_hash)
            VALUES ($1, $2, $3)
        "#,
            public_id,
            id,
            password_hash
        )
        .execute(&mut tx)
        .await?;

        query!(
            r#"
            INSERT INTO customers ( id, email, first_name, last_name, cart_id )
            VALUES ( $1, $2, $3, $4, $5)
            "#,
            id,
            email,
            first_name,
            last_name,
            cart_id
        )
        .execute(&mut tx)
        .await?;

        query!(
            r#"
            INSERT INTO shopping_carts (id, customer_id, cart_type, currency)
            VALUES ( $1, $2, $3, $4)
            "#,
            cart_id,
            id,
            CartType::Known as CartType,
            Currency::GBP as Currency
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(skip(pool), fields(repository = "customer"))]
    async fn update(id: Uuid, update: Vec<CustomerUpdate>, pool: &PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;
        let updates: Vec<(&str, String)> = update
            .into_iter()
            .filter_map(|update| {
                if let Some(query) = match update.key.to_lowercase().as_str() {
                    "firstname" => Some("UPDATE customers SET first_name = $1 WHERE id = $2"),
                    "lastname" => Some("UPDATE customers SET last_name = $1 WHERE id = $2"),
                    "email" => Some("UPDATE customers SET email = $1 WHERE id = $2"),
                    err => {
                        error!(
                            key = err,
                            "customer attempted to update key: '{}' but it's not a valid update",
                            err
                        );
                        None
                    }
                } {
                    return Some((query, update.value));
                }
                None
            })
            .collect();

        for (query, value) in updates {
            sqlx::query(query)
                .bind(value)
                .bind(id)
                .execute(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(skip(pool), fields(repository = "customer"))]
    async fn add_new_cart(
        customer_id: Uuid,
        cart_id: Uuid,
        currency: Currency,
        pool: &PgPool,
    ) -> Result<ShoppingCart> {
        use futures::future::join;

        let cloned_pool = pool.clone();
        let updated_customer_future = tokio::spawn(async move {
            query!(
                r#"
            UPDATE customers
            SET cart_id = $1
            WHERE id = $2;
            "#,
                cart_id,
                customer_id
            )
            .fetch_one(&cloned_pool)
            .await
        });
        let cloned_pool = pool.clone();
        let new_cart_future = tokio::spawn(async move {
            ShoppingCart::new_known::<ShoppingCartDatabase>(
                cart_id,
                customer_id,
                currency,
                &cloned_pool,
            )
            .await
        });

        let (_, cart) = join(updated_customer_future, new_cart_future).await;
        cart?
    }

    #[tracing::instrument(skip(pool), fields(repository = "customer"))]
    async fn check_cart(id: Uuid, pool: &PgPool) -> Option<Uuid> {
        if let Some(result) = query!(
            r#"
            SELECT cart_id FROM customers WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        {
            return result.cart_id;
        }
        None
    }
}
