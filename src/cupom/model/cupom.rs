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
pub struct Cupom {
    pub id: i32,
    pub code: String,
    pub discount: i32,
    pub max_usage_count: Option<i32>,
    pub expiration_date: Option<NaiveDateTime>,
    pub date_created: Option<NaiveDateTime>,
    pub date_updated: Option<NaiveDateTime>,
}

#[derive(Deserialize, Debug)]
pub struct CupomInsert {
   pub code: String,
   pub discount: i32,
}

#[derive(Deserialize, Debug)]
pub struct CupomRequest {
   pub code: String,
   pub discount: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CupomResponse {
    pub id: i32,
    pub code: String,
    pub discount: i32,
}

#[derive(thiserror::Error, Debug)]
pub enum CupomError {
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

impl CupomInsert {
    pub fn from_cupom(cupom: Cupom) -> CupomInsert {
        return CupomInsert {
            code: cupom.code
            , discount: cupom.discount
        };
    }
}

// Convert a Cupom to a CupomResponse
impl TryFrom<Cupom> for CupomResponse {
    type Error = String;
    fn try_from(cupom: Cupom) -> Result<Self, Self::Error> {
        return Ok( Self {
            id: cupom.id
            , code: cupom.code
            , discount: cupom.discount
        });
    }
}

impl ResponseError for CupomError {
    fn status_code(&self) -> StatusCode {
        match self {
            CupomError::NotFoundError(_) => StatusCode::NOT_FOUND,
            CupomError::ValidationError(_) => StatusCode::BAD_REQUEST,
            CupomError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// Same logic to get the full error chain on `Debug` 
// impl std::fmt::Debug for CupomError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         return error_chain_fmt(self, f);
//     }
// }
