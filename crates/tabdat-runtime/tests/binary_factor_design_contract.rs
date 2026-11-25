use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, EstimationState, ExecutionResult, LinearDesignTerm, RuntimeError};
use tabdat_language::parse_command;
use tabdat_runtime::Session;
use tabdat_stats::BinaryModel;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-binary-factor-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      include_str!("../../../examples/binary-factor-design/binary_factor_data.csv"),
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

fn close(actual: f64, expected: f64) {
  close_named(actual, expected, "numeric value");
}

fn close_named(actual: f64, expected: f64, context: &str) {
  assert!(
    (actual - expected).abs() <= 2e-8,
    "{context}: {actual:.17} != {expected:.17}"
  );
}

fn field<'a>(row: &'a [&str], index: usize) -> &'a str {
  row[index].trim_matches('"')
}

fn rows(text: &str) -> Vec<Vec<&str>> {
  text
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect())
    .collect()
}

fn fit_reference(model: &str, design: &str) -> Vec<Vec<&'static str>> {
  rows(include_str!(
    "../../../examples/binary-factor-design/binary_factor_fit_reference.csv"
  ))
  .into_iter()
  .filter(|row| field(row, 0) == model && field(row, 1) == design)
  .collect()
}

fn covariance_reference(
  model: &str,
  design: &str,
  covariance_type: &str,
) -> Vec<Vec<&'static str>> {
  rows(include_str!(
    "../../../examples/binary-factor-design/binary_factor_covariance_reference.csv"
  ))
  .into_iter()
  .filter(|row| {
    field(row, 0) == model && field(row, 1) == design && field(row, 2) == covariance_type
  })
  .collect()
}

fn assert_covariance_matches(
  result: &tabdat_stats::BinaryMleResult,
  model: &str,
  design: &str,
  covariance_type: &str,
) {
  let reference = covariance_reference(model, design, covariance_type);
  let parameter_count = result.coefficients.len();
  assert_eq!(reference.len(), parameter_count * parameter_count);
  for entry in reference {
    let row = field(&entry, 3).parse::<usize>().unwrap();
    let column = field(&entry, 4).parse::<usize>().unwrap();
    close_named(
      result.covariance[row][column],
      field(&entry, 5).parse().unwrap(),
      &format!("{model} {design} {covariance_type} covariance ({row}, {column})"),
    );
  }
}

fn prediction_reference(model: &str, design: &str) -> Vec<Vec<&'static str>> {
  rows(include_str!(
    "../../../examples/binary-factor-design/binary_factor_prediction_reference.csv"
  ))
  .into_iter()
  .filter(|row| field(row, 0) == model && field(row, 1) == design)
  .collect()
}

fn summary_reference(model: &str, design: &str) -> Vec<Vec<&'static str>> {
  rows(include_str!(
    "../../../examples/binary-factor-design/binary_factor_summary_reference.csv"
  ))
  .into_iter()
  .filter(|row| field(row, 0) == model && field(row, 1) == design)
  .collect()
}

fn retained_result(session: &Session) -> tabdat_stats::BinaryMleResult {
  match session.state().estimation.unwrap() {
    EstimationState::Logit(state) | EstimationState::Probit(state) => state.result,
    _ => panic!("expected retained binary model"),
  }
}

fn estimate_binary_design(model: &str, design: &str) -> (String, BinaryModel) {
  let command = match design {
    "default" | "numeric_factor" | "boolean_factor" => "x i.region".to_owned(),
    "west_reference" => "x ib(West).region".to_owned(),
    "noconstant" => "x i.region, noconstant".to_owned(),
    _ => unreachable!(),
  };
  let fit = match design {
    "numeric_factor" => "x i.group_code".to_owned(),
    "boolean_factor" => "x i.is_west".to_owned(),
    _ => command,
  };
  let model_kind = match model {
    "logit" => BinaryModel::Logit,
    "probit" => BinaryModel::Probit,
    _ => unreachable!(),
  };
  (format!("{model} y {fit}"), model_kind)
}

fn expected_predictors(design: &str) -> Vec<&'static str> {
  match design {
    "default" => vec!["x", "region_North", "region_West"],
    "west_reference" => vec!["x", "region_East", "region_North"],
    "noconstant" => vec!["x", "region_East", "region_North", "region_West"],
    "numeric_factor" => vec!["x", "group_code_2", "group_code_3"],
    "boolean_factor" => vec!["x", "is_west_true"],
    _ => unreachable!(),
  }
}

