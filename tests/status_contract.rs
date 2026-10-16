#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{ExecutionResult, StatusResult};
use tabdat_execution::Session;
use tabdat_language::parse_command;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-status-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(
      path.join("data.csv"),
      "x,y,w,g\n0,1,1,a\n1,2,2,a\n2,4,3,b\n3,6,4,b\n",
    )
    .unwrap();
    Self(path)
  }
  fn batch(&self, args: &[&str]) -> std::process::Output {
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
fn execute(session: &mut Session, source: &str) -> ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}
fn status(session: &mut Session) -> StatusResult {
  let ExecutionResult::Status(status) = execute(session, "status") else {
    panic!("status")
  };
  status
}
fn encoded(status: &StatusResult) -> Value {
  serde_json::from_str(
    &tabdat_reporting::json_results::serialize_execution_result(&ExecutionResult::Status(
      status.clone(),
    ))
    .unwrap(),
  )
  .unwrap()
}

#[test]
fn empty_status_is_read_only_backend_free_and_reports_explicit_absence() {
  let mut session = Session::new();
  let state = session.state();
  let result = status(&mut session);
  assert!(!result.data_initialized);
  assert!(session.initialized_capabilities().is_empty());
  assert_eq!(session.state(), state);
  assert_eq!(
    encoded(&result),
    json!({"schema_version":1,"result_type":"StatusResult","data":{
      "data_initialized":false,"dataset":null,"active_table_name":null,"named_tables":[],"estimation_family":null,"linear_model":null
    }})
  );
  let text = tabdat_reporting::terminal::render_execution_result(&ExecutionResult::Status(result));
  assert_eq!(
    text,
    "Session status\nData engine initialized: false\nActive dataset: none\nActive named table: none\nRegistered named tables: []\nRetained estimation family: none\nRetained linear model: none\n"
  );
}

#[test]
fn status_distinguishes_current_schema_and_rows_from_retained_original_fit() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(&mut session, "sql SELECT * FROM active into zeta");
  execute(&mut session, "sql SELECT * FROM active into alpha");
  execute(&mut session, "regress y x if x >= 1");
  execute(&mut session, "rename x current_x");
  execute(&mut session, "keep if current_x >= 2");
  execute(&mut session, "label variable current_x 'Current X'");
  let before = session.state();
  let result = status(&mut session);
  assert_eq!(session.state(), before);
  assert!(result.data_initialized);
  assert_eq!(result.active_table_name.as_deref(), Some("alpha"));
  assert_eq!(result.named_tables, ["alpha", "zeta"]);
  let dataset = result.dataset.as_ref().unwrap();
  assert_eq!(dataset.row_count, 2);
  assert_eq!(dataset.columns[0].name, "current_x");
  let model = result.linear_model.as_ref().unwrap();
  assert_eq!(model.outcome, "y");
  assert_eq!(model.predictors, ["x"]);
  assert!(model.include_intercept);
  assert_eq!(model.observation_count, 3);
  assert_eq!(model.degrees_of_freedom, 1);
  assert_eq!(model.sample_total_observations, Some(4));
  assert_eq!(model.sample_dropped_observations, Some(1));
  let value = encoded(&result);
  let model = value["data"]["linear_model"].as_object().unwrap();
  assert_eq!(model.len(), 7);
  for absent in [
    "estimator",
    "covariance",
    "coefficients",
    "retained_indices",
    "predictor_design",
    "weights",
    "last_operation",
  ] {
    assert!(!model.contains_key(absent));
  }
  assert!(
    session
      .execute(parse_command("predict missing").unwrap())
      .is_err()
  );
  assert_eq!(status(&mut session), result);
  execute(&mut session, "rename current_x x");
  execute(&mut session, "predict recovered");
  assert_eq!(status(&mut session).linear_model, result.linear_model);
  assert!(
    session
      .execute(parse_command("regress y absent").unwrap())
      .is_err()
  );
  let after = status(&mut session);
  assert!(after.linear_model.is_none());
  assert_eq!(after.dataset.as_ref().unwrap().row_count, 2);
}

