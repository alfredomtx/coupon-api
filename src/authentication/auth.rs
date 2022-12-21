use crate::{startup::ApplicationApiKey};
use actix_web::{
    web, 
    dev::{ServiceRequest},
};
use serde::{Deserialize, Serialize};
use anyhow::{Result};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Bearer {
    token: String,
}

#[tracing::instrument(name = "Authenticate", skip(request))]
// when sending a request to any route under auth middleware send a dummy bearer authentication token
pub async fn validator(request: ServiceRequest, _: actix_web_httpauth::extractors::bearer::BearerAuth,) -> Result<ServiceRequest, actix_web::Error> {
    // get Bearer token from `Authorization` header
    let autorization_header;
    match request.headers().get("Authorization") {
        Some(header) => autorization_header = header,
        _ => return Err(actix_web::error::ErrorBadRequest("`Authorization` header is missing.")),
    };

    let request_token;
    match autorization_header.to_str() {
        Ok(token) => request_token = token,
        _ => return Err(actix_web::error::ErrorBadRequest("`Authorization` header is invalid.")),
    };

    // get connection to the redis database
    let api_key;
    match request.app_data::<web::Data<ApplicationApiKey>>(){
        Some(data) => api_key = data,
        _ =>  return Err(actix_web::error::ErrorInternalServerError("Failed to get api_key data.")),
    }

    let api_key = String::from(&api_key.0);
    if (request_token != api_key){
        return Err(actix_web::error::ErrorUnauthorized("Request token is invalid"));
    }
    
    // request.extensions_mut().insert(Bearer { token: String::from(request_token) });
    Ok(request)
}
