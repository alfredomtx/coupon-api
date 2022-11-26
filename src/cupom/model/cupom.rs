use actix_web::{ 
    Responder, HttpResponse, HttpRequest, ResponseError,
    body::BoxBody,
    http:: {
        StatusCode,
        header::ContentType,
    },
};
use serde::{Serialize, Deserialize};
use crate::helpers::error_chain_fmt;
use anyhow::Context;

#[derive(Serialize, Deserialize, Debug)]
pub struct Cupom {
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

#[derive(thiserror::Error)]
pub enum CupomError {
    #[error("Something went wrong.")]
    GenericError(#[source] anyhow::Error),
    #[error("Not found.")]
    NotFoundError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

// // Implement Responder Trait for CupomResponse
// impl Responder for CupomResponse {
//     type Body = BoxBody;

//     fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
//         let res_body = serde_json::to_string(&self).unwrap();

//         // Create HttpResponse and set Content Type
//         return HttpResponse::Ok()
//            .content_type(ContentType::json())
//            .body(res_body);
//     }
// }

impl ResponseError for CupomError {
    fn status_code(&self) -> StatusCode {
        match self {
            CupomError::NotFoundError(_) => StatusCode::NOT_FOUND,
            CupomError::GenericError(_) => StatusCode::BAD_REQUEST,
            CupomError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// Same logic to get the full error chain on `Debug` 
impl std::fmt::Debug for CupomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return error_chain_fmt(self, f);
    }
}
