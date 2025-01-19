use actix_web::{error::HttpError, HttpResponse};

pub async fn insert_new_order() -> Result<HttpResponse, HttpError> {
    Ok(HttpResponse::Ok().finish())
}

pub async fn get_order_state() -> HttpResponse {
    HttpResponse::Ok().finish()
}
