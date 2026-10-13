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
      "tabdat-json-projection-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "x,y,id,label,wide,\"odd\"\"名\"\n1.0,2,4,A,9007199254740993,甲\n2.0,5,2,,9007199254740994,乙\n3.0,5,9,B,9007199254740995,\n4.0,9,1,A,9007199254740996,丙\n,8,7,C,9007199254740997,丁\n").unwrap();
    fs::write(root.join("empty.csv"), "x,y,id\n").unwrap();
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
fn lines(output: &[u8]) -> Vec<Value> {
  std::str::from_utf8(output)
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
fn names(dataset: &Value) -> Vec<&str> {
  dataset["columns"]
    .as_array()
    .unwrap()
    .iter()
    .map(|column| column["name"].as_str().unwrap())
    .collect()
}
fn execute(session: &mut Session, source: &str) -> tabdat_runtime::ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}

#[test]
fn projection_fit_predict_persistence_workflow_matches_nested_stream_and_values() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "keep y x id label",
    "drop label label",
    "regress y x",
    "predict fitted",
    "select id fitted y",
    "head 10",
    "save out.parquet, replace",
    "export out.csv, replace",
    "describe",
  ];
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  let results = success(&fixture.batch(&args));
  assert_eq!(results.len(), 10);
  for (index, tag, columns) in [
    (1, "KeepResult", vec!["y", "x", "id", "label"]),
    (2, "DropResult", vec!["y", "x", "id"]),
    (5, "SelectResult", vec!["id", "fitted", "y"]),
  ] {
    assert_eq!(results[index]["schema_version"], 1);
    assert_eq!(results[index]["result_type"], tag);
    let dataset = &results[index]["data"]["dataset"];
    assert_eq!(dataset["path"], "data.csv");
    assert_eq!(dataset["row_count"], 5);
    assert_eq!(names(dataset), columns);
    assert!(dataset["label_metadata"].is_null());
    assert!(dataset["panel_metadata"].is_null());
    assert_eq!(results[index]["data"].as_object().unwrap().len(), 1);
  }
  assert_eq!(results[3]["data"]["observation_count"], 4);
  assert_eq!(
    results[3]["data"]["coefficients"].as_array().unwrap().len(),
    2
  );
  // Independent OLS on x=[1,2,3,4], y=[2,5,5,9]: Sxy=10.5, Sxx=5,
  // slope=2.1 and intercept=5.25-2.1*2.5=0. Projection must not alter rows/sample.
  for (coefficient, expected) in results[3]["data"]["coefficients"]
    .as_array()
    .unwrap()
    .iter()
    .zip([0.0, 2.1])
  {
    assert!((coefficient["value"].as_f64().unwrap() - expected).abs() < 1e-10);
  }
  let rows = results[6]["data"]["rows"].as_array().unwrap();
  assert_eq!(
    rows
      .iter()
      .map(|row| row[0].as_i64().unwrap())
      .collect::<Vec<_>>(),
    [4, 2, 9, 1, 7]
  );
  for (row, expected) in rows.iter().zip([2.1, 4.2, 6.3, 8.4]) {
    assert!((row[1].as_f64().unwrap() - expected).abs() < 1e-10);
  }
  assert!(rows[4][1].is_null());
  assert_eq!(
    rows
      .iter()
      .map(|row| row[2].as_i64().unwrap())
      .collect::<Vec<_>>(),
    [2, 5, 5, 9, 8]
  );
  assert_eq!(results[5]["data"], results[9]["data"]);
  for file in ["out.csv", "out.parquet"] {
    let reload =
      success(&fixture.batch(&["--json", "-c", &format!("use {file}"), "-c", "head 10"]));
    assert_eq!(reload[1]["data"], results[6]["data"]);
    if file == "out.parquet" {
      assert_eq!(
        reload[0]["data"]["dataset"]["columns"],
        results[5]["data"]["dataset"]["columns"]
      );
    }
  }
  fs::write(fixture.0.join("grandchild.td"), commands[5..].join("\n")).unwrap();
  fs::write(
    fixture.0.join("child.td"),
    format!("{}\nrun grandchild.td\n", commands[1..5].join("\n")),
  )
  .unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
}

