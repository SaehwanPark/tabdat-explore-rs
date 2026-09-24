use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::parse_command;
use tabdat_runtime::{
  AppendResult, CellValue, CountResult, ExecutionResult, PreviewResult, RuntimeError, Session,
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
  root: PathBuf,
  parquet: PathBuf,
}

impl Fixture {
  fn new() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after the Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-append-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let parquet = root.join("patients.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES (30, 22.5, 'F', 100.0), (42, 25.0, 'M', 150.0), (54, 27.5, 'F', NULL)) AS patients(age, bmi, sex, cost)) TO ? (FORMAT PARQUET)",
        [&parquet_string],
      )
      .expect("fixture Parquet should be written");
    Self { root, parquet }
  }

  fn use_cmd(&self) -> String {
    format!("use {}", self.parquet.to_string_lossy())
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

#[test]
fn append_requires_an_active_dataset_without_initializing_backend() {
  let mut session = Session::new();
  let command = parse_command("append followup").expect("valid command");
  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "append" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn append_requires_a_registered_named_table() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  let before = session.active_dataset().unwrap().clone();

  let err = session
    .execute(parse_command("append missing_table").unwrap())
    .unwrap_err();
  assert_eq!(
    err,
    RuntimeError::UnknownTable {
      name: "missing_table".to_string()
    }
  );
  assert_eq!(err.to_string(), "unknown table: missing_table");
  assert_eq!(session.active_dataset().unwrap(), &before);
}

#[test]
fn append_reports_extra_variable_in_named_table() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(parse_command("sql select *, 1 as extra from active into extra_column").unwrap())
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  let before = session.active_dataset().unwrap().clone();

  let err = session
    .execute(parse_command("append extra_column").unwrap())
    .unwrap_err();
  assert_eq!(
    err,
    RuntimeError::AppendUnknownVariable {
      variables: vec!["extra".to_string()]
    }
  );
  assert_eq!(err.to_string(), "append unknown variable: extra");
  assert_eq!(session.active_dataset().unwrap(), &before);
}

#[test]
fn append_reports_missing_variable_in_named_table() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(parse_command("sql select age, bmi, sex from active into missing_cost").unwrap())
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  let before = session.active_dataset().unwrap().clone();

  let err = session
    .execute(parse_command("append missing_cost").unwrap())
    .unwrap_err();
  assert_eq!(
    err,
    RuntimeError::AppendUnknownVariableInTable {
      table_name: "missing_cost".to_string(),
      variables: vec!["cost".to_string()]
    }
  );
  assert_eq!(
    err.to_string(),
    "append unknown variable in missing_cost: cost"
  );
  assert_eq!(session.active_dataset().unwrap(), &before);
}

#[test]
fn append_reports_type_mismatch_for_variable() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(
      parse_command(
        "sql select cast(age as varchar) as age, bmi, sex, cost from active into bad_type",
      )
      .unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  let before = session.active_dataset().unwrap().clone();

  let err = session
    .execute(parse_command("append bad_type").unwrap())
    .unwrap_err();
  assert_eq!(
    err,
    RuntimeError::AppendTypeMismatch {
      variable: "age".to_string(),
      left_type: "INTEGER".to_string(),
      right_type: "VARCHAR".to_string(),
    }
  );
  assert_eq!(
    err.to_string(),
    "append type mismatch for age: INTEGER vs VARCHAR"
  );
  assert_eq!(session.active_dataset().unwrap(), &before);
}

