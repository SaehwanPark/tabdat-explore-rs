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
      "tabdat-json-ordering-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "id,key,text,flag,x,y,\"a\"\"名\",__TABDAT_SORT_ORDINAL,-名\n1,2,b,true,3.0,5,30,10,3\n2,1,a,false,1.0,2,10,11,1\n3,1,a,true,2.0,5,20,12,2\n4,,a,false,4.0,9,40,13,4\n5,1,,false,,8,,14,\n6,2,b,true,5.0,,50,15,5\n").unwrap();
    fs::write(
      root.join("empty.csv"),
      "id,key,\"a\"\"名\",__TABDAT_SORT_ORDINAL,-名\n",
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
fn ids(preview: &Value) -> Vec<i64> {
  preview["data"]["rows"]
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
fn execute(session: &mut Session, source: &str) -> tabdat_runtime::ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}

#[test]
fn ordered_predict_export_workflow_preserves_stable_ties_and_matches_nested_stream() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "regress y x",
    "sort key text",
    "head 10",
    "gsort -key +text -flag",
    "head 10",
    "sort key key",
    "head 10",
    "predict fitted",
    "head 10",
    "save sorted.parquet, replace",
    "export sorted.csv, replace",
    "lincom x",
  ];
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  let results = success(&fixture.batch(&args));
  assert_eq!(results.len(), commands.len());
  let baseline = &results[0]["data"]["dataset"];
  for (index, tag) in [(2, "SortResult"), (4, "GsortResult"), (6, "SortResult")] {
    assert_eq!(results[index]["schema_version"], 1);
    assert_eq!(results[index]["result_type"], tag);
    assert_eq!(results[index]["data"], json!({"dataset":baseline}));
  }
  assert_eq!(ids(&results[3]), [2, 3, 5, 1, 6, 4]);
  assert_eq!(ids(&results[5]), [1, 6, 3, 2, 5, 4]);
  assert_eq!(ids(&results[7]), [3, 2, 5, 1, 6, 4]);
  let original = success(&fixture.batch(&["--json", "-c", "use data.csv", "-c", "head 10"]));
  let original_rows = original[1]["data"]["rows"].as_array().unwrap();
  for preview in [&results[3], &results[5], &results[7]] {
    for row in preview["data"]["rows"].as_array().unwrap() {
      assert_eq!(row, &original_rows[row[0].as_u64().unwrap() as usize - 1]);
    }
  }
  assert_eq!(results[1]["data"]["observation_count"], 4);
  // Independent OLS on retained x=[3,1,2,4], y=[5,2,5,9]: Sxy=10.5,
  // Sxx=5, slope=2.1 and intercept=0. Sorting does not reinterpret the fit.
  close(&results[1]["data"]["coefficients"][0]["value"], 0.0);
  close(&results[1]["data"]["coefficients"][1]["value"], 2.1);
  close(&results[12]["data"]["estimate"], 2.1);
  assert_eq!(ids(&results[9]), [3, 2, 5, 1, 6, 4]);
  for row in results[9]["data"]["rows"].as_array().unwrap() {
    if row[4].is_null() {
      assert!(row[9].is_null());
    } else {
      close(&row[9], 2.1 * row[4].as_f64().unwrap());
    }
  }
  for path in ["sorted.csv", "sorted.parquet"] {
    let reload =
      success(&fixture.batch(&["--json", "-c", &format!("use {path}"), "-c", "head 10"]));
    assert_eq!(reload[1]["data"], results[9]["data"]);
    if path == "sorted.parquet" {
      assert_eq!(
        reload[0]["data"]["dataset"]["columns"],
        results[8]["data"]["dataset"]["columns"]
      );
    }
  }
  fs::write(fixture.0.join("grandchild.td"), commands[8..].join("\n")).unwrap();
  fs::write(
    fixture.0.join("child.td"),
    format!("{}\nrun grandchild.td\n", commands[2..8].join("\n")),
  )
  .unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    format!("{}\nrun child.td\n", commands[..2].join("\n")),
  )
  .unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
  let preordered = success(&fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "gsort -key +text -flag",
    "-c",
    "regress y x",
    "-c",
    "predict fitted",
    "-c",
    "head 10",
  ]));
  assert_eq!(preordered[2]["data"]["observation_count"], 4);
  close(&preordered[2]["data"]["coefficients"][0]["value"], 0.0);
  close(&preordered[2]["data"]["coefficients"][1]["value"], 2.1);
  assert_eq!(ids(&preordered[4]), [1, 6, 3, 2, 5, 4]);
  for row in preordered[4]["data"]["rows"].as_array().unwrap() {
    if row[4].is_null() {
      assert!(row[9].is_null());
    } else {
      close(&row[9], 2.1 * row[4].as_f64().unwrap());
    }
  }
}

