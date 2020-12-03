use async_graphql::{
    validators::{Email, StringMinLength},
    Context, InputObject, Object, Result,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;

use crate::models::{Currency, ShoppingCart};

#[derive(Debug, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct Customer {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub cart_id: Option<Uuid>,
}

#[derive(InputObject, Debug)]
pub struct CustomerUpdate {
    #[graphql(validator(Email))]
    pub email: String,
    #[graphql(validator(StringMinLength(length = "2")))]
    pub first_name: String,
    #[graphql(validator(StringMinLength(length = "2")))]
    pub last_name: String,
}

impl Customer {
    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Self>> {
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

    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    pub async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<Option<Self>> {
        let customer = query_as!(
            Customer,
            r#"
            SELECT * FROM customers WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(customer)
    }

    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    pub async fn find_by_email(email: String, pool: &PgPool) -> Result<Option<Self>> {
        let customer = query_as!(
            Customer,
            r#"
            SELECT * FROM customers WHERE email = $1;
            "#,
            email
        )
        .fetch_optional(pool)
        .await?;
        Ok(customer)
    }

    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    pub async fn new(
        email: String,
        first_name: String,
        last_name: String,
        pool: &PgPool,
    ) -> Result<Self> {
        let new_customer = query_as!(
            Customer,
            r#"
            INSERT INTO customers ( id, email, first_name, last_name )
            VALUES ( $1, $2, $3, $4 )
            RETURNING *;
            "#,
            Uuid::new_v4(),
            email,
            first_name,
            last_name,
        )
        .fetch_one(pool)
        .await?;
        Ok(new_customer)
    }

    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    pub async fn update(id: Uuid, update: CustomerUpdate, pool: &PgPool) -> Result<Self> {
        let updated_customer = query_as!(
            Customer,
            r#"
            UPDATE customers
            SET email = $1, first_name = $2, last_name = $3
            WHERE id = $4
            RETURNING *;
            "#,
            update.email,
            update.first_name,
            update.last_name,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(updated_customer)
    }

    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    pub async fn add_new_cart(id: Uuid, currency: Currency, pool: &PgPool) -> Result<ShoppingCart> {
        if let Some(cart_id) = Customer::check_cart(id, pool).await {
            let cart = ShoppingCart::find_by_id(cart_id, pool).await?;
            if let Some(cart) = cart {
                return Ok(cart);
            }
        }
        let cart_id = Uuid::new_v4();

        let cloned_pool = pool.clone();
        let updated_customer_future = tokio::spawn(async move {
            query!(
                r#"
            UPDATE customers
            SET cart_id = $1
            WHERE id = $2;
            "#,
                cart_id,
                id
            )
            .fetch_one(&cloned_pool)
            .await
        });
        let cloned_pool = pool.clone();
        let new_cart_future =
            tokio::spawn(
                async move { ShoppingCart::new_known(cart_id, currency, &cloned_pool).await },
            );

        let (_, cart) = futures::future::join(updated_customer_future, new_cart_future).await;
        cart?
    }
}

/// Private API
impl Customer {
    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    async fn check_cart(id: Uuid, pool: &PgPool) -> Option<Uuid> {
        let result = query!(
            r#"
            SELECT cart_id FROM customers WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
        if let Some(result) = result {
            return result.cart_id;
        }
        None
    }
}

/// Graphql Resolver
#[Object]
impl Customer {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn email(&self) -> String {
        self.email.clone()
    }

    async fn first_name(&self) -> String {
        self.first_name.clone()
    }

    async fn last_name(&self) -> String {
        self.last_name.clone()
    }
    async fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    async fn last_modified(&self) -> DateTime<Utc> {
        self.last_modified
    }

    async fn cart_id(&self) -> Option<Uuid> {
        self.cart_id
    }

    async fn cart(&self, ctx: &Context<'_>) -> Option<ShoppingCart> {
        let pool = ctx.data::<PgPool>().ok()?;
        if let Some(cart_id) = self.cart_id {
            return match ShoppingCart::find_by_id(cart_id, pool).await {
                Ok(Some(cart)) => Some(cart),
                Ok(None) => None,
                Err(_) => None,
            };
        }
        None
    }
}
