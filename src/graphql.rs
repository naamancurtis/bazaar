use async_graphql::{
    validators::{Email, StringMinLength},
    Context, EmptySubscription, Error, Object, Result, Schema,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    database::{CartItemDatabase, CustomerDatabase, ShoppingCartDatabase},
    error::generate_error_log,
    models::{cart_item::UpdateCartItem, Currency, Customer, CustomerUpdate, ShoppingCart},
};

pub type BazaarSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

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
        #[graphql(validator(StringMinLength(length = "8")))] password: String,
        #[graphql(validator(StringMinLength(length = "2")))] first_name: String,
        #[graphql(validator(StringMinLength(length = "2")))] last_name: String,
    ) -> Result<Uuid> {
        let pool = ctx.data::<PgPool>()?;
        Customer::new::<CustomerDatabase>(email, password, first_name, last_name, pool)
            .await
            .map_err(|err| {
                generate_error_log(err, None);
                Error::new("unable to create new customer")
            })
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn update_customer(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        update: CustomerUpdate,
    ) -> Result<Customer> {
        let pool = ctx.data::<PgPool>()?;
        Customer::update::<CustomerDatabase>(id, update, pool)
            .await
            .map_err(|err| {
                generate_error_log(err, None);
                Error::new("unable to update new customer")
            })
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn create_anonymous_cart(
        &self,
        ctx: &Context<'_>,
        currency: Currency,
    ) -> Result<ShoppingCart> {
        let pool = ctx.data::<PgPool>()?;
        ShoppingCart::new_anonymous::<ShoppingCartDatabase>(currency, pool)
            .await
            .map_err(|err| {
                generate_error_log(err, None);
                Error::new("unable to create cart")
            })
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn create_known_cart(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        currency: Currency,
    ) -> Result<ShoppingCart> {
        let pool = ctx.data::<PgPool>()?;
        Customer::add_new_cart::<CustomerDatabase, ShoppingCartDatabase>(id, currency, pool)
            .await
            .map_err(|err| {
                generate_error_log(err, None);
                Error::new("unable to create cart for customer")
            })
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn add_items_to_cart(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        new_items: Vec<UpdateCartItem>,
    ) -> Result<ShoppingCart> {
        let pool = ctx.data::<PgPool>()?;
        ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(id, new_items, pool)
            .await
            .map_err(|err| {
                generate_error_log(err, None);
                Error::new("unable to add items to cart")
            })
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn remove_items_from_cart(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        removed_items: Vec<UpdateCartItem>,
    ) -> Result<ShoppingCart> {
        let pool = ctx.data::<PgPool>()?;
        ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
            id,
            removed_items,
            pool,
        )
        .await
        .map_err(|err| {
            generate_error_log(err, None);
            Error::new("unable to remove items from cart")
        })
    }
}
