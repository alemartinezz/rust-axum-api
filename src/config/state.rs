// Start of file: /src/config/state.rs

/*
    * Defines the `AppState` that is cloned and passed around to handlers
    * (controllers) and middleware, allowing them to access shared resources.
*/

use crate::config::environment::EnvironmentVariables;

#[derive(Clone, Debug)]
pub struct AppState {
    pub env: EnvironmentVariables,
}

impl AppState {
    pub fn from_env() -> anyhow::Result<Self> {
        let env: EnvironmentVariables = EnvironmentVariables::from_env()?;
        Ok(Self { env })
    }
}

// End of file: /src/config/state.rs