#[test]
fn quoted_repeated_signed_and_empty_keys_do_not_leak_ordinal_or_change_schema() {
  let fixture = Fixture::new();
  // The quoted literal -名 key is ascending, not a descending prefix.
  for source in ["sort `a\"名` `a\"名`", "gsort `-名`"] {
    let results = success(&fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      source,
      "-c",
      "head 10",
    ]));
    assert_eq!(ids(&results[2]), [2, 3, 1, 4, 6, 5]);
    assert_eq!(results[1]["data"]["dataset"], results[0]["data"]["dataset"]);
  }
  for source in ["sort key key", "gsort -key", "sort `a\"名`", "gsort `-名`"] {
    let results = success(&fixture.batch(&[
      "--json",
      "-c",
      "use empty.csv",
      "-c",
      source,
      "-c",
      "head 10",
    ]));
    assert_eq!(results[1]["data"]["dataset"], results[0]["data"]["dataset"]);
    assert_eq!(results[1]["data"]["dataset"]["row_count"], 0);
    assert_eq!(results[2]["data"]["rows"], json!([]));
  }
}

#[test]
fn sort_failures_and_prefix_preflight_keep_locations_results_and_later_files_untouched() {
  let fixture = Fixture::new();
  for (source, code, kind) in [
    ("sort key absent", 1, "ExecutionError"),
    ("gsort -absent +key", 1, "ExecutionError"),
    ("sort", 2, "ParseError"),
    ("gsort", 2, "ParseError"),
    ("gsort -", 2, "ParseError"),
    ("gsort --key", 2, "ParseError"),
    ("sort key, stable", 2, "ParseError"),
    ("sort key if x > 0", 2, "ParseError"),
    ("gsort -key if x > 0", 2, "ParseError"),
    ("by key: sort x", 2, "JsonResultUnsupported"),
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
    assert_eq!(
      output.status.code(),
      Some(if kind == "JsonResultUnsupported" {
        1
      } else {
        code
      }),
      "{source}"
    );
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
  for source in ["sort key", "gsort -key"] {
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
fn ordering_keeps_labels_and_owned_fit_sample_while_prediction_uses_current_row_order() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(&mut session, "regress y x");
  execute(&mut session, "label variable key 'Grouping'");
  execute(&mut session, "label define groups 1 'one' 2 'two'");
  execute(&mut session, "label values key groups");
  let state = session.state();
  let original = execute(&mut session, "head 10");
  for source in ["sort key absent", "gsort -absent +key"] {
    assert!(session.execute(parse_command(source).unwrap()).is_err());
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "head 10"), original);
  }
  for (source, expected) in [
    ("sort key text", vec![2, 3, 5, 1, 6, 4]),
    ("gsort -key +text -flag", vec![1, 6, 3, 2, 5, 4]),
  ] {
    let result = execute(&mut session, source);
    let before_reporting = session.state();
    assert_eq!(before_reporting, state);
    let serialized = tabdat_reporting::json_results::serialize_execution_result(&result).unwrap();
    assert_eq!(session.state(), before_reporting);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert!(value["data"]["dataset"]["label_metadata"].is_null());
    let preview = execute(&mut session, "head 10");
    let value: Value = serde_json::from_str(
      &tabdat_reporting::json_results::serialize_execution_result(&preview).unwrap(),
    )
    .unwrap();
    assert_eq!(ids(&value), expected);
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
    execute(&mut session, "lincom x");
    assert_eq!(session.state(), state);
  }
  execute(&mut session, "predict fitted");
  assert_eq!(session.state().estimation, state.estimation);
  assert_eq!(session.state().labels, state.labels);
  let result = execute(&mut session, "head 10");
  let value: Value = serde_json::from_str(
    &tabdat_reporting::json_results::serialize_execution_result(&result).unwrap(),
  )
  .unwrap();
  assert_eq!(ids(&value), [1, 6, 3, 2, 5, 4]);
  for row in value["data"]["rows"].as_array().unwrap() {
    if row[4].is_null() {
      assert!(row[9].is_null());
    } else {
      close(&row[9], 2.1 * row[4].as_f64().unwrap());
    }
  }
  execute(&mut session, "regress y x");
  // A new fit uses sorted positions, unlike the preserved original fit above.
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 2, 3, 5]
  );
}
