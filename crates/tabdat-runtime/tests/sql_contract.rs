use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::parse_command;
use tabdat_runtime::{
  ActivateResult, CellValue, DescribeResult, ExecutionResult, PreviewResult, RuntimeError, Session,
  SqlCreateResult, TableResult,
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
      "tabdat-runtime-sql-{0}-{1}-{2}",
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
fn sql_requires_an_active_dataset() {
  let mut session = Session::new();
  let command = parse_command("sql select 1").expect("valid command");

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "sql" }
  );
}

#[test]
fn sql_rejects_queries_not_starting_with_select_or_with() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  let cases = [
    "sql drop table active",
    "sql insert into active values (1, 2, 'M', 10)",
    "sql update active set age = 100",
    "sql create table foo as select * from active",
    "sql delete from active",
  ];

  for case in cases {
    let command = parse_command(case).expect("valid parse");
    assert_eq!(
      session.execute(command).unwrap_err(),
      RuntimeError::SqlNotSelectOrWith,
      "failed on case: {case}"
    );
  }
}

#[test]
fn sql_queries_active_dataset() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  let command =
    parse_command("sql select sex, avg(bmi) as mean_bmi from active group by sex order by sex")
      .unwrap();
  let result = session.execute(command).unwrap();

  assert_eq!(
    result,
    ExecutionResult::Table(TableResult {
      headers: vec!["sex".to_owned(), "mean_bmi".to_owned()],
      rows: vec![
        vec![CellValue::Text("F".to_owned()), CellValue::Float(25.0)],
        vec![CellValue::Text("M".to_owned()), CellValue::Float(25.0)],
      ],
    })
  );

  // Verify active dataset was untouched
  let described = session.execute(parse_command("describe").unwrap()).unwrap();
  let ExecutionResult::Describe(DescribeResult { dataset }) = described else {
    panic!("expected DescribeResult");
  };
  let column_names: Vec<String> = dataset.columns.into_iter().map(|col| col.name).collect();
  assert_eq!(column_names, vec!["age", "bmi", "sex", "cost"]);
}

#[test]
fn sql_supports_with_ctes() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  let command = parse_command(
    "sql with female as (select * from active where sex = 'F') select count(*) as n from female",
  )
  .unwrap();
  let result = session.execute(command).unwrap();

  assert_eq!(
    result,
    ExecutionResult::Table(TableResult {
      headers: vec!["n".to_owned()],
      rows: vec![vec![CellValue::SignedInteger(2)]],
    })
  );
}

#[test]
fn sql_fails_on_invalid_sql() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  let command = parse_command("sql select missing from active").unwrap();
  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::SqlFailed
  );

  let command = parse_command("sql select * from non_existent_relation").unwrap();
  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::SqlFailed
  );
}

#[test]
fn sql_into_replaces_active_dataset() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  let command = parse_command(
    "sql select sex, count(*) as n from active group by sex order by sex into summary",
  )
  .unwrap();
  let result = session.execute(command).unwrap();

  let ExecutionResult::SqlCreate(SqlCreateResult {
    table_name,
    dataset,
  }) = result
  else {
    panic!("expected SqlCreateResult");
  };
  assert_eq!(table_name, "summary");
  assert_eq!(dataset.row_count, 2);
  let column_names: Vec<String> = dataset.columns.iter().map(|col| col.name.clone()).collect();
  assert_eq!(column_names, vec!["sex", "n"]);

  // Active dataset is replaced
  let described = session.execute(parse_command("describe").unwrap()).unwrap();
  let ExecutionResult::Describe(DescribeResult {
    dataset: desc_dataset,
  }) = described
  else {
    panic!("expected DescribeResult");
  };
  let desc_names: Vec<String> = desc_dataset
    .columns
    .iter()
    .map(|col| col.name.clone())
    .collect();
  assert_eq!(desc_names, vec!["sex", "n"]);

  // Head preview
  let preview = session.execute(parse_command("head 5").unwrap()).unwrap();
  let ExecutionResult::Head(PreviewResult { columns, rows }) = preview else {
    panic!("expected Head preview");
  };
  assert_eq!(columns, vec!["sex", "n"]);
  assert_eq!(
    rows,
    vec![
      vec![CellValue::Text("F".to_owned()), CellValue::SignedInteger(2)],
      vec![CellValue::Text("M".to_owned()), CellValue::SignedInteger(1)],
    ]
  );
}

