

use crate::helpers::{spawn_app};
// use wiremock::matchers::{method, path};
// use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn cupom_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    // TODO: implement HashMap as [field => value]
    let required_fields = vec!["discount", "code"];

    for field in required_fields {
        let test_cases = vec![(
            serde_json::json!({field: 1}),
            format!("missing {}", field),
        )
        ];
        
    }
    let test_cases = vec![(
        serde_json::json!({"discount": 1}),
        "missing code",
    ),
    (
        serde_json::json!({"code": 1}),
        "missing discount",
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
