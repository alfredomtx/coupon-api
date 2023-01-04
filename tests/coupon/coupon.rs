use crate::helpers::{spawn_app, build_query_params, TestApp, drop_test_database};
use chrono::{NaiveDateTime, Utc, Datelike};
use coupon_api::coupon::{Coupon, CouponInsertRequest, CouponResponse, CouponUpdateRequest};
use rand::distributions::{Alphanumeric, DistString};
use serde_json::json;

/**
 * GET
 */
#[tokio::test]
async fn get_coupon_by_id_returns_a_coupon() {
    
    let coupon_request = get_coupon_request(get_random_coupon_code());
    let (app, added_coupon) = spawn_app_and_post_coupon_with_coupon_request(coupon_request.clone()).await;

    // request for the added coupon using its id
    let coupon = app.get_and_deserialize_coupon("id", added_coupon.id.to_string()).await;

    // Assert
    assert_coupon_fields(coupon, coupon_request);
}

#[tokio::test]
async fn get_coupon_by_code_returns_a_coupon() {
    let code = get_random_coupon_code();
    let coupon_request = get_coupon_request(code.clone());
    let (app, added_coupon) = spawn_app_and_post_coupon_with_coupon_request(coupon_request.clone()).await;

    // request for the added coupon using its code
    let coupon = app.get_and_deserialize_coupon("code", added_coupon.code).await;

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
        (Some(vec![("id", "123456789".to_string())]), "not found by id", ""),
        (Some(vec![("code", "code that does not exist".to_string())]), "not found by code", "")
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
async fn post_returns_409_conflit_if_coupon_already_exists() {
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
    assert_eq!(409, response_status);
}

#[tokio::test]
async fn post_returns_4xx_for_invalid_coupon_data() {
    // Arrange
    let app = spawn_app().await;

    let test_cases = vec![
        (json!({
            "discount": 1,
            "active": true,
        }), "missing `code`", 400),
        (json!({
            "code": "test",
            "active": true,
        }), "missing `discount`", 400),
        (json!({
            "code": "string",
            "discount": 1,
        }), "missing `active`", 400),
        (json!({
            "discount": "string",
            "code": "test",
            "active": true,
        }), "invalid `discount` (string)", 400),
        (json!({
            "discount": -1,
            "code": "test",
            "active": true,
        }), "invalid `discount` (negative)", 422),
        (json!({
            "discount": 91,
            "code": "test",
            "active": true,
        }), "invalid `discount` (higher than 90)", 422),
        (json!({
            "discount": 1,
            "code": -1,
            "active": true,
        }), "invalid `code` (negative)", 400),
        (json!({
            "discount": 0,
            "code": "test",
            "active": "string",
        }), "invalid `active` (string)", 400),
        (json!({
            "discount": 0,
            "code": "test",
            "active": -1,
        }), "invalid `active` (negative)", 400),
        (json!({
            "discount": 0,
            "code": "test",
            "active": 2,
        }), "invalid `active` (2)", 400),
        (json!({
            "discount": 0,
            "code": "test",
            "active": "true",
        }), "invalid `active` (`true` string)", 400),
        (json!({
            "discount": 0,
            "code": "test",
            "active": "false",
        }), "invalid `active` (`false` string)", 400),
    ];

    // Act 
    for (invalid_body, error_message, expected_code) in test_cases {
        let response = app.post_coupon(invalid_body, false).await;
        // Assert
        assert_eq!(
            response.status().as_u16(),
            expected_code,
            "The API did not fail with `{}` when the payload was `{}`.",
            expected_code, error_message
        );
    }
}


/**
 * PATCH
 */
#[tokio::test]
async fn patch_by_id_updates_the_coupon_successfully() {
    patch_code_test_request("id").await;
}

#[tokio::test]
async fn patch_by_code_updates_the_coupon_successfully() {
    patch_code_test_request("code").await;
}

async fn patch_code_test_request(query_param: &str) {
    // Arrange
    let (app, added_coupon) = spawn_app_and_post_coupon().await;

    // update the added coupon with new data
    let mut coupon_update = get_default_coupon_data(added_coupon.code.clone());
    coupon_update.id = added_coupon.id;
    coupon_update.discount = 66;
    coupon_update.max_usage_count = Some(123);
    coupon_update.expiration_date = Some(NaiveDateTime::parse_from_str("2099-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap());
    coupon_update.active = false;

    let body = json!(serde_json::to_value(&coupon_update).unwrap());
    let response;
    if (query_param == "code"){
        response = app.patch_coupon(body, build_query_params("code", added_coupon.code)).await;
    } else {
        response = app.patch_coupon(body, build_query_params("id", added_coupon.id.to_string())).await;
    }
    let response_status = response.status().as_u16();
        
    // Assert
    assert_eq!(200, response_status);

    let coupon = app.get_and_deserialize_coupon("id", added_coupon.id.to_string()).await;

    assert_coupon_fields(coupon.clone(), coupon_update.into());
    // `date_updated` field now should have value
    let _ = coupon.date_updated.unwrap();
}

#[tokio::test]
async fn patch_do_not_update_the_code() {
    // Arrange
    let (app, added_coupon) = spawn_app_and_post_coupon().await;

    // update the added coupon with new `code`
    let mut coupon_update = get_default_coupon_data(get_random_coupon_code());
    coupon_update.id = added_coupon.id;

    let body = json!(serde_json::to_value(&coupon_update).unwrap());

    let response = app.patch_coupon(body, build_query_params("id", added_coupon.id.to_string()) ).await;
    let response_status = response.status().as_u16();
        
    // Assert
    assert_eq!(200, response_status);

    let coupon = app.get_and_deserialize_coupon("id", added_coupon.id.to_string()).await;

    // the code of the added coupon should not be updated
    assert_ne!(coupon.code, coupon_update.code);
}

#[tokio::test]
async fn patch_returns_404_for_coupon_not_found(){
    // Arrange
    let app = spawn_app().await;
    let coupon = get_default_coupon_data(get_random_coupon_code());
    let body = json!(serde_json::to_value(&coupon).unwrap());

    // Act 
    let response = app.patch_coupon(body, build_query_params("id", coupon.id.to_string()) ).await;

    // Assert
    assert_eq!(
        response.status().as_u16(),
        404,
        "`patch` did not fail with 404 Not Found."
    );
}

#[tokio::test]
async fn patch_returns_4xx_for_invalid_body_data() {
    // Arrange
    let (app, added_coupon) = spawn_app_and_post_coupon().await;

    let test_cases = vec![
        (json!({
            "discount": "string",
            "active": true,
        }), "invalid `discount` (string)", 400),
        (json!({
            "discount": -1,
            "active": true,
        }), "invalid `discount` (negative)", 422),
        (json!({
            "discount": 91,
            "active": true,
        }), "invalid `discount` (higher than 90)", 422),
        (json!({
            "discount": 0,
            "active": "string",
        }), "invalid `active` (string)", 400),
        (json!({
            "discount": 0,
            "active": -1,
        }), "invalid `active` (negative)", 400),
        (json!({
            "discount": 0,
            "active": 2,
        }), "invalid `active` (2)", 400),
        (json!({
            "discount": 0,
            "active": "true",
        }), "invalid `active` (`true` string)", 400),
        (json!({
            "discount": 0,
            "active": "false",
        }), "invalid `active` (`false` string)", 400),
    ];

    // Act 
    for (invalid_body, error_message, expected_code) in test_cases {
        let response = app.patch_coupon(invalid_body, build_query_params("id", added_coupon.id.to_string()) ).await;
        // Assert
        assert_eq!(
            response.status().as_u16(),
            expected_code,
            "The API did not fail with `{}` when the payload was `{}`.",
            expected_code, error_message
        );
    }
}


#[tokio::test]
async fn patch_returns_422_for_invalid_id_query_param_data() {
    // Arrange
    let app = spawn_app().await;

    let coupon = get_default_coupon_data(get_random_coupon_code());

    let coupon_update = CouponUpdateRequest {
        discount: coupon.discount,
        active: coupon.active,
        max_usage_count: coupon.max_usage_count,
        expiration_date: coupon.expiration_date
    };

    let body = json!(serde_json::to_value(&coupon_update).unwrap());

    let test_cases = vec![
        (json!({"id": "string",}), "invalid id (string)"),
        (json!({"id": -1,}), "invalid id (negative)"),
    ];

    // Act 
    for (invalid_query_value, error_message) in test_cases {
        let response = app.patch_coupon(body.clone(), build_query_params("id", invalid_query_value.to_string()) ).await;

        // Assert
        assert_eq!(
            response.status().as_u16(),
            422,
            "The API did not fail with 400 Bad Request when the payload was `{}`.",
            error_message
        );
    }
}


/**
 * DELETE
 */
#[tokio::test]
async fn delete_coupon_by_id_successfully() {
    // Arrange
    let (app, added_coupon) = spawn_app_and_post_coupon().await;

    // delete added coupon
    let response = app.delete_coupon("/id", json!({"id": added_coupon.id})).await;
    let response_status = response.status().as_u16();

    assert_eq!(200, response_status);

    // try to get the deleted coupon
    let response = app.get_coupon("", build_query_params("id", added_coupon.id.to_string()) ).await;
    let response_status = response.status().as_u16();

    // Assert 2
    assert_eq!(404, response_status);
}

#[tokio::test]
async fn delete_coupon_by_code_successfully() {
    // Arrange
    let (app, added_coupon) = spawn_app_and_post_coupon().await;

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
            response.status().as_u16(),
            400,
            "The API did not fail with 400 when the payload was `{}`.",
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
        (json!({"code": -1}), "invalid `code` (negative)"),
    ];

    // Act 
    for (invalid_body, error_message) in test_cases {
        let response = app.delete_coupon("/code", invalid_body).await;

        // Assert
        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 when the payload was `{}`.",
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

async fn spawn_app_and_post_coupon() -> (TestApp, CouponResponse) {
    let coupon_request = get_coupon_request(get_random_coupon_code());
    return spawn_app_and_post_coupon_with_coupon_request(coupon_request).await;
}

async fn spawn_app_and_post_coupon_with_coupon_request(coupon_request: CouponInsertRequest) -> (TestApp, CouponResponse) {
    // Arrange
    let app = spawn_app().await;

    // add the coupon before updating it
    let body = get_coupon_request_json(&coupon_request);
    let added_coupon = app.post_and_deserialize_coupon(body).await; 
    return (app, added_coupon);
}

async fn start_verify_test_and_post_coupon(coupon_request: CouponInsertRequest) -> String {
    let (app, _) = spawn_app_and_post_coupon_with_coupon_request(coupon_request.clone()).await;

    let response = app.get_coupon("/verify", build_query_params("code", coupon_request.code) ).await;
    let response_body = response.text().await.expect("Failed to get response_body");

    return response_body;
}

fn assert_coupon_fields(coupon_response: CouponResponse, coupon_expected: CouponInsertRequest){
    assert_eq!(coupon_response.code, coupon_expected.code);
    assert_eq!(coupon_response.discount, coupon_expected.discount);
    assert_eq!(coupon_response.active, coupon_expected.active);
    assert_eq!(coupon_response.max_usage_count, coupon_expected.max_usage_count);
    assert_eq!(coupon_response.expiration_date, coupon_expected.expiration_date);
}

fn get_default_coupon_data(code: String) -> Coupon {
    return Coupon { 
        id: 123456789,
        code,
        discount: 10,
        max_usage_count: Some(2),
        expiration_date: Some(NaiveDateTime::parse_from_str("2100-12-31 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
        active: true,
        date_created: None,
        date_updated: None,
    };
}

// Return a CouponRequest struct as JSON
fn get_coupon_request_json(coupon_request: &CouponInsertRequest) -> serde_json::Value {
    return json!(serde_json::to_value(&coupon_request).unwrap());
}

fn get_coupon_request(code: String) -> CouponInsertRequest {
    let coupon = get_default_coupon_data(code);
    return CouponInsertRequest {
        code: coupon.code,
        discount: coupon.discount,
        active: true,
        max_usage_count: coupon.max_usage_count,
        expiration_date: coupon.expiration_date,
    };
}

fn get_random_coupon_code() -> String {
    return Alphanumeric.sample_string(&mut rand::thread_rng(), 10);
}
