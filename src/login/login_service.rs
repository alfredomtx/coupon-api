use sqlx::{MySqlPool};
use anyhow::{Result, anyhow};

use super::model::{LoginRequest, LoginResponse, LoginError, Role};
use super::login_repository;

pub async fn authenticate(email: String, pool: &MySqlPool) -> Result<(), LoginError> {
    let result = login_repository::get_by_email(email.clone(), pool).await
        .map_err(|error| LoginError::UnexpectedError(error.into()))?;

    let (user_id, role) = result.ok_or(LoginError::AuthorizationError(anyhow!(format!("User with email `{}` not found", email))))?;

    return Ok(());
}


