// Start of file: src/models/env_vars.tf

/*
    * Defines the application's environment variables and provides a method
    * for loading them from the system (or .env) using dotenv.
*/

use std::borrow::Cow;
use anyhow::{bail, Result};
use dotenv::dotenv;

#[derive(Clone, Debug)]
pub struct EnvironmentVariables {
    pub host: Cow<'static, str>,
    pub port: u16,
    pub protocol: Cow<'static, str>,
    pub max_request_body_size: usize,
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
            db_host: match dotenv::var("DB_HOST") {
                Ok(host) => host.into(),
                Err(err) => bail!("Missing DB_HOST: {}", err),
            },
            db_port: match dotenv::var("DB_PORT") {
                Ok(port) => port.parse()?,
                Err(_) => 5432,
            },
            db_user: match dotenv::var("DB_USER") {
                Ok(user) => user.into(),
                Err(err) => bail!("Missing DB_USER: {}", err),
            },
            db_password: match dotenv::var("DB_PASSWORD") {
                Ok(pass) => pass.into(),
                Err(err) => bail!("Missing DB_PASSWORD: {}", err),
            }
        })
    }
}

// End of file: src/models/env_vars.tf
