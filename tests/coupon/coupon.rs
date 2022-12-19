use crate::helpers::{spawn_app};
use coupon_api::coupon::{Coupon, CouponUpdate, CouponRequest};
use rand::distributions::{Alphanumeric, DistString};
use serde_json::json;

/**
 * GET
 */
#[tokio::test]
async fn get_coupon_by_id_returns_a_coupon() {
    // Arrange
    let app = spawn_app().await;
    let code = get_random_coupon_code();
    let body = get_coupon_request_body(code.clone());
    
    // Act
    // add the coupon before getting it
    app.post_coupon(body, true).await;

    let coupon = app.get_and_deserialize_coupon(1).await;

    // Assert
    // the `code` field we are not asserting here, since it is not guaranteed
    // that the `post_coupon` added the first coupon(with id 1)
    assert_coupon_fields(coupon.code.clone(), coupon.code, coupon.discount, coupon.max_usage_count);
}

#[tokio::test]
async fn get_coupon_by_code_returns_a_coupon() {
    // Arrange
    let app = spawn_app().await;
    let code = get_random_coupon_code();
    let body = get_coupon_request_body(code.clone());
    
    // Act
    // add the coupon before getting it
    app.post_coupon(body, true).await;

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
    let code1 = get_random_coupon_code();
    let body1 = get_coupon_request_body(code1.clone());
    let code2 = get_random_coupon_code();
    let body2 = get_coupon_request_body(code2.clone());
    
    // Act
    // add 2 coupons
    app.post_coupon(body1, true).await;
    app.post_coupon(body2, true).await;

    // get all coupons
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
async fn get_coupon_not_found_returns_404(){
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


/**
 * POST
 */
#[tokio::test]
async fn post_persists_and_returns_the_new_coupon() {
    // Arrange
    let app = spawn_app().await;
    let code = get_random_coupon_code();   
    let body = get_coupon_request_body(code.clone());
    
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
async fn post_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;

    let test_cases = vec![
        (json!({"discount": 1}), "missing code"),
        (json!({"code": 1}), "missing discount"),
        (json!({"discount": "a"}), "invalid discount (string)"),
        (json!({"discount": -1}), "invalid discount (negative)"),
        (json!({"code": 1}), "invalid code (integer)"),
        (json!({"code": -1}), "invalid code (negative)"),
    ];

    // Act 
    for (invalid_body, error_message) in test_cases {
        let response = app.post_coupon(invalid_body, false).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was `{}`.",
            error_message
        );
    }
}


/**
 * PATCH
 */
#[tokio::test]
async fn patch_updates_the_coupon_successfully() {
    // Arrange
    let app = spawn_app().await;
    let code = get_random_coupon_code();
    let body = get_coupon_request_body(code.clone());
    
    // Act
    // add a coupon
    let added_coupon = app.post_and_deserialize_coupon(body).await; 
        
    // update the added coupon with new data
    let body = json!({   
        "id": added_coupon.id,
        "code": "CouponWithUpdatedData",
        "discount": 99,
        "max_usage_count": 123,
    });
    
    let response = app.patch_coupon(body).await;
    let response_status = response.status().as_u16();
        
    // Assert
    assert_eq!(200, response_status);

    let updated_coupon = app.get_and_deserialize_coupon(added_coupon.id).await;

    assert_eq!(added_coupon.id, updated_coupon.id);
    assert_eq!(updated_coupon.code, "CouponWithUpdatedData".to_string());
    assert_eq!(updated_coupon.discount, 99);
    assert_eq!(updated_coupon.max_usage_count, Some(123));  
    // TODO: assert that `date_updated` has changed
}

/**
 * DELETE
 */
#[tokio::test]
async fn delete_coupon_by_id_successfully() {
    // Arrange
    let app = spawn_app().await;
    let body = get_coupon_request_body(get_random_coupon_code().clone());
    
    // Act
    // add a coupon
    let added_coupon = app.post_and_deserialize_coupon(body).await; 
    // delete added coupon
    let response = app.delete_coupon("/id", json!({"id": added_coupon.id})).await;
    let response_status = response.status().as_u16();
    assert_eq!(200, response_status);

    // try to get the deleted coupon
    let response = app.get_coupon("/id", json!({"id": added_coupon.id})).await;
    let response_status = response.status().as_u16();

    // Assert 2
    assert_eq!(404, response_status);
}

#[tokio::test]
async fn delete_coupon_by_code_successfully() {
    // Arrange
    let app = spawn_app().await;
    let body = get_coupon_request_body(get_random_coupon_code().clone());
    
    // Act
    // add a coupon
    let added_coupon = app.post_and_deserialize_coupon(body).await; 
    // delete added coupon
    let response = app.delete_coupon("/code", json!({"code": added_coupon.code.clone()})).await;
    let response_status = response.status().as_u16();
    assert_eq!(200, response_status);

    // try to get the deleted coupon
    let response = app.get_coupon("/code", json!({"code": added_coupon.code})).await;
    let response_status = response.status().as_u16();

    // Assert 2
    assert_eq!(404, response_status);
}

#[tokio::test]
async fn delete_by_id_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;

    let test_cases = vec![
        (json!({
            "some random field": "",
        }), "missing id"),
        (json!({
            "id": "string",
        }), "invalid id (string)"),
        (json!({
            "id": -1,
        }), "invalid id (negative)"),
    ];

    // Act 
    for (invalid_body, error_message) in test_cases {
        let response = app.delete_coupon("/id", invalid_body).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was `{}`.",
            error_message
        );
    }
}

