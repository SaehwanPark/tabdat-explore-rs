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
use tabdat_runtime::{ExecutionResult, Session};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-collapse-count-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "g,x,y\n3,1,1\n1,2,\n2,,3\n1,4,5\n3,6,7\n4,,8\n2,9,10\n3,11,12\n,13,\n",
    )
    .unwrap();
    Self(root)
  }
  fn run(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn batch(&self, commands: &[&str]) -> Vec<Value> {
    let mut args = vec!["--json"];
    for source in commands {
      args.extend(["-c", source]);
    }
    success(self.run(&args))
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
fn lines(bytes: &[u8]) -> Vec<Value> {
  std::str::from_utf8(bytes)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn success(output: Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  lines(&output.stdout)
}
fn expected_rows() -> Value {
  json!([[1, 2, 1], [2, 1, 2], [3, 3, 3], [4, 0, 1], [null, 1, 0]])
}
#[test]
fn exact_count_metadata_rows_and_strict_admission() {
  let fixture = Fixture::new();
  let results = fixture.batch(&[
    "use data.csv",
    "by g: count",
    "collapse count x y, by(g)",
    "head 10",
  ]);
  assert_eq!(
    results[1]["data"]["rows"],
    json!([[1, 2], [2, 2], [3, 3], [4, 1], [null, 1]])
  );
  assert_eq!(results[2]["schema_version"], 1);
  assert_eq!(results[2]["result_type"], "CollapseResult");
  let mut expected = results[0]["data"]["dataset"].clone();
  expected["row_count"] = json!(5);
  expected["columns"] = json!([{"name":"g","data_type":"BIGINT"},{"name":"count_x","data_type":"BIGINT"},{"name":"count_y","data_type":"BIGINT"}]);
  assert_eq!(results[2]["data"], json!({"dataset":expected}));
  assert_eq!(results[3]["data"]["rows"], expected_rows());
  let mut dataset = fixture.session().active_dataset().unwrap().clone();
  dataset.row_count = u64::MAX;
  dataset.columns[0].name = "group\n\u{1b}雪".into();
  let owned = ExecutionResult::Collapse(tabdat_runtime::CollapseResult { dataset });
  let serialized = tabdat_reporting::json_results::serialize_execution_result(&owned).unwrap();
  assert_eq!(serialized.lines().count(), 1);
  assert_eq!(value(&owned)["data"]["dataset"]["row_count"], u64::MAX);
  assert_eq!(
    value(&owned)["data"]["dataset"]["columns"][0]["name"],
    "group\n\u{1b}雪"
  );
  assert_eq!(
    tabdat_reporting::terminal::render_execution_result(&owned),
    format!("Collapsed: {} rows, 3 columns\n", u64::MAX)
  );
  let grouped = parse_command("by g: collapse count x, by(g)").unwrap();
  assert!(!tabdat_reporting::json_results::supports_batch_command(
    &grouped
  ));
  assert!(!tabdat_reporting::json_results::supports_script_command(
    &grouped
  ));
  for statistic in ["count", "mean", "sum", "min", "max"] {
    let command = parse_command(&format!("collapse {statistic} x, by(g)")).unwrap();
    assert_eq!(
      tabdat_reporting::json_results::supports_batch_command(&command),
      matches!(statistic, "count" | "mean")
    );
    assert_eq!(
      tabdat_reporting::json_results::supports_script_command(&command),
      matches!(statistic, "count" | "mean")
    );
  }
}
#[test]
fn label_pruning_named_sync_cached_original_sample_and_explicit_refit_workflow() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(&mut session, "sql select * from active into cohort");
  execute(&mut session, "label variable g 'Group'");
  execute(&mut session, "label variable x 'Source predictor'");
  execute(&mut session, "label define groups 1 'First' 2 'Second'");
  execute(&mut session, "label values g groups");
  execute(&mut session, "regress y x");
  let cached = session.last_regression().unwrap().clone();
  let inference = execute(&mut session, "lincom x");
  let result = execute(&mut session, "collapse count x y, by(g)");
  assert_eq!(value(&result)["data"]["dataset"]["row_count"], 5);
  assert_eq!(session.active_table_name(), Some("cohort"));
  assert_eq!(session.last_regression().unwrap(), &cached);
  assert_eq!(
    cached.sample.as_ref().unwrap().retained_indices,
    [0, 3, 4, 6, 7]
  );
  assert_eq!(execute(&mut session, "lincom x"), inference);
  assert!(
    session
      .execute(parse_command("predict old").unwrap())
      .is_err()
  );
  let labels = session.active_label_metadata().unwrap();
  assert_eq!(labels.variable_labels, [("g".into(), "Group".into())]);
  assert_eq!(labels.attachments, [("g".into(), "groups".into())]);
  assert_eq!(labels.value_sets.len(), 1);
  let preview = execute(&mut session, "head 10");
  assert_eq!(value(&preview)["data"]["rows"], expected_rows());
  execute(&mut session, "use cohort");
  assert_eq!(execute(&mut session, "head 10"), preview);
  let fit = value(&execute(&mut session, "regress count_y count_x"));
  // Hand-derived OLS: xbar=ybar=7/5, centered Sxx=26/5, Sxy=16/5.
  for (actual, expected) in [
    (&fit["data"]["coefficients"][0]["value"], 7.0 / 13.0),
    (&fit["data"]["coefficients"][1]["value"], 8.0 / 13.0),
  ] {
    assert!((actual.as_f64().unwrap() - expected).abs() < 1e-10);
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
  execute(&mut session, "predict fitted");
  execute(
    &mut session,
    &format!("save {}", fixture.0.join("result.parquet").display()),
  );
  execute(
    &mut session,
    &format!("export {}", fixture.0.join("result.csv").display()),
  );
  let before = execute(&mut session, "head 10");
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("result.parquet").display()),
  );
  assert_eq!(execute(&mut session, "head 10"), before);
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("result.csv").display()),
  );
  assert_eq!(execute(&mut session, "head 10"), before);
}
#[test]
fn mean_mutation_preserves_named_labels_cached_fit_and_recovers_from_validation_and_staging_errors()
{
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(
    &mut session,
    "sql select *, g as mean_x, 'bad' as text from active into cohort",
  );
  execute(&mut session, "label variable g 'Group'");
  execute(&mut session, "label variable x 'Source predictor'");
  execute(&mut session, "label define groups 1 'First' 2 'Second'");
  execute(&mut session, "label values g groups");
  execute(&mut session, "regress y x");
  let cached = session.last_regression().unwrap().clone();
  let inference = execute(&mut session, "lincom x");
  let state = session.state();
  let labels = session.active_label_metadata().unwrap().clone();
  let before = execute(&mut session, "head 10");
  let signature = execute(&mut session, "datasignature");
  for source in [
    "collapse mean absent, by(g)",
    "collapse mean x, by(absent)",
    "collapse mean text, by(g)",
    "collapse mean x x, by(g)",
    "collapse mean x, by(g g)",
    "collapse mean x, by(mean_x)",
  ] {
    assert!(
      session.execute(parse_command(source).unwrap()).is_err(),
      "{source}"
    );
    assert_eq!(session.state(), state);
    assert_eq!(session.active_label_metadata().unwrap(), &labels);
    assert_eq!(execute(&mut session, "head 10"), before);
    assert_eq!(execute(&mut session, "datasignature"), signature);
    assert_eq!(execute(&mut session, "lincom x"), inference);
    assert_eq!(session.last_regression().unwrap(), &cached);
    fs::write(
      fixture.0.join("child.td"),
      format!("{source}\nexport forbidden.csv\n"),
    )
    .unwrap();
    fs::write(fixture.0.join("parent.td"), "use data.csv\nsql select *, g as mean_x, 'bad' as text from active into cohort\nrun child.td\n").unwrap();
    let output = fixture.run(&["--json", "-f", "parent.td"]);
    assert_eq!(output.status.code(), Some(1), "{source}");
    assert!(
      String::from_utf8(output.stderr)
        .unwrap()
        .contains("child.td:1:")
    );
    assert!(
      lines(&output.stdout)
        .iter()
        .all(|v| v["result_type"] != "CollapseResult")
    );
    assert!(!fixture.0.join("forbidden.csv").exists());
  }
  let mut expected = value(&execute(&mut session, "describe"))["data"]["dataset"].clone();
  let result = execute(&mut session, "collapse mean y x, by(g)");
  let metadata = value(&result);
  assert_eq!(metadata["schema_version"], 1);
  assert_eq!(metadata["result_type"], "CollapseResult");
  // The aggregate keeps its original source; only the native schema and row count change.
  expected["row_count"] = json!(5);
  expected["columns"] = json!([
    {"name":"g","data_type":"BIGINT"},
    {"name":"mean_y","data_type":"DOUBLE"},
    {"name":"mean_x","data_type":"DOUBLE"}
  ]);
  assert_eq!(metadata["data"], json!({"dataset":expected}));
  assert_eq!(session.active_table_name(), Some("cohort"));
  assert_eq!(session.last_regression().unwrap(), &cached);
  assert_eq!(
    cached.sample.as_ref().unwrap().retained_indices,
    [0, 3, 4, 6, 7]
  );
  assert_eq!(execute(&mut session, "lincom x"), inference);
  assert!(
    session
      .execute(parse_command("predict old").unwrap())
      .is_err()
  );
  assert_eq!(session.last_regression().unwrap(), &cached);
  let labels = session.active_label_metadata().unwrap();
  assert_eq!(labels.variable_labels, [("g".into(), "Group".into())]);
  assert_eq!(labels.attachments, [("g".into(), "groups".into())]);
  assert_eq!(labels.value_sets.len(), 1);
  let preview = execute(&mut session, "head 10");
  assert_eq!(
    value(&preview)["data"]["rows"],
    json!([
      [1, 5.0, 3.0],
      [2, 6.5, 9.0],
      [3, 20.0 / 3.0, 6.0],
      [4, 8.0, null],
      [null, null, 13.0]
    ])
  );
  execute(&mut session, "use cohort");
  assert_eq!(execute(&mut session, "head 10"), preview);
  let fit = value(&execute(&mut session, "regress mean_y mean_x"));
  // Three complete group means: (3,5), (9,6.5), (6,20/3); Sxx=18, Sxy=9/2.
  for (actual, expected) in [
    (&fit["data"]["coefficients"][0]["value"], 41.0 / 9.0),
    (&fit["data"]["coefficients"][1]["value"], 1.0 / 4.0),
  ] {
    assert!((actual.as_f64().unwrap() - expected).abs() < 1e-10);
  }
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
  execute(&mut session, "predict fitted");
  execute(
    &mut session,
    &format!("save {}", fixture.0.join("means.parquet").display()),
  );
  execute(
    &mut session,
    &format!("export {}", fixture.0.join("means.csv").display()),
  );
  let before = execute(&mut session, "head 10");
  for file in ["means.parquet", "means.csv"] {
    execute(
      &mut session,
      &format!("use {}", fixture.0.join(file).display()),
    );
    assert_eq!(execute(&mut session, "head 10"), before);
  }
}
#[test]
fn mean_preserves_requested_multikey_order_wide_keys_and_native_decimal_output_schema() {
  let fixture = Fixture::new();
  let results = fixture.batch(&[
    "use data.csv",
    "sql select * from (values (9007199254740993::bigint, 2, 1.25::decimal(8,2)), (9007199254740993::bigint, 2, 3.75::decimal(8,2)), (9007199254740993::bigint, 1, 3.50::decimal(8,2)), (-9007199254740993::bigint, 2, NULL), (NULL, 1, 2.75::decimal(8,2))) as t(g,h,x) into edge",
    "collapse mean x, by(h g)",
    "head 10",
  ]);
  assert_eq!(results[2]["result_type"], "CollapseResult");
  assert_eq!(
    results[2]["data"]["dataset"]["columns"],
    json!([
      {"name":"h","data_type":"INTEGER"}, {"name":"g","data_type":"BIGINT"},
      {"name":"mean_x","data_type":"DOUBLE"}
    ])
  );
  assert_eq!(results[2]["data"]["dataset"]["row_count"], 4);
  assert_eq!(results[3]["data"]["columns"], json!(["h", "g", "mean_x"]));
  assert_eq!(
    results[3]["data"]["rows"],
    json!([
      [1, 9007199254740993_i64, 3.5],
      [1, null, 2.75],
      [2, -9007199254740993_i64, null],
      [2, 9007199254740993_i64, 2.5]
    ])
  );
}
#[test]
fn native_null_nonfinite_empty_quoted_multiple_keys_and_wide_values() {
  let fixture = Fixture::new();
  let query = "sql select * from (values ('雪', '', cast('NaN' as double), 9007199254740993::bigint), ('雪', '', cast('Infinity' as double), NULL), ('雪',NULL,NULL,NULL), (NULL, '', cast('-Infinity' as double), -9007199254740993::bigint)) as t(\"group 雪\",s,n,w) into edge";
  let plain_query = query.replace("\"group 雪\"", "g");
  let results = fixture.batch(&[
    "use data.csv",
    &plain_query,
    "collapse count s n w, by(g s)",
    "head 10",
  ]);
  assert_eq!(
    results[3]["data"]["columns"],
    json!(["g", "s", "count_s", "count_n", "count_w"])
  );
  assert_eq!(
    results[3]["data"]["rows"],
    json!([
      ["雪", "", 2, 2, 1],
      ["雪", null, 0, 0, 0],
      [null, "", 1, 1, 1]
    ])
  );
  // The typed boundary supports quoted names; the existing by() parser does not.
  assert!(parse_command("collapse count s, by(\"group 雪\")").is_err());
  let mut session = fixture.session();
  execute(&mut session, query);
  let result = session
    .execute(tabdat_language::Command::Collapse {
      command: tabdat_domain::CollapseCommand {
        statistic: tabdat_domain::CollapseStatistic::Count,
        variables: vec!["s".into(), "n".into(), "w".into()],
        groups: vec!["group 雪".into(), "s".into()],
      },
    })
    .unwrap();
  assert_eq!(
    value(&result)["data"]["dataset"]["columns"][0]["name"],
    "group 雪"
  );
  assert_eq!(
    value(&execute(&mut session, "head 10"))["data"]["rows"],
    results[3]["data"]["rows"]
  );
  let wide = fixture.batch(&[
    "use data.csv",
    &plain_query,
    "collapse count n, by(w)",
    "head 10",
  ]);
  assert_eq!(
    wide[3]["data"]["rows"],
    json!([
      [-9007199254740993_i64, 1],
      [9007199254740993_i64, 1],
      [null, 1]
    ])
  );
  let empty = fixture.batch(&[
    "use data.csv",
    "sql select * from active where false into empty",
    "collapse count x y, by(g)",
    "head 10",
  ]);
  assert_eq!(empty[2]["data"]["dataset"]["row_count"], 0);
  assert_eq!(empty[3]["data"]["rows"], json!([]));
}
#[test]
fn validation_staging_failures_preserve_rows_labels_named_cache_and_stop_script_files() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(&mut session, "generate count_x = g");
  execute(&mut session, "sql select * from active into cohort");
  execute(&mut session, "label variable g 'Group'");
  execute(&mut session, "regress y x");
  let state = session.state();
  let labels = session.active_label_metadata().unwrap().clone();
  let preview = execute(&mut session, "head 10");
  let signature = execute(&mut session, "datasignature");
  let inference = execute(&mut session, "lincom x");
  for source in [
    "collapse count absent, by(g)",
    "collapse count x, by(absent)",
    "collapse count x x, by(g)",
    "collapse count x, by(g g)",
    "collapse count x, by(count_x)",
  ] {
    assert!(
      session.execute(parse_command(source).unwrap()).is_err(),
      "{source}"
    );
    assert_eq!(session.state(), state);
    assert_eq!(session.active_label_metadata().unwrap(), &labels);
    assert_eq!(execute(&mut session, "head 10"), preview);
    assert_eq!(execute(&mut session, "datasignature"), signature);
    assert_eq!(execute(&mut session, "lincom x"), inference);
    fs::write(
      fixture.0.join("child.td"),
      format!("{source}\nexport forbidden.csv\n"),
    )
    .unwrap();
    fs::write(
      fixture.0.join("parent.td"),
      "use data.csv\ngenerate count_x = g\nrun child.td\n",
    )
    .unwrap();
    let output = fixture.run(&["--json", "-f", "parent.td"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
      String::from_utf8(output.stderr)
        .unwrap()
        .contains("child.td:1:")
    );
    assert!(
      lines(&output.stdout)
        .iter()
        .all(|v| v["result_type"] != "CollapseResult")
    );
    assert!(!fixture.0.join("forbidden.csv").exists());
  }
  for statistic in ["sum", "min", "max"] {
    let source = format!("collapse {statistic} x, by(g)");
    let output = fixture.run(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      &source,
      "-c",
      "export forbidden.csv",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!fixture.0.join("forbidden.csv").exists());
    let emitted = lines(&output.stdout);
    assert_eq!(emitted.len(), 2);
    assert_eq!(emitted[0]["result_type"], "LoadResult");
    assert_eq!(emitted[1]["error"]["type"], "JsonResultUnsupported");
  }
  let output = fixture.run(&["--json", "-c", "collapse count x, by(g)"]);
  assert_eq!(output.status.code(), Some(1));
  for source in [
    "collapse count x, by()",
    "collapse count, by(g)",
    "collapse count x if x > 0, by(g)",
    "collapse count x, by(g) extra",
    "collapse mean x, by()",
    "collapse mean, by(g)",
    "collapse mean x if x > 0, by(g)",
    "collapse mean x, by(g) extra",
  ] {
    let output = fixture.run(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      source,
      "-c",
      "export forbidden.csv",
    ]);
    assert_eq!(output.status.code(), Some(2), "{source}");
    assert!(!fixture.0.join("forbidden.csv").exists());
    let emitted = lines(&output.stdout);
    assert_eq!(emitted.len(), 2);
    assert_eq!(emitted[1]["error"]["type"], "ParseError");
  }
}
#[test]
fn batch_nested_and_terminal_collapsed_dataset_workflows_agree() {
  for statistic in ["count", "mean"] {
    let fixture = Fixture::new();
    let collapse = format!("collapse {statistic} x y, by(g)");
    let regress = format!("regress {statistic}_y {statistic}_x");
    let commands = [
      "use data.csv",
      &collapse,
      "head 10",
      &regress,
      "predict fitted",
      "export nested.csv",
    ];
    let expected = fixture.batch(&commands);
    fs::remove_file(fixture.0.join("nested.csv")).unwrap();
    fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
    fs::write(fixture.0.join("child.td"), commands[1..].join("\n")).unwrap();
    assert_eq!(
      success(fixture.run(&["--json", "-f", "parent.td"])),
      expected
    );
    fs::remove_file(fixture.0.join("nested.csv")).unwrap();
    assert_eq!(fixture.batch(&["run parent.td"]), expected);
    let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&fixture.0)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    child
      .stdin
      .take()
      .unwrap()
      .write_all(
        format!("use data.csv\ncollapse {statistic} x x, by(g)\n{collapse}\nhead 10\nexit\n")
          .as_bytes(),
      )
      .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Collapsed"));
    assert!(text.contains(&format!("{statistic}_x")));
    assert!(text.contains(&format!("{statistic}_y")));
    assert!(!text.contains("terminal details"));
    assert!(!output.stderr.is_empty());
  }
}
