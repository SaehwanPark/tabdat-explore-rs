#![forbid(unsafe_code)]
//! Existing batch CLI and script interface. Interactive REPL remains later work.

pub mod cli;
pub use tabdat_reporting::{catalog, help, json_results};
