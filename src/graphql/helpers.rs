use async_graphql::Context;
use sqlx::PgPool;
use tracing::error;

use crate::{
    database::AuthDatabase,
    models::{parse_and_deserialize_token, BazaarToken, BearerToken, TokenType},
    BazaarError,
};

pub async fn extract_token_and_database_pool<'a>(
    context: &'a Context<'_>,
    token_type: TokenType,
) -> Result<(&'a PgPool, BazaarToken), BazaarError> {
    let pool = extract_database_pool(context)?;
    let token = extract_token(context, token_type, pool).await?;
    Ok((pool, token))
}

pub async fn extract_token(
    context: &Context<'_>,
    token_type: TokenType,
    pool: &PgPool,
) -> Result<BazaarToken, BazaarError> {
    let token = context.data::<BearerToken>().map_err(|e| {
        error!(err = ?e, "invalid token found");
        BazaarError::InvalidToken("Token was malformed or not found".to_owned())
    })?;
    parse_and_deserialize_token::<AuthDatabase>(token.clone(), token_type, pool).await
}

pub fn extract_database_pool<'a>(context: &'a Context<'_>) -> Result<&'a PgPool, BazaarError> {
    context.data::<PgPool>().map_err(|err| {
        error!(err = ?err, "failed to extract database pool from graphql context");
        BazaarError::ServerError(err.message)
    })
}
