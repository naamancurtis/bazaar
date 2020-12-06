use async_graphql::SimpleObject;
use serde::Serialize;

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

    /// Will always be `Bearer`
    pub token_type: String,
}
