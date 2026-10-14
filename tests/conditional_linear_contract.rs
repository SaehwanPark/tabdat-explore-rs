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
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
fn reference() -> Value {
  serde_json::from_str(include_str!("fixtures/conditional-linear-reference.json")).unwrap()
}
fn execute(session: &mut Session, source: &str) -> ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}
fn data(result: &ExecutionResult) -> Value {
  let value: Value = serde_json::from_str(
    &tabdat_reporting::json_results::serialize_execution_result(result).unwrap(),
  )
  .unwrap();
  value["data"].clone()
}
fn close(actual: &Value, expected: &Value) {
  let actual = actual.as_f64().unwrap();
  let expected = expected.as_f64().unwrap();
  let tolerance = 1e-10_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "{actual} != {expected}, tolerance {tolerance}"
  );
}
fn lines(bytes: &[u8]) -> Vec<Value> {
  std::str::from_utf8(bytes)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn batch(fixture: &Fixture, commands: &[&str]) -> Vec<Value> {
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  let output = fixture.batch(&args);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  lines(&output.stdout)
}
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-conditional-linear-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("seed.csv"), "y,x\n1,1\n").unwrap();
    Self(path)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  /// Write the reference records as a plain CSV for CLI batch workflows.
  fn cohort_csv(&self) -> String {
    let rows = reference()["records"]
      .as_array()
      .unwrap()
      .iter()
      .map(|row| {
        row
          .as_array()
          .unwrap()
          .iter()
          .map(|value| match value {
            Value::Null => String::new(),
            Value::String(value) => value.clone(),
            value => value.to_string(),
          })
          .collect::<Vec<_>>()
          .join(",")
      })
      .collect::<Vec<_>>()
      .join("\n");
    let path = self.0.join("cohort.csv");
    fs::write(&path, format!("id,eligible,y,x,z,w,sigma,g\n{rows}\n")).unwrap();
    path.file_name().unwrap().to_string_lossy().into_owned()
  }
  fn session(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", self.0.join("seed.csv").display()),
    );
    let rows = reference()["records"]
      .as_array()
      .unwrap()
      .iter()
      .map(|row| {
        let values = row
          .as_array()
          .unwrap()
          .iter()
          .enumerate()
          .map(|(index, value)| {
            let value = match value {
              Value::Null => "NULL".into(),
              Value::String(value) => format!("'{}'", value.replace('\'', "''")),
              value => value.to_string(),
            };
            if (2..7).contains(&index) {
              format!("CAST({value} AS DOUBLE)")
            } else {
              value
            }
          })
          .collect::<Vec<_>>()
          .join(",");
        format!("({values})")
      })
      .collect::<Vec<_>>()
      .join(",");
    execute(
      &mut session,
      &format!("sql select * from (values {rows}) as t(id,eligible,y,x,z,w,sigma,g) into cohort"),
    );
    session
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
#[test]
fn bare_models_share_reference_metrics_and_cluster_postestimation_inference_df() {
  let fixture = Fixture::new();
  for case in reference()["cases"].as_array().unwrap() {
    let mut session = fixture.session();
    execute(&mut session, "keep if eligible == 1");
    let command = case["command"]
      .as_str()
      .unwrap()
      .replace(" if eligible == 1", "");
    let result = data(&execute(&mut session, &command));
    for key in ["r_squared", "adjusted_r_squared", "root_mse"] {
      close(&result[key], &case[key]);
    }
    assert_eq!(result["df_residual"], case["df_residual"]);
    let combination = data(&execute(&mut session, "lincom x + z"));
    assert_eq!(combination["df_residual"], case["df_inference"]);
    for (index, key) in [
      "estimate",
      "standard_error",
      "statistic",
      "p_value",
      "ci_lower",
      "ci_upper",
    ]
    .iter()
    .enumerate()
    {
      close(&combination[key], &case["lincom"][index]);
    }
    let test = data(&execute(&mut session, "test x z"));
    assert_eq!(test["df_residual"], case["df_inference"]);
    close(&test["statistic"], &case["test"][0]);
    close(&test["p_value"], &case["test"][1]);
  }
}

