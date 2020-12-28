use async_graphql::{
    validators::{Email, StringMinLength},
    Context, ErrorExtensions, Object, Result,
};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::{
    auth::{generate_tokens, verify_password_and_fetch_details, BazaarTokens},
    database::{AuthDatabase, CartItemDatabase, CustomerDatabase, ShoppingCartDatabase},
    graphql::{extract_database_pool, validators::ValidCustomerUpdateType},
    models::{
        cart_item::{InternalCartItem, UpdateCartItem},
        Currency, Customer, CustomerUpdate, ShoppingCart,
    },
};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    #[tracing::instrument(skip(self, ctx, email, password))]
    async fn login(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> Result<BazaarTokens> {
        let pool = extract_database_pool(ctx).map_err(|e| e.extend())?;
        let customer_details =
            verify_password_and_fetch_details::<AuthDatabase>(&email, &password, pool)
                .await
                .map_err(|e| e.extend())?;
        let cart_id = ShoppingCart::find_cart_id_by_customer_id::<ShoppingCartDatabase>(
            customer_details.id,
            pool,
        )
        .await?;
        let tokens =
            generate_tokens(Some(customer_details.public_id), cart_id).map_err(|e| e.extend());
        tokens
    }

    // @TODO - Once there's an auth token, we need to ensure that if the user has an
    // anonymous cart, it's correctly added when they're signing up
    #[tracing::instrument(skip(self, ctx, password))]
    async fn create_customer(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(Email))] email: String,
        #[graphql(validator(StringMinLength(length = "8")))] password: String,
        #[graphql(validator(StringMinLength(length = "2")))] first_name: String,
        #[graphql(validator(StringMinLength(length = "2")))] last_name: String,
    ) -> Result<Customer> {
        let pool = ctx.data::<PgPool>()?;
        let id = Customer::new::<CustomerDatabase>(email, password, first_name, last_name, pool)
            .await
            .map_err(|err| {
                error!(?err, "failed to create new customer");
                err.extend()
            })?;
        Customer::find_by_id::<CustomerDatabase>(id, pool)
            .await
            .map_err(|err| {
                error!(?err, "failed to find newly created customer");
                err.extend()
            })
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn update_customer(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        #[graphql(validator(ValidCustomerUpdateType))] update: Vec<CustomerUpdate>,
    ) -> Result<Customer> {
        let pool = ctx.data::<PgPool>()?;
        Customer::update::<CustomerDatabase>(id, update, pool)
            .await
            .map_err(|err| {
                error!(?err, "failed to update customer");
                err.extend()
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
                error!(?err, "failed to create anonymous cart");
                err.extend()
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
                error!(?err, "failed to create known cart");
                err.extend()
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
        ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
            id,
            new_items.into_iter().map(Into::into).collect(),
            pool,
        )
        .await
        .map_err(|err| {
            error!(?err, "failed to add items to cart");
            err.extend()
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
        .map_err(|err| {
            error!(?err, "failed to remove items from cart");
            err.extend()
        })
    }
}
