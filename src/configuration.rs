use config::{Config, ConfigError};
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::mysql::MySqlSslMode;
use std::env;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

/// The possible runtime environment for our application.
#[derive(Debug, Clone, serde::Deserialize)]
pub enum Environment {
    Local,
    Production
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub test_db_name: String,
    // Determine if we demand the connection to be encrypted or not
    pub require_ssl: bool
}

impl DatabaseSettings {
    pub fn without_db(&self) -> MySqlConnectOptions {
        MySqlConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(MySqlSslMode::Disabled)
    }

    pub fn with_db(&self, test_database: bool) -> MySqlConnectOptions {
        let options = self.without_db()
            .database(if (test_database) { &self.test_db_name } else {&self.database_name} );
        return options;
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    
    // Detect the running environment s
    // Default to `local` if unspecified
    let environment: Environment = env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    
    let environment_filename = format!("{}.yaml", environment.as_str());

    let mut builder = Config::builder();

    // must re-assign to retain ownership
    builder = builder.add_source(config::File::from(configuration_directory.join("base.yaml")))
        .add_source(config::File::from(configuration_directory.join(&environment_filename)))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(config::Environment::with_prefix("APP").prefix_separator("_").separator("__"));
    
    let settings = builder.build()?;
    
    settings.try_deserialize::<Settings>()
}


impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production"
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either 'local' or 'production'.",
                other
            )),
        }
    }

}
