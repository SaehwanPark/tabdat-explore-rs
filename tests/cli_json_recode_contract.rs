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
      "tabdat-json-recode-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "x,y,label\n1.0,2,A\n2.0,5,\n3.0,5,A\n4.0,9,C\n,8,B\n5.0,,B\n",
    )
    .unwrap();
    fs::write(root.join("empty.csv"), "label\n").unwrap();
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
fn recode_clean_tabulate_fit_predict_persist_matches_rules_values_and_nested_stream() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "recode x (min/2=1) (2/4=2) (missing=0) (else=3), generate(group)",
    "recode label ('A'='') (missing='unknown'), replace",
    "recode y (2=3) (3=99), replace",
    "head 10",
    "missing label",
    "tabulate group",
    "regress y x",
    "predict fitted",
    "head 10",
    "save recoded.parquet, replace",
    "export recoded.csv, replace",
  ];
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  let results = success(&fixture.batch(&args));
  assert_eq!(results.len(), commands.len());
  for result in &results[1..=3] {
    assert_eq!(result["schema_version"], 1);
    assert_eq!(result["result_type"], "RecodeResult");
    assert_eq!(result["data"]["dataset"]["path"], "data.csv");
    assert_eq!(result["data"]["dataset"]["row_count"], 6);
    assert_eq!(result["data"].as_object().unwrap().len(), 1);
    assert_eq!(
      result["data"]["dataset"]["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect::<Vec<_>>(),
      ["x", "y", "label", "group"]
    );
  }
  assert_eq!(
    results[4]["data"]["rows"],
    json!([
      [1.0, 3, "", 1],
      [2.0, 5, "unknown", 1],
      [3.0, 5, "", 2],
      [4.0, 9, "C", 2],
      [null, 8, "B", 0],
      [5.0, null, "B", 3]
    ])
  );
  // x=2 matches the first overlapping range; y=2 becomes 3, not 99.
  // Stored empty strings remain nonmissing SQL values after replacement.
  assert_eq!(results[5]["data"]["rows"][0]["missing"], 0);
  let table = results[6]["data"]["rows"].as_array().unwrap();
  assert_eq!(
    table.iter().map(|row| row[0].clone()).collect::<Vec<_>>(),
    vec![json!(0), json!(1), json!(2), json!(3)]
  );
  assert_eq!(
    table
      .iter()
      .map(|row| row[1].as_u64().unwrap())
      .collect::<Vec<_>>(),
    [1, 2, 2, 1]
  );
  assert_eq!(results[7]["data"]["observation_count"], 4);
  // Independent OLS on retained x=[1,2,3,4], y=[3,5,5,9]: Sxy=9,
  // Sxx=5, slope=1.8 and intercept=5.5-1.8*2.5=1.
  close(&results[7]["data"]["coefficients"][0]["value"], 1.0);
  close(&results[7]["data"]["coefficients"][1]["value"], 1.8);
  for row in results[9]["data"]["rows"].as_array().unwrap() {
    if row[0].is_null() {
      assert!(row[4].is_null());
    } else {
      close(&row[4], 1.0 + 1.8 * row[0].as_f64().unwrap());
    }
  }
  for path in ["recoded.csv", "recoded.parquet"] {
    let reload =
      success(&fixture.batch(&["--json", "-c", &format!("use {path}"), "-c", "head 10"]));
    if path == "recoded.parquet" {
      assert_eq!(reload[1]["data"], results[9]["data"]);
      assert_eq!(
        reload[0]["data"]["dataset"]["columns"],
        results[8]["data"]["dataset"]["columns"]
      );
    } else {
      // CSV loader interprets empty fields as NULL; it is not a lossless string archive.
      let mut expected = results[9]["data"].clone();
      expected["rows"][0][2] = Value::Null;
      expected["rows"][2][2] = Value::Null;
      assert_eq!(reload[1]["data"], expected);
    }
  }
  fs::write(fixture.0.join("grandchild.td"), commands[7..].join("\n")).unwrap();
  fs::write(
    fixture.0.join("child.td"),
    format!("{}\nrun grandchild.td\n", commands[1..7].join("\n")),
  )
  .unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
}

