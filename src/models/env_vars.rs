// Start of file: src/models/env_vars.tf

/*
    * Defines the application's environment variables and provides a method
    * for loading them from the system (or .env) using dotenv.
*/

use std::borrow::Cow;
use anyhow::Result;
use dotenv::dotenv;
use tracing::warn;

#[derive(Clone, Debug)]
pub struct EnvironmentVariables {
    pub environment: Cow<'static, str>,
    pub host: Cow<'static, str>,
    pub port: u16,
    pub protocol: Cow<'static, str>,
    pub max_request_body_size: usize,
    pub default_timeout_seconds: u64,
    pub db_host: Cow<'static, str>,
    pub db_port: u16,
    pub db_user: Cow<'static, str>,
    pub db_password: Cow<'static, str>
}

/*
    * Load all environment variables or fall back to defaults where specified.
*/    
impl EnvironmentVariables {
    pub fn from_env() -> Result<Self> {
        dotenv().ok();

        Ok(Self {
            environment: match dotenv::var("ENVIRONMENT") {
                Ok(env) => env.into(),
                Err(_) => {
                    warn!("Missing ENVIRONMENT, defaulting to 'development'");
                    "development".into()
                }
            },
            host: match dotenv::var("HOST") {
                Ok(host) => host.into(),
                Err(_) => "127.0.0.1".into(),
            },
            port: match dotenv::var("PORT") {
                Ok(port) => port.parse()?,
                Err(_) => 3000,
            },
            protocol: match dotenv::var("PROTOCOL") {
                Ok(proto) => proto.into(),
                Err(_) => "http".into(),
            },
            max_request_body_size: match dotenv::var("MAX_REQUEST_BODY_SIZE") {
                Ok(size) => size.parse()?,
                Err(_) => 2_097_152, // 2MB default
            },
            default_timeout_seconds: match dotenv::var("DEFAULT_TIMEOUT_SECONDS") {
                Ok(seconds) => seconds.parse()?,
                Err(_) => 3, // 3 seconds default
            },
            db_host: match dotenv::var("DB_HOST") {
                Ok(host) => host.into(),
                Err(_) => {
                    warn!("Missing DB_HOST, defaulting to 'localhost'");
                    "localhost".into()
                }
            },
            db_port: match dotenv::var("DB_PORT") {
                Ok(port) => port.parse()?,
                Err(_) => 5432,
            },
            db_user: match dotenv::var("DB_USER") {
                Ok(user) => user.into(),
                Err(_) => {
                    warn!("Missing DB_USER, defaulting to 'postgres'");
                    "postgres".into()
                }
            },
            db_password: match dotenv::var("DB_PASSWORD") {
                Ok(pass) => pass.into(),
                Err(_) => {
                    warn!("Missing DB_PASSWORD, defaulting to 'postgres'");
                    "postgres".into()
                }
            }
        })
    }
}

// End of file: src/models/env_vars.tf
