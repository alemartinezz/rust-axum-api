// Start of file: /src/config/environment.rs

/*
    * Defines the application's environment variables and provides a method
    * for loading them from the system (or .env) using dotenv.
*/
use std::{borrow::Cow, collections::HashMap};
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
    pub db_password: Cow<'static, str>,
}

impl EnvironmentVariables {
    pub fn from_env() -> Result<Self> {
        dotenv().ok();
        let vars: HashMap<String, String> = dotenv::vars().collect();
        
        let get_var = |key: &str| vars.get(key).map(|s| s.as_str());

        Ok(Self {
            environment: match get_var("ENVIRONMENT") {
                Some(env) => Cow::from(env.to_owned()),
                None => {
                    warn!("Missing ENVIRONMENT, defaulting to 'development'");
                    Cow::Borrowed("development")
                }
            },
            host: match get_var("HOST") {
                Some(host) => Cow::from(host.to_owned()),
                None => Cow::Borrowed("127.0.0.1"),
            },
            port: match get_var("PORT") {
                Some(port) => port.parse()?,
                None => 3000,
            },
            protocol: match get_var("PROTOCOL") {
                Some(proto) => Cow::from(proto.to_owned()),
                None => Cow::Borrowed("http"),
            },
            max_request_body_size: match get_var("MAX_REQUEST_BODY_SIZE") {
                Some(size) => size.parse()?,
                None => 2_097_152,
            },
            default_timeout_seconds: match get_var("DEFAULT_TIMEOUT_SECONDS") {
                Some(seconds) => seconds.parse()?,
                None => 3,
            },
            db_host: match get_var("DB_HOST") {
                Some(host) => Cow::from(host.to_owned()),
                None => {
                    warn!("Missing DB_HOST, defaulting to 'localhost'");
                    Cow::Borrowed("localhost")
                }
            },
            db_port: match get_var("DB_PORT") {
                Some(port) => port.parse()?,
                None => 5432,
            },
            db_user: match get_var("DB_USER") {
                Some(user) => Cow::from(user.to_owned()),
                None => {
                    warn!("Missing DB_USER, defaulting to 'postgres'");
                    Cow::Borrowed("postgres")
                }
            },
            db_password: match get_var("DB_PASSWORD") {
                Some(pass) => Cow::from(pass.to_owned()),
                None => {
                    warn!("Missing DB_PASSWORD, defaulting to 'postgres'");
                    Cow::Borrowed("postgres")
                }
            },
        })
    }
}

// End of file: /src/config/environment.rs