#[test]
fn multi_column_generation_and_quoted_empty_data_report_actual_native_schema() {
  let fixture = Fixture::new();
  let results = success(&fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "recode y x (missing=0) (nonmissing=1), generate(yflag xflag)",
    "-c",
    "head 10",
  ]));
  assert_eq!(
    results[1]["data"]["dataset"]["columns"]
      .as_array()
      .unwrap()
      .iter()
      .map(|c| c["name"].as_str().unwrap())
      .collect::<Vec<_>>(),
    ["x", "y", "label", "yflag", "xflag"]
  );
  assert_eq!(
    results[1]["data"]["dataset"]["columns"][3]["data_type"],
    "BIGINT"
  );
  assert_eq!(
    results[1]["data"]["dataset"]["columns"][4]["data_type"],
    "DOUBLE"
  );
  let rows = results[2]["data"]["rows"].as_array().unwrap();
  assert_eq!(rows[4][3], 1);
  assert_eq!(rows[4][4], 0.0);
  assert_eq!(rows[5][3], 0);
  assert_eq!(rows[5][4], 1.0);
  let fallback = success(&fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "recode x (else=7) (1=9), generate(choice)",
    "-c",
    "head 10",
  ]));
  assert_eq!(
    fallback[2]["data"]["rows"]
      .as_array()
      .unwrap()
      .iter()
      .map(|row| row[3].clone())
      .collect::<Vec<_>>(),
    vec![json!(9), json!(7), json!(7), json!(7), json!(7), json!(7)]
  );
  let empty = success(&fixture.batch(&[
    "--json",
    "-c",
    "use empty.csv",
    "-c",
    "recode label (missing='M'), generate(`标\"记`)",
    "-c",
    "recode label ('A'='B'), replace",
    "-c",
    "head 10",
  ]));
  for result in &empty[1..=2] {
    assert_eq!(result["data"]["dataset"]["row_count"], 0);
    assert_eq!(result["data"]["dataset"]["columns"][1]["name"], "标\"记");
    assert_eq!(
      result["data"]["dataset"]["columns"][1]["data_type"],
      "VARCHAR"
    );
  }
  assert_eq!(empty[3]["data"]["rows"], json!([]));
}

#[test]
fn recode_validation_and_native_failures_stop_files_and_keep_nested_locations() {
  let fixture = Fixture::new();
  for (source, code) in [
    ("recode absent (1=2), replace", 1),
    ("recode label (1/2=3), replace", 1),
    ("recode x y (1=2), generate(one)", 1),
    ("recode x y (1=2), generate(dup dup)", 1),
    ("recode x (1=2), generate(y)", 1),
    ("recode x (1=2), generate(Y)", 1),
    ("recode x (1=2)", 2),
    ("recode x (1=2), generate(new) replace", 2),
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
    assert_eq!(
      results[1]["error"]["type"],
      if code == 2 {
        "ParseError"
      } else {
        "ExecutionError"
      }
    );
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
    assert_eq!(output.status.code(), Some(code), "{source}");
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
  let output = fixture.batch(&["--json", "-c", "recode x (1=2), replace"]);
  assert_eq!(output.status.code(), Some(1));
}

#[test]
fn recode_failure_state_and_generate_replace_label_cache_rules_are_observable() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(&mut session, "regress y x");
  execute(&mut session, "label variable y 'Outcome'");
  execute(&mut session, "label define ys 2 'two' 5 'five'");
  execute(&mut session, "label values y ys");
  let state = session.state();
  let rows = execute(&mut session, "head 10");
  for source in [
    "recode absent (1=2), replace",
    "recode label (1/2=3), replace",
    "recode x y (1=2), generate(one)",
    "recode x y (1=2), generate(dup dup)",
    "recode x (1=2), generate(y)",
    "recode x (1=2), generate(Y)",
  ] {
    assert!(session.execute(parse_command(source).unwrap()).is_err());
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "head 10"), rows);
  }
  for source in [
    "recode x (missing=0) (else=1), generate(flag)",
    "recode y (2=3) (3=99), replace",
  ] {
    let result = execute(&mut session, source);
    let before = session.state();
    assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
    assert_eq!(session.state(), before);
    assert_eq!(before.estimation, state.estimation);
    if source.ends_with("generate(flag)") {
      assert_eq!(before.labels, state.labels);
    }
  }
  let after = session.state();
  assert_eq!(after.labels.variable_labels, state.labels.variable_labels);
  assert_eq!(after.labels.value_sets, state.labels.value_sets);
  assert!(after.labels.attachments.is_empty());
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
  let old = execute(&mut session, "lincom x");
  let value: Value = serde_json::from_str(
    &tabdat_reporting::json_results::serialize_execution_result(&old).unwrap(),
  )
  .unwrap();
  close(&value["data"]["estimate"], 2.1);
  execute(&mut session, "regress y x");
  let fit = session.last_regression().unwrap();
  assert_eq!(fit.sample.as_ref().unwrap().retained_indices, [0, 1, 2, 3]);
  assert!((fit.coefficients[1].value - 1.8).abs() < 1e-10);
}
