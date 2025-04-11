// Start of file: /src/config/environment.rs

// * Optimized environment configuration with a singleton pattern
// * and zero-copy parsing.

use std::{borrow::Cow, collections::HashMap};
// * anyhow for convenient error handling
use anyhow::{Context, Result};
// * once_cell for lazy static initialization
use once_cell::sync::Lazy;
use tracing::warn;

// ! Default values for environment variables (used if variables aren't set):
const DEFAULT_ENVIRONMENT: &str = "development";
const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PROTOCOL: &str = "http";
const DEFAULT_DB_HOST: &str = "localhost";
const DEFAULT_DB_USER: &str = "postgres";
const DEFAULT_DB_PASSWORD: &str = "postgres";
const DEFAULT_PORT: u16 = 3000;
const DEFAULT_MAX_BODY_SIZE: usize = 2_097_152; // 2MB
const DEFAULT_TIMEOUT: u64 = 3; // 3 seconds
const DEFAULT_DB_PORT: u16 = 5432; // Default Postgres port

// * A struct containing all environment variables used by the app
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
    // * Loads environment variables once.
    // * Only reads .env if ENVIRONMENT != "production".
    fn load() -> Result<Self> {
        // ? In non-production environments, attempt to load .env
        if std::env::var("ENVIRONMENT").unwrap_or_default() != "production" {
            dotenv::dotenv().ok();
        }

        // * Collect all environment vars from the system and .env
        let vars: HashMap<String, String> = std::env::vars()
            .chain(dotenv::vars())
            .collect();

        // * A small helper closure to fetch a variable by key
        let get_var = |key: &str| vars.get(key).map(String::as_str);

        // * Build our EnvironmentVariables, providing defaults if missing
        Ok(Self {
            environment: get_var("ENVIRONMENT")
                .map(|s| Cow::Owned(s.into()))
                .unwrap_or_else(|| {
                    warn!("Missing ENVIRONMENT, defaulting to '{DEFAULT_ENVIRONMENT}'");
                    Cow::Borrowed(DEFAULT_ENVIRONMENT)
                }),

            host: get_var("HOST")
                .map(|s| Cow::Owned(s.into()))
                .unwrap_or(Cow::Borrowed(DEFAULT_HOST)),

            port: get_var("PORT")
                .map(|s| s.parse().context("Invalid PORT value"))
                .transpose()?
                .unwrap_or(DEFAULT_PORT),

            protocol: get_var("PROTOCOL")
                .map(|s| Cow::Owned(s.into()))
                .unwrap_or(Cow::Borrowed(DEFAULT_PROTOCOL)),

            max_request_body_size: get_var("MAX_REQUEST_BODY_SIZE")
                .map(|s| s.parse().context("Invalid MAX_REQUEST_BODY_SIZE"))
                .transpose()?
                .unwrap_or(DEFAULT_MAX_BODY_SIZE),

            default_timeout_seconds: get_var("DEFAULT_TIMEOUT_SECONDS")
                .map(|s| s.parse().context("Invalid DEFAULT_TIMEOUT_SECONDS"))
                .transpose()?
                .unwrap_or(DEFAULT_TIMEOUT),

            db_host: get_var("DB_HOST")
                .map(|s| Cow::Owned(s.into()))
                .unwrap_or_else(|| {
                    warn!("Missing DB_HOST, defaulting to '{DEFAULT_DB_HOST}'");
                    Cow::Borrowed(DEFAULT_DB_HOST)
                }),

            db_port: get_var("DB_PORT")
                .map(|s| s.parse().context("Invalid DB_PORT"))
                .transpose()?
                .unwrap_or(DEFAULT_DB_PORT),

            db_user: get_var("DB_USER")
                .map(|s| Cow::Owned(s.into()))
                .unwrap_or_else(|| {
                    warn!("Missing DB_USER, defaulting to '{DEFAULT_DB_USER}'");
                    Cow::Borrowed(DEFAULT_DB_USER)
                }),

            db_password: get_var("DB_PASSWORD")
                .map(|s| Cow::Owned(s.into()))
                .unwrap_or_else(|| {
                    warn!("Missing DB_PASSWORD, defaulting to '{DEFAULT_DB_PASSWORD}'");
                    Cow::Borrowed(DEFAULT_DB_PASSWORD)
                }),
        })
    }

    // * Returns a reference to the lazily-initialized environment configuration
    pub fn instance() -> &'static Self {
        static INSTANCE: Lazy<Result<EnvironmentVariables, anyhow::Error>> = Lazy::new(|| {
            let config: EnvironmentVariables = EnvironmentVariables::load()?;
            
            if cfg!(debug_assertions) {
                tracing::debug!("Loaded environment configuration: {:#?}", config);
            }
            
            Ok(config)
        });

        // ! Panics if loading fails
        INSTANCE.as_ref().expect("Failed to load environment configuration")
    }
}
// End of file: /src/config/environment.rs