#[test]
fn status_uses_actual_fit_metadata_without_inventing_estimator_or_covariance_tracking() {
  let fixture = Fixture::new();
  for options in [
    "",
    ", robust",
    ", cluster(g)",
    ", wls(w)",
    ", gls(w)",
    ", noconstant",
  ] {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", fixture.0.join("data.csv").display()),
    );
    execute(&mut session, &format!("regress y x{options}"));
    let before = session.state();
    let result = status(&mut session);
    let model = result.linear_model.unwrap();
    let fit = session.last_regression().unwrap();
    assert_eq!(
      model.observation_count,
      fit.fit_statistics.observation_count
    );
    assert_eq!(
      model.degrees_of_freedom,
      fit.fit_statistics.degrees_of_freedom
    );
    assert_eq!(model.include_intercept, options != ", noconstant");
    assert_eq!(model.sample_total_observations, Some(4));
    assert_eq!(model.sample_dropped_observations, Some(0));
    assert_eq!(session.state(), before);
  }
  let output = fixture.batch(&[
    "--json",
    "-c",
    "status",
    "-c",
    "use data.csv",
    "-c",
    "status",
  ]);
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let values: Vec<Value> = std::str::from_utf8(&output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  assert_eq!(values.len(), 3);
  assert_eq!(values[0]["data"]["data_initialized"], false);
  assert_eq!(values[2]["data"]["data_initialized"], true);
}

fn envelopes(output: &std::process::Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  std::str::from_utf8(&output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

#[test]
fn status_recovery_prediction_export_agree_across_batch_nested_streams_text_and_shell() {
  let fixture = Fixture::new();
  let commands = [
    "status",
    "use data.csv",
    "sql SELECT * FROM active into sample",
    "regress y x if x >= 1",
    "rename x current_x",
    "status",
    "rename current_x x",
    "predict fitted",
    "status",
    "export result.csv",
    "head 4",
  ];
  fs::write(
    fixture.0.join("parent.td"),
    "status\nuse data.csv\nrun child.td\n",
  )
  .unwrap();
  fs::write(fixture.0.join("child.td"), commands[2..].join("\n")).unwrap();
  let mut reference = None;
  for route in ["batch", "file", "run"] {
    let args: Vec<_> = if route == "batch" {
      std::iter::once("--json")
        .chain(commands.iter().flat_map(|command| ["-c", command]))
        .collect()
    } else if route == "file" {
      vec!["--json", "-f", "parent.td"]
    } else {
      vec!["--json", "-c", "run parent.td"]
    };
    let values = envelopes(&fixture.batch(&args));
    assert_eq!(values.len(), commands.len());
    assert_eq!(values[0]["data"]["data_initialized"], false);
    assert_eq!(values[5]["result_type"], "StatusResult");
    let data = &values[5]["data"];
    assert_eq!(data["active_table_name"], "sample");
    assert_eq!(data["named_tables"], json!(["sample"]));
    assert_eq!(data["dataset"]["columns"][0]["name"], "current_x");
    assert_eq!(data["linear_model"]["predictors"], json!(["x"]));
    assert_eq!(data["linear_model"]["observation_count"], 3);
    assert_eq!(data["linear_model"]["sample_total_observations"], 4);
    assert_eq!(values[8]["data"]["linear_model"], data["linear_model"]);
    assert_eq!(values[8]["data"]["dataset"]["columns"][4]["name"], "fitted");
    let csv = fs::read_to_string(fixture.0.join("result.csv")).unwrap();
    let rows: Vec<_> = csv.lines().collect();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0], "x,y,w,g,fitted");
    for (index, row) in rows[1..].iter().enumerate() {
      let fitted: f64 = row.rsplit(',').next().unwrap().parse().unwrap();
      // The three retained points lie exactly on y=2*x; status does not change fitting.
      assert!((fitted - 2.0 * index as f64).abs() < 1e-10);
    }
    if let Some((expected_values, expected_csv)) = &reference {
      assert_eq!(&values, expected_values, "{route}");
      assert_eq!(&csv, expected_csv, "{route}");
    } else {
      reference = Some((values, csv));
    }
    fs::remove_file(fixture.0.join("result.csv")).unwrap();
  }
  let expected_csv = reference.unwrap().1;
  let text = fixture.batch(&["--text", "-f", "parent.td"]);
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(text.stderr.is_empty());
  let stdout = String::from_utf8(text.stdout).unwrap();
  assert_eq!(stdout.matches("Session status\n").count(), 3);
  assert!(stdout.contains("  \"current_x\": "));
  assert!(stdout.contains("  \"fitted\": "));
  assert!(stdout.contains("Retained N = 3; residual df = 1"));
  assert!(stdout.contains("not current-data prediction eligibility"));
  assert_eq!(
    fs::read_to_string(fixture.0.join("result.csv")).unwrap(),
    expected_csv
  );
  fs::remove_file(fixture.0.join("result.csv")).unwrap();
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
    .write_all(format!("{}\nexit\n", commands.join("\n")).as_bytes())
    .unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(output.status.success());
  assert!(
    output.stderr.is_empty(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert_eq!(
    String::from_utf8(output.stdout)
      .unwrap()
      .matches("Session status\n")
      .count(),
    3
  );
  assert_eq!(
    fs::read_to_string(fixture.0.join("result.csv")).unwrap(),
    expected_csv
  );
  let quiet = fixture.batch(&["-c", "status"]);
  assert!(quiet.status.success());
  assert!(quiet.stdout.is_empty());
  assert!(quiet.stderr.is_empty());
}

#[test]
fn failed_load_and_partial_script_keep_honest_status_and_permit_explicit_recovery() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("partial.td"),
    "rename x partial_x\npredict broken\n",
  )
  .unwrap();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(&mut session, "regress y x");
  let model = status(&mut session).linear_model;
  let error = session
    .execute(parse_command(&format!("run {}", fixture.0.join("partial.td").display())).unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("predict unknown variable: x"));
  let before = session.state();
  let partial = status(&mut session);
  assert_eq!(session.state(), before);
  assert_eq!(
    partial.dataset.as_ref().unwrap().columns[0].name,
    "partial_x"
  );
  assert_eq!(partial.linear_model, model);
  assert!(
    session
      .execute(parse_command(&format!("use {}", fixture.0.join("missing.csv").display())).unwrap())
      .is_err()
  );
  assert_eq!(session.state(), before);
  assert_eq!(status(&mut session), partial);
  execute(&mut session, "rename partial_x x");
  execute(&mut session, "predict recovered");
  assert_eq!(status(&mut session).linear_model, model);
  fs::write(
    fixture.0.join("parent.td"),
    "status\nuse data.csv\nregress y x\nrun partial.td\nstatus\nexport never.csv\n",
  )
  .unwrap();
  for args in [
    vec!["--json", "-f", "parent.td"],
    vec!["--json", "-c", "run parent.td"],
  ] {
    let output = fixture.batch(&args);
    assert_eq!(output.status.code(), Some(1));
    let values: Vec<Value> = std::str::from_utf8(&output.stdout)
      .unwrap()
      .lines()
      .map(|line| serde_json::from_str(line).unwrap())
      .collect();
    assert_eq!(values.len(), 5);
    assert_eq!(values[0]["result_type"], "StatusResult");
    assert_eq!(values[3]["result_type"], "RenameResult");
    assert_eq!(values[4]["error"]["line"], 2);
    assert_eq!(
      values[4]["error"]["path"],
      fixture
        .0
        .canonicalize()
        .unwrap()
        .join("partial.td")
        .to_str()
        .unwrap()
    );
    assert!(
      values[4]["error"]["message"]
        .as_str()
        .unwrap()
        .contains("predict unknown variable: x")
    );
    assert!(!fixture.0.join("never.csv").exists());
  }
}

