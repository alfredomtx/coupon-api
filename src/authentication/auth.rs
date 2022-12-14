use super::model::{LoginRequest, Role};
use crate::{startup::ApplicationApiKey, authentication::LoginError};
use actix_web::{
    get, post, HttpResponse,Responder,
    web::{self, Data},
};
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use actix_jwt_auth_middleware::{AuthResult, AuthError, CookieSigner, FromRequest};
use jwt_compact::alg::Ed25519;

#[derive(Serialize, Deserialize, Clone, Debug, FromRequest)]
pub struct User {
    pub id: u32,
}

#[tracing::instrument(name = "Authenticate", skip(request, cookie_signer, api_key))]
#[post("/authenticate")]
async fn authenticate(request: web::Json<LoginRequest>, api_key: Data<ApplicationApiKey>, cookie_signer: web::Data<CookieSigner<User, Ed25519>>) -> AuthResult<HttpResponse> {
    tracing::Span::current().record("request api key", &tracing::field::display(&request.api_key));

    let api_key = String::from(&api_key.0);
    if (request.api_key != api_key){
        return Err(AuthError::Internal(LoginError::AuthorizationError(anyhow!("Invalid api_key.")).into()));
    }

    let user = User { id: 1 };
    Ok(HttpResponse::Ok()
        .cookie(cookie_signer.create_access_token_cookie(&user)?)
        .cookie(cookie_signer.create_refresh_token_cookie(&user)?)
        .body("You are now logged in"))
}

#[get("/hello")]
async fn hello(user: User) -> impl Responder {
    format!("Hello there, i see your user id is {}.", user.id)
}
