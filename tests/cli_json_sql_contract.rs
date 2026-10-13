#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-json-sql-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "x,y\n1,2\n2,5\n3,5\n4,9\n5,8\n6,\n").unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn lines(bytes: &[u8]) -> Vec<Value> {
  std::str::from_utf8(bytes)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn success(output: &Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  lines(&output.stdout)
}
fn close(actual: &Value, expected: f64) {
  assert!(
    (actual.as_f64().unwrap() - expected).abs() < 1e-10,
    "{actual} != {expected}"
  );
}
fn execute(session: &mut Session, source: &str) -> tabdat_runtime::ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}

#[test]
fn with_filtered_ordered_cohort_activation_fit_prediction_persistence_and_nested_streams() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "label variable x 'Predictor'",
    "sql with cohort as (select x,y from active where x between 2 and 4 and y is not null) select x,y from cohort order by x desc into cohort",
    "label list",
    "describe",
    "summarize x y",
    "sql select count(*) as n, avg(x) as mean_x from active",
    "use cohort",
    "regress y x",
    "predict fitted",
    "head 10",
    "save cohort.parquet, replace",
    "export cohort.csv, replace",
  ];
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  let results = success(&fixture.batch(&args));
  assert_eq!(results.len(), commands.len());
  for (index, name) in [(2, "SqlCreateResult"), (7, "ActivateResult")] {
    assert_eq!(results[index]["schema_version"], 1);
    assert_eq!(results[index]["result_type"], name);
    assert_eq!(
      results[index]["data"],
      json!({"table_name":"cohort","dataset":{"path":"cohort","row_count":3,"columns":[{"name":"x","data_type":"BIGINT"},{"name":"y","data_type":"BIGINT"}],"execution_mode":"eager","lazy_engine":null,"label_metadata":null,"panel_metadata":null}})
    );
  }
  assert!(results[3]["data"]["metadata"].is_null());
  assert_eq!(results[4]["data"]["dataset"], results[2]["data"]["dataset"]);
  assert_eq!(results[6]["result_type"], "TableResult");
  assert_eq!(
    results[6]["data"],
    json!({"headers":["n","mean_x"],"rows":[[3,3.0]]})
  );
  // Independent retained x=[4,3,2], y=[9,5,5]: Sxx=2, Sxy=4,
  // slope=2 and intercept=19/3-2*3=1/3, with three observations.
  assert_eq!(results[8]["data"]["observation_count"], 3);
  close(&results[8]["data"]["coefficients"][0]["value"], 1.0 / 3.0);
  close(&results[8]["data"]["coefficients"][1]["value"], 2.0);
  let rows = results[10]["data"]["rows"].as_array().unwrap();
  assert_eq!(rows.len(), 3);
  for (row, (x, y)) in rows.iter().zip([(4, 9), (3, 5), (2, 5)]) {
    assert_eq!(row[0], x);
    assert_eq!(row[1], y);
    close(&row[2], 1.0 / 3.0 + 2.0 * f64::from(x));
  }
  for path in ["cohort.parquet", "cohort.csv"] {
    let reload =
      success(&fixture.batch(&["--json", "-c", &format!("use {path}"), "-c", "head 10"]));
    assert_eq!(reload[1]["data"], results[10]["data"]);
  }
  fs::write(fixture.0.join("child.td"), commands[2..].join("\n")).unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    format!("{}\nrun child.td\n", commands[..2].join("\n")),
  )
  .unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
}

#[test]
fn query_rows_use_existing_scalar_policy_duplicate_headers_and_empty_named_tables() {
  let fixture = Fixture::new();
  let query = r#"sql select cast(9007199254740993 as bigint) as wide, cast('9007199254740993.0001' as decimal(38,4)) as exact, from_hex('0001ff') as bytes, cast('NaN' as double) as nonfinite, 0.0::double as zero, true as flag, '目录' as "a""b", null as missing"#;
  let results = success(&fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    query,
    "-c",
    "sql select x,x from active order by x limit 2",
    "-c",
    "sql select x as \"列\" from active where false",
    "-c",
    "sql select x,y from active where false into 空表",
    "-c",
    "use 空表",
    "-c",
    "head 10",
  ]));
  assert_eq!(
    results[1]["data"],
    json!({"headers":["wide","exact","bytes","nonfinite","zero","flag","a\"b","missing"],"rows":[[9007199254740993_i64,"9007199254740993.0001","base64:AAH/",null,0.0,true,"目录",null]]})
  );
  assert_eq!(
    results[2]["data"],
    json!({"headers":["x","x"],"rows":[[1,1],[2,2]]})
  );
  assert_eq!(results[3]["data"], json!({"headers":["列"],"rows":[]}));
  assert_eq!(results[4]["data"]["table_name"], "空表");
  assert_eq!(results[4]["data"]["dataset"]["row_count"], 0);
  assert_eq!(results[5]["result_type"], "ActivateResult");
  assert_eq!(results[5]["data"], results[4]["data"]);
  assert_eq!(results[6]["data"]["rows"], json!([]));
  let replaced = success(&fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "sql select x,y from active order by x into cohort",
    "-c",
    "sql select x from active where x=2 into cohort",
    "-c",
    "use cohort",
    "-c",
    "head 10",
  ]));
  assert_eq!(replaced[2]["data"]["dataset"]["row_count"], 1);
  assert_eq!(replaced[4]["data"], json!({"columns":["x"],"rows":[[2]]}));
}

