use serde_json::Value;
use tabdat_domain::ExecutionResult;
use tabdat_language::parse_command;
use tabdat_reporting::{
  json_results::{serialize_execution_result, supports_batch_command, supports_script_command},
  terminal::render_execution_result,
};
use tabdat_stats::{EstimationSample, NlsExpression, NlsModel, fit_nls};
fn result(robust: bool) -> ExecutionResult {
  let model = NlsModel {
    outcome: "y\nlabel".into(),
    predictor_names: vec![],
    parameter_names: vec!["a\tname".into()],
    expression: NlsExpression::Exp(Box::new(NlsExpression::Parameter(0))),
  };
  ExecutionResult::Nl(Box::new(
    fit_nls(
      &model,
      &[1.0, 2.0, 1.0, 3.0, 4.0],
      &vec![vec![]; 5],
      &[0.0],
      &EstimationSample::new(vec![0, 2, 3, 4, 6], 7, 2, None, None).unwrap(),
      robust,
    )
    .unwrap(),
  ))
}
#[test]
fn nls_cli_v1_has_full_owned_result_and_honest_gn_t_metadata() {
  for robust in [false, true] {
    let r = result(robust);
    let ExecutionResult::Nl(fit) = &r else {
      unreachable!()
    };
    let text = serialize_execution_result(&r).unwrap();
    assert!(!text.contains('\n'));
    let wire: Value = serde_json::from_str(&text).unwrap();
    let data = &wire["data"];
    assert_eq!(wire["schema_version"], 1);
    assert_eq!(wire["result_type"], "NlsResult");
    assert_eq!(data["model"]["outcome"], "y\nlabel");
    assert_eq!(data["coefficients"].as_array().unwrap().len(), 1);
    assert_eq!(data["covariance_matrix"].as_array().unwrap().len(), 1);
    assert_eq!(
      data["sample"]["retained_indices"],
      serde_json::json!([0, 2, 3, 4, 6])
    );
    assert_eq!(data["sample"]["total_observations"], 7);
    assert_eq!(
      data["covariance"],
      if robust {
        "hc1_gauss_newton"
      } else {
        "working_homoskedastic_gauss_newton"
      }
    );
    assert_eq!(data["covariance_bread"], "inverse_jacobian_cross_product");
    assert_eq!(data["covariance_correction"], fit.covariance_correction);
    assert_eq!(
      data["inference"]["reference_distribution"],
      "approximate_student_t"
    );
    assert_eq!(data["inference"]["degrees_of_freedom"], 4);
    assert_eq!(data["inference"]["confidence_level"], 0.95);
    for key in ["log_likelihood", "likelihood_ratio", "omnibus_f"] {
      assert!(data[key].is_null());
    }
    assert_eq!(data["fitted_values"].as_array().unwrap().len(), 5);
    assert_eq!(data["residuals"].as_array().unwrap().len(), 5);
    assert!(data["normalized_score"].as_f64().unwrap() <= 1e-12);
    assert!(data["relative_full_gauss_newton_step"].as_f64().unwrap() <= 1e-12);
  }
}
#[test]
fn nls_text_reports_formula_local_scope_sample_inference_and_prediction_semantics() {
  for robust in [false, true] {
    let text = render_execution_result(&result(robust));
    for expected in [
      "Nonlinear least squares",
      "regular local solution",
      "not a global optimum",
      "y\\nlabel",
      "mean = exp(a\\tname)",
      "no automatic intercept",
      "N = 5; candidate rows = 7; excluded joint missing/nonfinite = 2; residual df = 4",
      "Approximate Student t(4)",
      "correctly specified conditional mean",
      "relative full undamped step",
      "positive observed-loss curvature",
      "No likelihood, LR, omnibus F",
      "using current X",
      "residuals additionally require current y",
    ] {
      assert!(text.contains(expected), "missing {expected:?}: {text}");
    }
    assert!(text.contains(if robust {
      "HC1 Gauss-Newton sandwich N/(N-P)"
    } else {
      "working homoskedastic RSS/(N-P)"
    }));
  }
}
#[test]
fn nls_is_admitted_in_both_structured_execution_paths() {
  for text in [
    "nl y = exp(-a-b*x), params(a b) start(1 0)",
    "nl y = a/(b+x), params(a b) start(3 1) robust noconstant",
  ] {
    let command = parse_command(text).unwrap();
    assert!(supports_batch_command(&command));
    assert!(supports_script_command(&command));
  }
}