#[test]
fn sql_into_registers_named_table_for_later_activation() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  session
    .execute(
      parse_command(
        "sql select sex, count(*) as n from active group by sex order by sex into summary",
      )
      .unwrap(),
    )
    .unwrap();

  session.execute(parse_command("keep sex").unwrap()).unwrap();
  let transformed = session.execute(parse_command("describe").unwrap()).unwrap();
  let ExecutionResult::Describe(DescribeResult {
    dataset: trans_dataset,
  }) = transformed
  else {
    panic!("expected DescribeResult");
  };
  let trans_names: Vec<String> = trans_dataset
    .columns
    .iter()
    .map(|col| col.name.clone())
    .collect();
  assert_eq!(trans_names, vec!["sex"]);

  let activated = session
    .execute(parse_command("use summary").unwrap())
    .unwrap();
  let ExecutionResult::Activate(ActivateResult {
    table_name,
    dataset: act_dataset,
  }) = activated
  else {
    panic!("expected ActivateResult");
  };
  assert_eq!(table_name, "summary");
  let act_names: Vec<String> = act_dataset
    .columns
    .iter()
    .map(|col| col.name.clone())
    .collect();
  assert_eq!(act_names, vec!["sex"]);

  let preview = session.execute(parse_command("head 5").unwrap()).unwrap();
  let ExecutionResult::Head(PreviewResult { columns, rows }) = preview else {
    panic!("expected Head preview");
  };
  assert_eq!(columns, vec!["sex"]);
  assert_eq!(
    rows,
    vec![
      vec![CellValue::Text("F".to_owned())],
      vec![CellValue::Text("M".to_owned())],
    ]
  );
}

#[test]
fn use_unknown_named_table_reports_specific_error() {
  let mut session = Session::new();
  let command = parse_command("use missing_table").unwrap();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnknownTable {
      name: "missing_table".to_owned()
    }
  );
}

#[test]
fn use_named_table_rejects_options() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  session
    .execute(
      parse_command(
        "sql select sex, count(*) as n from active group by sex order by sex into summary",
      )
      .unwrap(),
    )
    .unwrap();

  let lazy_command = parse_command("use summary, lazy").unwrap();
  assert_eq!(
    session.execute(lazy_command).unwrap_err(),
    RuntimeError::UseOptionsNotSupportedForNamedTable
  );

  let delim_command = parse_command("use summary, delimiter(\",\")").unwrap();
  assert_eq!(
    session.execute(delim_command).unwrap_err(),
    RuntimeError::UseOptionsNotSupportedForNamedTable
  );
}

#[test]
fn sql_inside_script_execution() {
  let fixture = Fixture::new();
  let script_path = fixture.root.join("analysis.td");
  let script_content = format!(
    "use {}\nsql \"\"\"\nselect sex, count(*) as n\nfrom active\ngroup by sex\norder by sex\n\"\"\" into summary\nkeep sex\nuse summary\ncount\n",
    fixture.parquet.to_string_lossy()
  );
  fs::write(&script_path, script_content).expect("script should be written");

  let mut session = Session::new();
  let command = parse_command(&format!("run {}", script_path.to_string_lossy())).unwrap();
  let result = session.execute(command).unwrap();

  let ExecutionResult::Run(run_res) = result else {
    panic!("expected RunResult");
  };
  assert_eq!(run_res.executed_commands, 5);

  let active = session.active_dataset().expect("active dataset");
  assert_eq!(active.row_count, 2);
  let col_names: Vec<String> = active.columns.iter().map(|c| c.name.clone()).collect();
  assert_eq!(col_names, vec!["sex"]);
}
