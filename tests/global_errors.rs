//! tests/global_errors.rs
//! This file serves as an integration test crate that aggregates all
//! tests from the global_errors subdirectory.

// Use an inline module to import submodules from the global_errors folder.
// The paths are adjusted ("../global_errors/404.rs" etc.) because this file
// resides in the `tests/` folder.
#[cfg(test)]
mod global_errors {
    #[path = "../global_errors/404.rs"]
    mod e404;

    #[path = "../global_errors/408.rs"]
    mod e408;

    #[path = "../global_errors/413.rs"]
    mod e413;

    #[path = "../global_errors/500.rs"]
    mod e500;
}
