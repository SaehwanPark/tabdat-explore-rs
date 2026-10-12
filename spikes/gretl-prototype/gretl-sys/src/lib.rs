//! Private-pointer, RAII boundary over the pinned libgretl C ownership shim.
//! Native calls (including teardown) are serialized because libgretl uses
//! process-global state. Initialization is lazy and retained for process life;
//! there is intentionally no safe public cleanup operation to invalidate handles.

#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::CString;
use std::os::raw::{c_char, c_double, c_int, c_void};
use std::sync::{Mutex, Once};

static NATIVE: Mutex<()> = Mutex::new(());
static INIT: Once = Once::new();

fn native<T>(call: impl FnOnce() -> T) -> T {
  // Recover poisoning: panics in this wrapper occur only before/after completed
  // C calls, never by unwinding through C, and owned handles remain valid.
  let _guard = NATIVE.lock().unwrap_or_else(|error| error.into_inner());
  INIT.call_once(|| {
    // SAFETY: the process-global mutex excludes every other shim call; this
    // initialization executes once and no API can clean up live native state.
    unsafe { gretl_shim_init() }
  });
  call()
}

/// Eagerly initialize through the same serialized, once-only boundary for the
/// feasibility measurement executable. Ordinary constructors remain lazy and
/// never require callers to invoke this. There is still no cleanup API.
pub fn initialize_for_measurement() {
  native(|| ());
}

