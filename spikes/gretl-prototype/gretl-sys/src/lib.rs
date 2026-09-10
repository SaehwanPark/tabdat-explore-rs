//! Low-level FFI bindings to libgretl (gretl 2026b).
//!
//! This is the **only** crate in the spike that contains `unsafe` code. It
//! declares the `extern "C"` surface of the C ownership shim (`shim.c`) and
//! wraps the opaque native handles in RAII types so that native objects are
//! always released, on every path. No raw pointer or foreign struct type is
//! part of this crate's public API: callers receive [`GretlDataset`] and
//! [`GretlModel`] values that own their native memory.
//!
//! The C shim is compiled from the same pinned gretl source tree, so its view
//! of libgretl's `DATASET`/`MODEL` layouts is always correct; this crate never
//! re-declares those structs.

use std::ffi::CString;
use std::os::raw::{c_char, c_double, c_int, c_void};

unsafe extern "C" {
  fn gretl_shim_init();
  fn gretl_shim_cleanup();

  fn gretl_shim_dataset_new(nvar: c_int, nobs: c_int) -> *mut c_void;
  fn gretl_shim_dataset_set_varname(handle: *mut c_void, i: c_int, name: *const c_char);
  fn gretl_shim_dataset_set_value(handle: *mut c_void, var: c_int, obs: c_int, value: c_double);
  fn gretl_shim_dataset_free(handle: *mut c_void);

  fn gretl_shim_ols(dset: *mut c_void, list: *const c_int, nlist: c_int) -> *mut c_void;

  fn gretl_shim_model_ncoeff(handle: *const c_void) -> c_int;
  fn gretl_shim_model_nobs(handle: *const c_void) -> c_int;
  fn gretl_shim_model_dfn(handle: *const c_void) -> c_int;
  fn gretl_shim_model_dfd(handle: *const c_void) -> c_int;
  fn gretl_shim_model_errcode(handle: *const c_void) -> c_int;
  fn gretl_shim_model_coeff(handle: *const c_void, i: c_int) -> c_double;
  fn gretl_shim_model_sderr(handle: *const c_void, i: c_int) -> c_double;
  fn gretl_shim_model_vcv(handle: *const c_void, i: c_int, j: c_int) -> c_double;
  fn gretl_shim_model_scalar(handle: *const c_void, which: c_int) -> c_double;
  fn gretl_shim_model_free(handle: *mut c_void);
}

/// Scalar fit-statistic selectors for [`GretlModel::scalar`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Stat {
  RSquared = 0,
  AdjRSquared = 1,
  Sigma = 2,
  Ess = 3,
  Tss = 4,
  FStatistic = 5,
  LogLikelihood = 6,
}

impl Stat {
  fn code(self) -> c_int {
    self as c_int
  }
}

/// Initialize the libgretl process state.
///
/// # Safety contract
/// Must be called before any dataset/model operation. Safe to call more than
/// once; libgretl's init is idempotent for this spike's usage.
pub fn init() {
  unsafe { gretl_shim_init() }
}

/// Release libgretl process state. Intended for process shutdown / tests.
pub fn cleanup() {
  unsafe { gretl_shim_cleanup() }
}

/// An owned libgretl dataset.
///
/// The native `DATASET` is created in [`GretlDataset::new`] and released in
/// [`Drop`]. Variable 0 is reserved for the constant term.
pub struct GretlDataset {
  ptr: *mut c_void,
}

// SAFETY: The native DATASET is created and destroyed on the calling thread
// and is not shared. libgretl does not document thread-safety, so we do not
// mark this Send/Sync (see roadmap invariant 1.1).
impl GretlDataset {
  /// Create a dataset with `nvar` variables and `nobs` observations.
  pub fn new(nvar: usize, nobs: usize) -> Self {
    let ptr = unsafe { gretl_shim_dataset_new(nvar as c_int, nobs as c_int) };
    assert!(!ptr.is_null(), "gretl_shim_dataset_new returned NULL");
    Self { ptr }
  }

  fn handle(&self) -> *mut c_void {
    self.ptr
  }

