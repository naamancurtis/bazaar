use actix_web::{HttpResponse, Responder};

// @TODO
pub async fn log_out() -> impl Responder {
    HttpResponse::Ok().finish()
}