unsafe extern "C" {
  fn gretl_shim_init();
  fn gretl_shim_dataset_new(nvar: c_int, nobs: c_int) -> *mut c_void;
  fn gretl_shim_dataset_set_varname(handle: *mut c_void, i: c_int, name: *const c_char);
  fn gretl_shim_dataset_set_value(handle: *mut c_void, var: c_int, obs: c_int, value: c_double);
  fn gretl_shim_dataset_free(handle: *mut c_void);
  fn gretl_shim_ols(
    dset: *mut c_void,
    list: *const c_int,
    nlist: c_int,
    covariance: c_int,
    cluster_var: c_int,
  ) -> *mut c_void;
  fn gretl_shim_binary(
    dset: *mut c_void,
    list: *const c_int,
    nlist: c_int,
    probit: c_int,
  ) -> *mut c_void;
  #[cfg(feature = "count-plugin")]
  fn gretl_shim_count(
    dset: *mut c_void,
    list: *const c_int,
    nlist: c_int,
    nb2: c_int,
  ) -> *mut c_void;
  fn gretl_shim_iv(dset: *mut c_void, list: *const c_int, nlist: c_int) -> *mut c_void;
  fn gretl_shim_panel(
    dset: *mut c_void,
    list: *const c_int,
    nlist: c_int,
    periods: c_int,
    random: c_int,
  ) -> *mut c_void;
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

/// Owned dataset. Variable zero is Gretl's immutable implicit constant.
/// Raw-pointer fields deliberately keep native handles neither Send nor Sync.
/// Invalid prototype arguments panic before entering C.
pub struct GretlDataset {
  ptr: *mut c_void,
  nvar: usize,
  nobs: usize,
}

impl GretlDataset {
  pub fn new(nvar: usize, nobs: usize) -> Self {
    assert!(nvar > 0 && nobs > 0, "dataset dimensions must be positive");
    let vars = c_int::try_from(nvar).expect("too many variables");
    let obs = c_int::try_from(nobs).expect("too many observations");
    let ptr = native(|| {
      // SAFETY: dimensions are positive and representable in C; process state
      // is initialized and protected by native(). The returned allocation is owned.
      unsafe { gretl_shim_dataset_new(vars, obs) }
    });
    assert!(!ptr.is_null(), "gretl_shim_dataset_new returned NULL");
    Self { ptr, nvar, nobs }
  }

  /// Native names are truncated by the shim to VNAMELEN minus one bytes.
  pub fn set_varname(&mut self, i: usize, name: &str) {
    assert!(i < self.nvar, "variable index out of bounds");
    let name = CString::new(name).expect("variable name contains NUL");
    native(|| {
      // SAFETY: live owned dataset, checked index, and a NUL-terminated string
      // borrowed only for this serialized call; C copies the name.
      unsafe { gretl_shim_dataset_set_varname(self.ptr, i as c_int, name.as_ptr()) }
    });
  }

  pub fn set_value(&mut self, var: usize, obs: usize, value: f64) {
    assert!(
      var > 0 && var < self.nvar,
      "variable index is reserved or out of bounds"
    );
    assert!(obs < self.nobs, "observation index out of bounds");
    native(|| {
      // SAFETY: live owned dataset and checked indices within C-representable
      // dimensions; no concurrent native call can access process-global state.
      unsafe { gretl_shim_dataset_set_value(self.ptr, var as c_int, obs as c_int, value) }
    });
  }
}

impl Drop for GretlDataset {
  fn drop(&mut self) {
    native(|| {
      // SAFETY: this non-null allocation is owned exclusively and freed once;
      // process state stays initialized and teardown is serialized.
      unsafe { gretl_shim_dataset_free(self.ptr) }
    });
  }
}

#[derive(Clone, Copy)]
pub enum BinaryLink {
  Logit,
  Probit,
}

/// Closed native covariance request. Cluster indices are checked against the dataset.
#[derive(Clone, Copy)]
pub enum OlsCovariance {
  Classical,
  Hc1,
  Clustered { variable: usize },
}

/// Owned model, including native error results. The shim copies/owns model
/// buffers; no dataset borrow or raw native value escapes this wrapper.
pub struct GretlModel {
  ptr: *mut c_void,
}

impl GretlModel {
  /// `list[0]` counts remaining entries; the outcome must not be variable zero.
  pub fn ols(dset: &GretlDataset, list: &[c_int]) -> Self {
    Self::ols_with_covariance(dset, list, OlsCovariance::Classical)
  }

  pub fn ols_with_covariance(
    dset: &GretlDataset,
    list: &[c_int],
    covariance: OlsCovariance,
  ) -> Self {
    let (mode, cluster) = match covariance {
      OlsCovariance::Classical => (0, 0),
      OlsCovariance::Hc1 => (1, 0),
      OlsCovariance::Clustered { variable } => {
        assert!(
          variable > 0 && variable < dset.nvar,
          "cluster variable out of bounds"
        );
        (2, variable as c_int)
      }
    };
    let len = Self::checked_list_len(dset, list);
    let ptr = native(|| {
      // SAFETY: dataset is live; every list index and the exact count have been
      // checked. The slice lives through this call. C returns an owned model;
      // covariance settings are configured/restored within this single locked call.
      unsafe { gretl_shim_ols(dset.ptr, list.as_ptr(), len, mode, cluster) }
    });
    assert!(!ptr.is_null(), "gretl_shim_ols returned NULL");
    Self { ptr }
  }

  /// Classical binary MLE with a closed logit/probit selector.
  pub fn binary(dset: &GretlDataset, list: &[c_int], link: BinaryLink) -> Self {
    let len = Self::checked_list_len(dset, list);
    let probit = i32::from(matches!(link, BinaryLink::Probit));
    let ptr = native(|| {
      // SAFETY: live dataset, exact list count and checked indices. The closed
      // selector and borrowed slice are valid through this serialized call.
      // The shim returns an owned model, including native estimation errors.
      unsafe { gretl_shim_binary(dset.ptr, list.as_ptr(), len, probit) }
    });
    assert!(!ptr.is_null(), "gretl_shim_binary returned NULL");
    Self { ptr }
  }

  /// Classical Poisson through the pinned plugin, available only with the
  /// shared-core count capability. Plugin lifetime follows process initialization.
  #[cfg(feature = "count-plugin")]
  pub fn poisson(dset: &GretlDataset, list: &[c_int]) -> Self {
    Self::count(dset, list, false)
  }

  #[cfg(feature = "count-plugin")]
  pub fn negative_binomial(dset: &GretlDataset, list: &[c_int]) -> Self {
    Self::count(dset, list, true)
  }

  #[cfg(feature = "count-plugin")]
  fn count(dset: &GretlDataset, list: &[c_int], nb2: bool) -> Self {
    let len = Self::checked_list_len(dset, list);
    let ptr = native(|| {
      // SAFETY: live dataset, exact checked list/count, slice borrowed through
      // the call. Initialization set the bounded build-owned plugin path once;
      // all plugin dispatch/global state and teardown share the native mutex.
      // The shim returns an owned model even when estimation fails.
      unsafe { gretl_shim_count(dset.ptr, list.as_ptr(), len, i32::from(nb2)) }
    });
    assert!(!ptr.is_null(), "gretl_shim_count returned NULL");
    Self { ptr }
  }

  /// Classical unadjusted 2SLS, with one separator between structural and
  /// instrument lists. Invalid lists panic before native dispatch.
  pub fn iv(dset: &GretlDataset, list: &[c_int]) -> Self {
    let len = c_int::try_from(list.len()).expect("IV list too long");
    assert!(
      list.len() >= 5 && list[0] == len - 1,
      "invalid IV list count"
    );
    let separators: Vec<_> = list
      .iter()
      .enumerate()
      .filter(|(_, v)| **v == -100)
      .map(|(i, _)| i)
      .collect();
    assert_eq!(separators.len(), 1, "IV list needs exactly one separator");
    let separator = separators[0];
    assert!(
      separator >= 3 && separator < list.len() - 1,
      "empty IV list side"
    );
    assert!(list[1] > 0, "outcome cannot be reserved constant");
    assert!(
      list[1..]
        .iter()
        .all(|&i| i == -100 || (i >= 0 && (i as usize) < dset.nvar)),
      "IV variable index out of bounds"
    );
    let ptr = native(|| {
      // SAFETY: live owned dataset, one checked separator, exact count and all
      // variable indices checked. Native tsls owns its copied model/list buffers;
      // the slice is borrowed only through this serialized call.
      unsafe { gretl_shim_iv(dset.ptr, list.as_ptr(), len) }
    });
    assert!(!ptr.is_null(), "gretl_shim_iv returned NULL");
    Self { ptr }
  }

  pub fn panel(dset: &GretlDataset, list: &[c_int], periods: usize, random: bool) -> Self {
    let len = Self::checked_list_len(dset, list);
    assert!(
      list.len() >= 4,
      "panel requires a predictor beyond the constant"
    );
    assert!(list[2..].contains(&0), "panel baseline requires constant");
    assert!(
      periods >= 2 && dset.nobs.is_multiple_of(periods) && dset.nobs / periods >= 2,
      "balanced panel dimensions required"
    );
    let periods = c_int::try_from(periods).expect("panel period exceeds native limits");
    let ptr = native(|| {
      // SAFETY: live owned dataset, checked list/indices and balanced dimensions.
      // Metadata changes and restoration occur within this locked shim call;
      // the returned copied native model has the existing RAII ownership.
      unsafe { gretl_shim_panel(dset.ptr, list.as_ptr(), len, periods, i32::from(random)) }
    });
    assert!(!ptr.is_null(), "gretl_shim_panel returned NULL");
    Self { ptr }
  }

  fn checked_list_len(dset: &GretlDataset, list: &[c_int]) -> c_int {
    let len = c_int::try_from(list.len()).expect("variable list too long");
    assert!(list.len() >= 3, "need an outcome and regressor");
    assert_eq!(
      list[0],
      len - 1,
      "variable count does not match list length"
    );
    assert!(list[1] > 0, "outcome cannot be the reserved constant");
    assert!(
      list[1..]
        .iter()
        .all(|&i| i >= 0 && (i as usize) < dset.nvar),
      "variable list index out of bounds"
    );
    len
  }

  pub fn ncoeff(&self) -> usize {
    native(|| {
      // SAFETY: self owns a live model; accessor reads a scalar under the lock.
      usize::try_from(unsafe { gretl_shim_model_ncoeff(self.ptr) })
        .expect("negative coefficient count")
    })
  }

  pub fn nobs(&self) -> usize {
    native(|| {
      // SAFETY: self owns a live model; accessor reads a scalar under the lock.
      usize::try_from(unsafe { gretl_shim_model_nobs(self.ptr) })
        .expect("negative observation count")
    })
  }

  pub fn dfn(&self) -> usize {
    native(|| {
      // SAFETY: self owns a live model; accessor reads a scalar under the lock.
      usize::try_from(unsafe { gretl_shim_model_dfn(self.ptr) })
        .expect("negative model degrees of freedom")
    })
  }

  pub fn dfd(&self) -> usize {
    native(|| {
      // SAFETY: self owns a live model; accessor reads a scalar under the lock.
      usize::try_from(unsafe { gretl_shim_model_dfd(self.ptr) })
        .expect("negative residual degrees of freedom")
    })
  }

  pub fn errcode(&self) -> i32 {
    native(|| {
      // SAFETY: error models are still allocated and scalar access is valid.
      unsafe { gretl_shim_model_errcode(self.ptr) }
    })
  }

  fn check_index(&self, i: usize) {
    assert_eq!(self.errcode(), 0, "cannot access buffers on a failed model");
    assert!(i < self.ncoeff(), "coefficient index out of bounds");
  }

  pub fn coeff(&self, i: usize) -> f64 {
    self.check_index(i);
    native(|| {
      // SAFETY: successful live model with checked coefficient index. Model
      // ownership prevents mutation between checking and this serialized read.
      unsafe { gretl_shim_model_coeff(self.ptr, i as c_int) }
    })
  }

  pub fn sderr(&self, i: usize) -> f64 {
    self.check_index(i);
    native(|| {
      // SAFETY: successful live model and checked standard-error index.
      unsafe { gretl_shim_model_sderr(self.ptr, i as c_int) }
    })
  }

  pub fn vcv(&self, i: usize, j: usize) -> f64 {
    self.check_index(i);
    self.check_index(j);
    native(|| {
      // SAFETY: successful live model and checked covariance indices. The
      // accessor may populate native VCV storage, so it also holds the lock.
      unsafe { gretl_shim_model_vcv(self.ptr, i as c_int, j as c_int) }
    })
  }

  pub fn scalar(&self, stat: Stat) -> f64 {
    native(|| {
      // SAFETY: live model and a closed-enum scalar selector, serialized read.
      unsafe { gretl_shim_model_scalar(self.ptr, stat as c_int) }
    })
  }
}

impl Drop for GretlModel {
  fn drop(&mut self) {
    native(|| {
      // SAFETY: exclusively owned model is freed exactly once under the lock.
      unsafe { gretl_shim_model_free(self.ptr) }
    });
  }
}