#[tokio::test]
async fn delete_by_code_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;

    let test_cases = vec![
        (json!({
            "some random field": "",
        }), "missing code"),
        (json!({"code": 1}), "invalid code (integer)"),
        (json!({"code": -1}), "invalid code (negative)"),
    ];

    // Act 
    for (invalid_body, error_message) in test_cases {
        let response = app.delete_coupon("/code", invalid_body).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was `{}`.",
            error_message
        );
    }
}

#[tokio::test]
async fn patch_returns_404_for_coupon_not_found(){
    // Arrange
    let app = spawn_app().await;
    let coupon = get_default_coupon_data(get_random_coupon_code());
    let body = json!(serde_json::to_value(&coupon).unwrap());

    // Act 
    let response = app.patch_coupon(body).await;

    // Assert
    assert_eq!(
        404,
        response.status().as_u16(),
        "patch` did not fail with 404 Not Found."
    );
}

#[tokio::test]
async fn patch_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;

    let test_cases = vec![
        (json!({
            "id": "string",
            "code": "test",
            "discount": 1,
        }), "invalid id (string)"),
        (json!({
            "id": -1,
            "code": "test",
            "discount": 1,
        }), "invalid id (negative)"),
        (json!({
            "code": "test",
            "discount": 1,
        }), "missing id"),
        (json!({"discount": 1}), "missing code"),
        (json!({"code": 1}), "missing discount"),
        (json!({"discount": "a"}), "invalid discount (string)"),
        (json!({"discount": -1}), "invalid discount (negative)"),
        (json!({"code": 1}), "invalid code (integer)"),
        (json!({"code": -1}), "invalid code (negative)"),
    ];

    // Act 
    for (invalid_body, error_message) in test_cases {
        let response = app.patch_coupon(invalid_body).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was `{}`.",
            error_message
        );
    }
}

/**
 * Helper functions
 */
fn assert_coupon_fields(code: String, expected_code: String, discount: i32, max_usage_count: Option<i32>){
    assert_eq!(code, expected_code);
    assert_eq!(discount, 10);
    assert_eq!(max_usage_count, Some(2));
    // assert_eq!(expiration_date, Some(NaiveDateTime::from_str("31/12/2030 00:00:00").unwrap()));
}

fn get_default_coupon_data(code: String) -> CouponUpdate {
    return CouponUpdate { 
        id: 123456789,
        code,
        discount: 10,
        max_usage_count: 2,
    };
}

// Return a CouponRequest struct as JSON
fn get_coupon_request_body(code: String) -> serde_json::Value {
    let coupon = get_default_coupon_data(code);
    let coupon_request = CouponRequest {
        code: coupon.code,
        discount: coupon.discount,
        max_usage_count: Some(coupon.max_usage_count),
    };
    return json!(serde_json::to_value(&coupon_request).unwrap());
}

fn get_random_coupon_code() -> String {
    return Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
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
