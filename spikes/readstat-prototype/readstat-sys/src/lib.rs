//! Low-level ReadStat v1.0.0 FFI adapter.
//!
//! This crate is the only place in the spike that touches foreign pointers or
//! `unsafe`. It builds the pinned ReadStat release (see `build.rs`), declares
//! the raw C ABI, and exposes a safe `parse_dta` entry point that returns
//! owned Rust values. Raw parser/variable/metadata pointers and C string
//! pointers must not escape this crate.
//!
//! Safety model:
//! * every `unsafe` block carries an adjacent `// SAFETY:` rationale,
//! * `unsafe_op_in_unsafe_fn` is denied crate-wide,
//! * the parser handle is created and freed exactly once on every path,
//! * callback trampolines copy borrowed C data into owned values before
//!   returning and never let a Rust panic unwind across the FFI boundary.

#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

// ---------------------------------------------------------------------------
// Opaque foreign types
// ---------------------------------------------------------------------------

/// Opaque ReadStat parser handle. Owned by `parse_dta`; freed on every path.
#[repr(C)]
struct readstat_parser_t {
  _private: [u8; 0],
}

/// Opaque ReadStat variable handle. Borrowed inside callbacks only.
#[repr(C)]
struct readstat_variable_t {
  _private: [u8; 0],
}

/// Opaque ReadStat metadata handle. Borrowed inside the metadata callback.
#[repr(C)]
struct readstat_metadata_t {
  _private: [u8; 0],
}

/// Borrowed C callback value. Its size, union, bitfields and padding are known
/// only to `value_bridge.c`; Rust must never instantiate or dereference it.
#[repr(C)]
struct readstat_value_t {
  _private: [u8; 0],
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

#[allow(non_camel_case_types)]
type readstat_type_t = c_int;

const READSTAT_TYPE_STRING: readstat_type_t = 0;
const READSTAT_TYPE_INT8: readstat_type_t = 1;
const READSTAT_TYPE_INT16: readstat_type_t = 2;
const READSTAT_TYPE_INT32: readstat_type_t = 3;
const READSTAT_TYPE_FLOAT: readstat_type_t = 4;
const READSTAT_TYPE_DOUBLE: readstat_type_t = 5;

const READSTAT_OK: c_int = 0;
const READSTAT_HANDLER_OK: c_int = 0;
const READSTAT_HANDLER_ABORT: c_int = 1;

// ---------------------------------------------------------------------------
// Raw FFI declarations
// ---------------------------------------------------------------------------

unsafe extern "C" {
  fn readstat_parser_init() -> *mut readstat_parser_t;
  fn readstat_parser_free(parser: *mut readstat_parser_t);

  fn readstat_set_metadata_handler(
    parser: *mut readstat_parser_t,
    handler: extern "C" fn(*mut readstat_metadata_t, *mut c_void) -> c_int,
  ) -> c_int;
  fn readstat_set_variable_handler(
    parser: *mut readstat_parser_t,
    handler: extern "C" fn(c_int, *mut readstat_variable_t, *const c_char, *mut c_void) -> c_int,
  ) -> c_int;
  fn tabdat_readstat_set_value_handlers(parser: *mut readstat_parser_t) -> c_int;
  fn readstat_set_error_handler(
    parser: *mut readstat_parser_t,
    handler: extern "C" fn(*const c_char, *mut c_void),
  ) -> c_int;

  fn readstat_parse_dta(
    parser: *mut readstat_parser_t,
    path: *const c_char,
    user_ctx: *mut c_void,
  ) -> c_int;

  // Metadata accessors.
  fn readstat_get_row_count(metadata: *mut readstat_metadata_t) -> c_int;
  fn readstat_get_var_count(metadata: *mut readstat_metadata_t) -> c_int;
  fn readstat_get_file_label(metadata: *mut readstat_metadata_t) -> *const c_char;
  fn readstat_get_file_format_version(metadata: *mut readstat_metadata_t) -> c_int;
  fn readstat_get_file_format_is_64bit(metadata: *mut readstat_metadata_t) -> c_int;

  // Variable accessors.
  fn readstat_variable_get_index(variable: *const readstat_variable_t) -> c_int;
  fn readstat_variable_get_name(variable: *const readstat_variable_t) -> *const c_char;
  fn readstat_variable_get_label(variable: *const readstat_variable_t) -> *const c_char;
  fn readstat_variable_get_type(variable: *const readstat_variable_t) -> readstat_type_t;

  // Value accessors.
  fn tabdat_readstat_value_type(value: *const readstat_value_t) -> readstat_type_t;
  fn tabdat_readstat_value_is_system_missing(value: *const readstat_value_t) -> c_int;
  fn tabdat_readstat_value_is_tagged_missing(value: *const readstat_value_t) -> c_int;
  fn tabdat_readstat_value_is_defined_missing(
    value: *const readstat_value_t,
    variable: *mut readstat_variable_t,
  ) -> c_int;
  fn tabdat_readstat_value_tag(value: *const readstat_value_t) -> c_char;
  fn tabdat_readstat_int8_value(value: *const readstat_value_t) -> i8;
  fn tabdat_readstat_int16_value(value: *const readstat_value_t) -> i16;
  fn tabdat_readstat_int32_value(value: *const readstat_value_t) -> i32;
  fn tabdat_readstat_float_value(value: *const readstat_value_t) -> f32;
  fn tabdat_readstat_double_value(value: *const readstat_value_t) -> f64;
  fn tabdat_readstat_string_value(value: *const readstat_value_t) -> *const c_char;
}

// ---------------------------------------------------------------------------
// Owned result types
// ---------------------------------------------------------------------------

/// Error returned when the native parser reports a failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DtaError {
  /// The native parser returned a non-OK error code. `message` carries the
  /// parser's error-handler text when it was invoked.
  Native { code: i32, message: Option<String> },
  /// The path could not be converted to a NUL-terminated C string.
  Path(String),
}