#[test]
fn factor_logit_and_probit_designs_match_base_r_coefficients_covariance_and_sample() {
  let fixture = Fixture::new();
  for model in ["logit", "probit"] {
    for design in [
      "default",
      "west_reference",
      "noconstant",
      "numeric_factor",
      "boolean_factor",
    ] {
      let (command, expected_model) = estimate_binary_design(model, design);
      let mut session = fixture.load();
      run(&mut session, &command);
      let result = retained_result(&session);
      assert_eq!(result.model, expected_model, "{command}");
      let predictors = expected_predictors(design);
      assert_eq!(result.predictor_names, predictors, "{command}");
      let coefficient_names = result
        .coefficients
        .iter()
        .map(|coefficient| coefficient.name.as_str())
        .collect::<Vec<_>>();
      let expected_coefficients = if design == "noconstant" {
        predictors
      } else {
        std::iter::once("intercept")
          .chain(predictors.iter().copied())
          .collect()
      };
      assert_eq!(coefficient_names, expected_coefficients, "{command}");
      assert_eq!(result.observation_count, 237, "{command}");
      assert_eq!(result.event_count, 110, "{command}");
      let sample = result.sample.as_ref().unwrap();
      let expected_sample = (0..240)
        .filter(|index| ![4, 11, 17].contains(index))
        .collect::<Vec<_>>();
      assert_eq!(sample.retained_indices, expected_sample, "{command}");
      assert_eq!(sample.total_observations, 240, "{command}");
      assert_eq!(sample.dropped_observations, 3, "{command}");

      let reference = fit_reference(model, design);
      assert_eq!(reference.len(), result.coefficients.len(), "{command}");
      for (index, (estimate, expected)) in result.coefficients.iter().zip(&reference).enumerate() {
        close_named(
          estimate.value,
          field(expected, 3).parse().unwrap(),
          &format!("{command} coefficient {index}"),
        );
        close_named(
          estimate.standard_error,
          field(expected, 4).parse().unwrap(),
          &format!("{command} standard error {index}"),
        );
      }
      let expected_summary = summary_reference(model, design);
      assert_eq!(expected_summary.len(), 1);
      close_named(
        result.log_likelihood,
        field(&expected_summary[0], 4).parse().unwrap(),
        &format!("{command} log likelihood"),
      );

      assert_covariance_matches(&result, model, design, "model_based");
    }
  }
}

#[test]
fn factor_current_data_predictions_match_base_r_and_unknown_levels_are_null() {
  let fixture = Fixture::new();
  for (model, fit) in [
    ("logit", "logit y x i.region"),
    ("probit", "probit y x i.region"),
  ] {
    let mut session = fixture.load();
    run(&mut session, fit);
    let owned = session.state().estimation;
    run(&mut session, "predict eta, xb");
    run(&mut session, "predict p, pr");
    let ExecutionResult::Head(head) = run(&mut session, "head 300") else {
      panic!("expected head")
    };
    let expected = prediction_reference(model, "default");
    assert_eq!(head.rows.len(), 240);
    assert_eq!(expected.len(), 240);
    for (index, (actual, reference)) in head.rows.iter().zip(&expected).enumerate() {
      assert_eq!(field(reference, 2).parse::<usize>().unwrap(), index + 1);
      if field(reference, 3) == "TRUE" {
        assert!(!matches!(actual[1], CellValue::Null));
        let xb = match &actual[7] {
          CellValue::Float(value) => *value,
          other => panic!("expected finite xb, got {other:?}"),
        };
        let probability = match &actual[8] {
          CellValue::Float(value) => *value,
          other => panic!("expected finite probability, got {other:?}"),
        };
        close(xb, field(reference, 4).parse().unwrap());
        close(probability, field(reference, 5).parse().unwrap());
      }
      if index == 4 || index == 11 {
        assert_eq!(actual[7], CellValue::Null);
        assert_eq!(actual[8], CellValue::Null);
      }
      if index == 17 {
        // A missing outcome excludes the fit row but does not block prediction.
        assert!(matches!(actual[7], CellValue::Float(_)));
        assert!(matches!(actual[8], CellValue::Float(_)));
      }
    }
    assert_eq!(session.state().estimation, owned);

    run(&mut session, "replace region = 'Unused' if id == 1");
    run(&mut session, "predict new_probability, pr");
    let ExecutionResult::Head(head) = run(&mut session, "head 1") else {
      panic!("expected head")
    };
    assert_eq!(head.rows[0][9], CellValue::Null);
    assert_eq!(session.state().estimation, owned);
  }
}

