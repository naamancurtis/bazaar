use actix_cors::Cors;
use actix_web::{
    dev::Server,
    guard,
    http::header::{ACCESS_CONTROL_ALLOW_CREDENTIALS, COOKIE},
    web, App, HttpServer,
};
use async_graphql::{extensions::ApolloTracing, EmptySubscription, Schema};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{
    auth::REFRESH_TOKEN_DURATION_SECONDS, routes::*, BazaarSchema, MutationRoot, QueryRoot,
};

pub fn generate_schema(connection: Option<PgPool>) -> BazaarSchema {
    if let Some(connection) = connection {
        Schema::build(QueryRoot, MutationRoot, EmptySubscription)
            .extension(ApolloTracing)
            .data(connection)
            .finish()
    } else {
        Schema::build(QueryRoot, MutationRoot, EmptySubscription)
            .extension(ApolloTracing)
            .finish()
    }
}

pub fn build_app(listener: TcpListener, connection: PgPool) -> Result<Server, std::io::Error> {
    let schema = generate_schema(Some(connection.clone()));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger)
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
