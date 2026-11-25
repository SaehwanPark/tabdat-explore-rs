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

fn close_relative(actual: f64, expected: f64, relative_tolerance: f64, context: &str) {
  let tolerance = relative_tolerance * expected.abs();
  assert!(
    (actual - expected).abs() <= tolerance,
    "{context}: {actual:.17e} != {expected:.17e} (relative tolerance {relative_tolerance:.3e})"
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

fn factor_contrast_reference(
  model: &str,
  design: &str,
  covariance_type: &str,
) -> Vec<Vec<&'static str>> {
  rows(include_str!(
    "../../../examples/binary-factor-design/binary_factor_contrast_reference.csv"
  ))
  .into_iter()
  .filter(|row| {
    field(row, 0) == model && field(row, 1) == design && field(row, 2) == covariance_type
  })
  .collect()
}

fn continuous_marginal_effect_reference(
  model: &str,
  design: &str,
  covariance_type: &str,
) -> Vec<Vec<&'static str>> {
  rows(include_str!(
    "../../../examples/binary-factor-design/binary_factor_marginal_effect_reference.csv"
  ))
  .into_iter()
  .filter(|row| {
    field(row, 0) == model && field(row, 1) == design && field(row, 2) == covariance_type
  })
  .collect()
}

fn discrete_change_reference(
  model: &str,
  design: &str,
  covariance_type: &str,
) -> Vec<Vec<&'static str>> {
  rows(include_str!(
    "../../../examples/binary-factor-design/binary_factor_discrete_change_reference.csv"
  ))
  .into_iter()
  .filter(|row| {
    field(row, 0) == model && field(row, 1) == design && field(row, 2) == covariance_type
  })
  .collect()
}

fn binary_lincom_reference(model: &str, covariance_type: &str) -> Vec<Vec<&'static str>> {
  rows(include_str!(
    "../../../examples/binary-factor-design/binary_factor_lincom_reference.csv"
  ))
  .into_iter()
  .filter(|row| field(row, 0) == model && field(row, 1) == covariance_type)
  .collect()
}

fn binary_classification_reference(model: &str, cutoff: f64) -> Vec<Vec<&'static str>> {
  rows(include_str!(
    "../../../examples/binary-factor-design/binary_factor_classification_reference.csv"
  ))
  .into_iter()
  .filter(|row| field(row, 0) == model && field(row, 1).parse::<f64>().unwrap() == cutoff)
  .collect()
}

fn retained_result(session: &Session) -> tabdat_stats::BinaryMleResult {
  match session.state().estimation.unwrap() {
    EstimationState::Logit(state) | EstimationState::Probit(state) => state.result,
    _ => panic!("expected retained binary model"),
  }
}

