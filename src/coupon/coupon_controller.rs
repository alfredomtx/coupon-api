use super::model::{CouponRequest, CouponError, CouponUpdate, CouponQueryRequest};
use super::coupon_service;
use actix_web::{
    web, get, post, patch, delete, HttpResponse, Responder,
    web::Data,
};
use sqlx::MySqlPool;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Id { id: i32 }

#[derive(Deserialize, Debug)]
pub struct Code { code: String }

#[tracing::instrument(
    name = "Get all coupons", skip(pool)
)]
#[get("/coupon/all")]
pub async fn get_all_coupons(pool: Data::<MySqlPool>) -> Result<impl Responder, CouponError> {
    let coupons = coupon_service::get_all(&pool).await?;
    return Ok(web::Json(coupons));
}

#[tracing::instrument(
    name = "Get coupon", skip(pool)
)]
#[get("/coupon")]
pub async fn get_coupon(params: web::Query<CouponQueryRequest>, pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    let coupon = coupon_service::get_by_id_or_code(params.into_inner(), &pool).await?;
    return Ok(HttpResponse::Ok().json(coupon));
}

#[tracing::instrument(
    name = "Post coupon", skip(pool)
)]
#[post("/coupon")]
pub async fn add_coupon(request: web::Json<CouponRequest>, pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    let coupon = coupon_service::insert(request.0, &pool).await?;
    return Ok(HttpResponse::Created().json(coupon));
}

#[tracing::instrument(
    name = "Update coupon", skip(pool)
)]
#[patch("/coupon")]
pub async fn update_coupon(params: web::Query<CouponQueryRequest>, request: web::Json<CouponUpdate>, pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    coupon_service::update(params.into_inner(), request.0, &pool).await?;
    return Ok(HttpResponse::Ok().finish());
}

#[tracing::instrument(
    name = "Delete coupon by id", skip(pool)
)]
#[delete("/coupon/id")]
pub async fn delete_coupon_by_id(request: web::Json<Id>, pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    coupon_service::delete_by_id(request.id, &pool).await?;
    return Ok(HttpResponse::Ok().finish());
}

#[tracing::instrument(
    name = "Delete coupon by code", skip(pool)
)]
#[delete("/coupon/code")]
pub async fn delete_coupon_by_code(request: web::Json<Code>, pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    coupon_service::delete_by_code(request.code.clone(), &pool).await?;
    return Ok(HttpResponse::Ok().finish());
}

#[tracing::instrument(
    name = "Verify coupon", skip(pool)
)]
#[get("/coupon/verify")]
pub async fn verify_coupon(params: web::Query<CouponQueryRequest>, pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    let valid_coupon = coupon_service::is_valid(params.into_inner(), &pool).await?;
    return Ok(HttpResponse::Ok().body(valid_coupon.to_string()));
}


