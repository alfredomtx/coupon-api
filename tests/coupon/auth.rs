use crate::helpers::{spawn_app};

#[tokio::test]
async fn requests_missing_authorization_cookie_are_rejected() {
    // Arrange
    let app = spawn_app().await;

    // vector with all methods that the API accept,  we will loop through them,
    // make the requests and expect that it will return `401 Unauthorized` as response.
    let endpoints = vec![
        ("get", "/"),
        ("get", "/id"),
        ("get", "/code"),
        ("post", "/"),
        ("patch", "/"),
        ("delete", "/id"),
        ("delete", "/code"),
    ];

    // Act
    for (method, endpoint) in endpoints {
        let response: reqwest::Response;
        match method {
            "get" => {
                response = app.api_client.get(&format!("{}/coupon{}", &app.address, endpoint)).send().await.unwrap();
            },
            "post" => {
                response = app.api_client.post(&format!("{}/coupon{}", &app.address, endpoint)).send().await.unwrap();
            },
            "patch" => {
                response = app.api_client.patch(&format!("{}/coupon{}", &app.address, endpoint)).send().await.unwrap();
            },
            "delete" => {
                response = app.api_client.delete(&format!("{}/coupon{}", &app.address, endpoint)).send().await.unwrap();
            },
            _ => panic!("{}", format!("Invalid method: {}", method)),
        }

        // Assert
        assert_eq!(401, response.status().as_u16());
    }

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