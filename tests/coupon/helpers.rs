use coupon_api::{
    configuration::{get_configuration, DatabaseSettings},
    telemetry::{get_subscriber, init_subscriber},
    startup::{get_connection_pool, Application},
    coupon::{CouponResponse},
};
use reqwest::{
    Method,
    header:: HeaderMap,
};
use std::panic;
use serde_json::json;
use sqlx::{MySqlPool, MySqlConnection, Connection, Executor};
use once_cell::sync::Lazy;

pub struct TestApp {
    pub address: String,
    pub db_pool: MySqlPool,
    pub db_name: String,
    pub port: u16,
    pub api_client: reqwest::Client,
    pub api_key: String,
}

impl TestApp {
    pub async fn post_and_deserialize_coupon(&self, body: serde_json::Value) -> CouponResponse {
        let response = self.post_coupon(body).await;
        let response_body = response.text().await.expect("Failed to get response_body");
        let coupon: CouponResponse = serde_json::from_str(&response_body).expect("Failed to deserialize response to coupon");
        return coupon;
    }

    pub async fn get_and_deserialize_coupon(&self, id: i32) -> CouponResponse {
        let response = self.get_coupon("/id", json!({"id": id})).await;
        let response_body = response.text().await.expect("failed to get response_body");
        let coupon: CouponResponse = serde_json::from_str(&response_body).unwrap();
        return coupon;
    }

    pub async fn post_coupon(&self, body: serde_json::Value) -> reqwest::Response {
        return self.request_coupon(Method::POST, "", body).await;
    }
    
    pub async fn get_coupon(&self, endpoint: &str, body: serde_json::Value) -> reqwest::Response {
        return self.request_coupon(Method::GET, endpoint, body).await;
    }
        
    pub async fn patch_coupon(&self, body: serde_json::Value) -> reqwest::Response {
        return self.request_coupon(Method::PATCH, "", body).await;
    }

    pub async fn delete_coupon(&self, endpoint: &str, body: serde_json::Value) -> reqwest::Response {
        return self.request_coupon(Method::DELETE, endpoint, body).await;
    }

    pub async fn request_coupon(&self, method: Method, endpoint: &str, body: serde_json::Value) -> reqwest::Response {
        return self.api_client
            .request(method.clone(), &format!("{}/coupon{}", &self.address, endpoint))
            .json(&body)
            .send()
            .await
            .expect(format!("Failed to execute {} request", method.to_string()).as_str());
    }

}


// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "debug".to_string();
    let subscriber_name = "test".to_string();

    // We cannot assign the output of `get_subscriber` to a variable based on the value
    // of `TEST_LOG` because the sink is part of the type returned by `get_subscriber`,
    // therefore they are not the same type. We could work around it, but this is the
    // most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // Use a random OS port
        c.application.port = 0;
        c
    };

    // Create and migrate the database
    configure_test_database(&configuration.database).await;

    // Launch the application as a background task
    let application = Application::build(configuration.clone(), true)
        .await
        .expect("Failed to build application.");
    let application_port = application.port();

    // Get the port before spawning the application
    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    // setting default authorization header
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", configuration.application.api_key.parse().unwrap());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .default_headers(headers)
        .build()
        .unwrap();

    return TestApp {
        address,
        port: application_port,
        db_pool: get_connection_pool(&configuration.database, true),
        db_name: configuration.database.test_database_name,
        api_client: client,
        api_key: configuration.application.api_key,
    };
}

pub async fn configure_test_database(config: &DatabaseSettings) -> MySqlPool {
    // Create database
    let mut connection = MySqlConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to database.");

    if (!config.test_database_name.contains("TEST")){
        panic!("`TEST` string not found in Test Database name, is it correct?");
    }

    connection
        .execute(format!(r#"DROP DATABASE IF EXISTS {};"#, config.test_database_name).as_str())
        .await
        .expect("Failed to drop test database.");
        
    connection
        .execute(format!(r#"CREATE DATABASE IF NOT EXISTS {};"#, config.test_database_name).as_str())
        .await
        .expect("Failed to create test database.");
    
    // Migrate database
    let connection_pool = MySqlPool::connect_with(config.with_db(true))
        .await
        .expect("Failed to connect to test database.");
        
    let _ = sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await;
        // no .expect() here because we dont want a panic if the migration fails

    return connection_pool;
}