impl std::fmt::Display for DtaError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Native { code, message } => {
        if let Some(message) = message {
          write!(f, "readstat error {code}: {message}")
        } else {
          write!(f, "readstat error {code}")
        }
      }
      Self::Path(message) => write!(f, "invalid path: {message}"),
    }
  }
}

impl std::error::Error for DtaError {}

/// File-level metadata copied out of the metadata callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DtaMetadata {
  pub row_count: i64,
  pub var_count: i64,
  pub file_label: Option<String>,
  pub file_format_version: i64,
  pub is_64bit: bool,
}

/// Storage type of a variable, mirroring `readstat_type_t` for the types the
/// DTA reader delivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtaType {
  String,
  Int8,
  Int16,
  Int32,
  Float,
  Double,
}

impl DtaType {
  fn from_raw(raw: i32) -> Self {
    match raw {
      READSTAT_TYPE_STRING => Self::String,
      READSTAT_TYPE_INT8 => Self::Int8,
      READSTAT_TYPE_INT16 => Self::Int16,
      READSTAT_TYPE_INT32 => Self::Int32,
      READSTAT_TYPE_FLOAT => Self::Float,
      READSTAT_TYPE_DOUBLE => Self::Double,
      _ => Self::String,
    }
  }
}

/// A variable's identity and metadata, copied out of the variable callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DtaVariable {
  pub name: String,
  pub label: Option<String>,
  pub type_: DtaType,
  /// Name of the value-label set attached to this variable, if any.
  pub label_set: Option<String>,
}

/// A single value-label entry: the key value and its display text.
#[derive(Debug, Clone, PartialEq)]
pub struct DtaValueLabel {
  pub value: DtaValue,
  pub label: String,
}

/// A named value-label set.
#[derive(Debug, Clone, PartialEq)]
pub struct DtaLabelSet {
  pub name: String,
  pub labels: Vec<DtaValueLabel>,
}

/// An owned cell value with explicit missingness.
#[derive(Debug, Clone, PartialEq)]
pub enum DtaValue {
  Int8(i8),
  Int16(i16),
  Int32(i32),
  Float(f32),
  Double(f64),
  String(String),
  /// System missing (Stata `.`).
  SystemMissing,
  /// Tagged missing (Stata `.a`, `.b`, ...).
  TaggedMissing(char),
  /// Defined missing (SPSS concept; does not occur in DTA). Carries the
  /// underlying double so no data is lost.
  DefinedMissing(f64),
}

impl DtaValue {
  /// True for any missingness kind.
  pub fn is_missing(&self) -> bool {
    matches!(
      self,
      DtaValue::SystemMissing | DtaValue::TaggedMissing(_) | DtaValue::DefinedMissing(_)
    )
  }
}

/// The fully parsed, Rust-owned dataset.
#[derive(Debug, Clone, PartialEq)]
pub struct DtaDataset {
  pub metadata: DtaMetadata,
  pub variables: Vec<DtaVariable>,
  pub label_sets: Vec<DtaLabelSet>,
  /// `rows[obs][var]`; every row has `variables.len()` cells.
  pub rows: Vec<Vec<DtaValue>>,
}

