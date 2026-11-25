use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-continuous-interaction-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      include_str!("../../../examples/continuous-interaction/continuous_interaction_data.csv"),
    )
    .unwrap();
    Self(root)
  }

  fn load(&self) -> Session {
    let mut session = Session::new();
    run(
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

fn run(session: &mut Session, command: &str) -> ExecutionResult {
  session.execute(parse_command(command).unwrap()).unwrap()
}

fn fail(session: &mut Session, command: &str) -> RuntimeError {
  session
    .execute(parse_command(command).unwrap())
    .unwrap_err()
}

fn reference(model: &str, kind: &str, name: &str) -> (Option<f64>, Option<f64>) {
  let row =
    include_str!("../../../examples/continuous-interaction/continuous_interaction_reference.csv")
      .lines()
      .skip(1)
      .map(|line| {
        line
          .split(',')
          .map(|value| value.trim_matches('"'))
          .collect::<Vec<_>>()
      })
      .find(|row| row[0] == model && row[1] == kind && row[2] == name)
      .unwrap_or_else(|| panic!("missing reference {model}/{kind}/{name}"));
  let value = (!row[3].is_empty()).then(|| row[3].parse().unwrap());
  let standard_error = (!row[4].is_empty()).then(|| row[4].parse().unwrap());
  (value, standard_error)
}

fn close(actual: f64, expected: f64, label: &str) {
  assert!(
    (actual - expected).abs() <= 3e-8,
    "{label}: {actual:.17} != {expected:.17}"
  );
}

fn prediction_rows(session: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(result) = run(session, "head 300") else {
    panic!("expected head result")
  };
  result.rows
}

fn compare_predictions(session: &mut Session, model: &str, command: &str, output_column: usize) {
  run(session, command);
  compare_prediction_column(session, model, "prediction", output_column);
}

fn compare_prediction_column(session: &mut Session, model: &str, kind: &str, output_column: usize) {
  let rows = prediction_rows(session);
  assert_eq!(rows.len(), 240);
  for (index, row) in rows.iter().enumerate() {
    let (expected, _) = reference(model, kind, &(index + 1).to_string());
    match expected {
      None => assert_eq!(row[output_column], CellValue::Null, "row {}", index + 1),
      Some(expected) => {
        let CellValue::Float(actual) = row[output_column] else {
          panic!(
            "expected finite prediction at row {}, got {:?}",
            index + 1,
            row[output_column]
          );
        };
        close(
          actual,
          expected,
          &format!("{model} prediction row {}", index + 1),
        );
      }
    }
  }
}

fn compare_binary_uncertainty(session: &mut Session, model: &str) {
  run(session, "predict probability_se, pr se");
  compare_prediction_column(session, model, "prediction_se", 7);
  run(session, "predict probability95, pr interval");
  compare_prediction_column(session, model, "prediction", 8);
  compare_prediction_column(session, model, "prediction_lower", 9);
  compare_prediction_column(session, model, "prediction_upper", 10);
}

fn compare_margins(session: &mut Session, model: &str) {
  let output = run(session, "estat margins");
  let result = match output {
    ExecutionResult::LogitMarginalEffects(result) => result,
    ExecutionResult::ProbitMarginalEffects(result) => *result,
    other => panic!("expected continuous binary marginal effects, got {other:?}"),
  };
  assert!(result.original_variable_derivatives);
  let effects = result.effects;
  assert_eq!(effects.len(), 2);
  for effect in effects {
    let (value, standard_error) = reference(model, "ame", &effect.name);
    close(
      effect.value,
      value.unwrap(),
      &format!("{model} AME {}", effect.name),
    );
    close(
      effect.standard_error,
      standard_error.unwrap(),
      &format!("{model} AME SE {}", effect.name),
    );
  }
}

#[test]
fn continuous_interaction_fits_predictions_and_marginal_derivatives_match_base_r() {
  let fixture = Fixture::new();
  let mut ols = fixture.load();
  let ExecutionResult::Regression(result) = run(&mut ols, "regress y_ols x z c.x#c.z") else {
    panic!("expected OLS result")
  };
  assert_eq!(result.observation_count, 236);
  assert_eq!(
    result
      .coefficients
      .iter()
      .map(|coefficient| coefficient.name.as_str())
      .collect::<Vec<_>>(),
    ["intercept", "x", "z", "x__interaction__z"]
  );
  let names = ["intercept", "x", "z", "x__interaction__z"];
  for (index, coefficient) in result.coefficients.iter().enumerate() {
    let (value, standard_error) = reference("ols", "coefficient", names[index]);
    close(
      coefficient.value,
      value.unwrap(),
      &format!("OLS coefficient {}", names[index]),
    );
    close(
      coefficient.standard_error.unwrap(),
      standard_error.unwrap(),
      &format!("OLS SE {}", names[index]),
    );
    for (column, column_name) in names.iter().enumerate() {
      let (covariance, _) = reference(
        "ols",
        "covariance",
        &format!("{}|{}", names[index], column_name),
      );
      close(
        result.least_squares.covariance.matrix[index][column],
        covariance.unwrap(),
        &format!("OLS covariance {}|{}", names[index], column_name),
      );
    }
  }
  compare_predictions(&mut ols, "ols", "predict ols_xb, xb", 6);

  for (model, outcome, fit) in [
    ("logit", "y_logit", "logit y_logit x z c.x#c.z"),
    ("probit", "y_probit", "probit y_probit x z c.x#c.z"),
  ] {
    let mut session = fixture.load();
    let result = match run(&mut session, fit) {
      ExecutionResult::Logit(result) => *result,
      ExecutionResult::Probit(result) => *result,
      other => panic!("expected binary result for {outcome}, got {other:?}"),
    };
    assert_eq!(result.observation_count, 236);
    for (coefficient, name) in
      result
        .coefficients
        .iter()
        .zip(["intercept", "x", "z", "x__interaction__z"])
    {
      let (value, standard_error) = reference(model, "coefficient", name);
      close(
        coefficient.value,
        value.unwrap(),
        &format!("{model} coefficient {name}"),
      );
      close(
        coefficient.standard_error,
        standard_error.unwrap(),
        &format!("{model} SE {name}"),
      );
    }
    compare_predictions(&mut session, model, "predict probability, pr", 6);
    compare_binary_uncertainty(&mut session, model);
    let fitted_state = session.state().estimation;
    compare_margins(&mut session, model);
    assert_eq!(session.state().estimation, fitted_state);
    assert!(matches!(
      fail(&mut session, "estat margins, discrete(x)"),
      RuntimeError::EstatMarginsFailed { .. }
    ));
    assert_eq!(session.state().estimation, fitted_state);

    // Marginal effects use source values retained with the fit after active X changes.
    run(&mut session, "replace x = -x");
    compare_margins(&mut session, model);
  }
}

#[test]
fn interaction_only_binary_design_has_no_implicit_main_effects_and_correct_ames() {
  let fixture = Fixture::new();
  for (model, outcome, fit) in [
    ("logit_product_only", "y_logit", "logit y_logit c.x#c.z"),
    ("probit_product_only", "y_probit", "probit y_probit c.x#c.z"),
  ] {
    let mut session = fixture.load();
    let result = match run(&mut session, fit) {
      ExecutionResult::Logit(result) => *result,
      ExecutionResult::Probit(result) => *result,
      other => panic!("expected interaction-only binary result for {outcome}, got {other:?}"),
    };
    assert_eq!(result.coefficients.len(), 2);
    assert_eq!(result.predictor_names, ["x__interaction__z"]);
    compare_predictions(&mut session, model, "predict probability, pr", 6);
    compare_binary_uncertainty(&mut session, model);
    compare_margins(&mut session, model);
  }
}

#[test]
fn failed_interaction_fit_clears_old_fit_without_mutating_data() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  run(&mut session, "logit y_logit c.x#c.z");
  let before = session.state();
  let error = fail(&mut session, "logit y_logit c.x#c.z c.z#c.x");
  assert!(error.to_string().contains("duplicate design term"));
  let after = session.state();
  assert!(after.estimation.is_none());
  assert_eq!(after.active_dataset, before.active_dataset);
  assert_eq!(after.labels, before.labels);
}
