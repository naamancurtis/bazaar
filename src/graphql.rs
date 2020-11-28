use async_graphql::{Context, EmptySubscription, Object, Result, Schema, SimpleObject};
use chrono::{DateTime, Utc};
use sqlx::{query_as, PgPool};
use uuid::Uuid;

pub type BazarSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn customer(&self, ctx: &Context<'_>, id: Uuid) -> Result<Customer> {
        let pool = ctx.data::<PgPool>()?;
        match Customer::find(id, pool).await {
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
    async fn create_customer(
        &self,
        ctx: &Context<'_>,
        email: String,
        first_name: String,
        last_name: String,
    ) -> Result<Customer> {
        let pool = ctx.data::<PgPool>()?;
        Customer::new(email, first_name, last_name, pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.message))
    }
}

#[derive(Debug, SimpleObject)]
pub struct Customer {
    id: Uuid,
    email: String,
    first_name: String,
    last_name: String,
    created_at: DateTime<Utc>,
}

impl Customer {
    pub async fn find(id: Uuid, pool: &PgPool) -> Result<Self> {
        let customer = query_as!(Customer, r#"SELECT * FROM customers WHERE id = $1"#, id)
            .fetch_one(pool)
            .await?;
        Ok(customer)
    }

    pub async fn new(
        email: String,
        first_name: String,
        last_name: String,
        pool: &PgPool,
    ) -> Result<Customer> {
        let new_customer = query_as!(
            Customer,
            r#"
				INSERT INTO customers ( id, email, first_name, last_name, created_at )
				VALUES ( $1, $2, $3, $4, $5 )
				RETURNING *
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
}
