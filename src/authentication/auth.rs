
use actix_web::{
    web, post,
    dev::{ServiceRequest}, HttpResponse,
    web::Data,
};
use redis::{AsyncCommands};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use anyhow::{Result};
use uuid::Uuid;

use crate::configuration::ApiKey;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Bearer {
    token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiKeyRequest {
    pub api_key: String,
}

#[tracing::instrument(name = "Validator", skip(request))]
// when sending a request to any route under auth middleware send a dummy bearer authentication token
pub async fn validator(request: ServiceRequest, _: actix_web_httpauth::extractors::bearer::BearerAuth,) -> Result<ServiceRequest, actix_web::Error> {
    // get Bearer token from `Authorization` header
    let autorization_header;
    match request.headers().get("Authorization") {
        Some(header) => autorization_header = header,
        _ => return Err(actix_web::error::ErrorBadRequest("`Authorization` header is missing.")),
    };

    let request_bearer;
    match autorization_header.to_str() {
        Ok(token) => request_bearer = token,
        _ => return Err(actix_web::error::ErrorBadRequest("`Authorization` header is invalid.")),
    };

    let redis = request.app_data::<web::Data<redis::Client>>();
    if (redis.is_none()){
        return Err(actix_web::error::ErrorInternalServerError("Failed to get `redis` data from app data."));
    }

    // Decode the `Authorization` header value from base64
    let decoded = base64::decode(request_bearer.replace("Bearer ", ""))
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Failed to decode base64 header: {}.", e)))?;

    let bearer = String::from_utf8(decoded)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to parse decoded base64 to string: {}.", e)))?;

    // Convert base64 to string
    let session_id;
    // split Bearer token and get the first part, that contains the session id
    if let Some(bearer_request) = bearer.split(":").next(){
        session_id = bearer_request;
    } else {
        return Err(actix_web::error::ErrorBadRequest("Bearer header is invalid."));
    }

    // get connection to the redis database
    let mut con = redis.unwrap().get_async_connection()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to get `redis` connection: {}.", e)))?;

    // query redis using the `session_id` from Bearer as key
    let result: Option<String> = con.get(session_id).await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to query `redis`: {}.", e)))?;

    if let None = result {
        return Err(actix_web::error::ErrorUnauthorized("Bearer token is invalid or has expired."));
    }

    return Ok(request);
}


#[tracing::instrument(name = "Authenticate", skip(request, redis, api_key))]
// when sending a request to any route under auth middleware send a dummy bearer authentication token
#[post("/auth")]
pub async fn authenticate(request: web::Json<ApiKeyRequest>, redis: Data<redis::Client>, api_key: Data<ApiKey>) -> Result<HttpResponse, actix_web::Error> {

    let api_key = api_key.0.expose_secret().to_string();
    if (request.api_key != api_key){
        return Err(actix_web::error::ErrorUnauthorized("Request token is invalid"));
    }

    let mut conn = redis
        .get_async_connection()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to get `redis` connection: {}.", e)))?;

    let session_id = Uuid::new_v4();
    // we are only using the session id for its `key`, the `value` actually is not being used
    // so lets insert it empty to save some memory.
    let session_token = "".to_string();
    
    // 1 hour
    let expiration = 1 * 60 * 60;
    // insert on redis the session as session_id = session_token
    conn.set_ex(session_id.to_string(), session_token.to_string(), expiration)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to insert session token: {}.", e)))?;


    let bearer_base64 = base64::encode(format!("{}:{}", session_id.to_string(), session_token.to_string()));
    let bearer = format!("Bearer {}", bearer_base64);

    // request.extensions_mut().insert(Bearer { token: String::from(request_token) });
    return Ok(HttpResponse::Ok().json(bearer));
}

