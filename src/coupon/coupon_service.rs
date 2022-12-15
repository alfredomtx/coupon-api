use sqlx::{MySqlPool};
use actix_web::web::Json;
use anyhow::{Context, Result, anyhow};

use super::model::{CouponRequest, CouponResponse, CouponError, CouponInsert};
use super::{coupon_repository};
// use super::coupon_repository::Fields;


pub async fn get_all(pool: &MySqlPool) -> Result<Vec<CouponResponse>, CouponError> {
    let coupons = coupon_repository::get_all(pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?;

    let cumpoms_response = coupons
        .into_iter()
        // flat_map uses an iterator over the result of the mapping and as a consequence,
        // it will skip over elements for which the mapping closure returns empty or unsuccessful values
        .flat_map(|coupon| {
            match coupon.try_into() {
                Ok(coupon) => Some(coupon),
                Err(coupon) => {
                    tracing::error!("Failed to try_into() {:?}", coupon);
                    None
                }
            }
        })
        .collect();
    return Ok(cumpoms_response);
}

pub async fn get_by_id(id: i32, pool: &MySqlPool) -> Result<CouponResponse, CouponError> {
    let result = coupon_repository::get_by_id(id, pool).await
    // let result = coupon_repository::get_by_field(Fields::Id(id), pool).await
        .context("failed to get by id")?;

    let coupon = result.ok_or( CouponError::NotFoundError(anyhow!(format!("Coupon with id `{}` not found", id))))?;

    let coupon_response = coupon.try_into().map_err(CouponError::InternalError)?;
    return Ok(coupon_response);
}

pub async fn get_by_code(code: String, pool: &MySqlPool) -> Result<CouponResponse, CouponError> {
    let result = coupon_repository::get_by_code(code.clone(), pool).await
    // let result = coupon_repository::get_by_field(Fields::Code(code.clone()), pool).await
    .context("failed to get by code")?;

    let coupon = result.ok_or(CouponError::NotFoundError(anyhow!(format!("Coupon with code `{}` not found", code))))?;

    let coupon_response = coupon.try_into().map_err(CouponError::InternalError)?;
    return Ok(coupon_response);
}

pub async fn insert(coupon: Json<CouponRequest>, pool: &MySqlPool) -> Result<CouponResponse, anyhow::Error> {
    let coupon_insert = CouponInsert {
        code: coupon.code.to_string(),
        discount: coupon.discount,
        max_usage_count: coupon.max_usage_count
    };

    let inserted_id = coupon_repository::insert(coupon_insert, pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?;

    let inserted_coupon = coupon_repository::get_by_id(inserted_id as i32, pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?;

    let coupon = inserted_coupon.ok_or(CouponError::NotFoundError(anyhow!(format!("Inserted coupon with id `{}` not found", inserted_id))))?;

    let coupon_response = coupon.try_into().map_err(CouponError::InternalError)?;
    return Ok(coupon_response);
}
