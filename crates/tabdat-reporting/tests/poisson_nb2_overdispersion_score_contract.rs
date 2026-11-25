use serde_json::Value;
use tabdat_domain::{
  EstatSubcommand, ExecutionResult, PoissonCluster, PoissonOverdispersionScoreTest,
};
use tabdat_language::parse_command;
use tabdat_reporting::{
  catalog::describe_command_result,
  json_results::{serialize_execution_result, supports_batch_command, supports_script_command},
  terminal::render_execution_result,
};

fn result(clustered: bool) -> ExecutionResult {
  ExecutionResult::PoissonOverdispersionScoreTest(Box::new(PoissonOverdispersionScoreTest {
    method: "one_sided_nb2_variance_score".into(),
    null_hypothesis: "alpha = 0 (Poisson conditional variance)".into(),
    alternative_hypothesis: "alpha > 0 in Var(Y|X) = mu + alpha*mu^2".into(),
    statistic: 1.74139309981589,
    p_value: 0.0408073484465351,
    score: 27.2539065767643,
    score_variance: 244.942353974404,
    covariance: if clustered {
      "cluster_nb2_score_g_over_g_minus_1".into()
    } else {
      "robust_row_nb2_score_n_over_n_minus_1".into()
    },
    covariance_correction: if clustered {
      30.0 / 29.0
    } else {
      180.0 / 179.0
    },
    reference_distribution: "asymptotic_standard_normal".into(),
    observation_count: 180,
    offset_variable: Some("log_exposure".into()),
    exposure_variable: None,
    cluster: clustered.then(|| PoissonCluster {
      variable: "group".into(),
      group_count: 30,
    }),
  }))
}

#[test]
fn json_and_terminal_contract_name_score_direction_covariance_and_assumptions() {
  let execution = result(true);
  let wire = serialize_execution_result(&execution).unwrap();
  assert!(!wire.contains('\n'));
  let envelope: Value = serde_json::from_str(&wire).unwrap();
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(
    envelope["result_type"],
    "PoissonOverdispersionScoreTestResult"
  );
  let data = &envelope["data"];
  assert_eq!(data["estimator"], "poisson_nb2_overdispersion_score");
  assert_eq!(data["effect"], "one_sided_test_alpha_greater_than_zero");
  assert_eq!(data["sample"], "retained_estimation_design");
  assert_eq!(
    data["uncertainty"],
    "selected_score_variance_conditional_on_retained_fit"
  );
  assert_eq!(data["independent_unit"], "cluster");
  assert_eq!(data["method"], "one_sided_nb2_variance_score");
  assert_eq!(data["score"], 27.2539065767643);
  assert_eq!(data["score_variance"], 244.942353974404);
  assert_eq!(data["statistic"], 1.74139309981589);
  assert_eq!(data["p_value"], 0.0408073484465351);
  assert_eq!(data["covariance_correction"], 30.0 / 29.0);
  assert_eq!(data["reference_distribution"], "asymptotic_standard_normal");
  assert_eq!(data["offset_variable"], "log_exposure");
  assert_eq!(data["cluster"]["variable"], "group");
  assert_eq!(data["cluster"]["group_count"], 30);

  let text = render_execution_result(&execution);
  for expected in [
    "Poisson one-sided NB2 overdispersion score test",
    "Retained N = 180",
    "one-sided p =",
    "cluster_nb2_score_g_over_g_minus_1",
    "independent clusters, G = 30",
    "many-group approximation only, no few-cluster guarantee",
    "Correct Poisson conditional means are required",
    "Fixed offset log_exposure is included",
  ] {
    assert!(text.contains(expected), "missing {expected:?}: {text}");
  }

  let model_text = render_execution_result(&result(false));
  assert!(model_text.contains("independent rows"));
}

#[test]
fn overdispersion_is_admitted_for_cli_batch_and_scripts_and_discovered() {
  let command = parse_command("estat overdispersion").unwrap();
  assert!(supports_batch_command(&command));
  assert!(supports_script_command(&command));
  assert!(matches!(
    command,
    tabdat_domain::Command::Estat {
      command: tabdat_domain::EstatCommand {
        subcommand: EstatSubcommand::Overdispersion
      }
    }
  ));
  let schema = describe_command_result("estat").unwrap();
  assert!(schema.syntax.contains("estat overdispersion"));
}
