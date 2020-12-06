use actix_web::{http::Cookie, web, HttpMessage, HttpRequest, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    auth::{decode_token, refresh_tokens},
    database::{AuthDatabase, CustomerDatabase},
    models::TokenType,
    BazaarError,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RefreshRequest {
    pub login_redirect_url: String,
}

/// Checks for a valid refresh token provided to the request as a cookie with the
/// `refresh_token` key.
///
/// If a valid token is found, it generates a new access and refresh token, updating
/// the refresh token in the cookie and returning both in the body as well.
///
/// If no valid token is found, a redirection header is set to the `login_redirect_url`
/// provided in the request
#[tracing::instrument(skip(req, pool, req_body))]
pub async fn refresh(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    req_body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, BazaarError> {
    let raw_token = req
        .cookie("refresh_token")
        .map(|token| decode_token(token.value(), TokenType::Refresh(0)).ok())
        .flatten();
    if raw_token.is_none() {
        // Redirect to login
        return Ok(HttpResponse::SeeOther()
            .header("Location", req_body.login_redirect_url.clone())
            .finish());
    }
    let refresh_token_string = req.cookie("refresh_token").clone().unwrap().to_string();
    let current_refresh_token = raw_token.unwrap();

    let public_id = current_refresh_token.claims.sub;
    let cart_id = current_refresh_token.claims.cart_id;
    let tokens = refresh_tokens::<AuthDatabase, CustomerDatabase>(
        public_id,
        cart_id,
        refresh_token_string,
        current_refresh_token,
        &pool,
    )
    .await?;

    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("refresh_token", tokens.refresh_token.clone())
                .secure(true)
                .http_only(true)
                .finish(),
        )
        .json(tokens))
}
