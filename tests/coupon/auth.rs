use reqwest::header::HeaderMap;
use secrecy::ExposeSecret;
use serde_json::json;

use crate::helpers::{spawn_app};


#[tokio::test]
async fn auth_works_and_returns_valid_bearer_token() {
    // Arrange
    let app = spawn_app().await;

    // Act
    // post to `/auth` with correct api key
    let body = json!({"api_key": app.api_key.0.expose_secret()});
    let response = reqwest::Client::new()
            .post(&format!("{}/auth", &app.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to perform POST request to `/auth`.");
        
    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let response_body: String = response.json().await
        .expect("Failed to get `/auth` response text.");

    assert!(response_body.contains("Bearer "));

    let decoded = base64::decode(response_body.replace("Bearer ", ""))
        .expect("Failed to decode bearer base64");
        
    let bearer = String::from_utf8(decoded)
        .expect("Failed to parse decoded to string.");

    assert!(bearer.contains(":"));
}

#[tokio::test]
async fn request_missing_authorization_header_is_rejected() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    for (method, endpoint) in get_test_endpoints() {
        authorization_test_request(401, &client, &app.address, method, endpoint, "").await;
    }
}

#[tokio::test]
async fn request_with_invalid_authorization_header_is_rejected() {
    // Arrange
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", "".parse().unwrap());

    // Act 1
    for (method, endpoint) in get_test_endpoints() {
        authorization_test_request(401, &client, &app.address, method, endpoint, "Missing Bearer").await;
    }

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", "Bearer ".parse().unwrap());

    // Act 2
    for (method, endpoint) in get_test_endpoints() {
        authorization_test_request(401, &client, &app.address, method, endpoint, "Empty Bearer").await;
    }

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", "Bearer 💩".parse().unwrap());

    // Act 3
    for (method, endpoint) in get_test_endpoints() {
        authorization_test_request(401, &client, &app.address, method, endpoint, "Invalid Bearer").await;
    }
}

async fn authorization_test_request(expected_status: u16, client: &reqwest::Client, address: &str, method: &str, endpoint: &str, test_identifier: &str) -> reqwest::Response {
    let response: reqwest::Response;
    match method {
        "get" => {
            response = client.get(&format!("{}/coupon{}", address, endpoint)).send().await.unwrap();
        },
        "post" => {
            response = client.post(&format!("{}/coupon{}", address, endpoint)).send().await.unwrap();
        },
        "put" => {
            response = client.put(&format!("{}/coupon{}", address, endpoint)).send().await.unwrap();
        },
        "delete" => {
            response = client.delete(&format!("{}/coupon{}", address, endpoint)).send().await.unwrap();
        },
        _ => panic!("{}", format!("Invalid method: {}", method)),
    }
    assert_eq!(
        response.status().as_u16(),
        expected_status,
        "[Test `{}`]: The `{}` request to endpoint `{}` did not fail with `{}`.",
        test_identifier, method, endpoint, expected_status,
    );

    return response;
}

fn get_test_endpoints() -> Vec<(&'static str, &'static str)> {
    return vec![
        ("get", "/"),
        ("get", "/all"),
        ("post", "/"),
        ("put", "/"),
        ("delete", "/id"),
        ("delete", "/code"),
    ];
}
