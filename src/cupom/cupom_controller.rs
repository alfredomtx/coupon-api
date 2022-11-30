use actix_web::{
    web, get, post, HttpResponse, Responder
};
use super::model::{CupomRequest, CupomError};
use super::cupom_service;
use sqlx::MySqlPool;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Id { id: i32 }

#[derive(Deserialize, Debug)]
pub struct Code { code: String }

#[tracing::instrument(
    name = "Get all cupoms", skip(pool)
)]
#[get("cupom")]
pub async fn get_all_cupoms(pool: web::Data::<MySqlPool>) -> Result<impl Responder, CupomError> {
    let cupoms = cupom_service::get_all(&pool).await?;
    return Ok(web::Json(cupoms));
}

#[tracing::instrument(
    name = "Get cupom by id", skip(pool)
)]
#[get("cupom/id")]
pub async fn get_cupom_by_id(request: web::Json<Id>,  pool: web::Data::<MySqlPool>) -> Result<HttpResponse, CupomError> {
    let cupom = cupom_service::get_by_id(request.id, &pool).await?;
    return Ok(HttpResponse::Ok().json(cupom));
}

#[tracing::instrument(
    name = "Get cupom by code", skip(pool)
)]
#[get("cupom/code")]
pub async fn get_cupom_by_code(request: web::Json<Code>,  pool: web::Data::<MySqlPool>) -> Result<HttpResponse, CupomError> {
    let cupom = cupom_service::get_by_code(request.code.clone(), &pool).await?;
    return Ok(HttpResponse::Ok().json(cupom));
}

#[tracing::instrument(
    name = "Post cupom", skip(pool)
)]
#[post("cupom")]
pub async fn add_cupom(request: web::Json<CupomRequest>, pool: web::Data::<MySqlPool>) -> Result<HttpResponse, CupomError> {
    let cupom = cupom_service::insert(request, &pool).await?;
    return Ok(HttpResponse::Created().json(cupom));
}




