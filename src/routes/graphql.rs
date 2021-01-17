use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{Request, Response};
use opentelemetry::Context;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use std::sync::Arc;

use crate::{
    graphql::{BazaarSchema, OpenTelemetryConfig},
    models::{BazaarCookies, TokenType},
};

// Most of the Open Telemetry Context Stuff is pulled straight from `actix_web_opentelemetry`
//
// It doesn't quite seem correct to use it, given that this is a graphQL server, but a majority of
// it is still needed to transfer the distributed tracing information across
#[tracing::instrument(name = "graphql", skip(schema, http_request, graphql_request))]
pub async fn graphql_index(
    schema: web::Data<BazaarSchema>,
    http_request: HttpRequest,
    graphql_request: Request,
) -> Result<Response> {
    // Get the Open Telemetry Context
    let cx = Context::current();

    // For every request, tokens are extracted and attached to the graphql context
    // under the type `Arc<BazaarCookies`
    let cookies = Arc::new(extract_cookies(&http_request)?);

    // Get the current tracing Span
    let span = Span::current();
    // Attach the Otel context to the tracing span
    span.set_parent(cx);

    let otel_context = OpenTelemetryConfig::default().parent_span(span);

    let mut request = graphql_request.into_inner();
    request = request.data(Arc::clone(&cookies)).data(otel_context);

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
