//! Safe facade over the ReadStat v1.0.0 DTA reader.
//!
//! This crate is a feasibility spike, not product support. It proves that a
//! Rust-owned DTA ingestion boundary can preserve variable labels, value
//! labels, storage types, and system/tagged/defined missingness without
//! pandas, and that **all** `unsafe` stays in the low-level [`readstat_sys`]
//! adapter.
//!
//! This crate is `forbid(unsafe_code)`: it only re-exports the owned,
//! Rust-allocated result types produced by the adapter and forwards to its
//! safe `parse_dta` entry point. No foreign pointer, parser handle, or C
//! string can appear here — the compiler enforces that.

#![forbid(unsafe_code)]

use std::path::Path;

// Re-export the owned result types and the safe entry point. These are plain
// Rust-allocated values; the adapter has already copied all borrowed C data
// into them before returning.
pub use readstat_sys::{
  DtaDataset, DtaError, DtaLabelSet, DtaMetadata, DtaType, DtaValue, DtaValueLabel, DtaVariable,
  parse_dta,
};

/// Parse a Stata `.dta` file into an owned [`DtaDataset`].
///
/// This is a thin, safe forward to the low-level adapter. The native parser is
/// created, driven, and freed inside the adapter on every path (success,
/// native error, and callback abort).
pub fn load_dta(path: &Path) -> Result<DtaDataset, DtaError> {
  parse_dta(path)
}
