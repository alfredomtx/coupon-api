use coupon_api::{
    configuration::{get_configuration, DatabaseSettings, Settings, ApiKey},
    telemetry::{get_subscriber, init_subscriber},
    startup::{get_connection_pool, Application},
    coupon::{CouponResponse},
};
use reqwest::{
    Method,
    header:: HeaderMap,
};
use secrecy::ExposeSecret;
use serde_json::json;
use std::panic;
use sqlx::{MySqlPool, MySqlConnection, Connection, Executor};
use once_cell::sync::Lazy;

pub struct TestApp {
    pub address: String,
    pub db_pool: MySqlPool,
    pub db_name: String,
    pub port: u16,
    pub api_client: reqwest::Client,
    pub api_key: ApiKey,
}

pub fn build_query_params(key: &str, value: String) -> Option<Vec<(&str, String)>> {
    return Some(vec![(key, value)]);
}

impl TestApp {
    pub async fn post_and_deserialize_coupon(&self, body: serde_json::Value) -> CouponResponse {
        let response = self.post_coupon(body, false).await;
        let status = response.status().as_u16();
        let response_body = response.text().await.expect("Failed to get response_body");
        if (!status.to_string().starts_with("2")){
            dbg!(&response_body);
        }
        let coupon: CouponResponse = serde_json::from_str(&response_body).expect("POST: Failed to parse CouponResponse from response.");
        return coupon;
    }

    pub async fn get_and_deserialize_coupon(&self, query_param: &str, value: String) -> CouponResponse {
        let response;
        if (query_param == "code"){
            response = self.get_coupon("", build_query_params("code", value)).await;
        } else {
            response = self.get_coupon("", build_query_params("id", value)).await;
        }
        let response_body = response.text().await.expect("failed to get response_body");
        let coupon: CouponResponse = serde_json::from_str(&response_body).expect("GET: Failed to parse CouponResponse from response.");
        return coupon;
    }

    pub async fn post_coupon(&self, body: serde_json::Value, error_for_status: bool) -> reqwest::Response {
        return self.request_coupon(Method::POST, "", body, error_for_status).await;
    }
    
    pub async fn get_coupon(&self, endpoint: &str, query: Option<Vec<(&str, String)>>) -> reqwest::Response {
        if let Some(params) = query {
            return self.api_client
                .get(&format!("{}/coupon{}", &self.address, endpoint))
                .query(&params)
                .send()
                .await
                .expect("Failed to perform GET request");
        }
        return self.api_client
            .get(&format!("{}/coupon{}", &self.address, endpoint))
            .send()
            .await
            .expect("Failed to perform GET request");
    }
        
    pub async fn patch_coupon(&self, body: serde_json::Value, query: Option<Vec<(&str, String)>>) -> reqwest::Response {
        if let Some(params) = query {
            return self.api_client
                .patch(&format!("{}/coupon", &self.address))
                .json(&body)
                .query(&params)
                .send()
                .await
                .expect("Failed to perform GET request");
        }
        return self.api_client
            .patch(&format!("{}/coupon", &self.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to perform GET request");
    }

    pub async fn delete_coupon(&self, endpoint: &str, body: serde_json::Value) -> reqwest::Response {
        return self.request_coupon(Method::DELETE, endpoint, body, false).await;
    }

    pub async fn request_coupon(&self, method: Method, endpoint: &str, body: serde_json::Value, error_for_status: bool) -> reqwest::Response {
        if (error_for_status == true){
            return self.api_client
            .request(method.clone(), &format!("{}/coupon{}", &self.address, endpoint))
            .json(&body)
            .send()
            .await
            .expect(format!("Failed to perform {} request", method.to_string()).as_str())
            .error_for_status()
            .unwrap();
        }
        return self.api_client
            .request(method.clone(), &format!("{}/coupon{}", &self.address, endpoint))
            .json(&body)
            .send()
            .await
            .expect(format!("Failed to perform {} request", method.to_string()).as_str());
    }

}

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
        .expect("Failed to build TEST application.");
    let application_port = application.port();

    // Get the port before spawning the application
    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    return TestApp {
        address: address.clone(),
        port: application_port,
        db_pool: get_connection_pool(&configuration.database, true),
        db_name: configuration.database.test_database_name.clone(),
        api_client: create_reqwest_client(&configuration, &address).await,
        api_key: configuration.application.api_key,
    };
}

async fn create_reqwest_client(configuration: &Settings, address: &String) -> reqwest::Client {
    // request to `/auth` to get a Bearer token and set in in the header for next requests
    let body = json!({"api_key": &configuration.application.api_key.0.expose_secret()});
    let response = reqwest::Client::new()
        .post(&format!("{}/auth", address))
        .json(&body)
        .send()
        .await
        .expect("Failed to perform request to `/auth`.");

    let bearer: String = response.json().await
        .expect("Failed to get `/auth` response text.");

    // setting default Authorization header
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", bearer.parse().unwrap());

    return reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .default_headers(headers)
        .build()
        .unwrap();
}

async fn configure_test_database(config: &DatabaseSettings) -> MySqlPool {
    // Create database
    let mut connection = MySqlConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to database.");
    
    drop_test_database(&mut connection, config.test_database_name.clone()).await;

    connection
        .execute(format!(r#"CREATE DATABASE {};"#, config.test_database_name).as_str())
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


pub async fn drop_test_database(connection: &mut MySqlConnection, test_db_name: String) {
    if (!test_db_name.contains("TEST")){
        panic!("`TEST` string not found in Test Database name, for safety it must contains `TEST`.");
    }

    connection
        .execute(format!(r#"DROP DATABASE IF EXISTS {};"#, test_db_name).as_str())
        .await
        .expect("Failed to drop test database.");
}


// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "coupon-api".to_string();

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