#[test]
fn selected_samples_covariance_inference_vif_and_current_data_predictions_match_statsmodels() {
  let fixture = Fixture::new();
  let reference = reference();
  assert_eq!(reference["cases"].as_array().unwrap().len(), 18);
  assert_eq!(
    reference["eligible_indices"],
    json!([1, 2, 4, 5, 7, 8, 9, 10, 11, 12, 13, 15])
  );
  let mut session = fixture.session();
  execute(&mut session, "label variable y 'Outcome'");
  execute(&mut session, "label define groups 'A' 'First' 'B' 'Second'");
  execute(&mut session, "label values g groups");
  let dataset = session.state().active_dataset.clone();
  let labels = session.active_label_metadata().unwrap().clone();
  let signature = execute(&mut session, "datasignature");
  let preview = execute(&mut session, "head 20");
  for case in reference["cases"].as_array().unwrap() {
    let command = case["command"].as_str().unwrap();
    let result = data(&execute(&mut session, command));
    assert_eq!(result["observation_count"], case["n"], "{command}");
    assert_eq!(result["df_residual"], case["df_residual"], "{command}");
    assert_eq!(result["include_intercept"], case["include_intercept"]);
    assert_eq!(result["sample"]["total_observations"], 16);
    assert_eq!(
      result["sample"]["dropped_observations"].as_u64().unwrap(),
      16 - case["n"].as_u64().unwrap()
    );
    for (index, coefficient) in result["coefficients"]
      .as_array()
      .unwrap()
      .iter()
      .enumerate()
    {
      assert_eq!(coefficient["name"], case["coefficient_names"][index]);
      for (key, reference) in [
        ("value", "coefficients"),
        ("standard_error", "standard_errors"),
        ("statistic", "statistics"),
        ("p_value", "p_values"),
      ] {
        close(&coefficient[key], &case[reference][index]);
      }
    }
    for key in ["r_squared", "adjusted_r_squared", "root_mse"] {
      close(&result[key], &case[key]);
    }
    let cached = session.last_regression().unwrap();
    let sample = cached.sample.as_ref().unwrap();
    assert_eq!(
      json!(sample.retained_indices),
      case["retained_indices"],
      "{command}"
    );
    assert_eq!(json!(sample.cluster_groups), case["clusters"]);
    match &sample.weights {
      Some(weights) => {
        for (index, weight) in weights.iter().enumerate() {
          close(&json!(weight), &case["weights"][index]);
        }
      }
      None => assert!(case["weights"].is_null()),
    }
    for (row, values) in cached.covariance.matrix.iter().enumerate() {
      for (column, value) in values.iter().enumerate() {
        close(&json!(value), &case["covariance_matrix"][row][column]);
      }
    }
    let combination = data(&execute(&mut session, "lincom x + z"));
    assert_eq!(
      combination["df_residual"], case["df_inference"],
      "{command}"
    );
    for (index, key) in [
      "estimate",
      "standard_error",
      "statistic",
      "p_value",
      "ci_lower",
      "ci_upper",
    ]
    .iter()
    .enumerate()
    {
      close(&combination[key], &case["lincom"][index]);
    }
    let test = data(&execute(&mut session, "test x z"));
    assert_eq!(test["df_residual"], case["df_inference"], "{command}");
    close(&test["statistic"], &case["test"][0]);
    close(&test["p_value"], &case["test"][1]);
    let vif = data(&execute(&mut session, "estat vif"));
    for index in 0..2 {
      close(&vif["rows"][index][1], &case["vif"][index]);
    }
    assert_eq!(session.state().active_dataset, dataset);
    assert_eq!(session.active_label_metadata().unwrap(), &labels);
    assert_eq!(session.active_table_name(), Some("cohort"));
    assert_eq!(execute(&mut session, "datasignature"), signature);
    assert_eq!(execute(&mut session, "head 20"), preview);
    execute(&mut session, "predict fitted");
    let predictions = data(&execute(&mut session, "head 20"));
    for (index, row) in predictions["rows"].as_array().unwrap().iter().enumerate() {
      let actual = row.as_array().unwrap().last().unwrap();
      if case["predictions"][index].is_null() {
        assert!(actual.is_null());
      } else {
        close(actual, &case["predictions"][index]);
      }
    }
    execute(&mut session, "drop fitted");
  }
}

