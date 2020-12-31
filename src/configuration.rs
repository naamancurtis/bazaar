use config::{Config, File};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::convert::{TryFrom, TryInto};
use std::env::var;
use std::fmt;

#[derive(Deserialize)]
pub struct Configuration {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

#[derive(Debug)]
pub enum Environment {
    Local,
    CI,
    Production,
}

pub fn get_configuration() -> Result<Configuration, config::ConfigError> {
    let mut settings = Config::default();
    let base_path = std::env::current_dir().expect("failed to determine current directory");
    let configuration_directory = base_path.join("configuration");

    settings.merge(File::from(configuration_directory.join("base")).required(true))?;

    let environment: Environment = var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("failed to parse APP_ENVIRONMENT");

    settings
        .merge(File::from(configuration_directory.join(environment.as_str())).required(true))?;

    settings.merge(config::Environment::with_prefix("app").separator("__"))?;

    settings.try_into()
}

impl Configuration {
    pub fn set_database_name(&mut self, name: String) {
        self.database.database_name = name;
    }

    pub fn get_addr(&self) -> String {
        format!("{}:{}", self.application.host, self.application.port)
    }
}

impl DatabaseSettings {
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }

    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    #[cfg(test)]
    pub fn raw_pg_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.host, self.username, self.password, self.port, self.database_name
        )
    }
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::CI => "ci",
            Environment::Production => "production",
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Environment::Local => "local",
            Environment::CI => "ci",
            Environment::Production => "production",
        };
        write!(f, "{}", string)
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "ci" => Ok(Self::CI),
            "production" => Ok(Self::Production),
            other => Err(format!("{} is not a supported environment", other)),
        }
    }
}
