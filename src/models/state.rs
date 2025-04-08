use crate::models::env_vars::EnvironmentVariables;

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