#[test]
fn factor_reference_covariance_and_failed_fit_state_follow_existing_contracts() {
  let fixture = Fixture::new();
  for model in ["logit", "probit"] {
    let mut session = fixture.load();
    run(&mut session, &format!("{model} y x i.region"));
    let model_based = retained_result(&session);
    assert_covariance_matches(&model_based, model, "default", "model_based");

    run(&mut session, &format!("{model} y x i.region, robust"));
    let robust = retained_result(&session);
    assert!(robust.robust);
    assert_eq!(robust.predictor_names, model_based.predictor_names);
    for (actual, expected) in robust.coefficients.iter().zip(&model_based.coefficients) {
      close(actual.value, expected.value);
    }
    assert_covariance_matches(&robust, model, "default", "robust");

    run(
      &mut session,
      &format!("{model} y x i.region, cluster(cluster)"),
    );
    let clustered = retained_result(&session);
    assert!(clustered.cluster.is_some());
    assert_eq!(clustered.predictor_names, model_based.predictor_names);
    for (actual, expected) in clustered.coefficients.iter().zip(&model_based.coefficients) {
      close(actual.value, expected.value);
    }
    assert_covariance_matches(&clustered, model, "default", "cluster");
  }

  let mut explicit = fixture.load();
  run(&mut explicit, "probit y x ib(West).region");
  let Some(EstimationState::Probit(state)) = explicit.state().estimation else {
    panic!("expected probit state")
  };
  assert_eq!(
    state.design_terms,
    vec![
      LinearDesignTerm::Continuous { name: "x".into() },
      LinearDesignTerm::Factor {
        variable: "region".into(),
        reference: Some("West".into()),
        levels: vec![
          tabdat_domain::LinearFactorLevel::Text {
            label: "East".into()
          },
          tabdat_domain::LinearFactorLevel::Text {
            label: "North".into()
          },
          tabdat_domain::LinearFactorLevel::Text {
            label: "West".into()
          },
        ],
      },
    ]
  );
  assert!(matches!(
    fail(&mut explicit, "estat margins"),
    RuntimeError::EstatMarginsFailed { .. }
  ));

  let before = explicit.state();
  assert!(matches!(
    fail(&mut explicit, "probit y x ib(Absent).region"),
    RuntimeError::ProbitFailed { .. }
  ));
  assert_eq!(explicit.state().active_dataset, before.active_dataset);
  assert!(explicit.state().estimation.is_none());

  run(&mut explicit, "replace y = 2 if id == 1");
  let before_invalid_fit = explicit.state();
  assert!(matches!(
    fail(&mut explicit, "probit y x i.region"),
    RuntimeError::ProbitFailed { .. }
  ));
  assert_eq!(
    explicit.state().active_dataset,
    before_invalid_fit.active_dataset
  );
  assert!(explicit.state().estimation.is_none());
}

#[test]
fn factor_fits_keep_binary_diagnostics_tests_and_estimate_report_metadata() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  run(&mut session, "logit y x i.region");
  let Some(EstimationState::Logit(state)) = session.state().estimation else {
    panic!("expected logit state")
  };
  let design_terms = state.design_terms;

  let ExecutionResult::BinaryGoodnessOfFit(gof) = run(&mut session, "estat gof, group(10)") else {
    panic!("expected grouped binary diagnostic")
  };
  assert_eq!(gof.model, BinaryModel::Logit);
  assert!(gof.test.group_count >= 3);
  let ExecutionResult::Test(test) = run(&mut session, "test region_North = 0") else {
    panic!("expected retained-fit Wald test")
  };
  assert_eq!(test.df, 1);

  run(&mut session, "estimates store factors");
  let ExecutionResult::EstimateTable(table) = run(&mut session, "estimates table factors") else {
    panic!("expected estimate table")
  };
  assert_eq!(table.models.len(), 1);
  assert_eq!(table.models[0].linear_design_terms, design_terms);
}

#[test]
fn a_nonbinary_outcome_on_an_incomplete_factor_row_is_excluded() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  run(&mut session, "replace y = 2 if id == 5");
  let ExecutionResult::Logit(result) = run(&mut session, "logit y x i.region") else {
    panic!("expected logit")
  };
  assert_eq!(result.observation_count, 237);
  assert_eq!(result.sample.as_ref().unwrap().dropped_observations, 3);
}
