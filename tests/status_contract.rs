#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  path::PathBuf,
  process::Command,
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
      "data_initialized":false,"dataset":null,"active_table_name":null,"named_tables":[],"linear_model":null
    }})
  );
  let text = tabdat_reporting::terminal::render_execution_result(&ExecutionResult::Status(result));
  assert_eq!(
    text,
    "Session status\nData engine initialized: false\nActive dataset: none\nActive named table: none\nRegistered named tables: []\nRetained linear model: none\n"
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
