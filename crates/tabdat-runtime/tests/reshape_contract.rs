use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use duckdb::Connection;
use tabdat_language::parse_command;
use tabdat_runtime::{
  CellValue, CountResult, ExecutionResult, PreviewResult, ReshapeResult, RuntimeError, Session,
};

struct Fixture {
  root: PathBuf,
}

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

impl Fixture {
  fn new() -> Self {
    loop {
      // A clock is not a uniqueness source. Relaxed ordering grants distinct
      // candidates; exclusive mkdir, not the counter, establishes ownership.
      let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
      let root =
        std::env::temp_dir().join(format!("tabdat_reshape_test_{}_{}", std::process::id(), id));
      match Self::claim(root) {
        Ok(fixture) => return fixture,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
        Err(error) => panic!("fixture directory claim failed: {error}"),
      }
    }
  }

  fn claim(root: PathBuf) -> std::io::Result<Self> {
    // Never adopt an existing root: Drop is permitted to delete only our claim.
    fs::create_dir(&root)?;
    Ok(Self { root })
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

#[test]
fn existing_fixture_root_cannot_be_adopted_or_deleted_by_another_claim() {
  let owner = Fixture::new();
  let marker = owner.root.join("owner-marker");
  fs::write(&marker, "retained").unwrap();
  let result = Fixture::claim(owner.root.clone());
  assert!(matches!(result, Err(ref error) if error.kind() == std::io::ErrorKind::AlreadyExists));
  drop(result);
  let missing_parent = owner.root.join("uncreated-parent").join("fixture");
  assert!(
    matches!(Fixture::claim(missing_parent), Err(ref error) if error.kind() == std::io::ErrorKind::NotFound)
  );
  assert_eq!(fs::read_to_string(&marker).unwrap(), "retained");
}

#[test]
fn rejected_fixture_claim_preserves_parquet_and_runtime_reshape() {
  let owner = Fixture::new();
  let path = owner.root.join("owned.parquet");
  let path_string = path.to_string_lossy().into_owned();
  let connection = Connection::open_in_memory().unwrap();
  connection
    .execute(
      "COPY (SELECT 1 AS id, 10 AS income_2020, 11 AS income_2021) TO ? (FORMAT PARQUET)",
      [&path_string],
    )
    .unwrap();
  let rejected = Fixture::claim(owner.root.clone());
  assert!(matches!(rejected, Err(ref error) if error.kind() == std::io::ErrorKind::AlreadyExists));
  drop(rejected);
  assert!(path.is_file());
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_string}")).unwrap())
    .unwrap();
  let result = session
    .execute(parse_command("reshape long income, i(id) j(year)").unwrap())
    .unwrap();
  assert!(matches!(result, ExecutionResult::Reshape(_)));
  assert!(path.is_file());
}

#[test]
#[ignore = "manual live-clock causal probe; a timestamp collision is not a portable test requirement"]
fn former_live_clock_claims_reproduce_missing_parquet() {
  use std::sync::{Arc, Barrier};
  use std::time::{SystemTime, UNIX_EPOCH};

  // Deliberately reproduce the former helper only BELOW an exclusively owned
  // outer directory. Even its invalid ownership can never adopt external files.
  struct FormerFixture {
    root: PathBuf,
  }
  impl FormerFixture {
    fn new(parent: &std::path::Path) -> std::io::Result<Self> {
      let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(std::io::Error::other)?
        .as_nanos();
      let root = parent.join(format!(
        "tabdat_reshape_test_{}_{}",
        std::process::id(),
        stamp
      ));
      fs::create_dir_all(&root)?;
      Ok(Self { root })
    }
  }
  impl Drop for FormerFixture {
    fn drop(&mut self) {
      let _ = fs::remove_dir_all(&self.root);
    }
  }
  let outer = Fixture::new();
  let barrier = Arc::new(Barrier::new(32));
  let mut fixtures = std::thread::scope(|scope| {
    let workers: Vec<_> = (0..32)
      .map(|_| {
        let barrier = barrier.clone();
        let parent = &outer.root;
        scope.spawn(move || {
          let mut claims = Vec::new();
          for _ in 0..100 {
            barrier.wait();
            claims.push(FormerFixture::new(parent));
          }
          claims
        })
      })
      .collect();
    // Finish all barrier rounds before surfacing an I/O error, so one failed
    // claim cannot strand the remaining workers at their next barrier.
    workers
      .into_iter()
      .flat_map(|worker| worker.join().unwrap())
      .collect::<std::io::Result<Vec<_>>>()
      .expect("former helper claims")
  });
  let mut seen = std::collections::HashMap::new();
  let mut pair = None;
  let mut duplicates = 0;
  for (index, fixture) in fixtures.iter().enumerate() {
    if let Some(first) = seen.insert(fixture.root.clone(), index) {
      duplicates += 1;
      pair = Some((first, index));
    }
  }
  eprintln!(
    "former helper: {} live claims, {} shared-root claims",
    fixtures.len(),
    duplicates
  );
  let (first, second) =
    pair.expect("no live clock collision observed; this diagnostic is not a portable gate");
  let adopter = fixtures.swap_remove(second);
  let retained = fixtures.swap_remove(first);
  assert_eq!(retained.root, adopter.root);
  let path = retained.root.join("reshape_wide_collision.parquet");
  let path_string = path.to_string_lossy().into_owned();
  Connection::open_in_memory()
    .unwrap()
    .execute(
      "COPY (SELECT 1 AS id, 'order' AS year, 10 AS income) TO ? (FORMAT PARQUET)",
      [&path_string],
    )
    .unwrap();
  assert!(path.is_file());
  drop(adopter);
  assert!(
    !path.exists(),
    "one adopter deleted another live guard's Parquet"
  );
  let error = Connection::open_in_memory()
    .unwrap()
    .query_row(
      "SELECT COUNT(*) FROM read_parquet(?)",
      [&path_string],
      |row| row.get::<_, i64>(0),
    )
    .unwrap_err();
  // A fresh binder sees the already-deleted file. Deletion later in a read
  // can report "Cannot open file", consistent with the historical diagnostic.
  assert!(
    error
      .to_string()
      .contains("No files found that match the pattern"),
    "{error}"
  );
  let mut session = Session::new();
  assert!(matches!(
    session.execute(parse_command(&format!("use {path_string}")).unwrap()),
    Err(RuntimeError::FileNotFound { .. })
  ));
  // Current validation detects deletion before `use`; the historical error
  // variants are compatible with deletion later in the read pipeline.
  eprintln!(
    "native missing-file and runtime FileNotFound reproduced while the other guard remains live"
  );
  drop(retained);
}

#[test]
fn concurrent_fixture_claims_and_cleanup_are_independent() {
  let mut fixtures = std::thread::scope(|scope| {
    let workers: Vec<_> = (0..32).map(|_| scope.spawn(Fixture::new)).collect();
    workers
      .into_iter()
      .map(|worker| worker.join().unwrap())
      .collect::<Vec<_>>()
  });
  let roots: std::collections::HashSet<_> = fixtures.iter().map(|f| &f.root).collect();
  assert_eq!(roots.len(), 32);
  let retained = fixtures.pop().unwrap();
  let marker = retained.root.join("retained-marker");
  fs::write(&marker, "independent").unwrap();
  drop(fixtures);
  assert_eq!(fs::read_to_string(&marker).unwrap(), "independent");
}

#[test]
fn reshape_requires_an_active_dataset_without_initializing_backend() {
  let mut session = Session::new();
  let command = parse_command("reshape long income, i(id) j(year)").expect("valid command syntax");
  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "reshape" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn reshape_long_and_wide_roundtrip() {
  let fixture = Fixture::new();
  let path = fixture.root.join("wide.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT * FROM (VALUES \
        (1, 10.0::double, 12.0::double, 100.0::double, 120.0::double), \
        (2, 20.0::double, 21.0::double, 200.0::double, 210.0::double) \
      ) AS wide(id, income_2020, income_2021, cost_2020, cost_2021)) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();

  // 1. Reshape long
  let long_res = session
    .execute(parse_command("reshape long income cost, i(id) j(year)").unwrap())
    .unwrap();
  let ExecutionResult::Reshape(ReshapeResult { dataset: long_ds }) = long_res else {
    panic!("expected Reshape result");
  };
  let long_cols: Vec<&str> = long_ds.columns.iter().map(|c| c.name.as_str()).collect();
  assert_eq!(long_cols, vec!["id", "year", "income", "cost"]);
  assert_eq!(long_ds.row_count, 4);

  let preview = session.execute(parse_command("head 5").unwrap()).unwrap();
  let ExecutionResult::Head(PreviewResult {
    rows: long_rows, ..
  }) = preview
  else {
    panic!("expected Head result");
  };
  assert_eq!(
    long_rows,
    vec![
      vec![
        CellValue::SignedInteger(1),
        CellValue::Text("2020".to_string()),
        CellValue::Float(10.0),
        CellValue::Float(100.0),
      ],
      vec![
        CellValue::SignedInteger(1),
        CellValue::Text("2021".to_string()),
        CellValue::Float(12.0),
        CellValue::Float(120.0),
      ],
      vec![
        CellValue::SignedInteger(2),
        CellValue::Text("2020".to_string()),
        CellValue::Float(20.0),
        CellValue::Float(200.0),
      ],
      vec![
        CellValue::SignedInteger(2),
        CellValue::Text("2021".to_string()),
        CellValue::Float(21.0),
        CellValue::Float(210.0),
      ],
    ]
  );

  // 2. Reshape wide back
  let wide_res = session
    .execute(parse_command("reshape wide income cost, i(id) j(year)").unwrap())
    .unwrap();
  let ExecutionResult::Reshape(ReshapeResult { dataset: wide_ds }) = wide_res else {
    panic!("expected Reshape result");
  };
  let wide_cols: Vec<&str> = wide_ds.columns.iter().map(|c| c.name.as_str()).collect();
  assert_eq!(
    wide_cols,
    vec!["id", "income_2020", "income_2021", "cost_2020", "cost_2021"]
  );
  assert_eq!(wide_ds.row_count, 2);

  let preview_wide = session.execute(parse_command("head 5").unwrap()).unwrap();
  let ExecutionResult::Head(PreviewResult {
    rows: wide_rows, ..
  }) = preview_wide
  else {
    panic!("expected Head result");
  };
  assert_eq!(
    wide_rows,
    vec![
      vec![
        CellValue::SignedInteger(1),
        CellValue::Float(10.0),
        CellValue::Float(12.0),
        CellValue::Float(100.0),
        CellValue::Float(120.0),
      ],
      vec![
        CellValue::SignedInteger(2),
        CellValue::Float(20.0),
        CellValue::Float(21.0),
        CellValue::Float(200.0),
        CellValue::Float(210.0),
      ],
    ]
  );
}

#[test]
fn reshape_preserves_source_and_group_sequence() {
  let fixture = Fixture::new();
  let path = fixture.root.join("reshape_order.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT id, income_2021, income_2020, cost_2020, cost_2021 FROM (VALUES \
        (1, 2, 21.0::double, 20.0::double, 200.0::double, 210.0::double), \
        (2, 1, 11.0::double, 10.0::double, 100.0::double, 110.0::double) \
      ) AS wide(row_order, id, income_2021, income_2020, cost_2020, cost_2021) ORDER BY row_order) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();

  // Reshape long: source row order is preserved (id 2 then id 1)
  // and j values appear in order of appearance ("2021" then "2020")
  let long_res = session
    .execute(parse_command("reshape long income cost, i(id) j(year)").unwrap())
    .unwrap();
  let ExecutionResult::Reshape(ReshapeResult { dataset }) = long_res else {
    panic!("expected Reshape result");
  };
  assert_eq!(dataset.row_count, 4);

  let preview = session.execute(parse_command("head 10").unwrap()).unwrap();
  let ExecutionResult::Head(PreviewResult { rows, .. }) = preview else {
    panic!("expected Head result");
  };
  assert_eq!(
    rows,
    vec![
      vec![
        CellValue::SignedInteger(2),
        CellValue::Text("2021".to_string()),
        CellValue::Float(21.0),
        CellValue::Float(210.0),
      ],
      vec![
        CellValue::SignedInteger(2),
        CellValue::Text("2020".to_string()),
        CellValue::Float(20.0),
        CellValue::Float(200.0),
      ],
      vec![
        CellValue::SignedInteger(1),
        CellValue::Text("2021".to_string()),
        CellValue::Float(11.0),
        CellValue::Float(110.0),
      ],
      vec![
        CellValue::SignedInteger(1),
        CellValue::Text("2020".to_string()),
        CellValue::Float(10.0),
        CellValue::Float(100.0),
      ],
    ]
  );

  // Reshape wide: group order is preserved (id 2 was row 1, id 1 was row 2)
  let wide_res = session
    .execute(parse_command("reshape wide income cost, i(id) j(year)").unwrap())
    .unwrap();
  let ExecutionResult::Reshape(ReshapeResult { dataset: wide_ds }) = wide_res else {
    panic!("expected Reshape result");
  };
  assert_eq!(wide_ds.row_count, 2);

  let preview_wide = session.execute(parse_command("head 10").unwrap()).unwrap();
  let ExecutionResult::Head(PreviewResult {
    rows: wide_rows, ..
  }) = preview_wide
  else {
    panic!("expected Head result");
  };
  assert_eq!(
    wide_rows,
    vec![
      vec![
        CellValue::SignedInteger(2),
        CellValue::Float(20.0),
        CellValue::Float(21.0),
        CellValue::Float(200.0),
        CellValue::Float(210.0),
      ],
      vec![
        CellValue::SignedInteger(1),
        CellValue::Float(10.0),
        CellValue::Float(11.0),
        CellValue::Float(100.0),
        CellValue::Float(110.0),
      ],
    ]
  );
}

#[test]
fn reshape_handles_collision_free_internal_order_columns() {
  let fixture = Fixture::new();

  // Test 1: long reshape collision avoidance with __tabdat_reshape_row_order
  let long_path = fixture.root.join("reshape_long_collision.parquet");
  let long_path_str = long_path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT id, __tabdat_reshape_row_order_2020, __tabdat_reshape_row_order_2021 FROM (VALUES \
        (1, 10.0, 11.0) \
      ) AS wide(id, __tabdat_reshape_row_order_2020, __tabdat_reshape_row_order_2021)) TO ? (FORMAT PARQUET)",
      [&long_path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {long_path_str}")).unwrap())
    .unwrap();
  let res_long = session
    .execute(parse_command("reshape long __tabdat_reshape_row_order, i(id) j(year)").unwrap())
    .unwrap();
  let ExecutionResult::Reshape(ReshapeResult { dataset: ds_long }) = res_long else {
    panic!("expected Reshape result");
  };
  let long_cols: Vec<&str> = ds_long.columns.iter().map(|c| c.name.as_str()).collect();
  assert_eq!(long_cols, vec!["id", "year", "__tabdat_reshape_row_order"]);

  // Test 2: wide reshape collision avoidance with __tabdat_reshape_group_order
  let wide_path = fixture.root.join("reshape_wide_collision.parquet");
  let wide_path_str = wide_path.to_string_lossy().into_owned();
  conn
    .execute(
      "COPY (SELECT * FROM (VALUES \
        (2, 'order', 20.0), \
        (1, 'order', 10.0) \
      ) AS long_data(id, year, __tabdat_reshape_group) ORDER BY id DESC) TO ? (FORMAT PARQUET)",
      [&wide_path_str],
    )
    .unwrap();

  session
    .execute(parse_command(&format!("use {wide_path_str}")).unwrap())
    .unwrap();
  let res_wide = session
    .execute(parse_command("reshape wide __tabdat_reshape_group, i(id) j(year)").unwrap())
    .unwrap();
  let ExecutionResult::Reshape(ReshapeResult { dataset: ds_wide }) = res_wide else {
    panic!("expected Reshape result");
  };
  let wide_cols: Vec<&str> = ds_wide.columns.iter().map(|c| c.name.as_str()).collect();
  assert_eq!(wide_cols, vec!["id", "__tabdat_reshape_group_order"]);
}

#[test]
fn reshape_wide_preserves_null_and_duplicate_cell_behavior() {
  let fixture = Fixture::new();
  let path = fixture.root.join("reshape_wide_cells.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT * FROM (VALUES \
        (1, 2, '2021', 21.0), \
        (2, 2, '2021', null::double), \
        (3, 2, '2020', 20.0), \
        (4, 1, '2020', 10.0) \
      ) AS long_data(row_order, id, year, income) ORDER BY row_order) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();

  let res = session
    .execute(parse_command("reshape wide income, i(id) j(year)").unwrap())
    .unwrap();
  let ExecutionResult::Reshape(ReshapeResult { dataset }) = res else {
    panic!("expected Reshape result");
  };
  assert_eq!(dataset.row_count, 2);

  let preview = session.execute(parse_command("head 5").unwrap()).unwrap();
  let ExecutionResult::Head(PreviewResult { rows, .. }) = preview else {
    panic!("expected Head result");
  };
  assert_eq!(
    rows,
    vec![
      vec![
        CellValue::SignedInteger(2),
        CellValue::Float(20.0),
        CellValue::Float(21.0),
      ],
      vec![
        CellValue::SignedInteger(1),
        CellValue::Float(10.0),
        CellValue::Null,
      ],
    ]
  );
}

#[test]
fn reshape_reports_dataset_and_variable_validation_errors() {
  let fixture = Fixture::new();
  let path = fixture.root.join("partial.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT * FROM (VALUES \
        (1, 10.0, 12.0, 100.0, '2020', 10.0), \
        (2, 20.0, 21.0, 200.0, '2021', 20.0) \
      ) AS partial(id, income_2020, income_2021, cost_2020, year, income)) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();
  let before = session.active_dataset().unwrap().clone();

  // 1. Missing identifier
  let err = session
    .execute(parse_command("reshape long income, i(missing_id) j(time)").unwrap())
    .unwrap_err();
  assert_eq!(
    err,
    RuntimeError::ReshapeUnknownVariable {
      variables: vec!["missing_id".to_string()],
    }
  );
  assert_eq!(err.to_string(), "reshape unknown variable: missing_id");
  assert_eq!(session.active_dataset().unwrap(), &before);

  // 2. j_variable already exists in active relation for long reshape
  let err = session
    .execute(parse_command("reshape long income, i(id) j(year)").unwrap())
    .unwrap_err();
  assert_eq!(
    err,
    RuntimeError::ReshapeOutputColumnExists {
      variable: "year".to_string(),
    }
  );
  assert_eq!(
    err.to_string(),
    "reshape output column already exists: year"
  );
  assert_eq!(session.active_dataset().unwrap(), &before);

  // Drop year and income to test stub matching errors
  session
    .execute(parse_command("drop year income").unwrap())
    .unwrap();
  let after_drop = session.active_dataset().unwrap().clone();

  // 3. Stub matches no columns
  let err = session
    .execute(parse_command("reshape long bmi, i(id) j(year)").unwrap())
    .unwrap_err();
  assert_eq!(
    err,
    RuntimeError::ReshapeLongFoundNoColumnsForStub {
      stub: "bmi".to_string(),
    }
  );
  assert_eq!(
    err.to_string(),
    "reshape long found no columns for stub: bmi"
  );
  assert_eq!(session.active_dataset().unwrap(), &after_drop);

  // 4. Missing column across stubs: income has 2020 and 2021, cost only has 2020
  let err = session
    .execute(parse_command("reshape long income cost, i(id) j(year)").unwrap())
    .unwrap_err();
  assert_eq!(
    err,
    RuntimeError::ReshapeLongMissingColumn {
      column: "cost_2021".to_string(),
    }
  );
  assert_eq!(err.to_string(), "reshape long missing column: cost_2021");
  assert_eq!(session.active_dataset().unwrap(), &after_drop);
}

#[test]
fn reshape_long_validates_j_values_across_all_stubs() {
  let fixture = Fixture::new();
  let path = fixture.root.join("ragged_wide.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT * FROM (VALUES \
        (1, 10.0, 100.0, 300.0), \
        (2, 20.0, 200.0, 400.0) \
      ) AS ragged(id, income_2020, cost_2020, cost_2022)) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();
  let before = session.active_dataset().unwrap().clone();

  let err = session
    .execute(parse_command("reshape long income cost, i(id) j(year)").unwrap())
    .unwrap_err();
  assert_eq!(
    err,
    RuntimeError::ReshapeLongMissingColumn {
      column: "income_2022".to_string(),
    }
  );
  assert_eq!(err.to_string(), "reshape long missing column: income_2022");
  assert_eq!(session.active_dataset().unwrap(), &before);
}

#[test]
fn reshape_wide_reports_output_conflict() {
  let fixture = Fixture::new();
  let path = fixture.root.join("long_conflict.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT * FROM (VALUES \
        (1, '2020', 10.0, 999.0), \
        (1, '2021', 12.0, 999.0) \
      ) AS long_data(id, year, income, income_2020)) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();
  let before = session.active_dataset().unwrap().clone();

  let err = session
    .execute(parse_command("reshape wide income, i(id) j(year)").unwrap())
    .unwrap_err();
  assert_eq!(
    err,
    RuntimeError::ReshapeWideOutputColumnExists {
      variable: "income_2020".to_string(),
    }
  );
  assert_eq!(
    err.to_string(),
    "reshape wide output column already exists: income_2020"
  );
  assert_eq!(session.active_dataset().unwrap(), &before);
}

#[test]
fn reshape_wide_reports_no_j_values_on_all_null_column() {
  let fixture = Fixture::new();
  let path = fixture.root.join("all_null_j.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT * FROM (VALUES \
        (1, null::varchar, 10.0), \
        (2, null::varchar, 20.0) \
      ) AS long_data(id, year, income)) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();
  let before = session.active_dataset().unwrap().clone();

  let err = session
    .execute(parse_command("reshape wide income, i(id) j(year)").unwrap())
    .unwrap_err();
  assert_eq!(err, RuntimeError::ReshapeWideFoundNoJValues);
  assert_eq!(err.to_string(), "reshape wide found no j values");
  assert_eq!(session.active_dataset().unwrap(), &before);
}

#[test]
fn reshape_detaches_active_table_from_named_table() {
  let fixture = Fixture::new();
  let path = fixture.root.join("wide_detach.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT * FROM (VALUES \
        (1, 10.0, 12.0), \
        (2, 20.0, 21.0) \
      ) AS wide(id, income_2020, income_2021)) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();
  session
    .execute(parse_command("sql select * from active into base").unwrap())
    .unwrap();
  assert_eq!(session.active_table_name(), Some("base"));

  session
    .execute(parse_command("reshape long income, i(id) j(year)").unwrap())
    .unwrap();
  assert_eq!(session.active_table_name(), None);

  // Named table base remains unchanged with 2 rows and wide schema
  session.execute(parse_command("use base").unwrap()).unwrap();
  let count_res = session.execute(parse_command("count").unwrap()).unwrap();
  let ExecutionResult::Count(CountResult { row_count }) = count_res else {
    panic!("expected Count result");
  };
  assert_eq!(row_count, 2);
  let cols: Vec<&str> = session
    .active_dataset()
    .unwrap()
    .columns
    .iter()
    .map(|c| c.name.as_str())
    .collect();
  assert_eq!(cols, vec!["id", "income_2020", "income_2021"]);
}

#[test]
fn reshape_retains_surviving_variable_labels() {
  let fixture = Fixture::new();
  let path = fixture.root.join("wide_labels.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT * FROM (VALUES \
        (1, 10.0, 12.0), \
        (2, 20.0, 21.0) \
      ) AS wide(id, income_2020, income_2021)) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();
  session
    .execute(parse_command("label variable id \"Subject ID\"").unwrap())
    .unwrap();
  session
    .execute(parse_command("label variable income_2020 \"2020 Income\"").unwrap())
    .unwrap();

  session
    .execute(parse_command("reshape long income, i(id) j(year)").unwrap())
    .unwrap();

  let metadata = session.active_label_metadata().expect("labels retained");
  assert_eq!(
    metadata.variable_labels,
    vec![("id".to_string(), "Subject ID".to_string())]
  );
}

#[test]
fn reshape_executes_in_multiline_script() {
  let fixture = Fixture::new();
  let data_path = fixture.root.join("data.parquet");
  let data_path_str = data_path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT * FROM (VALUES \
        (1, 10.0, 12.0), \
        (2, 20.0, 21.0) \
      ) AS wide(id, val_1, val_2)) TO ? (FORMAT PARQUET)",
      [&data_path_str],
    )
    .unwrap();

  let script_path = fixture.root.join("test_script.td");
  let script_content = format!(
    "use {data_path_str}\n\
     reshape long val, i(id) j(t)\n\
     count\n"
  );
  fs::write(&script_path, script_content).unwrap();

  let mut session = Session::new();
  let result = session.execute_run(&script_path).unwrap();
  let ExecutionResult::Run(run_res) = result else {
    panic!("expected Run result");
  };
  assert_eq!(run_res.executed_commands, 3);
  assert_eq!(session.active_dataset().unwrap().row_count, 4);
}
