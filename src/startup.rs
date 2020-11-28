use actix_web::{dev::Server, guard, web, App, HttpServer};
use async_graphql::{EmptySubscription, Schema};
use sqlx::PgPool;
use std::net::TcpListener;

use crate::{routes::*, MutationRoot, QueryRoot};

pub fn build_app(listener: TcpListener, connection: PgPool) -> Result<Server, std::io::Error> {
    dbg!(&listener, &connection);
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(connection.clone())
        .finish();

    let server = HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .data(connection.clone())
            .service(web::resource("/").guard(guard::Post()).to(graphql_index))
            .service(
                web::resource("/")
                    .guard(guard::Get())
                    .to(graphql_playground),
            )
            .route("/health_check", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
