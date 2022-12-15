use crate::helpers::{spawn_app};
use coupon_api::coupon::{Coupon};
use rand::distributions::{Alphanumeric, DistString};
use serde_json::json;


#[tokio::test]
async fn post_persists_and_returns_the_new_coupon() {
    // Arrange
    let app = spawn_app().await;
    // generating a random string
    let code = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let body = get_coupon_request_body(Some(code.clone()));
    
    // Act
    let response = app.post_coupon(body, true).await;
    let response_status = response.status().as_u16();
    let response_body = response.text().await.expect("failed to get response_body");

    // Assert
    assert_eq!(201, response_status);

    let coupon: Coupon = serde_json::from_str(&response_body).unwrap();
    
    assert_coupon_fields(coupon.code, code, coupon.discount, coupon.max_usage_count);
}

#[tokio::test]
async fn post_persists_the_new_coupon() {
    // Arrange
    let app = spawn_app().await;
    let body = get_coupon_request_body(None);
    
    // Act
    let response = app.post_coupon(body, true).await;
    let response_status = response.status().as_u16();

    assert_eq!(201, response_status);

    let code = "TestCoupon";
    let error_message = format!("Failed to fetch saved coupon, no coupon was persisted.\nResponse Status:{}", response_status);
    let saved = sqlx::query!("SELECT * FROM coupon WHERE code = ?", code)
        .fetch_one(&app.db_pool)
        .await
        .expect(error_message.as_str());

    // Assert
    assert_coupon_fields(saved.code, "TestCoupon".to_string(), saved.discount, saved.max_usage_count);
}

#[tokio::test]
async fn get_coupon_by_id_returns_a_coupon() {
    // Arrange
    let app = spawn_app().await;
    let code = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let body = get_coupon_request_body(Some(code.clone()));
    
    // Act
    // add the coupon before getting it
    app.post_coupon(body, true).await;

    // get coupon with id 1
    let response = app.get_coupon("/id", json!({"id": 1})).await;
    let response_body = response.text().await.expect("failed to get response_body");
    let coupon: Coupon = serde_json::from_str(&response_body).unwrap();

    // Assert
    assert_coupon_fields(coupon.code, code, coupon.discount, coupon.max_usage_count);
}

#[tokio::test]
async fn get_coupon_by_code_returns_a_coupon() {
    // Arrange
    let app = spawn_app().await;
    let code = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let body = get_coupon_request_body(Some(code.clone()));
    
    // Act
    // add the coupon before getting it
    app.post_coupon(body, true).await;

    // get coupon with id 1
    let response = app.get_coupon("/code", json!({"code": code})).await;
    let response_body = response.text().await.expect("failed to get response_body");
    let coupon: Coupon = serde_json::from_str(&response_body).unwrap();

    // Assert
    assert_coupon_fields(coupon.code, code, coupon.discount, coupon.max_usage_count);
}


#[tokio::test]
async fn get_all_coupons_returns_a_list_of_coupons() {
    // Arrange
    let app = spawn_app().await;
    let code1 = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let body1 = get_coupon_request_body(Some(code1.clone()));
    let code2 = Alphanumeric.sample_string(&mut rand::thread_rng(), 26);
    let body2 = get_coupon_request_body(Some(code2.clone()));
    
    // Act
    // add 2 coupons
    app.post_coupon(body1, true).await;
    app.post_coupon(body2, true).await;

    // get coupons
    let response = app.get_coupon("", json!({})).await;
    let response_body = response.text().await.expect("failed to get response_body");
    let coupons: Vec<Coupon> = serde_json::from_str(&response_body).unwrap();

    // Assert
    assert!(coupons.len() > 1);

    // iterate through the cupons and get only the 2 coupon with the codes we added before
    let added_coupons: Vec<Coupon> = coupons.into_iter()
    .filter(|coupon| coupon.code == code1 || coupon.code == code2)
    .collect();

    assert!(added_coupons.len() == 2);
}



#[tokio::test]
async fn coupon_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;

    let test_cases = vec![
        (json!({"discount": 1}), "missing code"),
        (json!({"code": 1}), "missing discount"),
        (json!({"discount": "a"}), "discount is a string"),
        (json!({"code": 1}), "code is an integer"),
    ];

    // Act 
    for (invalid_body, error_message) in test_cases {
        let response = app.post_coupon(invalid_body, false).await;

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

    let test_cases = vec![
        (json!({"id": 123456789}), "not found by id", "/id"),
        (json!({"code": "code that does not exist"}), "not found by code", "/code")
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

fn assert_coupon_fields(code: String, expected_code: String, discount: i32, max_usage_count: Option<i32>){
    assert_eq!(code, expected_code);
    assert_eq!(discount, 10);
    assert_eq!(max_usage_count, Some(2));
    // assert_eq!(expiration_date, Some(NaiveDateTime::from_str("31/12/2030 00:00:00").unwrap()));
}

fn get_coupon_request_body(code: Option<String>) -> serde_json::Value {
    let coupon_code;
    match code {
        Some(code) => coupon_code = code,
        None => coupon_code = "TestCoupon".to_string()
    };
    return json!({
        "code": coupon_code,
        "discount": 10,
        "max_usage_count": 2,
        // "expiration_date": "31/12/2030 00:00:00",
    });
}

// #[tokio::test]
// async fn coupon_fails_if_there_is_a_fatal_database_error() {
//     // Arrange
//     let app = spawn_app().await;
//     let body = get_coupon_request_body(None);

//     // Sabotage the database
//     sqlx::query!("DROP TABLE coupon;",)
//         .execute(&app.db_pool)
//         .await
//         .unwrap();

//     // Act 
//     let response = app.post_coupon(body, false).await;

//     // Assert
//     assert_eq!(response.status().as_u16(), 500);
// }