/// Accumulator passed to the ReadStat callbacks as the opaque context.
///
/// Callback order for DTA is metadata → variables → rows → value labels, so
/// `rows` is populated before `label_sets`; both are complete when
/// `readstat_parse_dta` returns.
struct ParserState {
  metadata: Option<DtaMetadata>,
  variables: Vec<DtaVariable>,
  rows: Vec<Vec<DtaValue>>,
  label_sets: Vec<DtaLabelSet>,
  error: Option<String>,
}

// ---------------------------------------------------------------------------
// Safe extraction helpers (all `unsafe` confined here)
// ---------------------------------------------------------------------------

/// Copy a NUL-terminated C string into an owned `String`. Returns `None` for a
/// null pointer.
fn cstr_to_string(ptr: *const c_char) -> Option<String> {
  if ptr.is_null() {
    return None;
  }
  // SAFETY: callers only pass non-null pointers produced by ReadStat accessors,
  // which are valid NUL-terminated C strings while the callback is on the stack.
  let cstr = unsafe { CStr::from_ptr(ptr) };
  Some(cstr.to_string_lossy().into_owned())
}

/// Copy a value while the C bridge's borrowed stack value remains live.
fn extract_value(value: *const readstat_value_t, variable: *const readstat_variable_t) -> DtaValue {
  // SAFETY: `value`/`variable` are valid for the duration of the callback.
  let (is_system, is_tagged, is_defined, tag, type_) = unsafe {
    (
      tabdat_readstat_value_is_system_missing(value),
      tabdat_readstat_value_is_tagged_missing(value),
      if variable.is_null() {
        0
      } else {
        tabdat_readstat_value_is_defined_missing(value, variable as *mut readstat_variable_t)
      },
      tabdat_readstat_value_tag(value),
      tabdat_readstat_value_type(value),
    )
  };

  if is_system != 0 {
    return DtaValue::SystemMissing;
  }
  if is_tagged != 0 {
    // SAFETY: `tag` is a plain byte copied out of the value; ReadStat emits
    // ASCII tags for Stata, so the `u8 as char` conversion is sound.
    return DtaValue::TaggedMissing(tag as u8 as char);
  }
  if is_defined != 0 {
    // Defined missing is an SPSS concept and does not occur in DTA; carry the
    // underlying double so no data is lost if it ever appears.
    // SAFETY: `value` is valid for the duration of the callback.
    let underlying = unsafe { tabdat_readstat_double_value(value) };
    return DtaValue::DefinedMissing(underlying);
  }

  // SAFETY: `value` is valid for the duration of the callback.
  unsafe {
    match type_ {
      READSTAT_TYPE_STRING => {
        let ptr = tabdat_readstat_string_value(value);
        DtaValue::String(cstr_to_string(ptr).unwrap_or_default())
      }
      READSTAT_TYPE_INT8 => DtaValue::Int8(tabdat_readstat_int8_value(value)),
      READSTAT_TYPE_INT16 => DtaValue::Int16(tabdat_readstat_int16_value(value)),
      READSTAT_TYPE_INT32 => DtaValue::Int32(tabdat_readstat_int32_value(value)),
      READSTAT_TYPE_FLOAT => DtaValue::Float(tabdat_readstat_float_value(value)),
      READSTAT_TYPE_DOUBLE => DtaValue::Double(tabdat_readstat_double_value(value)),
      // STRING_REF and unknown types are not expected in the DTA read path.
      _ => DtaValue::SystemMissing,
    }
  }
}

/// Copy the metadata fields into an owned [`DtaMetadata`].
fn extract_metadata(metadata: *const readstat_metadata_t) -> DtaMetadata {
  let metadata = metadata as *mut readstat_metadata_t;
  // SAFETY: `metadata` is valid for the duration of the callback.
  let (row_count, var_count, file_label, version, is_64) = unsafe {
    (
      readstat_get_row_count(metadata),
      readstat_get_var_count(metadata),
      readstat_get_file_label(metadata),
      readstat_get_file_format_version(metadata),
      readstat_get_file_format_is_64bit(metadata),
    )
  };
  DtaMetadata {
    row_count: row_count as i64,
    var_count: var_count as i64,
    file_label: cstr_to_string(file_label),
    file_format_version: version as i64,
    is_64bit: is_64 != 0,
  }
}

