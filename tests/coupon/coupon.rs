
use coupon_api::coupon::{Coupon};
use crate::helpers::{spawn_app};
use rand::distributions::{Alphanumeric, DistString};

#[tokio::test]
async fn post_persists_and_returns_the_new_coupon() {
    // Arrange
    let app = spawn_app().await;
    // generating a random string
    let code = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let body = get_coupon_request_body(Some(code.clone()));
    
    // Act
    let response = app.post_coupon(body).await;
    let response_status = response.status().as_u16();
    let response_body = response.text()
        .await
        .expect("failed to get response_body");

    // Assert
    assert_eq!(201, response_status);

    let coupon: Coupon = serde_json::from_str(&response_body).unwrap();
    
    assert_eq!(coupon.code, code);
    assert_eq!(coupon.discount, 10);
    assert_eq!(coupon.max_usage_count, Some(2));
    // assert_eq!(coupon.expiration_date, Some(NaiveDateTime::from_str("31/12/2030 00:00:00").unwrap()));
}

#[tokio::test]
async fn post_persists_the_new_coupon() {
    // Arrange
    let app = spawn_app().await;
    let body = get_coupon_request_body(None);
    
    // Act
    let response = app.post_coupon(body).await;
    let response_status = response.status().as_u16();

    let _payload = response.text()
        .await
        .expect("failed to get payload");
    
    assert_eq!(201, response_status);

    let code = "TestCoupon";

    let error_message = format!("Failed to fetch saved coupon, no coupon was persisted.\nResponse Status:{}", response_status);
    let saved = sqlx::query!("SELECT * FROM coupon WHERE code = ?", code)
        .fetch_one(&app.db_pool)
        .await
        .expect(error_message.as_str());

    // Assert
    assert_eq!(saved.code, code);
    assert_eq!(saved.discount, 10);
    assert_eq!(saved.max_usage_count, Some(2));
    // assert_eq!(saved.expiration_date, Some(NaiveDateTime::from_str("31/12/2030 00:00:00").unwrap()));
}

#[tokio::test]
async fn coupon_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;

    let test_cases = vec![(
        serde_json::json!({"discount": 1}), "missing code",
    ),(
        serde_json::json!({"code": 1}), "missing discount",
    ),(
        serde_json::json!({"discount": "a"}), "discount is a string",
    ),(
        serde_json::json!({"code": 1}), "code is an integer",
    ),
    ];

    // Act 
    for (invalid_body, error_message) in test_cases {
        let response = app.post_coupon(invalid_body).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn coupon_not_found_returns_404(){
    // Arrange
    let app = spawn_app().await;

    let test_cases = vec![(
        serde_json::json!({"id": 123456789}), "not found by id", "/id"
    ),(
        serde_json::json!({"code": "code that does not exist"}), "not found by code", "/code"
    ),
    ];

    // Act 
    for (invalid_body, error_message, endpoint) in test_cases {
        let response = app.get_coupon(endpoint, invalid_body).await;

        // Assert
        assert_eq!(
            404,
            response.status().as_u16(),
            "The endpoint `{}` did not fail with 404 Not Found when the payload was `{}`.",
            endpoint, error_message
        );
    }
}

fn get_coupon_request_body(code: Option<String>) -> serde_json::Value {
    let coupon_code;
    match code {
        Some(code) => coupon_code = code,
        None => coupon_code = "TestCoupon".to_string()
    };
    return serde_json::json!({
        "code": coupon_code,
        "discount": 10,
        "max_usage_count": 2,
        // "expiration_date": "31/12/2030 00:00:00",
    });
}
