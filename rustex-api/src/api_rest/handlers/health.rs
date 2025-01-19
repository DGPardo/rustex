use actix_web::HttpResponse;

pub async fn service_health() -> HttpResponse {
    HttpResponse::Ok().finish()
}
