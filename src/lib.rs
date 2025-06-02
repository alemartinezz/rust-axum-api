// Start of file: /src/lib.rs

/*
* This lib.rs re-exports the major modules for the reorganized structure.
* New structure: api, core, config, utils
* Legacy aliases maintained for backward compatibility
*/

pub mod config;
pub mod core;
pub mod api;
pub mod utils;

// Legacy aliases for backward compatibility
pub mod features {
    pub use crate::api::*;
}

pub mod shared {
    pub use crate::utils::*;
}

// End of file: /src/lib.rs
