use actix_web::{web, HttpRequest, HttpResponse};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{Request, Response};
use std::convert::TryFrom;

use crate::{graphql::BazaarSchema, models::BearerToken};

#[tracing::instrument(name = "graphql", skip(schema, http_request, graphql_request))]
pub async fn graphql_index(
    schema: web::Data<BazaarSchema>,
    http_request: HttpRequest,
    graphql_request: Request,
) -> Response {
    // Extract the token from the header
    // If it exists, attach it to the context
    // if it doesn't exist, nothing will be attached to the context
    let token = http_request
        .headers()
        .get("Authorization")
        .and_then(|token| {
            token
                .to_str()
                .ok()
                .map(|t| BearerToken::try_from(t.to_string()).ok())
                .flatten()
        });
    let mut request = graphql_request.into_inner();
    if let Some(token) = token {
        request = request.data(token);
    }

    schema.execute(request).await.into()
}

pub async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        ))
}
