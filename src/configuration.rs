use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use std::error;

#[derive(Debug, Deserialize)]
pub struct Setting {
    pub app: AppSettings,
    pub database: DatabaseSettings,
}

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub db_name: String,
}

impl AppSettings {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

impl DatabaseSettings {
    pub fn dsn(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(), // Secret 没有实现 Display, 需要明确的使用
            self.host,
            self.port,
            self.db_name
        ))
    }
}

pub fn get_configuration() -> Result<Setting, Box<dyn error::Error>> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("configuration"))
        .add_source(config::Environment::with_prefix("APP"))
        .build()?
        .try_deserialize()?;

    tracing::info!("{:?}", settings);
    Ok(settings)
}
