use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{Request, Response};
use std::sync::Arc;

use crate::{
    graphql::BazaarSchema,
    models::{BazaarCookies, TokenType},
};

#[tracing::instrument(name = "graphql", skip(schema, http_request, graphql_request))]
pub async fn graphql_index(
    schema: web::Data<BazaarSchema>,
    http_request: HttpRequest,
    graphql_request: Request,
) -> Result<Response> {
    // For every request, tokens are extracted and attached to the graphql context
    // under the type `Arc<BazaarCookies`
    let cookies = Arc::new(extract_cookies(&http_request)?);

    let mut request = graphql_request.into_inner();
    request = request.data(Arc::clone(&cookies));

    let resp: Response = schema.execute(request).await.into();
    Ok(resp)
}

pub async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        ))
}

/// Pulls the Access Token & Refresh Token from the cookies sent on the request
fn extract_cookies(req: &HttpRequest) -> Result<BazaarCookies> {
    let access_cookie = req
        .cookie(TokenType::Access.as_str())
        .map(|c| c.value().to_string());
    let refresh_cookie = req
        .cookie(TokenType::Refresh(0).as_str())
        .map(|c| c.value().to_string());

    // @TODO - Come back and work out how to handle these errors appropriately
    let cookies = BazaarCookies::new(access_cookie, refresh_cookie)?;
    Ok(cookies)
}
