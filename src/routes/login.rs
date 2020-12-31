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

// @TODO
pub async fn log_in() -> Result<HttpResponse, BazaarError> {
    HttpResponse::Ok().finish()
}
