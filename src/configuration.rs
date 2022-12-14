use config::{Config, ConfigError};
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::mysql::MySqlSslMode;
use std::env;
use serde::{Deserialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub redis_uri: Secret<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiKey(pub Secret<String>);


/// The possible runtime environment for our application.
#[derive(Debug, Clone, Deserialize)]
pub enum Environment {
    Local,
    Production
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub api_key: ApiKey,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub test_database_name: String,
    // Determine if we demand the connection to be encrypted or not
    pub require_ssl: bool
}

impl DatabaseSettings {
    pub fn without_db(&self) -> MySqlConnectOptions {
        return MySqlConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(MySqlSslMode::Disabled);
    }

    pub fn with_db(&self, test_database: bool) -> MySqlConnectOptions {
        let options = self.without_db()
            .database(if (test_database) { &self.test_database_name } else { &self.database_name } );
        return options;
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    
    // Detect the running environment
    // Default to `local` if unspecified
    let environment: Environment = env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let environment_filename = format!("{}.yaml", environment.as_str());

    // If in Production, get the `port` variable from Heroku and set in our expected env format
    if let (Environment::Production) = environment {
        set_port_heroku();
    }

    let mut builder = Config::builder();
    // must re-assign to retain ownership
    // Add in settings from environment variables (with a prefix of APP and '__' as separator)
    // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
    builder = builder.add_source(config::Environment::with_prefix("APP").prefix_separator("_").separator("__"));
    builder = builder.add_source(config::File::from(configuration_directory.join(&environment_filename)));

    let settings = builder.build()?;

    return settings.try_deserialize::<Settings>();
}


pub fn set_port_heroku() {
    // Get the port from Heroku's `PORT` environment variable
    let port = env::var("PORT").expect("$PORT is not set.");
    env::set_var("APP_APPLICATION__PORT", port);
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        return match self {
            Environment::Local => "local",
            Environment::Production => "production"
        };
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        return match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either 'local' or 'production'.",
                other
            )),
        };
    }

}
