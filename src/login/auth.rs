
use secrecy::{Secret, ExposeSecret};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use actix_jwt_auth_middleware::{AuthError, Authority, FromRequest};
use actix_web::{
    get,
    App, HttpResponse, HttpServer, Responder,
    web::{self, Data},
};
use super::role::*;
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool};
use anyhow::{Result, anyhow};

use super::model::{LoginRequest, LoginError, Role};
use super::login_repository;
use anyhow::Context;



pub struct Credentials {
    pub email: String,
    pub password: Secret<String>,
}



#[derive(Serialize, Deserialize, Clone, FromRequest)]
pub struct UserClaims {
    pub id: u32,
    pub role: Role,
}

pub async fn verify_service_request(user_claims: UserClaims) -> bool {
    match user_claims.role {
        Role::Admin => true,
        Role::BaseUser => false,
    }
}

#[get("/hello")]
pub async fn hello(user_claims: UserClaims) -> impl Responder {
    format!("Hello user with id: {}!", user_claims.id)
}


// calling this route will give you access to the rest of the apps scopes
#[get("/login")]
pub async fn login_handler(request: web::Json<LoginRequest>, auth_authority: Data<Authority<UserClaims>>,  pool: Data::<MySqlPool>) -> Result<HttpResponse, AuthError> {



    if let Ok((user_id, role)) = authenticate(request.email.clone(), request.password.clone(), &pool).await {
        

        let cookie = auth_authority.create_signed_cookie(UserClaims {
            id: user_id as u32,
            role: Role::Admin,
        })?;

        let cookie_response = cookie.clone();

        Ok(HttpResponse::Accepted()
            .cookie(cookie)
            .body(format!("You are now logged in: {}", cookie_response  )))
        
    } else {
        return Err(AuthError::Unauthorized);
    }
    

}



pub async fn authenticate(email: String, password: String, pool: &MySqlPool) -> Result<(i32, String), LoginError> {
    let result = login_repository::get_by_email(email.clone(), pool).await
    .map_err(|error| LoginError::UnexpectedError(error.into()))?;

        
    let (user_id, role) = result.ok_or(LoginError::AuthorizationError(anyhow!(format!("User with email `{}` not found", email))))?;

    return Ok((user_id, role));
}


// calling this route will not give you access to the rest of the apps scopes because you are not an admin
#[get("/login-as-base-user")]
pub async fn login_as_base_user(
    auth_authority: Data<Authority<UserClaims>>,
) -> Result<HttpResponse, AuthError> {
    let cookie = auth_authority.create_signed_cookie(UserClaims {
        id: 69,
        role: Role::BaseUser,
    })?;

    Ok(HttpResponse::Accepted()
        .cookie(cookie)
        .body("You are now logged in"))
}


// We extracted the db-querying logic in its own function with its own span.
#[tracing::instrument(name = "Get stored credentials", skip(email, pool))]
async fn get_stored_credentials(email: &str, pool: &MySqlPool) -> Result<Option<(i32, Secret<String>)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, password, role
        FROM user
        WHERE email = ?
        "#,
        email,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to performed a query to retrieve stored credentials.")?
    .map(|row| (row.id, Secret::new(row.password)));
    return Ok(row);
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_credentials(credentials: Credentials, pool: &MySqlPool) -> Result<i32, LoginError> {
    let row: Option<_> = sqlx::query!(
        r#"
        SELECT id, password, role 
        FROM user 
        WHERE email = ?
        "#,
        credentials.email
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform query to retrieve user credentials")
    .map_err(LoginError::UnexpectedError)?;

    let (expected_password_hash, user_id) = match row {
        Some(row) => (row.password, row.id),
        None => {
            return Err(LoginError::AuthorizationError(anyhow::anyhow!("Unknown email")));
        }
    };

    let expected_password_hash = PasswordHash::new(&expected_password_hash)
        .context("Failed to parse hash in PHC string format.")
        .map_err(LoginError::UnexpectedError)?;

    tracing::info_span!("Verify password hash")
        .in_scope(|| {
            Argon2::default()
                .verify_password(credentials.password.expose_secret().as_bytes(), &expected_password_hash)
        })
        .context("Invalid password")
        .map_err(LoginError::AuthorizationError)?;

    return Ok(user_id);
}
