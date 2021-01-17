use chrono::Utc;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::{
    auth::{
        encode_token, ACCESS_TOKEN_DURATION_SECONDS, REFRESH_TOKEN_DURATION_SECONDS,
        TIME_TO_REFRESH, TOKEN_TYPE,
    },
    database::{AuthRepository, CustomerRepository},
    models::{BazaarToken, BazaarTokens, Customer, CustomerType, TokenType},
    BazaarError, Result,
};

/// Manages the creation of a brand new set of JWT tokens (Access & Refresh).
///
/// If there is a valid refresh token, then use `refresh_tokens` instead
///
/// This function will automatically invalidate any previous `Refresh Tokens`
/// issued to that customer
#[tracing::instrument(
    skip(public_id, pool, private_id), 
    fields(id = %private_id.map(|id| id.to_string()).unwrap_or_default())
)]
pub async fn generate_new_tokens<C: CustomerRepository>(
    public_id: Option<Uuid>,
    private_id: Option<Uuid>,
    cart_id: Uuid,
    pool: &PgPool,
) -> Result<BazaarTokens> {
    let refresh_counter = if let Some(id) = private_id {
        Customer::increment_refresh_token_counter::<C>(id, pool).await?
    } else {
        // In the case of anonymous refresh tokens
        1
    };
    let access_token = encode_token(public_id, cart_id, TokenType::Access)?;
    let refresh_token = encode_token(public_id, cart_id, TokenType::Refresh(refresh_counter))?;

    let tokens = BazaarTokens {
        issued_at: Utc::now().timestamp(),
        access_token,
        access_token_expires_in: ACCESS_TOKEN_DURATION_SECONDS,
        refresh_token,
        refresh_token_expires_in: REFRESH_TOKEN_DURATION_SECONDS,
        token_type: TOKEN_TYPE.to_string(),
    };

    Ok(tokens)
}

/// This function manages refreshing both JWTs (Access & Refresh).
///
/// If the `Refresh Token` is due to expire it will automatically refresh the
/// token, otherwise it will just return the one that was provided to it.
///
/// This function will error if the refresh token has been invalidated or has expired.
/// It's worth calling out that an Anonymous Customer's tokens have no way of being
/// invalidated, however this type of token is only tied to a shopping cart.
#[tracing::instrument(skip(refresh_token, refresh_token_string, pool))]
pub async fn refresh_tokens<A: AuthRepository, C: CustomerRepository>(
    refresh_token: BazaarToken,
    refresh_token_string: String,
    pool: &PgPool,
) -> Result<BazaarTokens> {
    let time_till_expiry = refresh_token.time_till_expiry();

    if refresh_token.id.is_none() && refresh_token.customer_type == CustomerType::Known {
        error!(
            cart_id = ?refresh_token.cart_id,
            "token was marked as known customer but no id was found"
        );
        return Err(BazaarError::InvalidToken(
            "Token is malformed, please log in again".to_owned(),
        ));
    }

    check_refresh_token_is_not_invalidated::<C>(refresh_token.id, refresh_token.count, pool)
        .await?;

    // If the expiry is more than `X` time period away, just return the current refresh token
    if time_till_expiry > *TIME_TO_REFRESH {
        let tokens = BazaarTokens {
            issued_at: Utc::now().timestamp(),
            access_token: encode_token(
                refresh_token.public_id(),
                refresh_token.cart_id,
                TokenType::Access,
            )?,
            access_token_expires_in: ACCESS_TOKEN_DURATION_SECONDS,
            refresh_token: refresh_token_string,
            refresh_token_expires_in: refresh_token.time_till_expiry().num_seconds(),
            token_type: TOKEN_TYPE.to_string(),
        };
        return Ok(tokens);
    }

    // Otherwise, also refresh the refresh token
    generate_new_tokens::<C>(
        refresh_token.public_id(),
        refresh_token.id,
        refresh_token.cart_id,
        pool,
    )
    .await
}

async fn check_refresh_token_is_not_invalidated<C: CustomerRepository>(
    private_id: Option<Uuid>,
    count: Option<i32>,
    pool: &PgPool,
) -> Result<()> {
    if let Some(id) = private_id {
        let current_refresh_counter = Customer::fetch_refresh_token_counter::<C>(id, pool).await?;
        if Some(current_refresh_counter) != count {
            return Err(BazaarError::InvalidToken(
                "Token has been invalidated".to_owned(),
            ));
        }
    }
    Ok(())
}
