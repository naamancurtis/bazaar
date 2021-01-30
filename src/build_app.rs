use actix_cors::Cors;
use actix_web::{
    dev::Server,
    guard,
    http::header::{ACCESS_CONTROL_ALLOW_CREDENTIALS, COOKIE},
    web, App, HttpServer,
};
use actix_web_opentelemetry::RequestTracing;
use async_graphql::{EmptySubscription, Schema};
use async_graphql_telemetry_extension::OpenTelemetryExtension;
use sqlx::PgPool;
use std::net::TcpListener;

use crate::{
    auth::REFRESH_TOKEN_DURATION_SECONDS, routes::*, AppConfig, BazaarSchema, MutationRoot,
    QueryRoot,
};

pub fn generate_schema(connection: Option<PgPool>, config: Option<AppConfig>) -> BazaarSchema {
    let mut schema =
        Schema::build(QueryRoot, MutationRoot, EmptySubscription).extension(OpenTelemetryExtension);
    if let Some(connection) = connection {
        schema = schema.data(connection);
    }
    if let Some(config) = config {
        schema = schema.data(config);
    }
    schema.finish()
}

pub fn build_app(
    listener: TcpListener,
    connection: PgPool,
    configuration: AppConfig,
) -> Result<Server, Box<dyn std::error::Error + Send + Sync>> {
    let schema = generate_schema(Some(connection.clone()), Some(configuration.clone()));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(RequestTracing::new())
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost")
                            || origin.as_bytes().starts_with(b"http://127.0.0.1")
                    })
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(&[ACCESS_CONTROL_ALLOW_CREDENTIALS, COOKIE])
                    .max_age(Some(REFRESH_TOKEN_DURATION_SECONDS as usize)), // @TODO - verify this is correct
            )
            .data(schema.clone())
            .data(connection.clone())
            .data(configuration.clone())
            .service(web::resource("/").guard(guard::Post()).to(graphql_index))
            .service(
                web::resource("/")
                    .guard(guard::Get())
                    .to(graphql_playground),
            )
    })
    .listen(listener)?
    .run();

    Ok(server)
}
