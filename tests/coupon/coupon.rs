use crate::helpers::{spawn_app};
use chrono::{NaiveDateTime, Utc, Datelike};
use coupon_api::coupon::{Coupon, CouponUpdate, CouponRequest, CouponResponse};
use rand::{distributions::{Alphanumeric, DistString}, Rng};
use serde_json::json;

/**
 * GET
 */
#[tokio::test]
async fn get_coupon_by_id_returns_a_coupon() {
    // Arrange
    let app = spawn_app().await;

    let coupon_request = get_coupon_request(get_random_coupon_code());
    let body = get_coupon_request_json(&coupon_request);
    
    // Act
    // add the coupon before getting it
    let response = app.post_coupon(body, true).await;
    let coupon_response = get_coupon_from_response(response).await;

    // request for the added coupon using its id
    let coupon = app.get_and_deserialize_coupon(coupon_response.id).await;

    // Assert
    assert_coupon_fields(coupon, coupon_request);
}

#[tokio::test]
async fn get_coupon_by_code_returns_a_coupon() {
    // Arrange
    let app = spawn_app().await;
    let code = get_random_coupon_code();
    let coupon_request = get_coupon_request(code.clone());
    let body = get_coupon_request_json(&coupon_request);
    
    // Act
    // add the coupon before getting it
    app.post_coupon(body, true).await;

    let response = app.get_coupon("", Some(vec![("code", code)]) ).await;
    let coupon = get_coupon_from_response(response).await;

    // Assert
     assert_coupon_fields(coupon, coupon_request);
}

#[tokio::test]
async fn get_all_coupons_returns_a_list_of_coupons() {
    // Arrange
    let app = spawn_app().await;
    let code1 = get_random_coupon_code();
    let coupon_request1 = get_coupon_request(code1.clone());
    let body1 = get_coupon_request_json(&coupon_request1);
    let code2 = get_random_coupon_code();
    let coupon_request2 = get_coupon_request(code2.clone());
    let body2 = get_coupon_request_json(&coupon_request2);
    
    // Act
    // add 2 coupons
    app.post_coupon(body1, true).await;
    app.post_coupon(body2, true).await;

    // get all coupons
    let response = app.get_coupon("/all", None).await;
    let response_body = response.text().await.expect("failed to get response_body");
    let coupons: Vec<Coupon> = serde_json::from_str(&response_body).expect("Failed to parse CouponResponse from response.");

    // Assert
    assert!(coupons.len() > 1);

    // iterate through the coupons and get only the 2 coupon with the codes we added before
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
        (Some(vec![("id", "123456789".to_string())]), "not found by id", "/id"),
        (Some(vec![("code", "code that does not exist".to_string())]), "not found by code", "/code")
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
     let coupon_request = get_coupon_request(get_random_coupon_code());
     let body = get_coupon_request_json(&coupon_request);
     
     // Act
     let response = app.post_coupon(body, false).await;
     let response_status = response.status().as_u16();
     let response_body = response.text().await.expect("failed to get response_body");
 
     // Assert
     assert_eq!(201, response_status);
 
     let coupon: CouponResponse = serde_json::from_str(&response_body).expect("Failed to parse CouponResponse from response.");
     
     assert_coupon_fields(coupon, coupon_request);
 }

