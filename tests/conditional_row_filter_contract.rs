#![forbid(unsafe_code)]
use serde_json::{Value, json};
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::{ExecutionResult, RuntimeError, Session};
static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-row-filter-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "id,x,y,g\n3,3,5,A\n1,1,2,B\n6,6,,A\n5,5,8,B\n2,2,5,A\n4,4,9,B\n",
    )
    .unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn session(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", self.0.join("data.csv").display()),
    );
    session
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn execute(session: &mut Session, source: &str) -> ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}
fn value(result: &ExecutionResult) -> Value {
  serde_json::from_str(&tabdat_reporting::json_results::serialize_execution_result(result).unwrap())
    .unwrap()
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
fn lines(bytes: &[u8]) -> Vec<Value> {
  std::str::from_utf8(bytes)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}
fn batch(fixture: &Fixture, commands: &[&str]) -> Vec<Value> {
  let mut args = vec!["--json"];
  for command in commands {
    args.extend(["-c", command]);
  }
  success(&fixture.batch(&args))
}
fn ids(session: &mut Session) -> Vec<i64> {
  value(&execute(session, "head 20"))["data"]["rows"]
    .as_array()
    .unwrap()
    .iter()
    .map(|row| row[0].as_i64().unwrap())
    .collect()
}
fn close(actual: &Value, expected: f64) {
  assert!(
    (actual.as_f64().unwrap() - expected).abs() < 1e-10,
    "{actual} != {expected}"
  );
}

#[test]
fn ordered_cohort_cached_fit_named_sync_refit_prediction_and_persistence_match_nested_stream() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "sql select * from active into primary",
    "label variable x 'Predictor'",
    "label define groups 'A' 'Group A' 'B' 'Group B'",
    "label values g groups",
    "regress y x",
    "keep if x >= 2",
    "lincom x",
    "drop if y == null",
    "head 10",
    "label list",
    "by g: count",
    "regress y x",
    "predict fitted",
    "head 10",
    "save cohort.parquet, replace",
    "export cohort.csv, replace",
    "use primary",
    "head 10",
  ];
  let results = batch(&fixture, &commands);
  assert_eq!(results.len(), commands.len());
  assert_eq!(results[6]["result_type"], "KeepResult");
  let mut expected = results[1]["data"]["dataset"].clone();
  expected["row_count"] = json!(5);
  assert_eq!(results[6]["data"], json!({"dataset":expected}));
  assert_eq!(results[8]["result_type"], "DropResult");
  expected["row_count"] = json!(4);
  assert_eq!(results[8]["data"], json!({"dataset":expected}));
  close(&results[7]["data"]["estimate"], 1.6);
  assert_eq!(
    results[9]["data"]["rows"],
    json!([
      [3, 3, 5, "A"],
      [5, 5, 8, "B"],
      [2, 2, 5, "A"],
      [4, 4, 9, "B"]
    ])
  );
  assert_eq!(
    results[10]["data"]["metadata"]["variable_labels"],
    json!([["x", "Predictor"]])
  );
  assert_eq!(
    results[10]["data"]["metadata"]["attachments"],
    json!([["g", "groups"]])
  );
  assert_eq!(results[11]["data"]["rows"], json!([["A", 2], ["B", 2]]));
  // Selected x=[2,3,4,5], y=[5,5,9,8]: Sxx=5, Sxy=6.5.
  // The independent closed-form slope is 1.3; intercept=6.75-1.3*3.5=2.2.
  assert_eq!(results[12]["data"]["observation_count"], 4);
  close(&results[12]["data"]["coefficients"][0]["value"], 2.2);
  close(&results[12]["data"]["coefficients"][1]["value"], 1.3);
  // Independent statsmodels 0.14.6 / NumPy 2.4.4 / SciPy 1.17.1 agrees;
  // SSE=4.3, residual DF=2, Sxx=5 gives these conventional OLS SEs.
  close(
    &results[12]["data"]["coefficients"][0]["standard_error"],
    2.409356760631351,
  );
  close(
    &results[12]["data"]["coefficients"][1]["standard_error"],
    0.6557438524302001,
  );
  close(&results[12]["data"]["r_squared"], 0.6627450980392158);
  close(&results[12]["data"]["root_mse"], 1.466287829861518);
  for row in results[14]["data"]["rows"].as_array().unwrap() {
    close(&row[4], 2.2 + 1.3 * row[1].as_f64().unwrap());
  }
  assert_eq!(results[18]["data"], results[14]["data"]);
  for path in ["cohort.csv", "cohort.parquet"] {
    let reloaded = batch(&fixture, &[&format!("use {path}"), "head 10"]);
    assert_eq!(reloaded[1]["data"], results[14]["data"]);
  }
  fs::write(fixture.0.join("child.td"), commands[6..].join("\n")).unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    format!("{}\nrun child.td\n", commands[..6].join("\n")),
  )
  .unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
  let mut session = fixture.session();
  execute(&mut session, "regress y x");
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 3, 4, 5]
  );
  execute(&mut session, "keep if x >= 2");
  execute(&mut session, "drop if y == null");
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 3, 4, 5]
  );
  execute(&mut session, "regress y x");
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 2, 3]
  );
}

