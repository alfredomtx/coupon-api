use actix_jwt_auth_middleware::{AuthError, Authority, FromRequest};
use actix_web::{
    get, HttpResponse,Responder,
    web::{self, Data},
};
use crate::startup::ApplicationApiKey;

use super::role::*;
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool};
use anyhow::{Result};

use super::model::{LoginRequest, LoginError, Role};


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
#[get("/authenticate")]
pub async fn authenticate(request: web::Json<LoginRequest>, auth_authority: Data<Authority<UserClaims>>, api_key: Data<ApplicationApiKey>,  pool: Data::<MySqlPool>) -> Result<HttpResponse, AuthError> {

    let api_key = String::from(&api_key.0);
    if (request.api_key != api_key){
        return Err(AuthError::Unauthorized);
    }

    let cookie = auth_authority.create_signed_cookie(UserClaims {
        id: 1 as u32,
        role: Role::Admin,
    })?;

    Ok(HttpResponse::Accepted()
        .cookie(cookie)
        .body("You are now logged in"))
}
