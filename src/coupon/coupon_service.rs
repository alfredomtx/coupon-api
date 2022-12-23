use super::model::{CouponRequest, CouponResponse, CouponError, CouponInsert, CouponUpdate};
use super::{coupon_repository, CouponQueryRequest};
use sqlx::{MySqlPool};
use actix_web::web::Json;
use anyhow::{Context, Result, anyhow};
use std::convert::TryFrom;

pub async fn get_all(pool: &MySqlPool) -> Result<Vec<CouponResponse>, CouponError> {
    let coupons = coupon_repository::get_all(pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?;

    let coupons_response = coupons
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
    return Ok(coupons_response);
}

pub async fn get_by_id(id: i32, pool: &MySqlPool) -> Result<CouponResponse, CouponError> {
    let result = coupon_repository::get_by_id(id, pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?;

    let coupon = result.ok_or( CouponError::NotFoundError(anyhow!(format!("Coupon with id `{}` not found", id))))?;

    let coupon_response = coupon.try_into().map_err(|e| CouponError::InternalError(anyhow!(format!("Failed to create CouponResponse: {}", e))))?;
    return Ok(coupon_response);
}

pub async fn get_by_code(code: String, pool: &MySqlPool) -> Result<CouponResponse, CouponError> {
    let result = coupon_repository::get_by_code(&code, pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?;

    let coupon = result.ok_or(CouponError::NotFoundError(anyhow!(format!("Coupon with code `{}` not found", code))))?;

    let coupon_response = coupon.try_into().map_err(|e| CouponError::InternalError(anyhow!(format!("Failed to create CouponResponse: {}", e))))?;
    return Ok(coupon_response);
}

pub async fn get_by_id_or_code(params: CouponQueryRequest, pool: &MySqlPool) -> Result<CouponResponse, CouponError> {
    // if the `id` param is present and it is an integer, then we get by id, otherwise by code
    if let Some(id_param) = params.id {
        match id_param.parse::<i32>() {
            Ok(id) => return get_by_id(id, pool).await,
            Err(_) => {},
        }
    };

    if let Some(code) = params.code {
        return get_by_code(code, pool).await;
    }

    return Err(CouponError::ValidationError("Both `id` and `code` params are missing from URL query, one is required.".to_string()));
}


pub async fn insert(coupon: Json<CouponRequest>, pool: &MySqlPool) -> Result<CouponResponse, anyhow::Error> {
    let coupon_insert = CouponInsert {
        code: coupon.code.to_string(),
        discount: coupon.discount,
        active: coupon.active,
        max_usage_count: coupon.max_usage_count,
        expiration_date: coupon.expiration_date,
    };

    let inserted_id = coupon_repository::insert(coupon_insert, pool).await
    .map_err(|e| CouponError::InternalError(anyhow!(format!("Something went wrong and the coupon was not inserted: {}", e))))?;

    let inserted_id = i32::try_from(inserted_id).or_else(|e| Err(CouponError::InternalError(anyhow!(format!("Failed to read inserted_id: {}", e)))))?;

    let inserted_coupon = coupon_repository::get_by_id(inserted_id, pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?;

    let coupon = inserted_coupon.ok_or(CouponError::NotFoundError(anyhow!(format!("Inserted coupon with id `{}` not found", inserted_id))))?;

    let coupon_response = coupon.try_into().map_err(|e| CouponError::InternalError(anyhow!(format!("Failed to create CouponResponse: {}", e))))?;
    return Ok(coupon_response);
}

pub async fn update(coupon: Json<CouponUpdate>, pool: &MySqlPool) -> Result<(), CouponError> {
    let coupon = coupon.0;
    // check if the coupon exists
    coupon_repository::get_by_id(coupon.id, pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?
        .ok_or(CouponError::NotFoundError(anyhow!(format!("Coupon with id `{}` not found", coupon.id))))?;

    let coupon_update = CouponUpdate {
        id: coupon.id,
        code: coupon.code,
        discount: coupon.discount,
        active: coupon.active,
        max_usage_count: coupon.max_usage_count,
        expiration_date: coupon.expiration_date,
    };

    coupon_repository::update(coupon_update, &pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?;

    return Ok(());
}

pub async fn delete_by_id(id: i32, pool: &MySqlPool) -> Result<(), CouponError> {
    coupon_repository::get_by_id(id, pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?
        .ok_or(CouponError::NotFoundError(anyhow!(format!("Coupon with id `{}` not found", id))))?;

    coupon_repository::delete_by_id(id, pool).await
        .context("Failed to delete by id")?;

    coupon_repository::delete_by_id(id, &pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?;
    return Ok(());
}

pub async fn delete_by_code(code: String, pool: &MySqlPool) -> Result<(), CouponError> {
    coupon_repository::get_by_code(&code, pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?
        .ok_or(CouponError::NotFoundError(anyhow!(format!("Coupon with code `{}` not found", &code))))?;

    coupon_repository::delete_by_code(&code, pool).await
        .context("Failed to delete by code")?;
            
    coupon_repository::delete_by_code(&code, &pool).await
        .map_err(|error| CouponError::UnexpectedError(error.into()))?;
    return Ok(());
}
