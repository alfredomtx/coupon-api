use crate::helpers::{spawn_app};

#[tokio::test]
async fn requests_missing_cookie_are_rejected() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = reqwest::Client::new()
        .get(&format!("{}/coupon", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn authenticate_returns_a_valid_cookie() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.authenticate_request().await;

    assert_eq!(200, response.status().as_u16());
    
    let cookie =  response.headers().get("Set-Cookie").unwrap().to_str().unwrap();
    let unsecure_cookie = cookie.replace(" Secure", "");

    // Act
    let response = reqwest::Client::new()
        .get(&format!("{}/hello", &app.address))
        .header("Cookie", unsecure_cookie)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());
}