#[test]
fn sql_and_activation_errors_preserve_batch_script_categories_and_later_file_guards() {
  let fixture = Fixture::new();
  for (source, code) in [
    ("sql select absent from active", 1),
    ("sql select * from absent", 1),
    ("sql select absent from active into cohort", 1),
    ("sql drop table active", 1),
    ("sql select [1,2] as unsupported", 1),
    ("sql select 1 into active", 2),
    ("sql select 1 into __tabdat_hidden", 2),
    ("sql select 1 into", 2),
    ("sql", 2),
    ("use absent", 1),
    ("use cohort, lazy", 1),
  ] {
    let setup = "use data.csv\nsql select x,y from active order by x into cohort\n";
    fs::write(
      fixture.0.join("child.td"),
      format!("count\n{source}\nexport never.csv\n"),
    )
    .unwrap();
    fs::write(
      fixture.0.join("parent.td"),
      format!("{setup}run child.td\nexport never.csv\n"),
    )
    .unwrap();
    let output = fixture.batch(&["--json", "-f", "parent.td"]);
    assert_eq!(
      output.status.code(),
      Some(code),
      "{source}: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 4);
    assert_eq!(results[3]["error"]["line"], 2);
    assert_eq!(
      results[3]["error"]["path"],
      fixture
        .0
        .join("child.td")
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .as_ref()
    );
    let mut args = vec!["--json"];
    for command in setup.lines() {
      args.extend(["-c", command]);
    }
    args.extend(["-c", source, "-c", "export never.csv"]);
    let output = fixture.batch(&args);
    assert_eq!(output.status.code(), Some(code));
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 3);
    assert_eq!(
      results[2]["error"]["type"],
      if code == 2 {
        "ParseError"
      } else {
        "ExecutionError"
      }
    );
    assert!(!fixture.0.join("never.csv").exists());
  }
  assert_eq!(
    fixture
      .batch(&["--json", "-c", "sql select 1"])
      .status
      .code(),
    Some(1)
  );
}

#[test]
fn query_reporting_validation_and_binder_failures_preserve_state_and_cached_original_fit() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(&mut session, "regress y x");
  execute(&mut session, "label variable x 'Predictor'");
  execute(&mut session, "label define keys 1 'One'");
  execute(&mut session, "label values x keys");
  let original = session.state();
  let preview = execute(&mut session, "head 10");
  let inference = execute(&mut session, "lincom x");
  let table = execute(
    &mut session,
    "sql with q as (select x from active where x>2) select x from q order by x desc",
  );
  assert!(tabdat_reporting::json_results::serialize_execution_result(&table).is_some());
  assert_eq!(session.state(), original);
  assert_eq!(execute(&mut session, "head 10"), preview);
  for source in [
    "sql select absent from active",
    "sql select * from absent",
    "sql select absent from active into cohort",
    "sql drop table active",
    "sql select [1,2] as unsupported",
  ] {
    assert!(session.execute(parse_command(source).unwrap()).is_err());
    assert_eq!(session.state(), original);
    assert_eq!(execute(&mut session, "head 10"), preview);
  }
  assert!(
    session
      .execute_sql("select x from active", Some("bad\0name"))
      .is_err()
  );
  assert_eq!(session.state(), original);
  assert_eq!(execute(&mut session, "head 10"), preview);
  for source in [
    "sql select x,y from active where x between 2 and 4 order by x desc into cohort",
    "use cohort",
  ] {
    let result = execute(&mut session, source);
    let before = session.state();
    assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
    assert_eq!(session.state(), before);
    assert_eq!(before.estimation, original.estimation);
    assert!(before.labels.is_empty());
    assert_eq!(before.active_table_name.as_deref(), Some("cohort"));
    assert_eq!(execute(&mut session, "lincom x"), inference);
  }
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 2, 3, 4]
  );
  let before = session.state();
  let preview = execute(&mut session, "head 10");
  for source in [
    "sql select absent from active into cohort",
    "use absent",
    "use cohort, lazy",
  ] {
    assert!(session.execute(parse_command(source).unwrap()).is_err());
    assert_eq!(session.state(), before);
    assert_eq!(execute(&mut session, "head 10"), preview);
  }
  execute(&mut session, "regress y x");
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 2]
  );
  let grouped = execute(&mut session, "by x: summarize x");
  assert!(tabdat_reporting::json_results::serialize_execution_result(&grouped).is_none());
}
