use actix_web::{http::Cookie, web, HttpMessage, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::{
    auth::{decode_token, generate_tokens},
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
pub async fn refresh(
    req: HttpRequest,
    req_body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, BazaarError> {
    let token = req
        .cookie("refresh_token")
        .map(|token| decode_token(token.value(), TokenType::Refresh).ok())
        .flatten();
    if token.is_none() {
        // Redirect to login
        return Ok(HttpResponse::SeeOther()
            .header("Location", req_body.login_redirect_url.clone())
            .finish());
    }
    let current_refresh_token = token.unwrap();
    let user_id = current_refresh_token.claims.sub;
    let cart_id = current_refresh_token.claims.cart_id;
    let tokens = generate_tokens(user_id, cart_id)?;

    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("refresh_token", tokens.refresh_token.clone())
                .secure(true)
                .http_only(true)
                .finish(),
        )
        .json(tokens))
}
