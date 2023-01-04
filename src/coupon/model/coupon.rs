use super::CouponDiscount;
use actix_web::{ 
    ResponseError,
    http::{StatusCode},
};
use serde::{Serialize, Deserialize};
// use chrono::NaiveDateTime;
use sqlx::types::chrono::{NaiveDateTime};


#[derive(Serialize, Deserialize, Debug)]
pub struct Coupon {
    pub id: i32,
    pub code: String,
    pub discount: i32,
    pub active: bool,
    pub max_usage_count: Option<i32>, // not actually being used currently, we will also need a new field to track the `current usage` count for the coupon
    pub expiration_date: Option<NaiveDateTime>,
    pub date_created: Option<NaiveDateTime>,
    pub date_updated: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CouponInsert {
    pub code: String,
    pub discount: CouponDiscount,
    pub active: bool,
    pub max_usage_count: Option<i32>,
    pub expiration_date: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CouponInsertRequest {
    pub code: String,
    pub discount: i32,
    pub active: bool,
    pub max_usage_count: Option<i32>,
    pub expiration_date: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CouponUpdate {
    pub discount: CouponDiscount,
    pub active: bool,
    pub max_usage_count: Option<i32>,
    pub expiration_date: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CouponUpdateRequest {
    pub discount: i32,
    pub active: bool,
    pub max_usage_count: Option<i32>,
    pub expiration_date: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CouponResponse {
    pub id: i32,
    pub code: String,
    pub discount: i32,
    pub active: bool,
    pub max_usage_count: Option<i32>,
    pub expiration_date: Option<NaiveDateTime>,
    pub date_created: Option<NaiveDateTime>,
    pub date_updated: Option<NaiveDateTime>,
}

// Convert a Coupon to a CouponResponse
impl TryFrom<Coupon> for CouponResponse {
    type Error = String;
    fn try_from(coupon: Coupon) -> Result<Self, Self::Error> {
        return Ok( Self {
            id: coupon.id,
            code: coupon.code,
            discount: coupon.discount,
            active: coupon.active,
            max_usage_count: coupon.max_usage_count,
            expiration_date: coupon.expiration_date,
            date_created: coupon.date_created,
            date_updated: coupon.date_updated,
        });
    }
}

impl TryFrom<CouponUpdateRequest> for CouponUpdate {
    type Error = String;
    fn try_from(coupon: CouponUpdateRequest) -> Result<Self, Self::Error> {
        let discount = CouponDiscount::parse(coupon.discount)?;
        return Ok( Self {
            discount,
            active: coupon.active,
            max_usage_count: coupon.max_usage_count,
            expiration_date: coupon.expiration_date,
        });
    }
}

impl TryFrom<CouponInsertRequest> for CouponInsert {
    type Error = String;
    fn try_from(coupon: CouponInsertRequest) -> Result<Self, Self::Error> {
        let discount = CouponDiscount::parse(coupon.discount)?;
        return Ok( Self {
            code: coupon.code,
            discount,
            active: coupon.active,
            max_usage_count: coupon.max_usage_count,
            expiration_date: coupon.expiration_date,
        });
    }
}


impl From<Coupon> for CouponInsertRequest {
    fn from(coupon: Coupon) -> Self {
        return Self {
            code: coupon.code,
            discount: coupon.discount,
            active: coupon.active,
            max_usage_count: coupon.max_usage_count,
            expiration_date: coupon.expiration_date,
        };
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CouponError {
    #[error("{0}")]
    AlreadyExistsError(#[source] anyhow::Error),
    #[error("{0}")]
    InternalError(#[source] anyhow::Error),
    // NotFoundError has one String parameter
    #[error("{0}")]
    NotFoundError(#[source] anyhow::Error),
    // ValidationError has one String parameter
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for CouponError {
    fn status_code(&self) -> StatusCode {
        match self {
            CouponError::AlreadyExistsError(_) => StatusCode::CONFLICT,
            CouponError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CouponError::NotFoundError(_) => StatusCode::NOT_FOUND,
            CouponError::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            CouponError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// Same logic to get the full error chain on `Debug` 
// impl std::fmt::Debug for CouponError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         return error_chain_fmt(self, f);
//     }
// }
