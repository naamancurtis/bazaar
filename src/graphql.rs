use async_graphql::{
    validators::{Email, StringMinLength},
    Context, EmptySubscription, Object, Result, Schema,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Currency, Customer, CustomerUpdate, ShoppingCart};

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

    #[tracing::instrument(name = "update_customer", skip(self, ctx))]
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

    #[tracing::instrument(name = "create_anonymous_cart", skip(self, ctx))]
    async fn create_anonymous_cart(
        &self,
        ctx: &Context<'_>,
        currency: Currency,
    ) -> Result<ShoppingCart> {
        let pool = ctx.data::<PgPool>()?;
        ShoppingCart::new_anonymous(currency, pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.message))
    }

    #[tracing::instrument(name = "create_known_cart", skip(self, ctx))]
    async fn create_known_cart(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        currency: Currency,
    ) -> Result<ShoppingCart> {
        let pool = ctx.data::<PgPool>()?;
        Customer::add_new_cart(id, currency, pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.message))
    }
}
