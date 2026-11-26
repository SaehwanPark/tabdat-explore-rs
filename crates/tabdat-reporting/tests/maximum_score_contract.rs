use serde_json::Value;
use tabdat_domain::{CommandEffect, ExecutionResult, MaximumScoreFit};
use tabdat_language::parse_command;
use tabdat_reporting::{
  catalog::{
    command_catalog_result, command_effect_catalog_result, describe_command_result,
    help_topic_result,
  },
  json_results::{serialize_execution_result, supports_batch_command, supports_script_command},
  terminal::render_execution_result,
};

#[test]
fn point_only_maxscore_has_a_stable_json_terminal_and_discovery_contract() {
  let command = parse_command("maxscore y x").unwrap();
  assert!(supports_batch_command(&command));
  assert!(supports_script_command(&command));
  let classification = parse_command("predict class, class").unwrap();
  assert!(supports_batch_command(&classification));
  assert!(supports_script_command(&classification));

  let result =
    tabdat_stats::fit_binary_maximum_score(&[0.0, 0.0, 1.0, 1.0], &[0.0, 1.0, 2.0, 3.0]).unwrap();
  let execution = ExecutionResult::MaximumScore(Box::new(MaximumScoreFit {
    outcome: "y".into(),
    predictor: "x".into(),
    sample: tabdat_stats::EstimationSample::all_retained(4),
    result,
  }));
  let encoded = serialize_execution_result(&execution).unwrap();
  let envelope: Value = serde_json::from_str(&encoded).unwrap();
  assert_eq!(envelope["result_type"], "MaximumScoreResult");
  let data = &envelope["data"];
  assert_eq!(
    data["estimator"],
    "binary_maximum_score_exhaustive_one_predictor"
  );
  assert_eq!(data["coefficient_normalization"], "euclidean_unit_norm");
  assert_eq!(data["inference"], "not_provided");
  assert_eq!(data["probability_predictions"], false);
  assert_eq!(data["sample"]["retained_observations"], 4);
  assert!(data["result"].get("optimal_regions").is_some());
  for unsupported in [
    "covariance",
    "standard_error",
    "p_value",
    "ci_lower",
    "ci_upper",
  ] {
    assert!(data.get(unsupported).is_none(), "{unsupported}");
    assert!(data["result"].get(unsupported).is_none(), "{unsupported}");
  }

  let text = render_execution_result(&execution);
  assert!(text.contains("All 1 finite-sample maximizing regions (region 1 selected"));
  assert!(text.contains("no standard errors"));
  assert!(text.contains("TP = 2; FP = 0; FN = 0; TN = 2"));

  assert_eq!(
    describe_command_result("maxscore").unwrap().syntax,
    "maxscore binary_y continuous_x"
  );
  assert!(
    help_topic_result("maxscore")
      .unwrap()
      .text
      .contains("point estimation")
  );
  assert!(tabdat_reporting::help::load_help_topic_text("maxscore").is_some());
  assert!(
    command_catalog_result()
      .commands
      .iter()
      .any(|entry| entry.name == "maxscore" && entry.help_topic.as_deref() == Some("maxscore"))
  );
  assert_eq!(
    command_effect_catalog_result()
      .commands
      .iter()
      .find(|entry| entry.name == "maxscore")
      .unwrap()
      .effects,
    [CommandEffect::Read]
  );
}
