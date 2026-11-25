use serde_json::Value;
use tabdat_domain::{BinaryGoodnessOfFitResult, EstatSubcommand, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_reporting::{
  catalog::describe_command_result,
  json_results::{serialize_execution_result, supports_batch_command, supports_script_command},
  terminal::render_execution_result,
};
use tabdat_stats::{
  BinaryHosmerLemeshowGroup, BinaryHosmerLemeshowResult, BinaryModel, EstimationSample,
};

fn result() -> ExecutionResult {
  ExecutionResult::BinaryGoodnessOfFit(Box::new(BinaryGoodnessOfFitResult {
    model: BinaryModel::Logit,
    outcome: "admit".into(),
    sample: Some(EstimationSample::new(vec![0, 2, 3], 4, 1, None, None).unwrap()),
    test: BinaryHosmerLemeshowResult {
      requested_group_count: 10,
      group_count: 3,
      statistic: 1.6486928104575164,
      degrees_of_freedom: 1,
      p_value: 0.19913693998396328,
      groups: vec![
        BinaryHosmerLemeshowGroup {
          group: 1,
          probability_upper_bound: 0.2,
          observation_count: 1,
          observed_events: 0,
          expected_events: 0.15,
          observed_nonevents: 1,
          expected_nonevents: 0.85,
        },
        BinaryHosmerLemeshowGroup {
          group: 2,
          probability_upper_bound: 0.5,
          observation_count: 1,
          observed_events: 1,
          expected_events: 0.45,
          observed_nonevents: 0,
          expected_nonevents: 0.55,
        },
        BinaryHosmerLemeshowGroup {
          group: 3,
          probability_upper_bound: 0.9,
          observation_count: 1,
          observed_events: 1,
          expected_events: 0.8,
          observed_nonevents: 0,
          expected_nonevents: 0.2,
        },
      ],
    },
  }))
}

#[test]
fn binary_gof_json_and_terminal_output_name_the_grouped_estimand_and_reference() {
  let result = result();
  let wire = serialize_execution_result(&result).unwrap();
  assert!(!wire.contains('\n'));
  let envelope: Value = serde_json::from_str(&wire).unwrap();
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["result_type"], "BinaryGoodnessOfFitResult");
  assert_eq!(envelope["data"]["estimator"], "logit");
  assert_eq!(
    envelope["data"]["method"],
    "hosmer_lemeshow_quantile_group_calibration"
  );
  assert_eq!(envelope["data"]["outcome"], "admit");
  assert_eq!(envelope["data"]["requested_group_count"], 10);
  assert_eq!(envelope["data"]["group_count"], 3);
  assert_eq!(envelope["data"]["degrees_of_freedom"], 1);
  assert_eq!(envelope["data"]["statistic"], 1.6486928104575164);
  assert_eq!(envelope["data"]["observation_count"], 3);
  assert_eq!(
    envelope["data"]["covariance"],
    "unweighted_model_based_bernoulli"
  );
  assert_eq!(
    envelope["data"]["reference_distribution"]["family"],
    "chi_square"
  );
  assert_eq!(
    envelope["data"]["sample_role"],
    "retained_estimation_sample"
  );
  assert_eq!(envelope["data"]["groups"][1]["observed_events"], 1);

  let text = render_execution_result(&result);
  for expected in [
    "Logit Hosmer-Lemeshow grouped goodness-of-fit test",
    "retained estimation N = 3",
    "Requested groups = 10; actual groups = 3",
    "Hosmer-Lemeshow chi-square(1) =",
    "Observed events",
    "sparse expected cells",
  ] {
    assert!(text.contains(expected), "missing {expected:?}: {text}");
  }
}

#[test]
fn grouped_gof_is_admitted_and_discoverable_without_admitting_bare_gof() {
  let command = parse_command("estat gof, group(10)").unwrap();
  assert!(supports_batch_command(&command));
  assert!(supports_script_command(&command));
  assert_eq!(
    command,
    tabdat_domain::Command::Estat {
      command: tabdat_domain::EstatCommand {
        subcommand: EstatSubcommand::BinaryGoodnessOfFit { group_count: 10 },
      },
    }
  );
  assert!(!supports_batch_command(
    &parse_command("estat gof").unwrap()
  ));
  let schema = describe_command_result("estat").unwrap();
  assert!(schema.syntax.contains("estat gof, group(10)"));
  assert!(schema.options.iter().any(|option| option.name == "group"));
}