fn continuous_marginal_effect_result(
  result: ExecutionResult,
) -> tabdat_stats::LogitMarginalEffects {
  match result {
    ExecutionResult::LogitMarginalEffects(result) => result,
    ExecutionResult::ProbitMarginalEffects(result) => *result,
    other => panic!("expected continuous probability effects, got {other:?}"),
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
  let ExecutionResult::ProbitMarginalEffects(margins) = run(&mut explicit, "estat margins") else {
    panic!("expected continuous probability effects alongside a retained factor")
  };
  assert_eq!(margins.effects.len(), 1);
  assert_eq!(margins.effects[0].name, "x");

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
fn continuous_margins_after_factor_fits_match_base_r_and_use_only_retained_continuous_terms() {
  let fixture = Fixture::new();
  for model in ["logit", "probit"] {
    for (design, covariance) in [
      ("default", "model_based"),
      ("default", "robust"),
      ("default", "cluster"),
      ("west_reference", "model_based"),
      ("noconstant", "model_based"),
      ("numeric_factor", "model_based"),
      ("boolean_factor", "model_based"),
    ] {
      let (base_fit, expected_model) = estimate_binary_design(model, design);
      let fit = match covariance {
        "model_based" => base_fit,
        "robust" => format!("{base_fit}, robust"),
        "cluster" => format!("{base_fit}, cluster(cluster)"),
        _ => unreachable!(),
      };
      let mut session = fixture.load();
      run(&mut session, &fit);
      let before = session.state().estimation;
      let actual = continuous_marginal_effect_result(run(&mut session, "estat margins"));
      let reference = continuous_marginal_effect_reference(model, design, covariance);
      assert_eq!(actual.observation_count, 237, "{fit}");
      assert_eq!(actual.robust, covariance != "model_based", "{fit}");
      assert_eq!(actual.cluster.is_some(), covariance == "cluster", "{fit}");
      assert_eq!(
        actual.cluster.as_ref().map(|cluster| cluster.group_count),
        (covariance == "cluster").then_some(24),
        "{fit} cluster groups"
      );
      assert_eq!(actual.effects.len(), 1, "{fit}");
      assert_eq!(actual.effects[0].name, "x", "{fit}");
      assert_eq!(reference.len(), 1, "{fit}");
      assert_eq!(
        expected_model,
        if model == "logit" {
          BinaryModel::Logit
        } else {
          BinaryModel::Probit
        },
        "{fit}"
      );
      for (name, value, index) in [
        ("estimate", actual.effects[0].value, 4),
        ("standard error", actual.effects[0].standard_error, 5),
        ("statistic", actual.effects[0].statistic, 6),
        ("p-value", actual.effects[0].p_value, 7),
        ("lower interval", actual.effects[0].confidence_lower, 8),
        ("upper interval", actual.effects[0].confidence_upper, 9),
      ] {
        close_named(
          value,
          field(&reference[0], index).parse().unwrap(),
          &format!("{fit} {name}"),
        );
      }
      assert_eq!(session.state().estimation, before, "{fit}");
      if design == "default" && covariance == "model_based" {
        run(&mut session, "replace region = 'Unused' if id == 1");
        run(&mut session, "replace x = x + 100 if id == 2");
        assert_eq!(
          continuous_marginal_effect_result(run(&mut session, "estat margins")),
          actual,
          "margins use retained design after current data edits"
        );
      }
    }
  }
}

#[test]
fn factor_only_margins_fail_without_changing_the_retained_fit() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  run(&mut session, "logit y i.region");
  let before = session.state().estimation;
  let error = fail(&mut session, "estat margins");
  assert!(matches!(
    error,
    RuntimeError::EstatMarginsFailed { message }
      if message.contains("at least one continuous predictor")
  ));
  assert_eq!(session.state().estimation, before);
}

#[test]
fn continuous_margins_map_predictors_when_the_factor_term_comes_first() {
  let fixture = Fixture::new();
  for model in ["logit", "probit"] {
    let mut session = fixture.load();
    run(&mut session, &format!("{model} y i.region x"));
    let actual = continuous_marginal_effect_result(run(&mut session, "estat margins"));
    let reference = continuous_marginal_effect_reference(model, "default", "model_based");
    assert_eq!(actual.effects.len(), 1);
    assert_eq!(actual.effects[0].name, "x");
    for (value, index) in [
      (actual.effects[0].value, 4),
      (actual.effects[0].standard_error, 5),
      (actual.effects[0].statistic, 6),
      (actual.effects[0].p_value, 7),
      (actual.effects[0].confidence_lower, 8),
      (actual.effects[0].confidence_upper, 9),
    ] {
      close_named(
        value,
        field(&reference[0], index).parse().unwrap(),
        &format!("{model} factor-first margin"),
      );
    }
  }
}

#[test]
fn factor_probability_contrasts_match_base_r_and_use_only_the_retained_design() {
  let fixture = Fixture::new();
  for model in ["logit", "probit"] {
    for (design, covariance, variable, fit) in [
      (
        "default",
        "model_based",
        "region",
        format!("{model} y x i.region"),
      ),
      (
        "default",
        "robust",
        "region",
        format!("{model} y x i.region, robust"),
      ),
      (
        "default",
        "cluster",
        "region",
        format!("{model} y x i.region, cluster(cluster)"),
      ),
      (
        "west_reference",
        "model_based",
        "region",
        format!("{model} y x ib(West).region"),
      ),
      (
        "noconstant",
        "model_based",
        "region",
        format!("{model} y x i.region, noconstant"),
      ),
      (
        "numeric_factor",
        "model_based",
        "group_code",
        format!("{model} y x i.group_code"),
      ),
      (
        "boolean_factor",
        "model_based",
        "is_west",
        format!("{model} y x i.is_west"),
      ),
    ] {
      let mut session = fixture.load();
      run(&mut session, &fit);
      let before = session.state().estimation;
      let ExecutionResult::BinaryFactorContrasts(actual) =
        run(&mut session, &format!("estat margins, factor({variable})"))
      else {
        panic!("expected factor probability contrasts for {fit}")
      };
      let reference = factor_contrast_reference(model, design, covariance);
      assert_eq!(
        actual.model,
        if model == "logit" {
          BinaryModel::Logit
        } else {
          BinaryModel::Probit
        }
      );
      assert_eq!(actual.factor_variable, variable);
      assert_eq!(actual.baseline_level, field(&reference[0], 4));
      assert_eq!(actual.observation_count, 237);
      assert_eq!(actual.robust, covariance != "model_based");
      assert_eq!(actual.cluster.is_some(), covariance == "cluster");
      assert_eq!(actual.contrasts.len(), reference.len(), "{fit}");
      for (contrast, expected) in actual.contrasts.iter().zip(&reference) {
        assert_eq!(contrast.level, field(expected, 5), "{fit}");
        close_named(
          contrast.effect.value,
          field(expected, 6).parse().unwrap(),
          &format!("{fit} contrast {} estimate", contrast.level),
        );
        close_named(
          contrast.effect.standard_error,
          field(expected, 7).parse().unwrap(),
          &format!("{fit} contrast {} standard error", contrast.level),
        );
        close_named(
          contrast.effect.statistic,
          field(expected, 8).parse().unwrap(),
          &format!("{fit} contrast {} statistic", contrast.level),
        );
        close_named(
          contrast.effect.p_value,
          field(expected, 9).parse().unwrap(),
          &format!("{fit} contrast {} p-value", contrast.level),
        );
        close_named(
          contrast.effect.confidence_lower,
          field(expected, 10).parse().unwrap(),
          &format!("{fit} contrast {} lower interval", contrast.level),
        );
        close_named(
          contrast.effect.confidence_upper,
          field(expected, 11).parse().unwrap(),
          &format!("{fit} contrast {} upper interval", contrast.level),
        );
      }
      assert_eq!(session.state().estimation, before);
      if design == "default" && covariance == "model_based" {
        run(&mut session, "replace region = 'Unused' if id == 1");
        let ExecutionResult::BinaryFactorContrasts(after_edit) =
          run(&mut session, &format!("estat margins, factor({variable})"))
        else {
          panic!("expected factor probability contrasts after data edit")
        };
        assert_eq!(*after_edit, *actual);
      }
    }
  }
}

#[test]
fn factor_probability_contrasts_require_a_retained_factor_and_preserve_fit_state() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  assert!(matches!(
    fail(&mut session, "estat margins, factor(region)"),
    RuntimeError::EstatMarginsFailed { .. }
  ));
  run(&mut session, "logit y x");
  let before = session.state().estimation;
  assert!(matches!(
    fail(&mut session, "estat margins, factor(region)"),
    RuntimeError::EstatMarginsFailed { .. }
  ));
  assert_eq!(session.state().estimation, before);
  assert!(matches!(
    fail(&mut session, "estat margins, factor(x)"),
    RuntimeError::EstatMarginsFailed { .. }
  ));
  assert_eq!(session.state().estimation, before);
}

