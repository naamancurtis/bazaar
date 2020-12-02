use async_graphql::{
    validators::{Email, StringMinLength},
    Context, EmptySubscription, InputObject, Object, Result, Schema, SimpleObject,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{query_as, PgPool};
use uuid::Uuid;

pub type BazaarSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    #[tracing::instrument(name = "customers", skip(self, ctx))]
    async fn customers(&self, ctx: &Context<'_>) -> Result<Vec<Customer>> {
        let pool = ctx.data::<PgPool>()?;
        match Customer::find_all(pool).await {
            Ok(customers) => Ok(customers),
            // @TODO improve error handling
            Err(e) => Err(async_graphql::Error::new(format!(
                "Message: {}, extensions: {:?}",
                e.message, e.extensions
            ))),
        }
    }

    #[tracing::instrument(name = "customer_by_id", skip(self, ctx))]
    async fn customer_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<Customer>> {
        let pool = ctx.data::<PgPool>()?;
        match Customer::find_by_id(id, pool).await {
            Ok(customer) => Ok(customer),
            // @TODO improve error handling
            Err(e) => Err(async_graphql::Error::new(format!(
                "Message: {}, extensions: {:?}",
                e.message, e.extensions
            ))),
        }
    }

    #[tracing::instrument(name = "customer_by_email", skip(self, ctx))]
    async fn customer_by_email(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(Email))] email: String,
    ) -> Result<Option<Customer>> {
        let pool = ctx.data::<PgPool>()?;
        match Customer::find_by_email(email, pool).await {
            Ok(customer) => Ok(customer),
            // @TODO improve error handling
            Err(e) => Err(async_graphql::Error::new(format!(
                "Message: {}, extensions: {:?}",
                e.message, e.extensions
            ))),
        }
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    #[tracing::instrument(name = "create_customer", skip(self, ctx))]
    async fn create_customer(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(Email))] email: String,
        #[graphql(validator(StringMinLength(length = "2")))] first_name: String,
        #[graphql(validator(StringMinLength(length = "2")))] last_name: String,
    ) -> Result<Customer> {
        let pool = ctx.data::<PgPool>()?;
        Customer::new(email, first_name, last_name, pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.message))
    }

    async fn update_customer(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        update: CustomerUpdate,
    ) -> Result<Customer> {
        let pool = ctx.data::<PgPool>()?;
        Customer::update(id, update, pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.message))
    }
}

#[derive(Debug, SimpleObject, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct Customer {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub created_at: DateTime<Utc>,
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
        let customer = query_as!(Customer, r#"SELECT * FROM customers"#)
            .fetch_all(pool)
            .await?;
        Ok(customer)
    }

    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    pub async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<Option<Self>> {
        let customer = query_as!(Customer, r#"SELECT * FROM customers WHERE id = $1"#, id)
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
        INSERT INTO customers ( id, email, first_name, last_name, created_at )
        VALUES ( $1, $2, $3, $4, $5 )
        RETURNING *;
        "#,
            Uuid::new_v4(),
            email,
            first_name,
            last_name,
            Utc::now()
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
}
