use crate::helpers::{spawn_app};



// #[tokio::test]
// async fn always_fails() {
//     // Arrange
//     let app = spawn_app().await;

//     run_test(|| {
//         assert!(false, "test error for validation...!");
//     }, &app.db_pool, &app.db_name ).await;
// }

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.api_client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length()); // no body

}