#[test]
fn status_serialization_preserves_large_counts_controls_and_absent_sample_metadata() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(&mut session, "regress y x");
  let before = session.state();
  let mut view = status(&mut session);
  // Probe transport-only limits on the owned projection, not the live fit.
  let dataset = view.dataset.as_mut().unwrap();
  dataset.source = PathBuf::from("path\t雪\".csv");
  dataset.row_count = u64::MAX;
  dataset.columns[0].name = "current\t雪\"".into();
  view.active_table_name = Some("active\t雪\"".into());
  view.named_tables = vec!["registered\t雪\"".into()];
  let model = view.linear_model.as_mut().unwrap();
  model.outcome = "outcome\t雪\"".into();
  model.predictors = vec!["predictor\t雪\"".into()];
  model.observation_count = usize::MAX;
  model.degrees_of_freedom = usize::MAX - 1;
  model.sample_total_observations = None;
  model.sample_dropped_observations = None;
  let value = encoded(&view);
  assert_eq!(
    value["data"]["dataset"]["row_count"].as_u64(),
    Some(u64::MAX)
  );
  assert_eq!(
    value["data"]["dataset"]["columns"][0]["name"],
    "current\t雪\""
  );
  assert_eq!(value["data"]["active_table_name"], "active\t雪\"");
  assert_eq!(value["data"]["named_tables"], json!(["registered\t雪\""]));
  let model = &value["data"]["linear_model"];
  assert_eq!(model["outcome"], "outcome\t雪\"");
  assert_eq!(model["predictors"], json!(["predictor\t雪\""]));
  assert_eq!(model["observation_count"].as_u64(), Some(usize::MAX as u64));
  assert_eq!(
    model["degrees_of_freedom"].as_u64(),
    Some((usize::MAX - 1) as u64)
  );
  assert!(model["sample_total_observations"].is_null());
  assert!(model["sample_dropped_observations"].is_null());
  let text = tabdat_reporting::terminal::render_execution_result(&ExecutionResult::Status(view));
  assert!(!text.contains('\t'));
  assert!(text.contains("path\\t雪\\\".csv"));
  assert!(text.contains("Original candidate rows = NA; excluded = NA"));
  assert_eq!(session.state(), before);
}
