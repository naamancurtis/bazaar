use async_graphql::{validators::Email, Context, Error, Object, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    database::{CustomerDatabase, ShoppingCartDatabase},
    error::generate_error_log,
    models::{Customer, ShoppingCart},
};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    #[tracing::instrument(name = "get_customers", skip(self, ctx))]
    async fn customers(&self, ctx: &Context<'_>) -> Result<Vec<Customer>> {
        let pool = ctx.data::<PgPool>()?;
        Customer::find_all::<CustomerDatabase>(pool)
            .await
            .map_err(|err| {
                generate_error_log(err, None);
                Error::new("unable to fetch customers")
            })
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn customer_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<Customer> {
        let pool = ctx.data::<PgPool>()?;
        Customer::find_by_id::<CustomerDatabase>(id, pool)
            .await
            .map_err(|err| {
                generate_error_log(err, None);
                Error::new("unable to find customer")
            })
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
                generate_error_log(err, None);
                Error::new("unable to find customer")
            })
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn cart_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<ShoppingCart> {
        let pool = ctx.data::<PgPool>()?;
        ShoppingCart::find_by_id::<ShoppingCartDatabase>(id, pool)
            .await
            .map_err(|err| {
                generate_error_log(err, None);
                Error::new("unable to find cart")
            })
    }
}
