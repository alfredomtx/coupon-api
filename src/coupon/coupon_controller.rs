use super::model::{CouponInsertRequest, CouponError, CouponUpdateRequest};
use super::coupon_service;
use actix_web::{
    web, get, post, put, delete, HttpResponse, Responder,
    web::Data,
};
use sqlx::MySqlPool;


#[tracing::instrument( name = "Get all coupons", skip(pool) )]
#[get("")]
pub async fn get_all_coupons(pool: Data::<MySqlPool>) -> Result<impl Responder, CouponError> {
    let coupons = coupon_service::get_all(&pool).await?;
    return Ok(web::Json(coupons));
}

#[tracing::instrument( name = "Get coupon", skip(pool) )]
#[get("/{id_or_code}")]
pub async fn get_coupon(param: web::Path<String>, pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    let coupon = coupon_service::get_by_id_or_code(param.into_inner(), &pool).await?;
    return Ok(HttpResponse::Ok().json(coupon));
}

#[tracing::instrument( name = "Put coupon", skip(pool) )]
#[put("/{id_or_code}")]
pub async fn update_coupon(params: web::Path<String>, request: web::Json<CouponUpdateRequest>, pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    coupon_service::update(params.into_inner(), request.0, &pool).await?;
    return Ok(HttpResponse::Ok().finish());
}

#[tracing::instrument( name = "Delete coupon", skip(pool) )]
#[delete("/{id_or_code}")]
pub async fn delete_coupon(param: web::Path<String>, pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    coupon_service::delete(param.into_inner(), &pool).await?;
    return Ok(HttpResponse::NoContent().finish());
}

#[tracing::instrument( name = "Post coupon", skip(pool) )]
#[post("")]
pub async fn add_coupon(request: web::Json<CouponInsertRequest>, pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    let coupon = coupon_service::insert(request.0, &pool).await?;
    return Ok(HttpResponse::Created().json(coupon));
}

#[tracing::instrument( name = "Verify coupon", skip(pool) )]
#[get("/verify/{id_or_code}")]
pub async fn verify_coupon(param: web::Path<String>, pool: Data::<MySqlPool>) -> Result<HttpResponse, CouponError> {
    let valid_coupon = coupon_service::is_valid(param.into_inner(), &pool).await?;
    return Ok(HttpResponse::Ok().body(valid_coupon.to_string()));
}


