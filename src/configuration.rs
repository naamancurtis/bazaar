use config::{Config, File};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let mut settings = Config::default();
    settings.merge(File::with_name("configuration"))?;
    settings.try_into()
}

impl Settings {
    pub fn set_database_name(&mut self, name: String) {
        self.database.database_name = name;
    }
}

impl DatabaseSettings {
    pub fn generate_connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub fn generate_connection_without_database(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port,
        )
    }
}