#[test]
fn true_false_unknown_complements_explicit_null_and_nonfinite_comparison_policy() {
  let fixture = Fixture::new();
  let setup = "sql select * from (values (1,cast('NaN' as double),''),(2,cast('Infinity' as double),'A'),(3,cast('-Infinity' as double),'A'),(4,cast(null as double),cast(null as varchar)),(5,0.0,'B'),(6,2.0,'A')) as s(id,x,g) order by id into valueset";
  for (predicate, keep_ids, drop_ids) in [
    ("x > 0", vec![1, 2, 6], vec![3, 4, 5]),
    ("x == null", vec![4], vec![1, 2, 3, 5, 6]),
    ("(x + 0) == null", vec![1, 2, 3, 4], vec![5, 6]),
    ("g == ''", vec![1], vec![2, 3, 4, 5, 6]),
    ("g != 'A'", vec![1, 5], vec![2, 3, 4, 6]),
    ("null", vec![], vec![1, 2, 3, 4, 5, 6]),
    ("null == null", vec![1, 2, 3, 4, 5, 6], vec![]),
  ] {
    for (action, expected) in [("keep", keep_ids), ("drop", drop_ids)] {
      let mut session = fixture.session();
      execute(&mut session, setup);
      let before = session.active_dataset().unwrap().clone();
      execute(&mut session, &format!("{action} if {predicate}"));
      assert_eq!(ids(&mut session), expected, "{action} if {predicate}");
      assert_eq!(session.active_dataset().unwrap().columns, before.columns);
    }
  }
  for (action, expected) in [("keep", vec![1]), ("drop", vec![3, 6, 5, 2, 4])] {
    let mut session = fixture.session();
    execute(
      &mut session,
      &format!("{action} if x * 99999999999999999999999999999999999999 > 0"),
    );
    assert_eq!(ids(&mut session), expected);
  }
  let mut session = fixture.session();
  execute(
    &mut session,
    "sql select id,x>3 as flag from active order by id into flags",
  );
  execute(&mut session, "keep if flag");
  assert_eq!(ids(&mut session), [4, 5, 6]);
}

#[test]
fn quoted_exact_wide_scalars_internal_name_collision_and_empty_repeated_filters() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(
    &mut session,
    "sql select id,cast(9007199254740992+id as bigint) as \"x\"\"名\",'O''Brien' as g,id as __tabdat_filter_row_order from active order by id into quoted",
  );
  execute(&mut session, "keep if `x\"名` == 9007199254740993");
  assert_eq!(ids(&mut session), [1]);
  execute(&mut session, "keep if g == \"O'Brien\"");
  assert_eq!(ids(&mut session), [1]);
  execute(&mut session, "drop if id == 1");
  assert!(ids(&mut session).is_empty());
  let before = session.active_dataset().unwrap().clone();
  for command in ["keep if null == null", "drop if null", "keep if id > 0"] {
    execute(&mut session, command);
    assert_eq!(session.active_dataset().unwrap(), &before);
  }
  let mut session = fixture.session();
  execute(
    &mut session,
    "sql select id,cast(x as decimal(38,1)) as x from active order by id into decimals",
  );
  execute(&mut session, "keep if x / 2 >= 2");
  assert_eq!(ids(&mut session), [4, 5, 6]);
  assert_eq!(
    session.active_dataset().unwrap().columns[1].data_type,
    "DECIMAL(38,1)"
  );
}

