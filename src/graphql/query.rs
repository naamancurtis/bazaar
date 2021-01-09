use async_graphql::{Context, Error, ErrorExtensions, Object, Result};
use sqlx::PgPool;
use tracing::error;

use crate::{
    database::{CustomerDatabase, ShoppingCartDatabase},
    graphql::extract_token_and_database_pool,
    models::{Customer, CustomerType, ShoppingCart},
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
        let mut context = extract_token_and_database_pool(ctx, true, false)
            .await
            .map_err(|e| e.extend())?;
        let token = context.access_token().map_err(|e| e.extend())?;
        let pool = context.pool;

        if let Some(id) = token.id {
            let mut customer = Customer::find_by_id::<CustomerDatabase>(id, pool)
                .await
                .map_err(|err| {
                    error!(?err, "failed to find customer");
                    BazaarError::NotFound.extend()
                })?;
            customer.id = token.public_id();
            return Ok(customer);
        }
        if token.customer_type == CustomerType::Anonymous {
            return Err(BazaarError::AnonymousError.extend());
        }
        Err(BazaarError::Unauthorized.extend())
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn cart(&self, ctx: &Context<'_>) -> Result<ShoppingCart> {
        let mut context = extract_token_and_database_pool(ctx, true, false)
            .await
            .map_err(|e| e.extend())?;
        let token = context.access_token().map_err(|e| e.extend())?;
        let pool = context.pool;

        ShoppingCart::find_by_id::<ShoppingCartDatabase>(token.cart_id, pool)
            .await
            .map_err(|err| {
                error!(?err, "failed to find customer's cart");
                err.extend()
            })
    }
}