#[test]
fn discrete_changes_in_mixed_factor_designs_match_base_r_and_use_only_the_retained_design() {
  let fixture = Fixture::new();
  for model in ["logit", "probit"] {
    for (design, covariance) in [
      ("default", "model_based"),
      ("default", "robust"),
      ("default", "cluster"),
      ("west_reference", "model_based"),
      ("noconstant", "model_based"),
      ("numeric_factor", "model_based"),
      ("boolean_factor", "model_based"),
    ] {
      let factor_terms = match design {
        "default" | "robust" | "cluster" => "i.region",
        "west_reference" => "ib(West).region",
        "noconstant" => "i.region",
        "numeric_factor" => "i.group_code",
        "boolean_factor" => "i.is_west",
        _ => unreachable!(),
      };
      let options = match (design, covariance) {
        ("noconstant", "model_based") => ", noconstant",
        (_, "model_based") => "",
        (_, "robust") => ", robust",
        (_, "cluster") => ", cluster(cluster)",
        _ => unreachable!(),
      };
      let fit = format!("{model} y treated x {factor_terms}{options}");
      let mut session = fixture.load();
      run(&mut session, "generate treated = 0");
      run(&mut session, "replace treated = 1 if x > 0");
      run(&mut session, &fit);
      let before = session.state().estimation;
      let ExecutionResult::BinaryDiscreteChange(actual) =
        run(&mut session, "estat margins, discrete(treated)")
      else {
        panic!("expected discrete probability change for {fit}")
      };
      let reference_design = if covariance == "robust" || covariance == "cluster" {
        "default"
      } else {
        design
      };
      let reference = discrete_change_reference(model, reference_design, covariance);
      assert_eq!(reference.len(), 1, "{fit}");
      assert_eq!(
        actual.model,
        if model == "logit" {
          BinaryModel::Logit
        } else {
          BinaryModel::Probit
        }
      );
      assert_eq!(actual.predictor, "treated");
      assert_eq!((actual.from_value, actual.to_value), (0, 1));
      assert_eq!(actual.observation_count, 237, "{fit}");
      assert_eq!(actual.robust, covariance != "model_based", "{fit}");
      assert_eq!(actual.cluster.is_some(), covariance == "cluster", "{fit}");
      assert_eq!(
        actual.cluster.as_ref().map(|cluster| cluster.group_count),
        (covariance == "cluster").then_some(24),
        "{fit} cluster groups"
      );
      for (name, value, index) in [
        ("estimate", actual.effect.value, 4),
        ("standard error", actual.effect.standard_error, 5),
        ("statistic", actual.effect.statistic, 6),
        ("p-value", actual.effect.p_value, 7),
        ("lower interval", actual.effect.confidence_lower, 8),
        ("upper interval", actual.effect.confidence_upper, 9),
      ] {
        close_named(
          value,
          field(&reference[0], index).parse().unwrap(),
          &format!("{fit} {name}"),
        );
      }
      assert_eq!(session.state().estimation, before, "{fit}");
      if design == "default" && covariance == "model_based" {
        run(&mut session, "replace treated = 1 - treated");
        run(&mut session, "replace region = 'Unused' if id == 1");
        run(&mut session, "replace x = x + 100 if id == 2");
        let ExecutionResult::BinaryDiscreteChange(after_edit) =
          run(&mut session, "estat margins, discrete(treated)")
        else {
          panic!("expected retained discrete probability change after data edits")
        };
        assert_eq!(*after_edit, *actual);
      }
    }
  }
}

