use actix_web::{HttpResponse, Responder};

// @TODO
pub async fn log_in() -> impl Responder {
    HttpResponse::Ok().finish()
}
