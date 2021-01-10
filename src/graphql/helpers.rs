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
    pub(crate) access_token_raw: Option<String>,
    refresh_token: Option<Result<BazaarToken>>,
    pub(crate) refresh_token_raw: Option<String>,
    cookies: &'a Arc<BazaarCookies>,
}

impl<'a> GraphqlContext<'a> {
    /// Returns the access token from the context
    pub fn access_token(&mut self) -> Result<BazaarToken> {
        if self.access_token.is_none() {
            error!(err = "no access token was found", "no access token found");
            return Err(BazaarError::InvalidToken("Not found".to_owned()));
        }
        self.access_token
            .clone()
            .expect("already checked that it is some")
    }

    /// Returns the refresh token from the context
    pub fn refresh_token(&mut self) -> Result<BazaarToken> {
        if self.refresh_token.is_none() {
            error!(err = "no refresh token was found", "no refresh token found");
            return Err(BazaarError::InvalidToken("Not found".to_owned()));
        }
        self.refresh_token
            .clone()
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
        access_token_raw: cookies.get_access_cookie()?,
        refresh_token: None,
        refresh_token_raw: cookies.get_refresh_cookie()?,
        cookies: &cookies,
    };
    if extract_access_token {
        result.access_token =
            Some(extract_token(&result.access_token_raw, TokenType::Access, pool).await);
    }
    if extract_refresh_token {
        result.refresh_token =
            Some(extract_token(&result.refresh_token_raw, TokenType::Refresh(0), pool).await);
    }

    // @TODO Remove this once async-graphql is updated
    // Now we have extracted the access tokens from the cookies, we will set the context state to
    // `None`, ready to decide whether we need to change the cookies on the response
    cookies.set_cookies_to_not_be_changed()?;
    Ok(result)
}

pub async fn extract_token(
    cookie_raw: &Option<String>,
    token_type: TokenType,
    pool: &PgPool,
) -> Result<BazaarToken> {
    if let Some(cookie) = cookie_raw {
        return verify_and_deserialize_token::<AuthDatabase>(cookie, token_type, pool).await;
    }
    Err(BazaarError::InvalidToken("No token found".to_owned()))
}

pub fn extract_database_pool<'a>(context: &'a Context<'_>) -> Result<&'a PgPool> {
    context.data::<PgPool>().map_err(|err| {
        error!(err = ?err, "failed to extract database pool from graphql context");
        BazaarError::ServerError(err.message)
    })
}