#[test]
fn discrete_changes_reject_factor_dummies_and_nonbinary_continuous_terms_without_losing_fit() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  run(&mut session, "generate treated = 0");
  run(&mut session, "replace treated = 1 if x > 0");
  run(&mut session, "logit y treated x i.region");
  let before = session.state().estimation;

  let factor_error = fail(&mut session, "estat margins, discrete(region_North)");
  assert!(matches!(
    factor_error,
    RuntimeError::EstatMarginsFailed { message }
      if message.contains("original continuous predictor")
  ));
  assert_eq!(session.state().estimation, before);

  let nonbinary_error = fail(&mut session, "estat margins, discrete(x)");
  assert!(matches!(
    nonbinary_error,
    RuntimeError::EstatMarginsFailed { message }
      if message.contains("only 0 and 1")
  ));
  assert_eq!(session.state().estimation, before);

  let mut factor_only = fixture.load();
  run(&mut factor_only, "logit y i.region");
  let factor_only_before = factor_only.state().estimation;
  assert!(matches!(
    fail(&mut factor_only, "estat margins, discrete(x)"),
    RuntimeError::EstatMarginsFailed { .. }
  ));
  assert_eq!(factor_only.state().estimation, factor_only_before);
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

#[test]
fn binary_lincom_matches_base_r_for_factor_coefficients_and_selected_covariances() {
  let fixture = Fixture::new();
  let combinations = [
    ("slope_offset", "lincom x + 0.5"),
    (
      "factor_contrast",
      "lincom x + region_North - region_West + 0.5",
    ),
    ("intercept_slope", "lincom intercept + x"),
  ];

  for model in ["logit", "probit"] {
    for covariance in ["model_based", "robust", "cluster"] {
      let options = match covariance {
        "model_based" => "",
        "robust" => ", robust",
        "cluster" => ", cluster(cluster)",
        _ => unreachable!(),
      };
      let mut session = fixture.load();
      run(&mut session, &format!("{model} y x i.region{options}"));
      run(&mut session, "replace y = 1 - y if id == 1");
      let retained_state = session.state();
      let references = binary_lincom_reference(model, covariance);
      assert_eq!(references.len(), combinations.len(), "{model} {covariance}");

      for (combination, command) in combinations {
        let reference = references
          .iter()
          .find(|row| field(row, 2) == combination)
          .unwrap_or_else(|| panic!("missing {model} {covariance} {combination} reference"));
        let ExecutionResult::Lincom(actual) = run(&mut session, command) else {
          panic!("expected linear combination after {model}")
        };
        for (name, value, column) in [
          ("estimate", actual.estimate, 3),
          ("standard error", actual.standard_error, 4),
          ("z statistic", actual.statistic, 5),
          ("lower interval", actual.ci_lower, 7),
          ("upper interval", actual.ci_upper, 8),
        ] {
          close_named(
            value,
            field(reference, column).parse().unwrap(),
            &format!("{model} {covariance} {combination} {name}"),
          );
        }
        close_relative(
          actual.p_value,
          field(reference, 6).parse().unwrap(),
          1e-8,
          &format!("{model} {covariance} {combination} p-value"),
        );
        close(actual.ci_level, field(reference, 9).parse().unwrap());
        assert_eq!(
          actual.df_residual, None,
          "{model} {covariance} {combination}"
        );
        assert_eq!(
          session.state(),
          retained_state,
          "{model} {covariance} {combination}"
        );
      }

      let ExecutionResult::Lincom(zero_se) = run(&mut session, "lincom 2") else {
        panic!("expected zero-SE binary linear combination")
      };
      assert_eq!(zero_se.estimate, 2.0);
      assert_eq!(zero_se.standard_error, 0.0);
      assert!(zero_se.statistic.is_nan());
      assert!(zero_se.p_value.is_nan());
      assert_eq!((zero_se.ci_lower, zero_se.ci_upper), (2.0, 2.0));
      assert_eq!(zero_se.df_residual, None);
      assert_eq!(session.state(), retained_state);

      for command in ["lincom absent", "lincom x * intercept"] {
        assert!(matches!(
          fail(&mut session, command),
          RuntimeError::LincomFailed { .. }
        ));
        assert_eq!(
          session.state(),
          retained_state,
          "{model} {covariance} {command}"
        );
      }
    }
  }
}

