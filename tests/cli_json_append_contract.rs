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
      "tabdat-json-append-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("left.csv"), "x,y\n1,2\n2,5\n3,5\n").unwrap();
    fs::write(root.join("right.csv"), "y,x\n9,4\n8,5\n,6\n").unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn setup(&self) -> &'static str {
    "use left.csv\nsql select x,y from active order by x desc into primary\nuse right.csv\nsql select y,x from active order by x desc into followup\nuse primary\n"
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
fn batch_commands(fixture: &Fixture, commands: &[&str]) -> Vec<Value> {
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  success(&fixture.batch(&args))
}

#[test]
fn append_alignment_order_cached_fit_refit_persistence_and_named_detachment_match_nested_stream() {
  let fixture = Fixture::new();
  let commands = [
    "use left.csv",
    "sql select x,y from active order by x desc into primary",
    "use right.csv",
    "sql select y,x from active order by x desc into followup",
    "use primary",
    "label variable x 'Predictor'",
    "regress y x",
    "append followup",
    "lincom x",
    "head 10",
    "label list",
    "summarize x y",
    "regress y x",
    "predict fitted",
    "head 10",
    "save stacked.parquet, replace",
    "export stacked.csv, replace",
    "use primary",
    "head 10",
  ];
  let results = batch_commands(&fixture, &commands);
  assert_eq!(results.len(), commands.len());
  assert_eq!(results[7]["schema_version"], 1);
  assert_eq!(results[7]["result_type"], "AppendResult");
  assert_eq!(
    results[7]["data"],
    json!({"dataset":{"path":"primary","row_count":6,"columns":[{"name":"x","data_type":"BIGINT"},{"name":"y","data_type":"BIGINT"}],"execution_mode":"eager","lazy_engine":null,"label_metadata":null,"panel_metadata":null}})
  );
  assert_eq!(
    results[9]["data"]["rows"],
    json!([[3, 5], [2, 5], [1, 2], [6, null], [5, 8], [4, 9]])
  );
  close(&results[8]["data"]["estimate"], 1.5);
  assert_eq!(results[6]["data"]["observation_count"], 3);
  assert_eq!(
    results[10]["data"]["metadata"]["variable_labels"],
    json!([["x", "Predictor"]])
  );
  // Retained combined x=[3,2,1,5,4], y=[5,5,2,8,9] has Sxx=10,
  // Sxy=16, slope=1.6 and intercept=5.8-1.6*3=1.0. Missing y stays out.
  assert_eq!(results[12]["data"]["observation_count"], 5);
  close(&results[12]["data"]["coefficients"][0]["value"], 1.0);
  close(&results[12]["data"]["coefficients"][1]["value"], 1.6);
  let rows = results[14]["data"]["rows"].as_array().unwrap();
  assert_eq!(rows.len(), 6);
  for row in rows {
    close(&row[2], 1.0 + 1.6 * row[0].as_f64().unwrap());
  }
  for path in ["stacked.csv", "stacked.parquet"] {
    let reload = batch_commands(&fixture, &[&format!("use {path}"), "head 10"]);
    assert_eq!(reload[1]["data"], results[14]["data"]);
  }
  assert_eq!(results[17]["result_type"], "ActivateResult");
  assert_eq!(results[17]["data"]["dataset"]["row_count"], 3);
  assert_eq!(
    results[18]["data"],
    json!({"columns":["x","y"],"rows":[[3,5],[2,5],[1,2]]})
  );
  fs::write(fixture.0.join("child.td"), commands[4..].join("\n")).unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    format!("{}\nrun child.td\n", commands[..4].join("\n")),
  )
  .unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
}

