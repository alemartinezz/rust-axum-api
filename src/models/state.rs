// Start of file: src/models/state.tf

/*
    * Defines the `AppState` that is cloned and passed around to handlers
    * (controllers) and middleware, allowing them to access shared resources
    * such as database connections, environment config, etc.
*/

use crate::models::env_vars::EnvironmentVariables;

#[derive(Clone, Debug)]
pub struct AppState {
    pub env: EnvironmentVariables,
}
 
/*
    * Helper constructor that loads environment variables
    * and packages them into AppState.
*/ 
impl AppState {
    pub fn from_env() -> anyhow::Result<Self> {
        let env: EnvironmentVariables = EnvironmentVariables::from_env()?;
        Ok(Self { env })
    }
}
 
// End of file: src/models/state.tf
 