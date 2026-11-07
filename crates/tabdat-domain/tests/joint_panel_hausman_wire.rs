use tabdat_domain::{DomainEnvelope, ExecutionResult, RandomEffectsState};

#[test]
fn joint_result_has_additive_lossless_tag_and_owned_equal_group_state_is_legacy_readable() {
  let rows: Vec<_> = include_str!(
    "../../tabdat-stats/tests/fixtures/panel_joint_equal_group_hausman/balanced_input.csv"
  )
  .lines()
  .skip(1)
  .map(|l| l.split(',').collect::<Vec<_>>())
  .collect();
  let n = rows.len();
  let sample = tabdat_stats::EstimationSample::new(
    (0..n).collect(),
    n,
    0,
    None,
    Some(rows.iter().map(|r| r[8].to_owned()).collect()),
  )
  .unwrap();
  let problem = tabdat_stats::EstimationProblem::new(
    "lnexp12m",
    ["lnhhexp", "age", "sex", "hhsize", "farm", "educ"]
      .map(str::to_owned)
      .to_vec(),
    rows.iter().map(|r| r[1].parse().unwrap()).collect(),
    rows
      .iter()
      .map(|r| [7, 2, 3, 4, 5, 6].map(|j| r[j].parse().unwrap()).to_vec())
      .collect(),
  )
  .unwrap()
  .with_sample(sample);
  let re = tabdat_stats::fit_random_effects_equal_group_between(&problem, "commune").unwrap();
  let joint = tabdat_stats::joint_equal_group_panel_hausman(&problem, &re).unwrap();
  let result = ExecutionResult::JointPanelHausman(joint.clone());
  let value = serde_json::to_value(DomainEnvelope::new(result.clone())).unwrap();
  assert_eq!(value["schema_version"], 1);
  assert_eq!(value["data"]["kind"], "joint_panel_hausman");
  assert_eq!(
    value["data"]["payload"]["covariance_scale_convention"],
    "separate_native_fe_and_re_residual_scales"
  );
  assert_eq!(
    value["data"]["payload"]["re_component_method"],
    "equal_group_between"
  );
  assert_eq!(
    value["data"]["payload"]["fe_covariance"]
      .as_array()
      .unwrap()
      .len(),
    6
  );
  assert_eq!(
    serde_json::from_value::<DomainEnvelope<ExecutionResult>>(value)
      .unwrap()
      .data,
    result
  );
  let state = RandomEffectsState {
    time_variable: "source_row".into(),
    entity_data_type: "BIGINT".into(),
    result: re,
    hausman_problem: Some(problem),
  };
  let mut value = serde_json::to_value(&state).unwrap();
  assert!(value.get("hausman_problem").is_some());
  let roundtrip: RandomEffectsState = serde_json::from_value(value.clone()).unwrap();
  assert_eq!(roundtrip, state);
  assert_eq!(
    tabdat_stats::joint_equal_group_panel_hausman(
      roundtrip.hausman_problem.as_ref().unwrap(),
      &roundtrip.result
    )
    .unwrap(),
    joint
  );
  value.as_object_mut().unwrap().remove("hausman_problem");
  let old: RandomEffectsState = serde_json::from_value(value.clone()).unwrap();
  assert!(old.hausman_problem.is_none());
  assert_eq!(serde_json::to_value(old).unwrap(), value);
}