#[test]
fn guarded_attempts_clear_cached_models_and_preserve_read_only_state() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(&mut session, "label variable y 'Outcome'");
  execute(&mut session, "regress y x z if eligible == 1");
  let inference = execute(&mut session, "lincom x + z");
  // A failed attempted regression clears the prior fit under the existing
  // policy, so the guarded-state baseline is taken after the first failure.
  assert!(
    session
      .execute(parse_command("regress y x z if eligible == 9").unwrap())
      .is_err()
  );
  assert!(session.last_regression().is_none());
  let state = session.state();
  let metadata = session.active_label_metadata().unwrap().clone();
  let snapshot = "sql select id,cast(y as varchar),cast(x as varchar),cast(z as varchar),g from active order by id";
  let original = execute(&mut session, snapshot);
  for (source, expected) in [
    (
      "regress y x z if absent > 0",
      RuntimeError::RegressUnknownVariable {
        variables: vec!["absent".into()],
      },
    ),
    ("regress y x z if y", RuntimeError::RegressRequiresBoolean),
    ("regress y x z if g", RuntimeError::RegressRequiresBoolean),
    (
      "regress y x z if inrange(x, 1, 5)",
      RuntimeError::RegressUnsupportedPredicate,
    ),
    (
      "regress y x z if eligible == 9",
      RuntimeError::RegressNoObservations,
    ),
    ("regress y x z if null", RuntimeError::RegressNoObservations),
    (
      "regress y x z x",
      RuntimeError::RegressFailed {
        message: String::new(),
      },
    ),
    (
      "regress g x z if eligible == 1",
      RuntimeError::RegressRequiresNumeric {
        variables: vec!["g".into()],
      },
    ),
  ] {
    let error = session.execute(parse_command(source).unwrap()).unwrap_err();
    match (&error, &expected) {
      (RuntimeError::RegressFailed { .. }, RuntimeError::RegressFailed { .. }) => {}
      _ => assert_eq!(error, expected, "{source}"),
    }
    assert_eq!(session.state(), state, "{source}");
    assert_eq!(session.active_label_metadata().unwrap(), &metadata);
    assert_eq!(execute(&mut session, snapshot), original);
    assert!(session.last_regression().is_none(), "{source}");
  }
  assert!(matches!(
    session
      .execute(parse_command("regress y x z if eligible == 'A'").unwrap())
      .unwrap_err(),
    RuntimeError::RegressPredicateTypeMismatch { .. }
  ));
  assert_eq!(session.state(), state);
  // A successful conditional fit restores the cache with identical inference.
  execute(&mut session, "regress y x z if eligible == 1");
  assert_eq!(execute(&mut session, "lincom x + z"), inference);
  assert_eq!(session.state(), session.state());
}

#[test]
fn weight_and_sigma_validation_is_scoped_to_the_selected_cohort() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("weights.csv"),
    "y,x,w,s,keep\n1,1,1,1,1\n2,2,2,1,1\n3,3,-2,0.5,0\n4,4,1,-1,0\n5,5,1,1,1\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("invalid.csv"),
    "y,x,w,s,keep\n1,1,1,1,1\n2,2,-3,0.5,1\n3,3,1,-1,1\n4,4,1,1,0\n",
  )
  .unwrap();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("weights.csv").display()),
  );
  // Invalid weights and sigma outside the TRUE cohort are excluded before
  // weight validation; sample indices still refer to the original relation.
  let result = data(&execute(&mut session, "regress y x if keep == 1, wls(w)"));
  assert_eq!(result["observation_count"], 3);
  assert_eq!(result["sample"]["total_observations"], 5);
  assert_eq!(result["sample"]["dropped_observations"], 2);
  let sample = session.last_regression().unwrap().sample.as_ref().unwrap();
  assert_eq!(sample.retained_indices, [0, 1, 4]);
  assert_eq!(sample.cluster_groups, None);
  let weights = sample.weights.as_ref().unwrap();
  for (index, expected) in [1.0, 2.0, 1.0].into_iter().enumerate() {
    close(&json!(weights[index]), &json!(expected));
  }
  let gls = data(&execute(&mut session, "regress y x if keep == 1, gls(s)"));
  assert_eq!(gls["observation_count"], 3);
  // Invalid values inside the cohort fail closed.
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("invalid.csv").display()),
  );
  for (source, expected) in [
    (
      "regress y x if keep == 1, wls(w)",
      RuntimeError::RegressRequiresPositiveWeights,
    ),
    (
      "regress y x if keep == 1, gls(s)",
      RuntimeError::RegressRequiresPositiveSigma,
    ),
  ] {
    assert_eq!(
      session.execute(parse_command(source).unwrap()).unwrap_err(),
      expected
    );
  }
}

