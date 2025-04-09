// Start of file: src/utils/utils.rs

use serde_json::{
    ser::PrettyFormatter, Serializer
};
use serde::Serialize;
use tracing::{error, info};

use crate::models::response_format::ResponseFormat;

/*
    * Logs the final `ResponseFormat` in a pretty JSON.
*/
pub fn log_json(wrapped: &ResponseFormat) {
    match to_two_space_indented_json(wrapped) {
        Ok(spaced_json) => {
            info!("\nFinal response:\n{}", spaced_json);
        }
        Err(err) => {
            // Add explicit error logging
            error!("Failed to format response JSON: {:?}", err);
        }
    }
}

/*
    * Convert any `Serialize` type into a two-space-indented JSON string.
*/
fn to_two_space_indented_json<T: Serialize>(value: &T) -> serde_json::Result<String> {
    let mut writer: Vec<u8> = Vec::new();
    
    let formatter: PrettyFormatter<'_> = PrettyFormatter::with_indent(b"  ");
    
    let mut ser: Serializer<&mut Vec<u8>, PrettyFormatter<'_>> =
        Serializer::with_formatter(&mut writer, formatter);

    value.serialize(&mut ser)?;

    Ok(String::from_utf8(writer).expect("should be valid UTF-8"))
}

// End of file: src/utils/utils.rs