#[test]
fn retained_classification_matches_base_r_at_default_and_custom_cutoffs() {
  let fixture = Fixture::new();
  for model in ["logit", "probit"] {
    let mut session = fixture.load();
    run(&mut session, &format!("{model} y x i.region"));
    let fit_state = session.state().estimation;

    for (cutoff, command) in [
      (0.5, "estat classification"),
      (0.35, "estat classification, cutoff(.35)"),
    ] {
      let ExecutionResult::BinaryClassification(actual) = run(&mut session, command) else {
        panic!("expected retained classification after {model}")
      };
      let reference = binary_classification_reference(model, cutoff);
      assert_eq!(reference.len(), 1, "{model} cutoff {cutoff}");
      assert_eq!(
        actual.model,
        if model == "logit" {
          BinaryModel::Logit
        } else {
          BinaryModel::Probit
        }
      );
      assert_eq!(actual.outcome, "y");
      assert_eq!(actual.metrics.cutoff, cutoff);
      assert_eq!(actual.metrics.observation_count, 237);
      assert_eq!(actual.sample.as_ref().unwrap().retained_indices.len(), 237);
      for (name, value, index) in [
        ("N", actual.metrics.observation_count, 2),
        ("actual positive", actual.metrics.actual_positive_count, 3),
        ("actual negative", actual.metrics.actual_negative_count, 4),
        (
          "predicted positive",
          actual.metrics.predicted_positive_count,
          5,
        ),
        (
          "predicted negative",
          actual.metrics.predicted_negative_count,
          6,
        ),
        ("true positive", actual.metrics.true_positive, 7),
        ("false positive", actual.metrics.false_positive, 8),
        ("false negative", actual.metrics.false_negative, 9),
        ("true negative", actual.metrics.true_negative, 10),
      ] {
        assert_eq!(
          value,
          field(&reference[0], index).parse::<usize>().unwrap(),
          "{model} {cutoff} {name}"
        );
      }
      for (name, value, index) in [
        ("sensitivity", actual.metrics.sensitivity, 11),
        ("specificity", actual.metrics.specificity, 12),
        (
          "positive predictive value",
          actual.metrics.positive_predictive_value,
          13,
        ),
        (
          "negative predictive value",
          actual.metrics.negative_predictive_value,
          14,
        ),
        (
          "false-positive rate among actual negatives",
          actual.metrics.false_positive_rate_given_negative,
          15,
        ),
        (
          "false-negative rate among actual positives",
          actual.metrics.false_negative_rate_given_positive,
          16,
        ),
        (
          "false-positive rate among predicted positives",
          actual.metrics.false_positive_rate_given_positive,
          17,
        ),
        (
          "false-negative rate among predicted negatives",
          actual.metrics.false_negative_rate_given_negative,
          18,
        ),
        (
          "correct classification",
          actual.metrics.correctly_classified,
          19,
        ),
      ] {
        close_named(
          value.unwrap(),
          field(&reference[0], index).parse().unwrap(),
          &format!("{model} {cutoff} {name}"),
        );
      }
      assert_eq!(session.state().estimation, fit_state);
    }

    run(&mut session, "replace y = 1 - y if id == 1");
    run(&mut session, "replace x = x + 100 if id == 2");
    run(&mut session, "replace region = 'Unused' if id == 3");
    let ExecutionResult::BinaryClassification(after_edit) =
      run(&mut session, "estat classification, cutoff(.35)")
    else {
      panic!("expected retained classification after {model} data edits")
    };
    assert_eq!(after_edit.metrics.cutoff, 0.35);
    assert_eq!(after_edit.metrics.observation_count, 237);
    assert_eq!(session.state().estimation, fit_state);
  }
}

