use async_graphql::Context;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::error;

use crate::{
    auth::verify_and_deserialize_token,
    database::AuthDatabase,
    models::{BazaarCookies, BazaarToken, TokenType},
    BazaarError, Result,
};

pub struct GraphqlContext<'a> {
    pub pool: &'a PgPool,
    access_token: Option<Result<BazaarToken>>,
    refresh_token: Option<Result<BazaarToken>>,
    cookies: &'a Arc<BazaarCookies>,
}

impl<'a> GraphqlContext<'a> {
    /// Consumes the access token from the context, returning it
    pub fn access_token(&mut self) -> Result<BazaarToken> {
        if self.access_token.is_none() {
            error!(err = "no access token was found", "no access token found");
            return Err(BazaarError::Unauthorized);
        }
        self.access_token
            .take()
            .expect("already checked that it is some")
    }

    pub fn refresh_token(&mut self) -> Result<BazaarToken> {
        if self.refresh_token.is_none() {
            error!(err = "no refresh token was found", "no refresh token found");
            return Err(BazaarError::Unauthorized);
        }
        self.refresh_token
            .take()
            .expect("already checked that it is some")
    }

    pub fn set_new_cookies(
        &'a self,
        access_token: Option<String>,
        refresh_token: Option<String>,
    ) -> Result<()> {
        self.cookies.set_access_cookie(access_token)?;
        self.cookies.set_refresh_cookie(refresh_token)?;
        Ok(())
    }
}

/// The most common call signature for this function will be:
/// `extract_token_and_database_pool(ctx, true, false).await?;`
/// which will extract the database pool and the access token from
/// the raw GraphQL context
// Not sure if it's a bug or not, but removing the lifetimes breaks compilation
#[allow(clippy::needless_lifetimes)]
#[tracing::instrument(skip(context))]
pub async fn extract_token_and_database_pool<'a>(
    context: &'a Context<'_>,
    extract_access_token: bool,
    extract_refresh_token: bool,
) -> Result<GraphqlContext<'a>> {
    let pool = extract_database_pool(context)?;
    let cookies = context.data::<Arc<BazaarCookies>>().map_err(|e| {
        error!(err=?e, "failed to retrieve request cookies from graphql context");
        BazaarError::BadRequest("Failed to validate access cookies".to_owned())
    })?;
    let mut result = GraphqlContext {
        pool,
        access_token: None,
        refresh_token: None,
        cookies: &cookies,
    };
    if extract_access_token {
        result.access_token = Some(extract_token(&cookies, TokenType::Access, pool).await);
    }
    if extract_refresh_token {
        result.refresh_token = Some(extract_token(&cookies, TokenType::Refresh(0), pool).await);
    }

    // Now we have extracted the access tokens from the cookies, we will set the context state to
    // `None`, ready to decide whether we need to change the cookies on the response
    cookies.set_cookies_to_not_be_changed()?;
    Ok(result)
}

pub async fn extract_token(
    cookies: &BazaarCookies,
    token_type: TokenType,
    pool: &PgPool,
) -> Result<BazaarToken> {
    if token_type == TokenType::Access {
        if let Some(access_cookie) = cookies.get_access_cookie()? {
            return verify_and_deserialize_token::<AuthDatabase>(access_cookie, token_type, pool)
                .await;
        }
    }
    if let Some(refresh_cookie) = cookies.get_refresh_cookie()? {
        return verify_and_deserialize_token::<AuthDatabase>(refresh_cookie, token_type, pool)
            .await;
    }
    Err(BazaarError::Unauthorized)
}

pub fn extract_database_pool<'a>(context: &'a Context<'_>) -> Result<&'a PgPool> {
    context.data::<PgPool>().map_err(|err| {
        error!(err = ?err, "failed to extract database pool from graphql context");
        BazaarError::ServerError(err.message)
    })
}
