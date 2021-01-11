use async_graphql::Object;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BazaarTokens {
    // Timestamp of when these tokens were issued
    pub issued_at: i64,

    /// This token is automatically set in the cookies
    #[serde(skip)]
    pub access_token: String,

    /// Time until expiry (in seconds)
    pub access_token_expires_in: i64,

    /// This token is automatically set in the cookies
    #[serde(skip)]
    pub refresh_token: String,

    /// Time until expiry (in seconds)
    pub refresh_token_expires_in: i64,

    /// Will always be `cookies`
    pub token_type: String,
}

#[Object]
impl BazaarTokens {
    async fn issued_at(&self) -> i64 {
        self.issued_at
    }

    async fn access_token_expires_in(&self) -> i64 {
        self.access_token_expires_in
    }

    async fn refresh_token_expires_in(&self) -> i64 {
        self.refresh_token_expires_in
    }

    async fn token_type(&self) -> String {
        self.token_type.clone()
    }
}