/// Copy the variable identity fields into an owned [`DtaVariable`].
fn extract_variable(
  variable: *const readstat_variable_t,
  label_set_name: *const c_char,
) -> DtaVariable {
  // SAFETY: `variable` is valid for the duration of the callback.
  let (name, label, type_) = unsafe {
    (
      readstat_variable_get_name(variable),
      readstat_variable_get_label(variable),
      readstat_variable_get_type(variable),
    )
  };
  DtaVariable {
    name: cstr_to_string(name).unwrap_or_default(),
    label: cstr_to_string(label),
    type_: DtaType::from_raw(type_),
    label_set: cstr_to_string(label_set_name),
  }
}

// ---------------------------------------------------------------------------
// Callback trampolines
// ---------------------------------------------------------------------------
//
// Each trampoline recovers the `ParserState` from the opaque context, copies
// the borrowed C data into owned Rust values via the helpers above, and never
// lets a Rust panic unwind across the FFI boundary (a panic aborts the parse,
// which ReadStat treats as a user abort).

extern "C" fn metadata_trampoline(metadata: *mut readstat_metadata_t, ctx: *mut c_void) -> c_int {
  let outcome = catch_unwind(AssertUnwindSafe(|| {
    // SAFETY: `ctx` is the `ParserState` allocated by `parse_dta` and lives for
    // the whole parse; ReadStat invokes callbacks single-threaded and
    // reentrantly on the calling stack.
    let state = unsafe { &mut *(ctx as *mut ParserState) };
    // SAFETY: `metadata` is valid for the duration of this callback.
    let meta = extract_metadata(metadata as *const readstat_metadata_t);
    state.metadata = Some(meta);
  }));
  if outcome.is_err() {
    READSTAT_HANDLER_ABORT
  } else {
    READSTAT_HANDLER_OK
  }
}

extern "C" fn variable_trampoline(
  _index: c_int,
  variable: *mut readstat_variable_t,
  label_set_name: *const c_char,
  ctx: *mut c_void,
) -> c_int {
  let outcome = catch_unwind(AssertUnwindSafe(|| {
    // SAFETY: as in `metadata_trampoline`.
    let state = unsafe { &mut *(ctx as *mut ParserState) };
    // SAFETY: `variable`/`label_set_name` are valid for this callback.
    let var = extract_variable(variable as *const readstat_variable_t, label_set_name);
    state.variables.push(var);
  }));
  if outcome.is_err() {
    READSTAT_HANDLER_ABORT
  } else {
    READSTAT_HANDLER_OK
  }
}

// SAFETY: this unique symbol is called only by the header-compiled bridge,
// with the pointer live for this synchronous callback; no Rust ABI is exposed.
#[unsafe(export_name = "tabdat_readstat_value_callback")]
extern "C" fn value_trampoline(
  obs_index: c_int,
  variable: *mut readstat_variable_t,
  value: *const readstat_value_t,
  ctx: *mut c_void,
) -> c_int {
  let outcome = catch_unwind(AssertUnwindSafe(|| {
    // SAFETY: as in `metadata_trampoline`.
    let state = unsafe { &mut *(ctx as *mut ParserState) };
    // SAFETY: `variable` is valid for this callback.
    let col =
      unsafe { readstat_variable_get_index(variable as *const readstat_variable_t) } as usize;
    // SAFETY: `value`/`variable` are valid for this callback.
    let cell = extract_value(value, variable as *const readstat_variable_t);
    let row = obs_index as usize;
    while state.rows.len() <= row {
      state.rows.push(Vec::new());
    }
    let row = &mut state.rows[row];
    while row.len() <= col {
      row.push(DtaValue::SystemMissing);
    }
    row[col] = cell;
  }));
  if outcome.is_err() {
    READSTAT_HANDLER_ABORT
  } else {
    READSTAT_HANDLER_OK
  }
}

