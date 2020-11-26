use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

pub fn build_app(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .listen(listener)?
        .run();
    Ok(server)
}
