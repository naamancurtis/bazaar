use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use bazaar::{
    auth::{generate_tokens, BazaarTokens},
    database::ShoppingCartDatabase,
    models::{Currency, ShoppingCart},
};

use crate::helpers::{insert_default_customer, CustomerData};

#[derive(Debug)]
pub struct AnonymousTokenData {
    pub cart_id: Uuid,
    pub tokens: BazaarTokens,
}

#[derive(Debug)]
pub struct KnownTokenData {
    pub customer: CustomerData,
    pub tokens: BazaarTokens,
}

pub async fn get_anonymous_token(pool: &PgPool) -> Result<AnonymousTokenData> {
    let cart = ShoppingCart::new_anonymous::<ShoppingCartDatabase>(Currency::GBP, pool).await?;
    let tokens = generate_tokens(None, cart.id)?;
    Ok(AnonymousTokenData {
        cart_id: cart.id,
        tokens,
    })
}

/// Inserts a new customer into all the relevant tables and generates
/// valid tokens for them
pub async fn get_known_token(pool: &PgPool) -> Result<KnownTokenData> {
    let customer = insert_default_customer(pool).await?;
    let tokens = generate_tokens(customer.id, customer.cart_id.unwrap())?;

    Ok(KnownTokenData { customer, tokens })
}
