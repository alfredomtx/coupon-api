
use crate::helpers::{spawn_app};

#[tokio::test]
async fn post_persists_the_new_cupom() {
    // Arrange
    let app = spawn_app().await;
    let body = get_cupom_request_body();
    
    // Act
    let response = app.post_cupom(body).await;
    let response_status = response.status().as_u16();

    let _payload = response.text()
        .await
        .expect("failed to get payload");
    
    assert_eq!(201, response_status);

    let error_message = format!("Failed to fetch saved cupom, no cupom was persisted.\nResponse Status:{}", response_status);
    let saved = sqlx::query!("SELECT * FROM cupom")
        .fetch_one(&app.db_pool)
        .await
        .expect(error_message.as_str());

    // Assert
    assert_eq!(saved.code, "TestCupom");
    assert_eq!(saved.discount, 10);
    assert_eq!(saved.max_usage_count, Some(2));
    // assert_eq!(saved.expiration_date, Some(NaiveDateTime::from_str("31/12/2030 00:00:00").unwrap()));
}

#[tokio::test]
async fn cupom_returns_400_for_invalid_data() {
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
        let response = reqwest::Client::new()
            .post(&format!("{}/cupom", &app.address))
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

fn get_cupom_request_body() -> serde_json::Value {
    return serde_json::json!({
        "code": "TestCupom",
        "discount": 10,
        "max_usage_count": 2,
        // "expiration_date": "31/12/2030 00:00:00",
    });
}
