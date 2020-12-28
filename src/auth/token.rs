use async_graphql::SimpleObject;
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    auth::{encode_token, ACCESS_TOKEN_DURATION, REFRESH_TOKEN_DURATION},
    models::TokenType,
    BazaarError,
};

#[derive(Debug, Serialize, SimpleObject)]
pub struct BazaarTokens {
    // Timestamp of when these tokens were issued
    pub issued_at: i64,

    pub access_token: String,

    /// Time until expiry (in seconds)
    pub access_token_expires_in: i64,

    /// This token is automatically set in the cookies
    pub refresh_token: String,

    /// Time until expiry (in seconds)
    pub refresh_token_expires_in: i64,
}

pub fn generate_tokens(
    // This should always be the **public** id, never the private one
    id: Option<Uuid>,
    cart_id: Uuid,
) -> Result<BazaarTokens, BazaarError> {
    let access_token = encode_token(id, cart_id, TokenType::Access)?;
    // @TODO need to add refresh token + expiry to whitelist/blacklist
    let refresh_token = encode_token(id, cart_id, TokenType::Refresh)?;

    let tokens = BazaarTokens {
        issued_at: Utc::now().timestamp(),
        access_token,
        access_token_expires_in: ACCESS_TOKEN_DURATION,
        refresh_token: refresh_token.clone(),
        refresh_token_expires_in: REFRESH_TOKEN_DURATION,
    };
    Ok(tokens)
}
