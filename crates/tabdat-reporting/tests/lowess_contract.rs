use serde_json::{Value, json};
use tabdat_domain::{ColumnInfo, DatasetInfo, ExecutionMode, ExecutionResult, LowessResult};
use tabdat_language::parse_command;
use tabdat_reporting::{
  catalog::describe_command_result,
  json_results::{serialize_execution_result, supports_batch_command, supports_script_command},
  terminal::{render_execution_result, render_interactive_help},
};
use tabdat_stats::EstimationSample;

fn result() -> ExecutionResult {
  ExecutionResult::Lowess(Box::new(LowessResult {
    dataset: DatasetInfo {
      source: "input.csv".into(),
      row_count: 9,
      columns: vec![
        ColumnInfo {
          name: "outcome\nvar".into(),
          data_type: "DOUBLE".into(),
        },
        ColumnInfo {
          name: "x".into(),
          data_type: "DOUBLE".into(),
        },
        ColumnInfo {
          name: "fit\tcol".into(),
          data_type: "DOUBLE".into(),
        },
      ],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    },
    outcome: "outcome\nvar".into(),
    predictor: "x".into(),
    target_variable: "fit\tcol".into(),
    bandwidth: 0.4,
    sample: EstimationSample::new(vec![0, 2, 3, 4, 5, 7, 8], 9, 2, None, None).unwrap(),
    half_window: 1,
    local_constant_windows: 7,
  }))
}
#[test]
fn lowess_cli_v1_reports_exact_sample_and_method_without_invented_inference() {
  let wire = serialize_execution_result(&result()).unwrap();
  assert!(!wire.contains('\n'));
  let v: Value = serde_json::from_str(&wire).unwrap();
  assert_eq!(v["schema_version"], 1);
  assert_eq!(v["result_type"], "LowessResult");
  let d = &v["data"];
  assert_eq!(d["outcome"], "outcome\nvar");
  assert_eq!(d["predictor"], "x");
  assert_eq!(d["target_variable"], "fit\tcol");
  assert_eq!(d["bandwidth"], 0.4);
  assert_eq!(d["observation_count"], 7);
  assert_eq!(
    d["sample"]["retained_indices"],
    json!([0, 2, 3, 4, 5, 7, 8])
  );
  assert_eq!(d["sample"]["total_observations"], 9);
  assert_eq!(d["sample"]["dropped_observations"], 2);
  assert!(d["sample"]["weights"].is_null() && d["sample"]["cluster_groups"].is_null());
  assert_eq!(d["half_window"], 1);
  assert_eq!(d["local_constant_windows"], 7);
  assert_eq!(d["algorithm"], "centered_rank_local_linear_tricube");
  assert_eq!(d["kernel_endpoint_inflation"], 1.0001);
  assert_eq!(d["robust_iterations"], 0);
  assert_eq!(d["interpolation"], "none");
  assert_eq!(d["neighborhood"], "rank_centered_shrinking_at_endpoints");
  assert_eq!(
    d["ties"],
    "stable_numeric_predictor_then_eligible_input_order_sensitive"
  );
  assert_eq!(d["fallback"], "exact_zero_radius_window_local_mean");
  assert_eq!(d["estimation_state"], "unchanged");
  assert_eq!(d["out_of_sample_prediction"], false);
  assert_eq!(d["inference"], "none_training_row_smoother");
  assert_eq!(d["dataset"]["row_count"], 9);
  for absent in [
    "coefficients",
    "covariance",
    "standard_errors",
    "p_values",
    "log_likelihood",
    "model_statistics",
  ] {
    assert!(d.get(absent).is_none());
  }
}
#[test]
fn lowess_human_output_explains_order_state_and_scope_and_escapes_names() {
  let text = render_execution_result(&result());
  for expected in [
    "Centered-rank LOWESS",
    "nonrobust local linear / tricube",
    "Retained 7 / 9; excluded 2",
    "Bandwidth fraction 0.4; half-window 1; coincident-X local means 7",
    "Tied-X fits may differ and depend on input order",
    "no inference or out-of-sample model",
    "previous estimation state unchanged",
    "outcome\\nvar",
    "fit\\tcol",
  ] {
    assert!(text.contains(expected), "missing {expected}: {text}");
  }
  assert!(!text.contains("outcome\nvar"));
  assert!(!text.contains("fit\tcol"));
}
#[test]
fn lowess_batch_script_admission_schema_and_help_match_installed_parser() {
  for command in ["lowess y x, gen(fit)", "lowess y x, gen(fit) bandwidth=0.4"] {
    let parsed = parse_command(command).unwrap();
    assert!(supports_batch_command(&parsed));
    assert!(supports_script_command(&parsed));
  }
  let schema = describe_command_result("lowess").unwrap();
  assert_eq!(schema.arguments.len(), 2);
  assert!(schema.arguments.iter().all(|a| a.required));
  assert!(schema.options.iter().any(|o| o.name == "gen" && o.required));
  assert!(
    schema
      .options
      .iter()
      .any(|o| o.name == "bandwidth" && !o.required)
  );
  assert_eq!(schema.syntax, "lowess y x, gen(newvar) [bandwidth=<0,1>]");
  let help = render_interactive_help(Some("lowess")).unwrap();
  for text in [
    "default 2/3",
    "within-tie row order",
    "window-local",
    "23,000,000",
    "PREVIOUS retained model",
    "nearest-neighbour",
  ] {
    assert!(help.contains(text), "missing {text}: {help}");
  }
}
