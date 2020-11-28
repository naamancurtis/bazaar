use actix_web::{dev::Server, guard, web, App, HttpServer};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use std::net::TcpListener;

use crate::{routes::*, QueryRoot};

pub fn build_app(listener: TcpListener) -> Result<Server, std::io::Error> {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
    let server = HttpServer::new(move || {
        App::new()
            .data(schema.clone())
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
