use async_graphql::{
    validators::{Email, StringMinLength},
    Context, ErrorExtensions, Object, Result,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    auth::{generate_new_tokens, verify_password_and_fetch_details},
    database::{AuthDatabase, CartItemDatabase, CustomerDatabase, ShoppingCartDatabase},
    graphql::{extract_token_and_database_pool, validators::ValidCustomerUpdateType},
    models::{
        cart_item::{InternalCartItem, UpdateCartItem},
        BazaarTokens, Currency, Customer, CustomerType, CustomerUpdate, ShoppingCart, TokenType,
    },
    BazaarError,
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
        let (pool, token) = extract_token_and_database_pool(ctx, TokenType::Access)
            .await
            .map_err(|e| e.extend())?;
        let anonymous_cart_id = if let Ok(token) = token {
            if token.customer_type == CustomerType::Known {
                error!(
                    err = "already logged in customer hit login mutation",
                    id = ?token.id,
                    "customer already has valid tokens"
                );
                return Err(BazaarError::BadRequest(
                    "Customer already has valid tokens".to_string(),
                )
                .extend());
            }
            Some(token.cart_id)
        } else {
            None
        };
        let customer_details =
            verify_password_and_fetch_details::<AuthDatabase>(&email, &password, pool)
                .await
                .map_err(|e| e.extend())?;
        let cart_id = ShoppingCart::find_cart_id_by_customer_id::<ShoppingCartDatabase>(
            customer_details.id,
            pool,
        )
        .await?;
        // If the customer did some browsing while anonymous (ie. the token is valid), need
        // to merge the two carts together
        if let Some(anonymous_cart_id) = anonymous_cart_id {
            let id = ShoppingCart::merge_shopping_carts::<ShoppingCartDatabase, CartItemDatabase>(
                cart_id,
                anonymous_cart_id,
                pool,
            )
            .await?;
            assert_eq!(id, cart_id);
        }
        generate_new_tokens::<CustomerDatabase>(
            Some(customer_details.public_id),
            Some(customer_details.id),
            cart_id,
            pool,
        )
        .await
        .map_err(|e| e.extend())
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn anonymous_login(&self, ctx: &Context<'_>) -> Result<BazaarTokens> {
        // There is an edge case where an anonymous user had a pair of tokens
        // and both have expired. However when they access the site after that
        // time period the client they're using hasn't cleared the tokens and
        // expired tokens are sent. In this case we do want to log them in again.
        let (pool, token) = extract_token_and_database_pool(ctx, TokenType::Access)
            .await
            .map_err(|e| e.extend())?;
        if token.is_ok() {
            // If the token is `Ok` it means the token is valid, in which case
            // we want them to use those tokens
            return Err(BazaarError::BadRequest("Valid token already exists".to_string()).extend());
        };
        let cart = ShoppingCart::new_anonymous::<ShoppingCartDatabase>(Currency::GBP, pool).await?;
        generate_new_tokens::<CustomerDatabase>(None, None, cart.id, pool)
            .await
            .map_err(|e| e.extend())
    }

    #[tracing::instrument(skip(self, ctx, password, first_name, last_name, email))]
    async fn sign_up(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(Email))] email: String,
        #[graphql(validator(StringMinLength(length = "8")))] password: String,
        #[graphql(validator(StringMinLength(length = "2")))] first_name: String,
        #[graphql(validator(StringMinLength(length = "2")))] last_name: String,
    ) -> Result<BazaarTokens> {
        let (pool, token) = extract_token_and_database_pool(ctx, TokenType::Access)
            .await
            .map_err(|e| e.extend())?;

        // Need to know whether to create a new cart, or update an existing one
        let cart_id = if let Ok(token) = token {
            if token.customer_type == CustomerType::Known {
                error!(
                    err = "signed up customer with valid token hit sign up mutation",
                    id = ?token.id.unwrap_or_default(),
                    "customer already has valid tokens"
                );
                return Err(
                    BazaarError::BadRequest("Customer already exists".to_string()).extend(),
                );
            }
            Some(token.cart_id)
        } else {
            None
        };

        let ids = Customer::new::<CustomerDatabase>(
            Uuid::new_v4(),
            email,
            password,
            first_name,
            last_name,
            cart_id,
            pool,
        )
        .await
        .map_err(|err| {
            error!(?err, "failed to create new customer");
            err.extend()
        })?;
        generate_new_tokens::<CustomerDatabase>(
            Some(ids.public_id),
            Some(ids.id),
            ids.cart_id,
            pool,
        )
        .await
        .map_err(|e| e.extend())
    }

    #[tracing::instrument(skip(self, ctx, update))]
    async fn update_customer(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(ValidCustomerUpdateType))] update: Vec<CustomerUpdate>,
    ) -> Result<Customer> {
        let (pool, token) = extract_token_and_database_pool(ctx, TokenType::Access)
            .await
            .map_err(|e| e.extend())?;
        let token = token.map_err(|e| e.extend())?;
        if let Some(id) = token.id {
            return Customer::update::<CustomerDatabase>(id, update, pool)
                .await
                .map_err(|err| {
                    error!(?err, "failed to update customer");
                    err.extend()
                });
        }
        Err(BazaarError::AnonymousError.extend())
    }

    #[tracing::instrument(skip(self, ctx))]
    async fn add_items_to_cart(
        &self,
        ctx: &Context<'_>,
        new_items: Vec<UpdateCartItem>,
    ) -> Result<ShoppingCart> {
        let (pool, token) = extract_token_and_database_pool(ctx, TokenType::Access)
            .await
            .map_err(|e| e.extend())?;
        let token = token.map_err(|e| e.extend())?;
        ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
            token.cart_id,
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
        removed_items: Vec<UpdateCartItem>,
    ) -> Result<ShoppingCart> {
        let (pool, token) = extract_token_and_database_pool(ctx, TokenType::Access)
            .await
            .map_err(|e| e.extend())?;
        let token = token.map_err(|e| e.extend())?;
        ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
            token.cart_id,
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