  /// Set the name of variable `i`.
  pub fn set_varname(&mut self, i: usize, name: &str) {
    let c_name = CString::new(name).expect("variable name contains NUL");
    // SAFETY: self.ptr is a valid DATASET handle; c_name is a valid NUL-
    // terminated C string for the duration of the call; i is a valid index.
    unsafe { gretl_shim_dataset_set_varname(self.ptr, i as c_int, c_name.as_ptr()) };
  }

  /// Write observation `obs` of variable `var`.
  pub fn set_value(&mut self, var: usize, obs: usize, value: f64) {
    // SAFETY: self.ptr is a valid DATASET handle; var/obs are valid indices.
    unsafe { gretl_shim_dataset_set_value(self.ptr, var as c_int, obs as c_int, value) };
  }
}

impl Drop for GretlDataset {
  fn drop(&mut self) {
    // SAFETY: self.ptr is a valid DATASET handle created by new(); Drop runs
    // exactly once, so the handle is freed exactly once.
    unsafe { gretl_shim_dataset_free(self.ptr) };
  }
}

/// An owned libgretl estimation result (an OLS `MODEL`).
///
/// The native `MODEL` is released in [`Drop`]. The handle remains valid when
/// estimation failed; inspect [`GretlModel::errcode`].
pub struct GretlModel {
  ptr: *mut c_void,
}

impl GretlModel {
  /// Run OLS on `dset` with the given gretl variable `list`.
  ///
  /// `list[0]` must be the count of the entries that follow.
  pub fn ols(dset: &GretlDataset, list: &[c_int]) -> Self {
    assert!(!list.is_empty(), "variable list is empty");
    // SAFETY: dset.ptr is a valid DATASET handle; list is a valid slice of
    // c_int with list[0] == its logical length.
    let ptr = unsafe { gretl_shim_ols(dset.handle(), list.as_ptr(), list.len() as c_int) };
    assert!(!ptr.is_null(), "gretl_shim_ols returned NULL");
    Self { ptr }
  }

  pub fn ncoeff(&self) -> usize {
    // SAFETY: self.ptr is a valid MODEL handle.
    (unsafe { gretl_shim_model_ncoeff(self.ptr) }) as usize
  }

  pub fn nobs(&self) -> usize {
    // SAFETY: self.ptr is a valid MODEL handle.
    (unsafe { gretl_shim_model_nobs(self.ptr) }) as usize
  }

  pub fn dfn(&self) -> usize {
    // SAFETY: self.ptr is a valid MODEL handle.
    (unsafe { gretl_shim_model_dfn(self.ptr) }) as usize
  }

  pub fn dfd(&self) -> usize {
    // SAFETY: self.ptr is a valid MODEL handle.
    (unsafe { gretl_shim_model_dfd(self.ptr) }) as usize
  }

  /// libgretl error code; 0 means success.
  pub fn errcode(&self) -> i32 {
    // SAFETY: self.ptr is a valid MODEL handle.
    unsafe { gretl_shim_model_errcode(self.ptr) }
  }

  /// Coefficient estimate `i`.
  pub fn coeff(&self, i: usize) -> f64 {
    // SAFETY: self.ptr is a valid MODEL handle; i < ncoeff().
    unsafe { gretl_shim_model_coeff(self.ptr, i as c_int) }
  }

  /// Standard error of coefficient `i`.
  pub fn sderr(&self, i: usize) -> f64 {
    // SAFETY: self.ptr is a valid MODEL handle; i < ncoeff().
    unsafe { gretl_shim_model_sderr(self.ptr, i as c_int) }
  }

  /// (i, j) element of the coefficient covariance matrix.
  pub fn vcv(&self, i: usize, j: usize) -> f64 {
    // SAFETY: self.ptr is a valid MODEL handle; i, j < ncoeff().
    unsafe { gretl_shim_model_vcv(self.ptr, i as c_int, j as c_int) }
  }

  /// A scalar fit statistic.
  pub fn scalar(&self, stat: Stat) -> f64 {
    // SAFETY: self.ptr is a valid MODEL handle; stat is a valid selector.
    unsafe { gretl_shim_model_scalar(self.ptr, stat.code()) }
  }
}

impl Drop for GretlModel {
  fn drop(&mut self) {
    // SAFETY: self.ptr is a valid MODEL handle created by ols(); Drop runs
    // exactly once, so the handle is freed exactly once.
    unsafe { gretl_shim_model_free(self.ptr) };
  }
}
