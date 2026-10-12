use super::*;

unsafe extern "C" {
  fn tabdat_readstat_exercise_values(
    handler: extern "C" fn(*const readstat_value_t, *mut c_void) -> c_int,
    ctx: *mut c_void,
  ) -> c_int;
}

extern "C" fn collect(value: *const readstat_value_t, ctx: *mut c_void) -> c_int {
  let outcome = catch_unwind(AssertUnwindSafe(|| {
    // SAFETY: exercise_values calls synchronously with this Vec's unique
    // context and a native value live until this callback returns.
    let values = unsafe { &mut *ctx.cast::<Vec<DtaValue>>() };
    values.push(extract_value(value, std::ptr::null()));
  }));
  if outcome.is_ok() {
    READSTAT_HANDLER_OK
  } else {
    READSTAT_HANDLER_ABORT
  }
}

#[test]
fn all_scalar_types_and_missing_flags_copy_from_native_values() {
  let mut values = Vec::<DtaValue>::new();
  // SAFETY: the C driver retains neither callback nor context; collect copies
  // each value before the driver mutates/frees its local string and stack.
  let code =
    unsafe { tabdat_readstat_exercise_values(collect, (&mut values as *mut Vec<DtaValue>).cast()) };
  assert_eq!(code, READSTAT_HANDLER_OK);
  assert_eq!(
    values,
    [
      DtaValue::Int8(-7),
      DtaValue::Int16(-320),
      DtaValue::Int32(-70000),
      DtaValue::Float(1.25),
      DtaValue::Double(-2.5),
      DtaValue::String("owned".into()),
      DtaValue::SystemMissing,
      DtaValue::TaggedMissing('z'),
    ]
  );
}

extern "C" fn abort_first(_value: *const readstat_value_t, ctx: *mut c_void) -> c_int {
  // SAFETY: the synchronous C driver receives the unique counter below.
  unsafe {
    *ctx.cast::<usize>() += 1;
  }
  READSTAT_HANDLER_ABORT
}

#[test]
fn native_value_driver_propagates_abort_without_more_callbacks() {
  let mut count = 0_usize;
  // SAFETY: count outlives the synchronous driver; no pointers are retained.
  let code =
    unsafe { tabdat_readstat_exercise_values(abort_first, (&mut count as *mut usize).cast()) };
  assert_eq!(code, READSTAT_HANDLER_ABORT);
  assert_eq!(count, 1);
}

extern "C" fn abort_metadata(_meta: *mut readstat_metadata_t, _ctx: *mut c_void) -> c_int {
  READSTAT_HANDLER_ABORT
}

#[test]
fn actual_native_abort_and_repeated_parse_recover() {
  let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/labeled.dta");
  for _ in 0..20 {
    let error = parse_dta_with_metadata_handler(&path, abort_metadata).unwrap_err();
    // READSTAT_ERROR_USER_ABORT in the pinned readstat_error_e.
    assert!(matches!(error, DtaError::Native { code: 4, .. }));
    let data = parse_dta(&path).expect("parse recovers after native abort");
    assert_eq!(data.rows.len(), 4);
    assert_eq!(data.rows[0][4], DtaValue::Double(88.25));
  }
}

fn empty_state() -> ParserState {
  ParserState {
    metadata: None,
    variables: vec![],
    rows: vec![],
    label_sets: vec![],
    error: None,
    boundary_error: None,
  }
}

fn metadata(rows: i64, columns: i64) -> DtaMetadata {
  DtaMetadata {
    row_count: rows,
    var_count: columns,
    file_label: None,
    file_format_version: 118,
    is_64bit: true,
  }
}

#[test]
fn dimensions_and_callback_indices_reject_before_row_growth() {
  for (rows, columns) in [
    (-1, 1),
    (1, -1),
    (1, 0),
    (1, 32768),
    (i64::MAX, 32767),
    (10_000_001, 1),
  ] {
    let mut state = empty_state();
    assert!(state.set_metadata(metadata(rows, columns)).is_err());
    assert!(state.metadata.is_none());
    assert!(state.rows.is_empty());
  }
  let mut state = empty_state();
  assert!(state.check_variable(0).is_err());
  assert!(state.store_cell(0, 0, DtaValue::Int8(1)).is_err());
  state.set_metadata(metadata(2, 1)).unwrap();
  assert!(state.set_metadata(metadata(2, 1)).is_err());
  assert!(state.check_variable(-1).is_err());
  assert!(state.check_variable(1).is_err());
  assert!(state.store_cell(0, 0, DtaValue::Int8(1)).is_err());
  state.check_variable(0).unwrap();
  state.variables.push(DtaVariable {
    name: "x".into(),
    label: None,
    type_: DtaType::Int8,
    label_set: None,
  });
  assert!(state.check_variable(0).is_err());
  for (row, column) in [(-1, 0), (i32::MAX, 0), (2, 0), (0, -1), (0, 1)] {
    assert!(state.store_cell(row, column, DtaValue::Int8(1)).is_err());
    assert!(state.rows.is_empty());
  }
  state.store_cell(1, 0, DtaValue::Int8(7)).unwrap();
  assert_eq!(
    state.rows,
    [vec![DtaValue::SystemMissing], vec![DtaValue::Int8(7)]]
  );
}

#[cfg(unix)]
#[test]
fn non_utf8_path_is_rejected_without_lossy_file_substitution() {
  use std::os::unix::ffi::OsStrExt;
  let path = Path::new(std::ffi::OsStr::from_bytes(b"wrong-\xff.dta"));
  assert!(matches!(parse_dta(path), Err(DtaError::Path(_))));
}
