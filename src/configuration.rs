use std::{env, error};

use anyhow::anyhow;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};

#[derive(Debug, Deserialize)]
pub struct Setting {
    pub app: AppSettings,
    pub database: DatabaseSettings,
}

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub db_name: String,
    pub require_ssl: bool,
}

impl AppSettings {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
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
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            // Try an encrypted connection, fallback to unencrypted if it fails
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
    // Renamed from `connection_string`
    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.db_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}

pub fn get_configuration() -> Result<Setting, Box<dyn error::Error>> {
    let work_dir = env::current_dir()?;
    let config_dir = work_dir.join("conf");

    let deploy: Deploy = env::var("APP_DEPLOY")
        .unwrap_or_else(|_| "dev".into())
        .try_into()?;

    let settings = config::Config::builder()
        .add_source(config::File::from(config_dir.join("base")))
        .add_source(config::File::from(config_dir.join(deploy.as_str())))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("."),
        )
        .build()?
        .try_deserialize()?;

    tracing::info!("{:?}", settings);
    Ok(settings)
}

/// The possible runtime environment for our application.
pub enum Deploy {
    Local,
    Dev,
    Prod,
}

impl Deploy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Deploy::Local => "local",
            Deploy::Dev => "dev",
            Deploy::Prod => "prod",
        }
    }
}

impl TryFrom<String> for Deploy {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "dev" => Ok(Self::Dev),
            "prod" => Ok(Self::Prod),
            other => Err(anyhow!(
                "{} is not a supported environment. Use either `local` or `prod`.",
                other
            )),
        }
    }
}
