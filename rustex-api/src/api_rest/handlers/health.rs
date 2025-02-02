use actix_web::HttpResponse;
use serde_json::json;

pub async fn service_health() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "Status": "ok"
    }))
}
