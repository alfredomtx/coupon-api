use actix_web::{
    web, get, post, HttpResponse,
};
use super::model::{LoginRequest, LoginError};
use super::login_service;
use sqlx::MySqlPool;




#[tracing::instrument(
    name = "Post login", skip(pool)
)]
#[post("login")]
pub async fn login_handler(request: web::Json<LoginRequest>, pool: web::Data::<MySqlPool>) -> Result<HttpResponse, LoginError> {
    
    return Ok(HttpResponse::Created().json(""));
}