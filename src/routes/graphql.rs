use actix_web::{web, HttpResponse};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{Request, Response};

use crate::graphql::BazaarSchema;

#[tracing::instrument(name = "graphql", skip(schema, req))]
pub async fn graphql_index(schema: web::Data<BazaarSchema>, req: Request) -> Response {
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        ))
}