#[test]
fn quoted_duplicate_and_empty_projections_preserve_native_schema_and_scalar_values() {
  let fixture = Fixture::new();
  for command in ["keep", "select"] {
    let results = success(&fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      &format!("{command} `odd\"名` wide wide label"),
      "-c",
      "head 10",
    ]));
    let dataset = &results[1]["data"]["dataset"];
    assert_eq!(names(dataset), ["odd\"名", "wide", "wide_1", "label"]);
    assert_eq!(
      dataset["columns"],
      json!([
        {"name":"odd\"名","data_type":"VARCHAR"},
        {"name":"wide","data_type":"BIGINT"},
        {"name":"wide_1","data_type":"BIGINT"},
        {"name":"label","data_type":"VARCHAR"}
      ])
    );
    assert_eq!(
      results[2]["data"]["rows"],
      json!([
        ["甲", 9007199254740993_i64, 9007199254740993_i64, "A"],
        ["乙", 9007199254740994_i64, 9007199254740994_i64, null],
        [null, 9007199254740995_i64, 9007199254740995_i64, "B"],
        ["丙", 9007199254740996_i64, 9007199254740996_i64, "A"],
        ["丁", 9007199254740997_i64, 9007199254740997_i64, "C"]
      ])
    );
  }
  for source in ["keep id x id", "select y x", "drop y y"] {
    let results = success(&fixture.batch(&[
      "--json",
      "-c",
      "use empty.csv",
      "-c",
      source,
      "-c",
      "head 10",
    ]));
    assert_eq!(results[1]["data"]["dataset"]["row_count"], 0);
    assert_eq!(results[1]["data"]["dataset"]["path"], "empty.csv");
    assert_eq!(results[2]["data"]["rows"], json!([]));
    let expected = match source {
      "keep id x id" => vec!["id", "x", "id_1"],
      "select y x" => vec!["y", "x"],
      _ => vec!["x", "id"],
    };
    assert_eq!(names(&results[1]["data"]["dataset"]), expected);
  }
}

#[test]
fn projection_errors_and_conditional_preflight_preserve_prior_results_and_stop_files() {
  let fixture = Fixture::new();
  for (source, code, kind) in [
    ("keep x absent", 1, "ExecutionError"),
    ("select absent", 1, "ExecutionError"),
    ("drop x absent", 1, "ExecutionError"),
    ("drop x y id label wide `odd\"名`", 1, "ExecutionError"),
    ("keep", 2, "ParseError"),
    ("drop", 2, "ParseError"),
    ("select", 2, "ParseError"),
    ("select x if x > 0", 2, "ParseError"),
    ("keep if x > 0", 2, "JsonResultUnsupported"),
    ("drop if x == null", 2, "JsonResultUnsupported"),
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
    assert_eq!(
      output.status.code(),
      Some(code),
      "{source}: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 2);
    assert_eq!(results[1]["error"]["type"], kind);
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
    let script_code = if kind == "JsonResultUnsupported" {
      1
    } else {
      code
    };
    assert_eq!(output.status.code(), Some(script_code), "{source}");
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
  for source in ["keep x", "drop x", "select x"] {
    let output = fixture.batch(&["--json", "-c", source]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
      lines(&output.stdout)[0]["error"]["message"]
        .as_str()
        .unwrap()
        .contains("active dataset")
    );
  }
}

#[test]
fn projection_failure_preserves_state_and_success_prunes_labels_without_refitting() {
  let fixture = Fixture::new();
  for projection in ["keep y x id", "select y x id", "drop label wide `odd\"名`"] {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", fixture.0.join("data.csv").display()),
    );
    execute(&mut session, "regress y x");
    execute(&mut session, "label variable y 'Outcome'");
    execute(&mut session, "label variable wide 'Extra'");
    execute(&mut session, "label define ys 2 'two' 5 'five'");
    execute(&mut session, "label values y ys");
    execute(&mut session, "label values wide ys");
    let state = session.state();
    let rows = execute(&mut session, "head 10");
    for source in [
      "keep x absent",
      "select absent",
      "drop x absent",
      "drop x y id label wide `odd\"名`",
    ] {
      assert!(session.execute(parse_command(source).unwrap()).is_err());
      assert_eq!(session.state(), state);
      assert_eq!(execute(&mut session, "head 10"), rows);
    }
    let result = execute(&mut session, projection);
    let before_reporting = session.state();
    let serialized = tabdat_reporting::json_results::serialize_execution_result(&result).unwrap();
    assert_eq!(session.state(), before_reporting);
    assert_eq!(before_reporting.estimation, state.estimation);
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
    assert_eq!(
      before_reporting.labels.variable_labels,
      vec![("y".to_owned(), "Outcome".to_owned())]
    );
    assert_eq!(
      before_reporting.labels.attachments,
      vec![("y".to_owned(), "ys".to_owned())]
    );
    assert_eq!(before_reporting.labels.value_sets, state.labels.value_sets);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert!(value["data"]["dataset"]["label_metadata"].is_null());
    execute(&mut session, "select y id");
    assert_eq!(session.state().estimation, state.estimation);
    execute(&mut session, "lincom x");
    assert_eq!(session.state().estimation, state.estimation);
    assert!(
      session
        .execute(parse_command("predict fitted").unwrap())
        .is_err()
    );
    assert_eq!(session.state().estimation, state.estimation);
  }
}
