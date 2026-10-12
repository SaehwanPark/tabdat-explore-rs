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
  fn gretl_shim_dataset_nvar(handle: *const c_void) -> c_int;
  #[cfg(feature = "dpanel-plugin")]
  fn gretl_shim_dataset_panel_shape(handle: *const c_void, pd: *mut c_int, structure: *mut c_int);
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
  #[cfg(feature = "tobit-plugin")]
  fn gretl_shim_tobit(
    dset: *mut c_void,
    list: *const c_int,
    nlist: c_int,
    lower: c_double,
    upper: c_double,
    has_upper: c_int,
  ) -> *mut c_void;
  #[cfg(feature = "tobit-plugin")]
  fn gretl_shim_tobit_vcv(handle: *mut c_void, i: c_int, j: c_int) -> c_double;
  #[cfg(feature = "quantile-plugin")]
  fn gretl_shim_quantile(
    dset: *mut c_void,
    list: *const c_int,
    nlist: c_int,
    q: c_double,
  ) -> *mut c_void;
  #[cfg(feature = "quantile-plugin")]
  fn gretl_shim_normal_quantile(probability: c_double) -> c_double;
  #[cfg(feature = "survival-plugin")]
  fn gretl_shim_survival(
    dset: *mut c_void,
    list: *const c_int,
    nlist: c_int,
    exponential: c_int,
  ) -> *mut c_void;
  #[cfg(feature = "dpanel-plugin")]
  fn gretl_shim_dynamic_panel(
    dset: *mut c_void,
    list: *const c_int,
    nlist: c_int,
    periods: c_int,
  ) -> *mut c_void;
  #[cfg(feature = "dpanel-plugin")]
  fn gretl_shim_dpanel_instruments(handle: *const c_void) -> c_int;
  #[cfg(feature = "dpanel-plugin")]
  fn gretl_shim_model_sample_mask(handle: *const c_void, n: c_int, out: *mut u8) -> c_int;
  #[cfg(feature = "heckman-plugin")]
  fn gretl_shim_heckman(dset: *mut c_void, list: *const c_int, nlist: c_int) -> *mut c_void;
  #[cfg(feature = "heckman-plugin")]
  fn gretl_shim_selection_nobs(handle: *const c_void) -> c_int;
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

