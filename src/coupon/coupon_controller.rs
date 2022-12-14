use actix_web::{
    web, get, post, HttpResponse, Responder,
    web::Data,
};
use super::model::{CouponRequest, CouponError};
use super::coupon_service;
use sqlx::MySqlPool;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Id { id: i32 }

#[derive(Deserialize, Debug)]
pub struct Code { code: String }

#[tracing::instrument(
    name = "Get all coupons", skip(pool)
)]
#[get("/coupon")]
pub async fn get_all_coupons(pool: Data::<MySqlPool>) -> Result<impl Responder, CouponError> {
    let coupons = coupon_service::get_all(&pool).await?;
    return Ok(web::Json(coupons));
}

#[tracing::instrument(
    name = "Get coupon by id", skip(pool)
)]
#[get("/coupon/id")]
pub async fn get_coupon_by_id(request: web::Json<Id>,  pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    let coupon = coupon_service::get_by_id(request.id, &pool).await?;
    return Ok(HttpResponse::Ok().json(coupon));
}

#[tracing::instrument(
    name = "Get coupon by code", skip(pool)
)]
#[get("/coupon/code")]
pub async fn get_coupon_by_code(request: web::Json<Code>,  pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    let coupon = coupon_service::get_by_code(request.code.clone(), &pool).await?;
    return Ok(HttpResponse::Ok().json(coupon));
}

#[tracing::instrument(
    name = "Post coupon", skip(pool)
)]
#[post("/coupon")]
pub async fn add_coupon(request: web::Json<CouponRequest>, pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    let coupon = coupon_service::insert(request, &pool).await?;
    return Ok(HttpResponse::Created().json(coupon));
}



