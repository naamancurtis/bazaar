use async_graphql::{validators::Email, Context, Error, ErrorExtensions, Object, Result};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::{
    database::{CustomerDatabase, ShoppingCartDatabase},
    graphql::extract_token_and_database_pool,
    models::{Customer, ShoppingCart, TokenType},
    BazaarError,
};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    // @TODO Remove this - only here for QoL while developing
    #[tracing::instrument(name = "get_customers", skip(self, ctx))]
    async fn customers(&self, ctx: &Context<'_>) -> Result<Vec<Customer>> {
        let pool = ctx.data::<PgPool>()?;
        Customer::find_all::<CustomerDatabase>(pool)
            .await
            .map_err(|err| {
                error!(?err, "failed to fetch all customers");
                Error::new("unable to fetch customers")
            })
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn customer<'ctx>(&self, ctx: &'ctx Context<'_>) -> Result<Customer> {
        let (pool, token) = extract_token_and_database_pool(ctx, TokenType::Access)
            .await
            .map_err(|e| e.extend())?;

        if let Some(id) = token.id() {
            return Customer::find_by_id::<CustomerDatabase>(id, pool)
                .await
                .map_err(|err| {
                    error!(?err, "failed to find customer");
                    BazaarError::NotFound.extend()
                });
        }
        Err(BazaarError::Unauthorized.extend())
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn customer_by_email(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(Email))] email: String,
    ) -> Result<Customer> {
        let pool = ctx.data::<PgPool>()?;
        Customer::find_by_email::<CustomerDatabase>(email, pool)
            .await
            .map_err(|err| {
                error!(?err, "failed to find customer");
                Error::new("unable to find customer")
            })
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn cart_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<ShoppingCart> {
        let pool = ctx.data::<PgPool>()?;
        ShoppingCart::find_by_id::<ShoppingCartDatabase>(id, pool)
            .await
            .map_err(|err| {
                error!(?err, "failed to find customer's cart");
                Error::new("unable to find cart")
            })
    }
}