#[tokio::test]
async fn post_returns_500_if_coupon_already_exists() {
    // Arrange
    let app = spawn_app().await;
    let coupon_request = get_coupon_request(get_random_coupon_code());
    let body = get_coupon_request_json(&coupon_request);
    
    // Act 1
    let response = app.post_coupon(body.clone(), false).await;
    let response_status = response.status().as_u16();
    
    // Assert 1
    assert_eq!(201, response_status);

    // Act 2
    // adding the same coupon twice
    let response = app.post_coupon(body, false).await;
    let response_status = response.status().as_u16();

    // Assert 2
    assert_eq!(500, response_status);
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
    let coupon_request = get_coupon_request(get_random_coupon_code());
    let body = get_coupon_request_json(&coupon_request);
    
    // Act
    // add a coupon
    let added_coupon = app.post_and_deserialize_coupon(body).await; 

    // update the added coupon with new data
    let mut coupon_update = get_default_coupon_data("CouponWithUpdatedData".to_string());
    coupon_update.id = added_coupon.id;
    coupon_update.discount = 99;
    coupon_update.max_usage_count = Some(123);
    coupon_update.active = false;

    let body = json!(serde_json::to_value(&coupon_update).unwrap());
    
    let response = app.patch_coupon(body).await;
    let response_status = response.status().as_u16();
        
    // Assert
    assert_eq!(200, response_status);

    let coupon = app.get_and_deserialize_coupon(added_coupon.id).await;

    assert_coupon_fields(coupon.clone(), coupon_update.into());
    // date_updated should have value
    let _ = coupon.date_updated.unwrap();
}

#[tokio::test]
async fn patch_coupon_not_found_returns_404(){
    // Arrange
    let app = spawn_app().await;

    let mut coupon_update = get_default_coupon_data("".to_string());
    // assign random id that won't be found in database
    coupon_update.id = rand::thread_rng().gen_range(100000..i32::MAX);

    let body = json!(serde_json::to_value(&coupon_update).unwrap());

    let response = app.patch_coupon(body).await;
    let response_status = response.status().as_u16();

    // Assert
    assert_eq!(404, response_status);
}


/**
 * DELETE
 */
#[tokio::test]
async fn delete_coupon_by_id_successfully() {
    // Arrange
    let app = spawn_app().await;
    let coupon_request = get_coupon_request(get_random_coupon_code());
    let body = get_coupon_request_json(&coupon_request);
    
    // Act
    // add a coupon
    let added_coupon = app.post_and_deserialize_coupon(body).await; 
    // delete added coupon
    let response = app.delete_coupon("/id", json!({"id": added_coupon.id})).await;
    let response_status = response.status().as_u16();

    assert_eq!(200, response_status);

    // try to get the deleted coupon
    let response = app.get_coupon("", Some(vec![("id", added_coupon.id.to_string())]) ).await;
    let response_status = response.status().as_u16();

    // Assert 2
    assert_eq!(404, response_status);
}

#[tokio::test]
async fn delete_coupon_by_code_successfully() {
    // Arrange
    let app = spawn_app().await;
    let coupon_request = get_coupon_request(get_random_coupon_code());
    let body = get_coupon_request_json(&coupon_request);
    // dbg!(&body);
    
    // Act
    // add a coupon
    let added_coupon = app.post_and_deserialize_coupon(body).await; 
    // delete added coupon
    let response = app.delete_coupon("/code", json!({"code": added_coupon.code.clone()})).await;
    let response_status = response.status().as_u16();

    assert_eq!(200, response_status);

    // try to get the deleted coupon
    let response = app.get_coupon("", Some(vec![("code", added_coupon.code)]) ).await;
    let response_status = response.status().as_u16();

    // Assert 2
    assert_eq!(404, response_status);
}

