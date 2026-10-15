#![forbid(unsafe_code)]
//! Batch CLI, scripts and a minimal persistent line-oriented interactive session.

pub mod cli;
mod completion;
mod interactive;
pub use tabdat_reporting::{catalog, help, json_results};
