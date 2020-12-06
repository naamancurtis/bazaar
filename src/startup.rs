use actix_web::{dev::Server, guard, web, App, HttpServer};
use async_graphql::{extensions::ApolloTracing, EmptySubscription, Schema};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{routes::*, BazaarSchema, MutationRoot, QueryRoot};

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
            .data(schema.clone())
            .data(connection.clone())
            .service(web::resource("/").guard(guard::Post()).to(graphql_index))
            .service(
                web::resource("/playground")
                    .guard(guard::Get())
                    .to(graphql_playground),
            )
            .route("/health_check", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
