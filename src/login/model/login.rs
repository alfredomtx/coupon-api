
use actix_web::{ 
    ResponseError,
    http:: {
        StatusCode,
    },
};
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    // #[error("Not found.)]
    // NotFoundError,
    #[error("Not found.")]
    NotFoundError(#[source] anyhow::Error),
    #[error("Invalid credentials.")]
    AuthorizationError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::NotFoundError(_) => StatusCode::NOT_FOUND,
            LoginError::AuthorizationError(_) => StatusCode::UNAUTHORIZED,
            LoginError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}