// SAFETY: unique bridge symbol; value and strings live until this callback
// returns, and only owned copies are retained in ParserState.
#[unsafe(export_name = "tabdat_readstat_label_callback")]
extern "C" fn value_label_trampoline(
  set_name: *const c_char,
  value: *const readstat_value_t,
  label: *const c_char,
  ctx: *mut c_void,
) -> c_int {
  let outcome = catch_unwind(AssertUnwindSafe(|| {
    // SAFETY: as in `metadata_trampoline`.
    let state = unsafe { &mut *(ctx as *mut ParserState) };
    let name = cstr_to_string(set_name).unwrap_or_default();
    // SAFETY: `value` is valid for this callback; label-set keys have no
    // associated variable, so pass null for the defined-missing check.
    let key = extract_value(value, std::ptr::null());
    let text = cstr_to_string(label).unwrap_or_default();
    let inserted = !state.label_sets.iter().any(|set| set.name == name);
    if inserted {
      state.label_sets.push(DtaLabelSet {
        name: name.clone(),
        labels: Vec::new(),
      });
    }
    // The set was just pushed (or already existed); it is the last match.
    if let Some(entry) = state.label_sets.iter_mut().find(|set| set.name == name) {
      entry.labels.push(DtaValueLabel {
        value: key,
        label: text,
      });
    }
  }));
  if outcome.is_err() {
    READSTAT_HANDLER_ABORT
  } else {
    READSTAT_HANDLER_OK
  }
}

extern "C" fn error_trampoline(message: *const c_char, ctx: *mut c_void) {
  let outcome = catch_unwind(AssertUnwindSafe(|| {
    // SAFETY: as in `metadata_trampoline`.
    let state = unsafe { &mut *(ctx as *mut ParserState) };
    state.error = cstr_to_string(message);
  }));
  let _ = outcome;
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Parse a Stata `.dta` file into an owned [`DtaDataset`].
///
/// The native parser is created, driven, and freed within this call on every
/// path (success, native error, and callback abort).
pub fn parse_dta(path: &Path) -> Result<DtaDataset, DtaError> {
  parse_dta_with_metadata_handler(path, metadata_trampoline)
}

// Private injection seam tests a real native handler abort and teardown without
// exposing callback pointers or parser handles through the safe API.
fn parse_dta_with_metadata_handler(
  path: &Path,
  metadata_handler: extern "C" fn(*mut readstat_metadata_t, *mut c_void) -> c_int,
) -> Result<DtaDataset, DtaError> {
  let c_path = CString::new(path.as_os_str().to_string_lossy().as_bytes())
    .map_err(|_| DtaError::Path("path contains NUL".into()))?;

  let mut state = ParserState {
    metadata: None,
    variables: Vec::new(),
    rows: Vec::new(),
    label_sets: Vec::new(),
    error: None,
  };

  // SAFETY: `readstat_parser_init` returns a heap-allocated parser that we own
  // and free exactly once below; a null return is treated as allocation
  // failure rather than dereferenced.
  let parser = unsafe { readstat_parser_init() };
  if parser.is_null() {
    return Err(DtaError::Native {
      code: -1,
      message: Some("readstat_parser_init returned null".into()),
    });
  }

  // SAFETY: the trampolines are `extern "C"` functions with the exact
  // signatures ReadStat expects; `parser` is valid until freed below.
  let setup = unsafe {
    [
      readstat_set_metadata_handler(parser, metadata_handler),
      readstat_set_variable_handler(parser, variable_trampoline),
      tabdat_readstat_set_value_handlers(parser),
      readstat_set_error_handler(parser, error_trampoline),
    ]
    .into_iter()
    .find(|code| *code != READSTAT_OK)
    .unwrap_or(READSTAT_OK)
  };
  if setup != READSTAT_OK {
    // SAFETY: `parser` is the valid handle created above and is freed exactly
    // once on this early-exit path.
    unsafe { readstat_parser_free(parser) };
    return Err(DtaError::Native {
      code: setup,
      message: state.error,
    });
  }

  // SAFETY: `parser` is valid, `c_path` is a valid NUL-terminated C string that
  // outlives the call, and `&mut state` lives until after the call returns.
  // ReadStat invokes the callbacks single-threaded on this stack; the
  // trampolines copy all borrowed data before returning.
  let code = unsafe {
    readstat_parse_dta(
      parser,
      c_path.as_ptr(),
      &mut state as *mut ParserState as *mut c_void,
    )
  };

  // SAFETY: `parser` is the valid handle created above and is freed exactly
  // once here, after `readstat_parse_dta` has returned and no callback can be
  // on the stack.
  unsafe { readstat_parser_free(parser) };

  if code != READSTAT_OK {
    return Err(DtaError::Native {
      code,
      message: state.error,
    });
  }

  let metadata = state.metadata.unwrap_or(DtaMetadata {
    row_count: state.rows.len() as i64,
    var_count: state.variables.len() as i64,
    file_label: None,
    file_format_version: 0,
    is_64bit: false,
  });

  Ok(DtaDataset {
    metadata,
    variables: state.variables,
    label_sets: state.label_sets,
    rows: state.rows,
  })
}

#[cfg(test)]
mod value_tests;