#[tokio::test]
async fn delete_by_id_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;

    let test_cases = vec![
        (json!({"some random field": ""}), "missing id"),
        (json!({"id": "string"}), "invalid id (string)"),
        (json!({"id": "-1"}), "invalid id (negative)"),
        (json!({"id": ""}), "invalid id (empty)"),
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
        (json!({"some random field": ""}), "missing code"),
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
 * Verify Coupon
 */
#[tokio::test]
async fn verify_coupon_returns_true_for_a_valid_coupon() {
    // Arrange
    let coupon_request = get_coupon_request(get_random_coupon_code());
    // Act
    let response_body = start_verify_test_and_post_coupon(coupon_request).await;

    // Assert
    assert_eq!(response_body, "true");
}

#[tokio::test]
async fn verify_coupon_returns_false_if_not_active() {
    // Arrange
    let mut coupon_request = get_coupon_request(get_random_coupon_code());
    coupon_request.active = false;
    // Act
    let response_body = start_verify_test_and_post_coupon(coupon_request).await;

    // Assert
    assert_eq!(response_body, "false");
}

#[tokio::test]
async fn verify_coupon_returns_false_if_expired() {
    // Arrange
    let mut coupon_request = get_coupon_request(get_random_coupon_code());
    coupon_request.expiration_date = Some(NaiveDateTime::parse_from_str("2000-12-31 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap());
    // Act
    let response_body = start_verify_test_and_post_coupon(coupon_request).await;
    
    // Assert
    assert_eq!(response_body, "false");
}

#[tokio::test]
async fn verify_coupon_validates_if_today_is_friday() {
    // Arrange
    // "SEXTOU" is a special coupon that is only valid on Friday
    let coupon_request = get_coupon_request("SEXTOU".to_string());
    // Act
    let response_body = start_verify_test_and_post_coupon(coupon_request).await;
    
    // Assert
    let weekday = Utc::now().date_naive().weekday().to_string();
    if (weekday.to_uppercase() == "FRIDAY"){
        assert_eq!(response_body, "true");
    } else {
        assert_eq!(response_body, "false");
    }
}



/**
 * Helper functions
 */
async fn start_verify_test_and_post_coupon(coupon_request: CouponRequest) -> String {
    let app = spawn_app().await;

    let body = get_coupon_request_json(&coupon_request);
    // add the coupon before verifying it
    app.post_coupon(body, true).await;

    let response = app.get_coupon("/verify", Some(vec![("code", coupon_request.code)]) ).await;
    let response_body = response.text().await.expect("Failed to get response_body");

    return response_body;
}

fn assert_coupon_fields(coupon_response: CouponResponse, coupon_expected: CouponRequest){
    assert_eq!(coupon_response.code, coupon_expected.code);
    assert_eq!(coupon_response.discount, coupon_expected.discount);
    assert_eq!(coupon_response.active, coupon_expected.active);
    assert_eq!(coupon_response.max_usage_count, coupon_expected.max_usage_count);
    assert_eq!(coupon_response.expiration_date, coupon_expected.expiration_date);
}

fn get_default_coupon_data(code: String) -> CouponUpdate {
    return CouponUpdate { 
        id: 123456789,
        code,
        discount: 10,
        max_usage_count: Some(2),
        expiration_date: Some(NaiveDateTime::parse_from_str("2030-12-31 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
        active: true,
    };
}

// Return a CouponRequest struct as JSON
fn get_coupon_request_json(coupon_request: &CouponRequest) -> serde_json::Value {
    return json!(serde_json::to_value(&coupon_request).unwrap());
}

fn get_coupon_request(code: String) -> CouponRequest {
    let coupon = get_default_coupon_data(code);
    return CouponRequest {
        code: coupon.code,
        discount: coupon.discount,
        active: true,
        max_usage_count: coupon.max_usage_count,
        expiration_date: coupon.expiration_date,
    };
}

fn get_random_coupon_code() -> String {
    return Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
}

async fn get_coupon_from_response(response: reqwest::Response) -> CouponResponse {
    let response_body = response.text().await.expect("failed to get response_body");
    let coupon_response: CouponResponse = serde_json::from_str(&response_body).expect("Failed to parse CouponResponse from response.");
    return coupon_response;
}

// #[tokio::test]
// async fn coupon_fails_if_there_is_a_fatal_database_error() {
//     // Arrange
//     let app = spawn_app().await;
//     let body = get_coupon_request_json(None);

//     // Sabotage the database
//     sqlx::query!("DROP TABLE coupon;",)
//         .execute(&app.db_pool)
//         .await
//         .unwrap();

//     // Act 
//     let response = app.post_coupon(body).await;

//     // Assert
//     assert_eq!(response.status().as_u16(), 500);
// }