#[test]
fn append_preserves_left_then_right_sequence() {
  let fixture = Fixture::new();
  let path = fixture.root.join("row_order.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT value, label FROM (VALUES (1, 3, 'first'), (2, NULL, 'missing'), (3, 1, 'second'), (4, 2, 'third'), (5, 4, 'fourth')) AS base_data(row_id, value, label) ORDER BY row_id) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();
  let append_query = "\
    sql select value, label from (\
      values (1, 9, 'ninth'), (2, 4, 'fourth'), (3, 8, 'eighth')\
    ) as append_data(row_id, value, label) order by row_id into followup";
  session
    .execute(parse_command(append_query).unwrap())
    .unwrap();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("append followup").unwrap())
    .unwrap();
  let ExecutionResult::Append(AppendResult { dataset }) = result else {
    panic!("expected Append result");
  };
  assert_eq!(dataset.row_count, 8);

  let head = session.execute(parse_command("head 5").unwrap()).unwrap();
  let ExecutionResult::Head(PreviewResult {
    rows: head_rows, ..
  }) = head
  else {
    panic!("expected Head result");
  };
  assert_eq!(
    head_rows,
    vec![
      vec![
        CellValue::SignedInteger(3),
        CellValue::Text("first".to_string())
      ],
      vec![CellValue::Null, CellValue::Text("missing".to_string())],
      vec![
        CellValue::SignedInteger(1),
        CellValue::Text("second".to_string())
      ],
      vec![
        CellValue::SignedInteger(2),
        CellValue::Text("third".to_string())
      ],
      vec![
        CellValue::SignedInteger(4),
        CellValue::Text("fourth".to_string())
      ],
    ]
  );

  let tail = session.execute(parse_command("tail 4").unwrap()).unwrap();
  let ExecutionResult::Tail(PreviewResult {
    rows: tail_rows, ..
  }) = tail
  else {
    panic!("expected Tail result");
  };
  assert_eq!(
    tail_rows,
    vec![
      vec![
        CellValue::SignedInteger(4),
        CellValue::Text("fourth".to_string())
      ],
      vec![
        CellValue::SignedInteger(9),
        CellValue::Text("ninth".to_string())
      ],
      vec![
        CellValue::SignedInteger(4),
        CellValue::Text("fourth".to_string())
      ],
      vec![
        CellValue::SignedInteger(8),
        CellValue::Text("eighth".to_string())
      ],
    ]
  );
}

#[test]
fn append_aligns_columns_when_named_table_has_different_column_order() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(
      parse_command("sql select sex, cost, bmi, age from active where age > 42 into followup")
        .unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("append followup").unwrap())
    .unwrap();
  let ExecutionResult::Append(AppendResult { dataset }) = result else {
    panic!("expected Append result");
  };
  assert_eq!(dataset.row_count, 4);
  let column_names: Vec<&str> = dataset.columns.iter().map(|c| c.name.as_str()).collect();
  assert_eq!(column_names, vec!["age", "bmi", "sex", "cost"]);

  let preview = session.execute(parse_command("tail 1").unwrap()).unwrap();
  let ExecutionResult::Tail(PreviewResult { rows, .. }) = preview else {
    panic!("expected Tail result");
  };
  assert_eq!(
    rows,
    vec![vec![
      CellValue::SignedInteger(54),
      CellValue::Decimal {
        width: 3,
        scale: 1,
        value: 275,
      },
      CellValue::Text("F".to_string()),
      CellValue::Null,
    ]]
  );
}

#[test]
fn append_detaches_active_table_from_named_table() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(parse_command("sql select * from active where age = 30 into base").unwrap())
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(parse_command("sql select * from active where age = 42 into followup").unwrap())
    .unwrap();

  session.execute(parse_command("use base").unwrap()).unwrap();
  assert_eq!(session.active_table_name(), Some("base"));

  let result = session
    .execute(parse_command("append followup").unwrap())
    .unwrap();
  let ExecutionResult::Append(AppendResult { dataset }) = result else {
    panic!("expected Append result");
  };
  assert_eq!(dataset.row_count, 2);
  assert_eq!(session.active_table_name(), None);

  let count = session.execute(parse_command("count").unwrap()).unwrap();
  let ExecutionResult::Count(CountResult { row_count }) = count else {
    panic!("expected Count result");
  };
  assert_eq!(row_count, 2);

  // Re-activating base reveals that its snapshot was preserved at 1 row
  session.execute(parse_command("use base").unwrap()).unwrap();
  assert_eq!(session.active_table_name(), Some("base"));
  let base_count = session.execute(parse_command("count").unwrap()).unwrap();
  let ExecutionResult::Count(CountResult { row_count: bc }) = base_count else {
    panic!("expected Count result");
  };
  assert_eq!(bc, 1);
}