#[cfg(feature = "quantile-plugin")]
pub fn normal_quantile(probability: f64) -> f64 {
  assert!(
    probability.is_finite() && probability > 0.0 && probability < 1.0,
    "normal probability must be strictly between zero and one"
  );
  native(|| {
    // SAFETY: finite strictly interior probability, scalar-only native function;
    // no ownership transfer, and existing process serialization still applies.
    unsafe { gretl_shim_normal_quantile(probability) }
  })
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
  Rho = 7,
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
  /// Owned scalar metadata for restoration probes, confined to the low-level API.
  #[cfg(feature = "dpanel-plugin")]
  pub fn panel_shape(&self) -> (i32, i32) {
    native(|| {
      let mut pd = 0;
      let mut structure = 0;
      // SAFETY: live owned dataset, two valid scalar out pointers; serialized
      // read copies metadata only, without borrowing native state.
      unsafe { gretl_shim_dataset_panel_shape(self.ptr, &mut pd, &mut structure) };
      (pd, structure)
    })
  }
  /// Native shape observation for invariant probes; no foreign data escapes.
  pub fn variable_count(&self) -> usize {
    native(|| {
      // SAFETY: self owns a live dataset; scalar native shape read is serialized.
      usize::try_from(unsafe { gretl_shim_dataset_nvar(self.ptr) })
        .expect("negative dataset variable count")
    })
  }

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

  #[cfg(feature = "tobit-plugin")]
  pub fn tobit(dset: &GretlDataset, list: &[c_int], lower: f64, upper: Option<f64>) -> Self {
    let len = Self::checked_list_len(dset, list);
    assert!(
      lower.is_finite() && upper.is_none_or(|u| u.is_finite() && u > lower),
      "invalid Tobit limits"
    );
    let ptr = native(|| {
      // SAFETY: live owned dataset, checked borrowed list/indices and finite
      // ordered bounds. The shim owns the by-value model and all its buffers;
      // source-matched plugin loading and estimation share the global lock.
      unsafe {
        gretl_shim_tobit(
          dset.ptr,
          list.as_ptr(),
          len,
          lower,
          upper.unwrap_or(0.0),
          i32::from(upper.is_some()),
        )
      }
    });
    assert!(!ptr.is_null(), "gretl_shim_tobit returned NULL");
    Self { ptr }
  }

  #[cfg(feature = "tobit-plugin")]
  pub fn tobit_vcv(&self, i: usize, j: usize) -> f64 {
    assert_eq!(self.errcode(), 0, "cannot access failed Tobit model");
    let k = self
      .ncoeff()
      .checked_add(1)
      .expect("Tobit dimension overflow");
    assert!(i < k && j < k, "Tobit covariance index out of bounds");
    let i = c_int::try_from(i).expect("Tobit index exceeds native limits");
    let j = c_int::try_from(j).expect("Tobit index exceeds native limits");
    native(|| {
      // SAFETY: live successful model and checked indices. The shim additionally
      // validates the borrowed full_vcv matrix shape before reading one scalar;
      // no foreign matrix storage escapes or outlives its RAII model owner.
      unsafe { gretl_shim_tobit_vcv(self.ptr, i, j) }
    })
  }

  #[cfg(feature = "quantile-plugin")]
  pub fn quantile(dset: &GretlDataset, list: &[c_int], q: f64) -> Self {
    let len = Self::checked_list_len(dset, list);
    assert!(
      q.is_finite() && (0.01..=0.99).contains(&q),
      "native quantile must be .01 through .99"
    );
    let ptr = native(|| {
      // SAFETY: live owned dataset, checked list/indices and native q range.
      // The shim allocates/frees the one-element tau matrix within this locked
      // call; the model stores a scalar or owned copy, never a borrowed tau.
      unsafe { gretl_shim_quantile(dset.ptr, list.as_ptr(), len, q) }
    });
    assert!(!ptr.is_null(), "gretl_shim_quantile returned NULL");
    Self { ptr }
  }

  #[cfg(feature = "survival-plugin")]
  pub fn survival(dset: &GretlDataset, list: &[c_int], exponential: bool) -> Self {
    let len = c_int::try_from(list.len()).expect("duration list exceeds native limits");
    assert!(
      list.len() >= 3 && list[0] == len - 1,
      "invalid duration list count/length"
    );
    assert!(list[1] > 0, "duration outcome cannot be constant");
    let separators: Vec<_> = list
      .iter()
      .enumerate()
      .filter(|(_, v)| **v == -100)
      .map(|(i, _)| i)
      .collect();
    assert!(
      separators.is_empty()
        || (separators.len() == 1 && separators[0] == list.len() - 2 && list[list.len() - 1] > 0),
      "censor separator must be second last with nonconstant dummy"
    );
    let end = separators.first().copied().unwrap_or(list.len());
    assert!(
      list[2..end].contains(&0),
      "native duration requires constant"
    );
    assert!(
      list[1..]
        .iter()
        .all(|&i| i == -100 || (i >= 0 && (i as usize) < dset.nvar)),
      "duration index out of bounds"
    );
    let ptr = native(|| {
      // SAFETY: live owned dataset, exact count, validated separator/constant
      // and all indices checked. Native validates the dummy/times and owns its
      // copied MODEL buffers. Loader/fit/destructors share this process lock.
      unsafe { gretl_shim_survival(dset.ptr, list.as_ptr(), len, i32::from(exponential)) }
    });
    assert!(!ptr.is_null(), "gretl_shim_survival returned NULL");
    Self { ptr }
  }

  #[cfg(feature = "dpanel-plugin")]
  pub fn dynamic_panel(dset: &GretlDataset, list: &[c_int], periods: usize) -> Self {
    assert!(
      (4..=32).contains(&periods) && dset.nobs.is_multiple_of(periods),
      "invalid balanced dynamic panel periods"
    );
    assert!(
      (8..=4096).contains(&(dset.nobs / periods)) && dset.nvar <= 32,
      "prototype dynamic panel dimension cap"
    );
    assert!(
      (7..=37).contains(&list.len()) && (list.len() - 5).is_multiple_of(2),
      "dynamic panel parameter cap/list length"
    );
    let len = c_int::try_from(list.len()).expect("dynamic list exceeds native limits");
    assert!(
      list[0] == len - 1 && list[1] == 1 && list[2] == -100,
      "dynamic list needs exactly one lag then separator"
    );
    let p = (list.len() - 5) / 2;
    assert!(
      list[4 + p] == -100 && list[5 + p..] == list[4..4 + p],
      "dynamic exogenous instrument roles must match"
    );
    assert!(
      list[3..4 + p]
        .iter()
        .all(|&v| v > 0 && (v as usize) < dset.nvar),
      "dynamic variable index/constant invalid"
    );
    let ptr = native(|| {
      // SAFETY: live bounded dataset, checked balanced metadata and closed one-lag
      // list/spec with checked indices. Shim restores metadata and owns MODEL;
      // fit/loader/cleanup share serialization and no input slice is mutated.
      unsafe { gretl_shim_dynamic_panel(dset.ptr, list.as_ptr(), len, periods as c_int) }
    });
    assert!(!ptr.is_null(), "gretl_shim_dynamic_panel returned NULL");
    Self { ptr }
  }
  #[cfg(feature = "dpanel-plugin")]
  pub fn dynamic_panel_instruments(&self) -> usize {
    assert_eq!(
      self.errcode(),
      0,
      "cannot read failed dynamic model metadata"
    );
    native(|| {
      // SAFETY: live successful owned MODEL; fixed integer metadata key read is
      // serialized, with no borrowed metadata storage exposed.
      usize::try_from(unsafe { gretl_shim_dpanel_instruments(self.ptr) })
        .expect("negative instruments")
    })
  }
  #[cfg(feature = "dpanel-plugin")]
  pub fn sample_mask(&self, n: usize) -> Vec<bool> {
    assert_eq!(self.errcode(), 0, "cannot access failed model sample");
    let len = c_int::try_from(n).expect("sample mask exceeds native limits");
    let mut mask = vec![0u8; n];
    let ok = native(|| {
      // SAFETY: owned caller buffer has exactly n bytes. C validates successful
      // MODEL, full_n and residual allocation before bounded reads/writes;
      // serialization covers the copy and no foreign ownership escapes.
      unsafe { gretl_shim_model_sample_mask(self.ptr, len, mask.as_mut_ptr()) }
    });
    assert_eq!(ok, 1, "native sample shape/residual buffer unavailable");
    mask.into_iter().map(|v| v != 0).collect()
  }

  #[cfg(feature = "heckman-plugin")]
  pub fn heckman(dset: &GretlDataset, list: &[c_int]) -> Self {
    let len = c_int::try_from(list.len()).expect("selection list exceeds native limits");
    assert!(
      list.len() >= 6 && list[0] == len - 1,
      "invalid selection list count/length"
    );
    let separators: Vec<_> = list
      .iter()
      .enumerate()
      .filter(|(_, v)| **v == -100)
      .map(|(i, _)| i)
      .collect();
    assert!(
      separators.len() == 1,
      "selection needs exactly one separator"
    );
    // heckit.c computes (tot*tot+tot)/2 in signed int before allocating packed
    // VCV. Bound the nominal combined parameters even if a fit could drop rank.
    let total = list.len() - 3;
    assert!(
      total
        .checked_mul(total + 1)
        .is_some_and(|cells| c_int::try_from(cells).is_ok()),
      "two-step packed covariance exceeds native integer arithmetic"
    );
    let sep = separators[0];
    assert!(
      sep >= 3 && sep + 3 < list.len(),
      "selection equation sides must be nonempty with predictors"
    );
    assert!(
      list[1] > 0 && list[sep + 1] > 0,
      "equation outcomes cannot be constant"
    );
    assert!(
      list[2..sep].contains(&0) && list[sep + 2..].contains(&0),
      "two-step spike requires both intercepts"
    );
    assert!(
      list[1..]
        .iter()
        .all(|&i| i == -100 || (i >= 0 && (i as usize) < dset.nvar)),
      "selection index out of bounds"
    );
    let ptr = native(|| {
      // SAFETY: live dataset, exact count, two validated equation sides with
      // checked indices/one separator. Source-matched OPT_T owns copied MODEL
      // buffers and cleans temporary Mills series within this serialized call.
      unsafe { gretl_shim_heckman(dset.ptr, list.as_ptr(), len) }
    });
    assert!(!ptr.is_null(), "gretl_shim_heckman returned NULL");
    Self { ptr }
  }

  #[cfg(feature = "heckman-plugin")]
  pub fn selection_nobs(&self) -> usize {
    assert_eq!(
      self.errcode(),
      0,
      "cannot access failed selection model metadata"
    );
    native(|| {
      // SAFETY: live successful MODEL owns its integer totobs metadata; scalar
      // read is serialized and no data-item pointer escapes.
      usize::try_from(unsafe { gretl_shim_selection_nobs(self.ptr) })
        .expect("negative selection sample count")
    })
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
