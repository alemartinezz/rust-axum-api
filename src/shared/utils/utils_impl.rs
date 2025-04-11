// Start of file: /src/shared/utils/mod.rs

// * The utils module organizes useful functions, used by other modules.

use serde_json::{
    ser::PrettyFormatter, Serializer
};
use serde::Serialize;

// * Convert any `Serialize` type into a two-space-indented JSON string.
pub fn to_two_space_indented_json<T: Serialize>(value: &T) -> serde_json::Result<String> {
    let mut writer: Vec<u8> = Vec::new();

    let formatter: PrettyFormatter<'_> = PrettyFormatter::with_indent(b"  ");
    let mut ser: Serializer<&mut Vec<u8>, PrettyFormatter<'_>> = Serializer::with_formatter(&mut writer, formatter);

    value.serialize(&mut ser)?;
    
    Ok(String::from_utf8(writer).expect("Should always be valid UTF-8"))
}

// End of file: /src/shared/utils/mod.rs
