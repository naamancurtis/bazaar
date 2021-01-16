use async_graphql::Context;
use http::header::SET_COOKIE;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::error;

use crate::{
    auth::verify_and_deserialize_token,
    database::AuthDatabase,
    models::{BazaarCookies, BazaarToken, BazaarTokens, TokenType},
    AppConfig, BazaarError, Environment, Result,
};

/// An internal struct that holds state that is pulled off the
/// GraphQL context for most requests
pub struct GraphqlContext<'a> {
    pub pool: &'a PgPool,
    access_token: Option<Result<BazaarToken>>,
    pub(crate) access_token_raw: Option<String>,
    refresh_token: Option<Result<BazaarToken>>,
    pub(crate) refresh_token_raw: Option<String>,
}

impl<'a> GraphqlContext<'a> {
    /// Returns the access token from the context
    pub fn access_token(&self) -> Result<BazaarToken> {
        if self.access_token.is_none() {
            error!(err = "no access token was found", "no access token found");
            return Err(BazaarError::InvalidToken("Not found".to_owned()));
        }
        self.access_token
            .clone()
            .expect("already checked that it is some")
    }

    /// Returns the refresh token from the context
    pub fn refresh_token(&self) -> Result<BazaarToken> {
        if self.refresh_token.is_none() {
            error!(err = "no refresh token was found", "no refresh token found");
            return Err(BazaarError::InvalidToken("Not found".to_owned()));
        }
        self.refresh_token
            .clone()
            .expect("already checked that it is some")
    }
}

/// The most common call signature for this function will be:
/// `extract_token_and_database_pool(ctx, true, false).await?;`
/// which will extract the database pool and the access token from
/// the raw GraphQL context - most of the time you won't need the
/// refresh token
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
    };
    if extract_access_token {
        result.access_token =
            Some(extract_token(&result.access_token_raw, TokenType::Access, pool).await);
    }
    if extract_refresh_token {
        result.refresh_token =
            Some(extract_token(&result.refresh_token_raw, TokenType::Refresh(0), pool).await);
    }

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
    Err(BazaarError::InvalidToken("No token was found".to_owned()))
}

pub fn extract_database_pool<'a>(context: &'a Context<'_>) -> Result<&'a PgPool> {
    context.data::<PgPool>().map_err(|err| {
        error!(err = ?err, "failed to extract database pool from graphql context");
        BazaarError::ServerError(err.message)
    })
}

#[tracing::instrument(skip(ctx, tokens))]
pub fn set_auth_cookies_on_response(ctx: &Context<'_>, tokens: &BazaarTokens) {
    let app_env = ctx
        .data::<AppConfig>()
        .expect("configuration should always be present in context")
        .env;
    let access = generate_auth_cookie_string(
        &tokens.access_token,
        TokenType::Access,
        tokens.access_token_expires_in,
        app_env,
    );
    ctx.append_http_header(SET_COOKIE, access);
    let refresh = generate_auth_cookie_string(
        &tokens.refresh_token,
        TokenType::Refresh(0),
        tokens.refresh_token_expires_in,
        app_env,
    );
    ctx.append_http_header(SET_COOKIE, refresh);
}

/// As cookies are set via the `Set-Cookie` header, this helper function generates the string that
/// is expected as the value
fn generate_auth_cookie_string(
    cookie: &str,
    token_type: TokenType,
    expiry: i64,
    env: Environment,
) -> String {
    // This is hacky, and ideally we'd be able to get rid of it, but with `Secure` set on the
    // cookies, and no TLS cert on the server, none of the cookies get set within the tests.
    // Ideally we'd push all the traffic to https even on tests
    let secure = match env {
        Environment::Local | Environment::Test => "",
        _ => "Secure; ",
    };
    format!(
        "{}={}; {}HttpOnly; MaxAge={}",
        token_type.as_str(),
        cookie,
        secure,
        expiry
    )
}
