use async_graphql::{
    validators::{Email, StringMinLength},
    Context, EmptySubscription, Object, Result, Schema,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    cart_item::{InternalCartItem, UpdateCartItem},
    Currency, Customer, CustomerUpdate, ShoppingCart,
};

pub type BazaarSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    #[tracing::instrument(name = "get_customers", skip(self, ctx))]
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

    #[tracing::instrument(skip(self, ctx))]
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

    #[tracing::instrument(skip(self, ctx))]
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
    // @TODO - Once there's an auth token, we need to ensure that if the user has an
    // anonymous cart, it's correctly added when they're signing up
    #[tracing::instrument(skip(self, ctx))]
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

    #[tracing::instrument(skip(self, ctx))]
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

    #[tracing::instrument(skip(self, ctx))]
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

    #[tracing::instrument(skip(self, ctx))]
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

    #[tracing::instrument(skip(self, ctx))]
    async fn add_items_to_cart(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        new_items: Vec<UpdateCartItem>,
    ) -> Result<ShoppingCart> {
        let pool = ctx.data::<PgPool>()?;
        ShoppingCart::edit_cart_items(id, new_items.into_iter().map(|i| i.into()).collect(), pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.message))
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn remove_items_from_cart(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        removed_items: Vec<UpdateCartItem>,
    ) -> Result<ShoppingCart> {
        let pool = ctx.data::<PgPool>()?;
        ShoppingCart::edit_cart_items(
            id,
            removed_items
                .into_iter()
                .map(|i| {
                    let mut item: InternalCartItem = i.into();
                    item.quantity = -item.quantity;
                    item
                })
                .collect(),
            pool,
        )
        .await
        .map_err(|e| async_graphql::Error::new(e.message))
    }
}
