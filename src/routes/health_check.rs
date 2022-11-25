use actix_web::{get, HttpResponse, Responder};

#[tracing::instrument(
    name = "Health check",
)]
#[get("/health_check")]
pub async fn health_check() -> impl Responder {
    return HttpResponse::Ok().finish();
}