#[test]
fn append_handles_collision_free_internal_order_columns() {
  let fixture = Fixture::new();
  let path = fixture.root.join("append_collision.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT __tabdat_append_side, __tabdat_append_row, label FROM (VALUES (1, 10, 100, 'active1'), (2, 20, 200, 'active2')) AS d(row_id, __tabdat_append_side, __tabdat_append_row, label) ORDER BY row_id) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();
  session
    .execute(
      parse_command(
        "sql select __tabdat_append_side, __tabdat_append_row, label from (VALUES (1, 30, 300, 'app1'), (2, 40, 400, 'app2')) AS d(row_id, __tabdat_append_side, __tabdat_append_row, label) ORDER BY row_id into followup",
      )
      .unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("append followup").unwrap())
    .unwrap();
  let ExecutionResult::Append(AppendResult { dataset }) = result else {
    panic!("expected Append result");
  };
  assert_eq!(dataset.row_count, 4);
  let column_names: Vec<&str> = dataset.columns.iter().map(|c| c.name.as_str()).collect();
  assert_eq!(
    column_names,
    vec!["__tabdat_append_side", "__tabdat_append_row", "label"]
  );

  let head = session.execute(parse_command("head 4").unwrap()).unwrap();
  let ExecutionResult::Head(PreviewResult { rows, .. }) = head else {
    panic!("expected Head result");
  };
  assert_eq!(
    rows,
    vec![
      vec![
        CellValue::SignedInteger(10),
        CellValue::SignedInteger(100),
        CellValue::Text("active1".to_string()),
      ],
      vec![
        CellValue::SignedInteger(20),
        CellValue::SignedInteger(200),
        CellValue::Text("active2".to_string()),
      ],
      vec![
        CellValue::SignedInteger(30),
        CellValue::SignedInteger(300),
        CellValue::Text("app1".to_string()),
      ],
      vec![
        CellValue::SignedInteger(40),
        CellValue::SignedInteger(400),
        CellValue::Text("app2".to_string()),
      ],
    ]
  );
}

#[test]
fn append_retains_left_variable_labels() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(parse_command("label variable age \"Patient Age\"").unwrap())
    .unwrap();
  session
    .execute(parse_command("label variable sex \"Biological Sex\"").unwrap())
    .unwrap();
  session
    .execute(parse_command("sql select * from active where age > 42 into followup").unwrap())
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(parse_command("label variable age \"Patient Age\"").unwrap())
    .unwrap();
  session
    .execute(parse_command("label variable sex \"Biological Sex\"").unwrap())
    .unwrap();

  session
    .execute(parse_command("append followup").unwrap())
    .unwrap();

  let labels = session
    .active_label_metadata()
    .expect("labels should exist");
  assert_eq!(
    labels.variable_labels,
    vec![
      ("age".to_string(), "Patient Age".to_string()),
      ("sex".to_string(), "Biological Sex".to_string()),
    ]
  );
}

#[test]
fn append_executes_in_multiline_script() {
  let fixture = Fixture::new();
  let script_path = fixture.root.join("analysis.td");
  let script_content = format!(
    "use {}\n\
     sql select * from active where age > 42 into followup\n\
     use {}\n\
     append followup\n\
     count\n",
    fixture.parquet.to_string_lossy(),
    fixture.parquet.to_string_lossy(),
  );
  fs::write(&script_path, script_content).expect("script should be written");

  let mut session = Session::new();
  let result = session
    .execute_run(&script_path)
    .expect("script should run");
  let ExecutionResult::Run(run_result) = result else {
    panic!("expected Run result");
  };
  assert_eq!(run_result.executed_commands, 5);

  let active = session
    .active_dataset()
    .expect("active dataset should exist");
  assert_eq!(active.row_count, 4);
}
