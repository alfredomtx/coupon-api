use actix_web::{ 
    ResponseError,
    http:: {
        StatusCode,
    },
};
// use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use sqlx::types::chrono::{NaiveDateTime};
#[derive(Debug)]
pub struct Coupon {
    pub id: i32,
    pub code: String,
    pub discount: i32,
    pub max_usage_count: Option<i32>,
    pub expiration_date: Option<NaiveDateTime>,
    pub date_created: Option<NaiveDateTime>,
    pub date_updated: Option<NaiveDateTime>,
}

#[derive(Deserialize, Debug)]
pub struct CouponInsert {
    pub code: String,
    pub discount: i32,
    pub max_usage_count: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CouponRequest {
    pub code: String,
    pub discount: i32,
    pub max_usage_count: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CouponResponse {
    pub id: i32,
    pub code: String,
    pub discount: i32,
    pub max_usage_count: Option<i32>,
}

#[derive(thiserror::Error, Debug)]
pub enum CouponError {
    // #[error("Not found.)]
    // NotFoundError,
    #[error("Not found.")]
    NotFoundError(#[source] anyhow::Error),
    // ValidationError has one String parameter
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl CouponInsert {
    pub fn from_coupon(coupon: Coupon) -> CouponInsert {
        return CouponInsert {
            code: coupon.code
            , discount: coupon.discount
            , max_usage_count: coupon.max_usage_count
        };
    }
}

// Convert a Coupon to a CouponResponse
impl TryFrom<Coupon> for CouponResponse {
    type Error = String;
    fn try_from(coupon: Coupon) -> Result<Self, Self::Error> {
        return Ok( Self {
            id: coupon.id
            , code: coupon.code
            , discount: coupon.discount
            , max_usage_count: coupon.max_usage_count
        });
    }
}

impl ResponseError for CouponError {
    fn status_code(&self) -> StatusCode {
        match self {
            CouponError::NotFoundError(_) => StatusCode::NOT_FOUND,
            CouponError::ValidationError(_) => StatusCode::BAD_REQUEST,
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