#[test]
fn classification_boundary_cutoffs_and_missing_fit_fail_without_mutation() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  assert!(matches!(
    fail(&mut session, "estat classification"),
    RuntimeError::EstatClassificationRequiresPriorBinary
  ));
  assert!(session.state().estimation.is_none());

  run(&mut session, "logit y x");
  let fit_state = session.state().estimation;
  let ExecutionResult::BinaryClassification(at_zero) =
    run(&mut session, "estat classification, cutoff(0)")
  else {
    panic!("expected classification at zero cutoff")
  };
  assert_eq!(at_zero.metrics.predicted_positive_count, 238);
  assert_eq!(at_zero.metrics.true_negative, 0);
  assert_eq!(at_zero.metrics.specificity, Some(0.0));
  assert_eq!(at_zero.metrics.negative_predictive_value, None);
  assert_eq!(at_zero.metrics.false_negative_rate_given_negative, None);
  let ExecutionResult::BinaryClassification(at_one) =
    run(&mut session, "estat classification, cutoff(1)")
  else {
    panic!("expected classification at one cutoff")
  };
  assert_eq!(at_one.metrics.predicted_positive_count, 0);
  assert_eq!(at_one.metrics.true_positive, 0);
  assert_eq!(at_one.metrics.positive_predictive_value, None);
  assert_eq!(at_one.metrics.false_positive_rate_given_positive, None);
  assert_eq!(session.state().estimation, fit_state);
}
