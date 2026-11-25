use serde_json::Value;
use tabdat_domain::{EstatSubcommand, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_reporting::{
  catalog::describe_command_result,
  json_results::{serialize_execution_result, supports_batch_command, supports_script_command},
  terminal::render_execution_result,
};
use tabdat_stats::{BinaryCluster, BinaryDiscreteChange, BinaryModel, LogitEstimate};

fn result(model: BinaryModel, robust: bool, cluster: Option<BinaryCluster>) -> ExecutionResult {
  ExecutionResult::BinaryDiscreteChange(Box::new(BinaryDiscreteChange {
    model,
    predictor: "treated".into(),
    from_value: 0,
    to_value: 1,
    effect: LogitEstimate {
      name: "treated".into(),
      value: 0.183329669025602,
      standard_error: 0.0882884728319724,
      statistic: 2.076,
      p_value: 0.038,
      confidence_lower: 0.0103,
      confidence_upper: 0.3563,
    },
    robust,
    cluster,
    observation_count: 120,
  }))
}

#[test]
fn binary_discrete_change_json_and_terminal_output_name_estimand_and_inference() {
  let clustered = result(
    BinaryModel::Logit,
    true,
    Some(BinaryCluster {
      variable: "group".into(),
      group_count: 20,
    }),
  );
  let wire = serialize_execution_result(&clustered).unwrap();
  assert!(!wire.contains('\n'));
  let envelope: Value = serde_json::from_str(&wire).unwrap();
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["result_type"], "BinaryDiscreteChangeResult");
  let data = &envelope["data"];
  assert_eq!(data["estimator"], "logit");
  assert_eq!(data["effect"], "average_discrete_probability_change");
  assert_eq!(data["predictor"], "treated");
  assert_eq!(
    (data["from_value"].as_u64(), data["to_value"].as_u64()),
    (Some(0), Some(1))
  );
  assert_eq!(data["sample"], "retained_estimation_design");
  assert_eq!(
    data["uncertainty"],
    "conditional_on_observed_design_delta_method"
  );
  assert_eq!(data["covariance"], "cluster_sandwich_g_over_g_minus_1");
  assert_eq!(data["cluster"]["group_count"], 20);
  assert_eq!(data["reference_distribution"], "asymptotic_standard_normal");
  assert_eq!(data["causal_interpretation"], false);
  assert_eq!(data["effect_estimate"]["value"], 0.183329669025602);
  assert_eq!(
    data["effect_estimate"]["standard_error"],
    0.0882884728319724
  );

  let text = render_execution_result(&clustered);
  for expected in [
    "Logit discrete average probability change",
    "retained estimation N = 120",
    "Predictor treated set from 0 to 1",
    "other predictors stay at their observed retained-row values",
    "G/(G-1) correction",
    "asymptotic normal 95% interval",
    "This is not a causal effect",
  ] {
    assert!(text.contains(expected), "missing {expected:?}: {text}");
  }

  let robust_probit = result(BinaryModel::Probit, true, None);
  let envelope: Value =
    serde_json::from_str(&serialize_execution_result(&robust_probit).unwrap()).unwrap();
  assert_eq!(envelope["data"]["estimator"], "probit");
  assert_eq!(
    envelope["data"]["covariance"],
    "robust_sandwich_n_over_n_minus_1"
  );
  assert!(render_execution_result(&robust_probit).contains("Probit discrete average"));
}

#[test]
fn discrete_margins_is_admitted_for_json_and_scripts_without_changing_bare_margins() {
  let command = parse_command("estat margins, discrete(treated)").unwrap();
  assert!(supports_batch_command(&command));
  assert!(supports_script_command(&command));
  assert_eq!(
    parse_command("estat margins").unwrap(),
    tabdat_domain::Command::Estat {
      command: tabdat_domain::EstatCommand {
        subcommand: EstatSubcommand::Margins,
      },
    }
  );
  let schema = describe_command_result("estat").unwrap();
  assert!(schema.syntax.contains("estat margins, discrete(variable)"));
  assert!(
    schema
      .options
      .iter()
      .any(|option| option.name == "discrete")
  );
}

#[test]
fn factor_margins_is_admitted_for_json_and_scripts_without_changing_bare_margins() {
  let command = parse_command("estat margins, factor(region)").unwrap();
  assert!(supports_batch_command(&command));
  assert!(supports_script_command(&command));
  assert_eq!(
    parse_command("estat margins").unwrap(),
    tabdat_domain::Command::Estat {
      command: tabdat_domain::EstatCommand {
        subcommand: EstatSubcommand::Margins,
      },
    }
  );
  let schema = describe_command_result("estat").unwrap();
  assert!(schema.syntax.contains("estat margins, factor(variable)"));
  assert!(schema.options.iter().any(|option| option.name == "factor"));
}