#[test]
fn native_fit_failures_inside_cohorts_report_bounded_errors_without_state() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  let state = session.state();
  // Duplicate predictors are rank deficient within the selected cohort.
  assert!(matches!(
    session
      .execute(parse_command("regress y x x if eligible == 1").unwrap())
      .unwrap_err(),
    RuntimeError::RegressFailed { .. }
  ));
  // Fewer than two distinct clusters inside the cohort cannot produce a
  // cluster covariance.
  assert!(matches!(
    session
      .execute(parse_command("regress y x z if id == 1, cluster(g)").unwrap())
      .unwrap_err(),
    RuntimeError::RegressFailed { .. }
  ));
  assert_eq!(session.state(), state);
}

#[test]
fn batch_guards_stop_files_and_recovery_persistence_keep_the_full_active_relation() {
  let fixture = Fixture::new();
  let cohort_csv = fixture.cohort_csv();
  let plain = batch(&fixture, &[&format!("use {cohort_csv}"), "head 20"]);
  for (source, code, kind) in [
    ("regress y x z if absent > 0", 1, "ExecutionError"),
    ("regress y x z if", 2, "ParseError"),
    ("regress y x z if eligible == 9", 1, "ExecutionError"),
  ] {
    let output = fixture.batch(&[
      "--json",
      "-c",
      &format!("use {cohort_csv}"),
      "-c",
      source,
      "-c",
      "export never.csv",
    ]);
    assert_eq!(output.status.code(), Some(code), "{source}");
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 2);
    assert_eq!(results[1]["error"]["type"], kind);
    assert!(!fixture.0.join("never.csv").exists());
    fs::write(
      fixture.0.join("child.td"),
      format!("count\n{source}\nexport never.csv\n"),
    )
    .unwrap();
    fs::write(
      fixture.0.join("parent.td"),
      format!("use {cohort_csv}\nrun child.td\nexport never.csv\n"),
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
  // Interactive recovery: a failed conditional fit does not end the session,
  // a successful one reports the selected sample, and exports stay full.
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&fixture.0)
    .env("TERM", "dumb")
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
      format!(
        "use {cohort_csv}\nregress y x z if absent > 0\ncount\nregress y x z if eligible == 1\nexport terminal.csv\nexit\n"
      )
      .as_bytes(),
    )
    .unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(output.status.success());
  assert!(String::from_utf8_lossy(&output.stderr).contains("unknown variable"));
  assert!(String::from_utf8_lossy(&output.stdout).contains("Candidate rows = 16"));
  // Persistence after a conditional fit keeps the full active relation, not
  // the selected cohort, in saved and exported files.
  let persisted = batch(
    &fixture,
    &[
      &format!("use {cohort_csv}"),
      "regress y x z if eligible == 1",
      "save saved.parquet, replace",
      "export saved.csv, replace",
      "use saved.csv",
      "head 20",
    ],
  );
  assert_eq!(persisted[1]["result_type"], "RegressionResult");
  assert_eq!(persisted[1]["data"]["sample"]["total_observations"], 16);
  assert_eq!(persisted[5]["data"], plain[1]["data"]);
  let reloaded = batch(&fixture, &["use terminal.csv", "head 20"]);
  assert_eq!(reloaded[1]["data"], plain[1]["data"]);
}
