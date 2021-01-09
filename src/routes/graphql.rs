use actix_web::{
    dev::HttpResponseBuilder, http::StatusCode, web, HttpMessage, HttpRequest, HttpResponse, Result,
};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{Request, Response};
use std::sync::Arc;

use crate::{
    auth::{ACCESS_TOKEN_DURATION_SECONDS, REFRESH_TOKEN_DURATION_SECONDS},
    graphql::BazaarSchema,
    models::{BazaarCookies, TokenType},
};

#[tracing::instrument(name = "graphql", skip(schema, http_request, graphql_request))]
pub async fn graphql_index(
    schema: web::Data<BazaarSchema>,
    http_request: HttpRequest,
    graphql_request: Request,
) -> Result<HttpResponse> {
    // // Extract the token from the header
    // // If it exists, attach it to the context
    // // if it doesn't exist, nothing will be attached to the context
    // let token = http_request
    //     .headers()
    //     .get("Authorization")
    //     .and_then(|token| {
    //         token
    //             .to_str()
    //             .ok()
    //             .map(|t| BearerToken::try_from(t.to_string()).ok())
    //             .flatten()
    //     });
    let access_cookie = http_request
        .cookie(TokenType::Access.as_str())
        .map(|c| c.value().to_string());
    let refresh_cookie = http_request
        .cookie(TokenType::Refresh(0).as_str())
        .map(|c| c.value().to_string());

    // @TODO - Come back and work out how to handle these errors appropriately
    let cookies = Arc::new(BazaarCookies::new(access_cookie, refresh_cookie)?);

    let mut request = graphql_request.into_inner();
    request = request.data(Arc::clone(&cookies));

    // if let Some(token) = token {
    //     request = request.data(token);
    // }

    let res: Response = schema.execute(request).await.into();
    let mut response = HttpResponse::build(StatusCode::OK);
    response.content_type("application/json");
    if res.0.is_ok() {
        if let Some(cache_control) = res.0.cache_control().value() {
            response.header("cache-control", cache_control);
        }
    }

    set_cookie_on_response(
        &mut response,
        cookies.get_access_cookie().ok().flatten(),
        TokenType::Access,
    );
    set_cookie_on_response(
        &mut response,
        cookies.get_refresh_cookie().ok().flatten(),
        TokenType::Refresh(0),
    );
    response.json(&res.0).await
}

pub async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        ))
}

fn set_cookie_on_response(
    response: &mut HttpResponseBuilder,
    cookie: Option<String>,
    token_type: TokenType,
) {
    if let Some(cookie) = cookie {
        let duration = if token_type == TokenType::Access {
            ACCESS_TOKEN_DURATION_SECONDS
        } else {
            REFRESH_TOKEN_DURATION_SECONDS
        };
        let cookie_string = format!(
            "{}={}; SameSite=Strict; Secure; HttpOnly; MaxAge={}",
            token_type.as_str(),
            cookie,
            duration
        );
        response.header("Set-Cookie", cookie_string);
    }
}
