use super::model::{LoginRequest, Role};
use crate::startup::ApplicationApiKey;
use actix_jwt_auth_middleware::{AuthError, Authority, FromRequest};
use actix_web::{
    get, post, HttpResponse,Responder,
    web::{self, Data},
};
use serde::{Deserialize, Serialize};
use anyhow::{Result};

#[derive(Serialize, Deserialize, Clone, FromRequest)]
pub struct UserClaims {
    pub id: u32,
    pub role: Role,
}

pub async fn verify_request(user_claims: UserClaims) -> bool {
    match user_claims.role {
        Role::Admin => true,
        Role::BaseUser => false,
    }
}

#[get("/hello")]
pub async fn hello(user_claims: UserClaims) -> impl Responder {
    format!("User with id: {}, role: {}", user_claims.id, user_claims.role)
}

#[get("/login")]
async fn login_handler(cookie_signer: web::Data<CookieSigner<User, Ed25519>>) -> AuthResult<HttpResponse> {
    let user = User { id: 1 };
    Ok(HttpResponse::Ok()
        .cookie(cookie_signer.create_access_token_cookie(&user)?)
        .cookie(cookie_signer.create_refresh_token_cookie(&user)?)
        .body("You are now logged in"))
}

// calling this route will give you access to the rest of the apps scopes
#[tracing::instrument(name = "Authenticate", skip(auth_authority, api_key))]
#[post("/authenticate")]
pub async fn authenticate(request: web::Json<LoginRequest>, auth_authority: Data<Authority<UserClaims>>, api_key: Data<ApplicationApiKey>) -> Result<HttpResponse, AuthError> {

    tracing::Span::current().record("request api key", &tracing::field::display(&request.api_key));

    let api_key = String::from(&api_key.0);
    if (request.api_key != api_key){
        return Err(AuthError::Unauthorized);
    }

    let cookie = auth_authority.create_signed_cookie(UserClaims {
        id: 1,
        role: Role::Admin,
    })?;

    return Ok(HttpResponse::Accepted()
        .cookie(cookie)
        .body("You are now logged in"));
}