#[test]
fn repeated_self_and_empty_quoted_appends_preserve_duplicates_native_schema_and_source() {
  let fixture = Fixture::new();
  let mut commands = fixture.setup().lines().collect::<Vec<_>>();
  commands.extend([
    "append followup",
    "append followup",
    "head 10",
    "use followup",
    "head 10",
  ]);
  let results = batch_commands(&fixture, &commands);
  assert_eq!(results[6]["data"]["dataset"]["row_count"], 9);
  assert_eq!(
    results[7]["data"]["rows"],
    json!([
      [3, 5],
      [2, 5],
      [1, 2],
      [6, null],
      [5, 8],
      [4, 9],
      [6, null],
      [5, 8],
      [4, 9]
    ])
  );
  assert_eq!(
    results[9]["data"],
    json!({"columns":["y","x"],"rows":[[null,6],[8,5],[9,4]]})
  );
  let self_append = batch_commands(
    &fixture,
    &[
      "use left.csv",
      "sql select x,y from active order by x into base",
      "append base",
      "head 10",
      "use base",
      "count",
    ],
  );
  assert_eq!(
    self_append[3]["data"]["rows"],
    json!([[1, 2], [2, 5], [3, 5], [1, 2], [2, 5], [3, 5]])
  );
  assert_eq!(self_append[5]["data"]["row_count"], 3);
  let empty = batch_commands(
    &fixture,
    &[
      "use left.csv",
      "sql select x as \"a\"\"b\",y from active where false into empty",
      "use left.csv",
      "sql select y,x as \"a\"\"b\" from active order by x into other",
      "use empty",
      "append other",
      "head 10",
    ],
  );
  assert_eq!(empty[5]["data"]["dataset"]["row_count"], 3);
  assert_eq!(empty[5]["data"]["dataset"]["columns"][0]["name"], "a\"b");
  assert_eq!(empty[6]["data"]["rows"], json!([[1, 2], [2, 5], [3, 5]]));
  let right_empty = batch_commands(
    &fixture,
    &[
      "use left.csv",
      "sql select x,y from active where false into empty",
      "use left.csv",
      "append empty",
      "head 10",
    ],
  );
  assert_eq!(right_empty[3]["data"]["dataset"]["path"], "left.csv");
  assert_eq!(
    right_empty[4]["data"]["rows"],
    json!([[1, 2], [2, 5], [3, 5]])
  );
}

#[test]
fn append_validation_errors_stream_locations_and_library_invariants_preserve_state_and_files() {
  let fixture = Fixture::new();
  let setup = format!(
    "{}sql select x,y,1 as extra from active into extra\nsql select x from active into missing\nuse primary\nsql select cast(x as integer) as x,y from active into narrow\nuse primary\nsql select cast(x as double) as x,y from active into double_type\nuse primary\n",
    fixture.setup()
  );
  let setup_count = setup.lines().count();
  for (source, code) in [
    ("append absent", 1),
    ("append extra", 1),
    ("append missing", 1),
    ("append narrow", 1),
    ("append double_type", 1),
    ("append", 2),
    ("append followup, replace", 2),
  ] {
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
    assert_eq!(results.len(), setup_count + 2);
    let error = &results.last().unwrap()["error"];
    assert_eq!(error["line"], 2);
    assert_eq!(
      error["path"],
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
    assert_eq!(results.len(), setup_count + 1);
    assert_eq!(
      results.last().unwrap()["error"]["type"],
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
      .batch(&["--json", "-c", "append followup"])
      .status
      .code(),
    Some(1)
  );
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("left.csv").display()),
  );
  execute(
    &mut session,
    "sql select x,y from active order by x desc into primary",
  );
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("right.csv").display()),
  );
  execute(
    &mut session,
    "sql select y,x from active order by x desc into followup",
  );
  execute(&mut session, "use primary");
  for source in setup.lines().skip(5) {
    execute(&mut session, source);
  }
  execute(&mut session, "regress y x");
  execute(&mut session, "label variable x 'Predictor'");
  execute(&mut session, "label define xs 1 'One'");
  execute(&mut session, "label values x xs");
  let before = session.state();
  let preview = execute(&mut session, "head 10");
  let inference = execute(&mut session, "lincom x");
  for source in [
    "append absent",
    "append extra",
    "append missing",
    "append narrow",
    "append double_type",
  ] {
    assert!(session.execute(parse_command(source).unwrap()).is_err());
    assert_eq!(session.state(), before);
    assert_eq!(execute(&mut session, "head 10"), preview);
  }
  let result = execute(&mut session, "append followup");
  let after = session.state();
  assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
  assert_eq!(session.state(), after);
  assert_eq!(after.estimation, before.estimation);
  assert_eq!(after.labels, before.labels);
  assert_eq!(after.named_tables, before.named_tables);
  assert!(after.active_table_name.is_none());
  assert_eq!(
    after.active_dataset.as_ref().unwrap().source,
    before.active_dataset.as_ref().unwrap().source
  );
  assert_eq!(execute(&mut session, "lincom x"), inference);
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
  execute(&mut session, "regress y x");
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 2, 4, 5]
  );
}