#[test]
fn validation_and_native_query_failures_leave_rows_schema_labels_named_and_fit_unchanged() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(&mut session, "sql select * from active into primary");
  execute(&mut session, "label variable x 'Predictor'");
  execute(&mut session, "regress y x");
  let state = session.state();
  let preview = execute(&mut session, "head 20");
  let inference = execute(&mut session, "lincom x");
  for (source, expected) in [
    (
      "keep if absent > 0",
      RuntimeError::RowFilterUnknownVariable {
        variables: vec!["absent".into()],
      },
    ),
    ("drop if x", RuntimeError::RowFilterRequiresBoolean),
    (
      "keep if inrange(x, 1, 5)",
      RuntimeError::RowFilterUnsupportedExpression,
    ),
  ] {
    assert_eq!(
      session.execute(parse_command(source).unwrap()).unwrap_err(),
      expected
    );
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "head 20"), preview);
    assert_eq!(execute(&mut session, "lincom x"), inference);
  }
  for source in ["keep if x == 'A'", "drop if null < 1"] {
    assert!(matches!(
      session.execute(parse_command(source).unwrap()),
      Err(RuntimeError::RowFilterTypeMismatch { .. })
    ));
    assert_eq!(session.state(), state);
  }
  assert_eq!(
    session
      .execute(parse_command("keep if g == 'A\0'").unwrap())
      .unwrap_err(),
    RuntimeError::RowFilterFailed
  );
  assert_eq!(session.state(), state);
  assert_eq!(execute(&mut session, "head 20"), preview);
  assert_eq!(execute(&mut session, "lincom x"), inference);
  execute(&mut session, "use primary");
  assert_eq!(execute(&mut session, "head 20"), preview);
  execute(
    &mut session,
    "sql select id,cast(x as ubigint) as x from active into unsigned",
  );
  let state = session.state();
  let preview = execute(&mut session, "head 20");
  for source in ["keep if -x > 0", "drop if x-1 > 0", "keep if x > -1"] {
    assert!(matches!(
      session.execute(parse_command(source).unwrap()),
      Err(RuntimeError::RowFilterTypeMismatch { .. })
    ));
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "head 20"), preview);
  }
  execute(&mut session, "keep if x > 2");
  assert_eq!(ids(&mut session), [3, 6, 5, 4]);
}

#[test]
fn error_streams_stop_later_files_and_terminal_recovery_uses_same_filters() {
  let fixture = Fixture::new();
  assert_eq!(
    fixture
      .batch(&["--json", "-c", "keep if x > 0"])
      .status
      .code(),
    Some(1)
  );
  for (source, code) in [
    ("keep if absent > 0", 1),
    ("drop if x", 1),
    ("keep if inrange(x, 1, 5)", 1),
    ("keep if x >", 2),
    ("drop if x > 0, force", 2),
  ] {
    let output = fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      source,
      "-c",
      "export never.csv",
    ]);
    assert_eq!(output.status.code(), Some(code));
    assert_eq!(lines(&output.stdout).len(), 2);
    fs::write(
      fixture.0.join("child.td"),
      format!("count\n{source}\nexport never.csv\n"),
    )
    .unwrap();
    fs::write(
      fixture.0.join("parent.td"),
      "use data.csv\nrun child.td\nexport never.csv\n",
    )
    .unwrap();
    let output = fixture.batch(&["--json", "-f", "parent.td"]);
    assert_eq!(output.status.code(), Some(code));
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 3);
    assert_eq!(results[2]["error"]["line"], 2);
    assert_eq!(
      results[2]["error"]["path"],
      fixture
        .0
        .join("child.td")
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .as_ref()
    );
    assert!(!fixture.0.join("never.csv").exists());
  }
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&fixture.0)
    .env("TERM", "dumb")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child.stdin.take().unwrap().write_all(b"use data.csv\nkeep if x >= 2\ndrop if y == null\nkeep if absent > 0\ncount\nexport terminal.csv\nexit\n").unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(output.status.success());
  assert!(String::from_utf8_lossy(&output.stderr).contains("row filter unknown variable"));
  let reloaded = batch(&fixture, &["use terminal.csv", "head 10"]);
  assert_eq!(
    reloaded[1]["data"]["rows"],
    json!([
      [3, 3, 5, "A"],
      [5, 5, 8, "B"],
      [2, 2, 5, "A"],
      [4, 4, 9, "B"]
    ])
  );
}
