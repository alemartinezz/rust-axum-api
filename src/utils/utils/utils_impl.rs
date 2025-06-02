// Start of file: /src/utils/utils/utils_impl.rs

// The utils module organizes useful functions, used by other modules.

use serde::Serialize;
use anyhow::Result;

// Convert any `Serialize` type into a two-space-indented JSON string.
pub fn to_two_space_indented_json<T: Serialize>(value: &T) -> Result<String> {
    let json_value: serde_json::Value = serde_json::to_value(value)?;
    let pretty_json: String = serde_json::to_string_pretty(&json_value)?;
    Ok(pretty_json)
}

// End of file: /src/utils/utils/utils_impl.rs
