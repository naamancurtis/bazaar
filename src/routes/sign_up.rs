use actix_web::{HttpResponse, Responder};

// @TODO
pub async fn sign_up(
    email: String,
    password: String,
    first_name: String,
    last_name: String,
) -> impl Responder {
    // create entry in auth table
    // create customer in customers table
    // generate auth token
    
    HttpResponse::Ok().finish()
}
