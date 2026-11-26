use tabdat_domain::Command;
use tabdat_language::parse_command;
#[test]
fn bootstrap_requires_explicit_exact_seed_and_bounded_iid_ols_options() {
  let Command::Bootstrap { command } =
    parse_command("bootstrap `my y` x, seed(18446744073709551615) reps(40) noconstant").unwrap()
  else {
    panic!("bootstrap")
  };
  assert_eq!(command.seed, u64::MAX);
  assert_eq!(command.replications, 40);
  assert!(!command.regression.include_intercept);
  assert_eq!(command.regression.outcome, "my y");
  for text in [
    "bootstrap y x",
    "bootstrap y x, seed(-1)",
    "bootstrap y x, seed(18446744073709551616)",
    "bootstrap y x, seed(1.5)",
    "bootstrap y x, seed(1) seed(2)",
    "bootstrap y x, seed(1) reps(1)",
    "bootstrap y x, seed(1) reps(5001)",
    "bootstrap y x, seed(1) reps(2.5)",
    "bootstrap y x, seed(1) robust",
    "bootstrap y x, seed(1) cluster(id other)",
    "bootstrap y x, seed(1) cluster(1)",
    "bootstrap y x, seed(1) cluster(id) cluster(id)",
    "bootstrap y x if x>0, seed(1)",
    "bootstrap y x, seed(1) noconstant(true)",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}

#[test]
fn whole_cluster_option_uses_one_exact_identifier() {
  let Command::Bootstrap { command } =
    parse_command("bootstrap y x, seed(7) cluster(`unit key`)").unwrap()
  else {
    panic!("bootstrap")
  };
  assert_eq!(
    command.regression.cluster_variable.as_deref(),
    Some("unit key")
  );
}

#[test]
fn joint_quantiles_are_ordered_bounded_and_keep_exact_seed_spelling() {
  let Command::QuantileBootstrap { command } = parse_command(
    "bootstrap y x, quantiles(.05 .25 .5 .75 .95) seed(18446744073709551615) reps(40) noconstant",
  )
  .unwrap() else {
    panic!("joint quantiles")
  };
  assert_eq!(command.quantiles, [".05", ".25", ".5", ".75", ".95"]);
  assert_eq!(command.bootstrap.seed, u64::MAX);
  assert!(!command.bootstrap.regression.include_intercept);
  assert!(matches!(
    parse_command("bootstrap y x, quantiles(.5) seed(7)").unwrap(),
    Command::QuantileBootstrap { .. }
  ));
  let Command::QuantileBootstrap { command } =
    parse_command("bootstrap y x, quantiles(.25 .75) cluster(id) seed(7)").unwrap()
  else {
    panic!("clustered joint quantiles")
  };
  assert_eq!(
    command.bootstrap.regression.cluster_variable.as_deref(),
    Some("id")
  );
  for text in [
    "bootstrap y x, quantiles(.5 .5) seed(7)",
    "bootstrap y x, quantiles(.9 .1) seed(7)",
    "bootstrap y x, quantiles(.01 .5) seed(7)",
    "bootstrap y x, quantiles() seed(7)",
    "bootstrap y x, quantiles(x) seed(7)",
    "bootstrap y x, quantiles(.5)",
    "bootstrap y x, quantiles(.5) quantiles(.9) seed(7)",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}

#[test]
fn contrast_requires_one_exact_coefficient_and_two_quantiles() {
  let Command::QuantileBootstrap { command } =
    parse_command("bootstrap y `my x`, quantiles(.25 .75) contrast(`my x`) seed(7)").unwrap()
  else {
    panic!("contrast")
  };
  assert_eq!(command.contrast.as_deref(), Some("my x"));
  let Command::QuantileBootstrap { command } =
    parse_command("bootstrap y x, quantiles(.25 .75) seed(7)").unwrap()
  else {
    panic!("old command")
  };
  let mut wire = serde_json::to_value(&command).unwrap();
  assert!(wire.get("contrast").is_none());
  let old: tabdat_domain::QuantileBootstrapCommand = serde_json::from_value(wire.clone()).unwrap();
  assert_eq!(old, command);
  wire["contrast"] = serde_json::Value::Null;
  assert_eq!(
    serde_json::from_value::<tabdat_domain::QuantileBootstrapCommand>(wire).unwrap(),
    command
  );
  for text in [
    "bootstrap y x, contrast(x) seed(7)",
    "bootstrap y x, quantiles(.5) contrast(x) seed(7)",
    "bootstrap y x, quantiles(.25 .5 .75) contrast(x) seed(7)",
    "bootstrap y x, quantiles(.25 .75) contrast(x z) seed(7)",
    "bootstrap y x, quantiles(.25 .75) contrast(1) seed(7)",
    "bootstrap y x, quantiles(.25 .75) contrast seed(7)",
    "bootstrap y x, quantiles(.25 .75) contrast(x) contrast(x) seed(7)",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
  let Command::QuantileBootstrap { command } =
    parse_command("bootstrap y x, quantiles(.25 .75) contrast(x) cluster(g) seed(7)").unwrap()
  else {
    panic!("clustered coefficient contrast")
  };
  assert_eq!(command.contrast.as_deref(), Some("x"));
  assert_eq!(
    command.bootstrap.regression.cluster_variable.as_deref(),
    Some("g")
  );
}

#[test]
fn qregplot_parses_selected_slope_and_pointwise_band_workflow() {
  let Command::QregPlot { command } = parse_command(
    "qregplot lnmed lntotal age, coefficient(lntotal) quantiles(.05 .1 .5 .9 .95) seed(10101) reps(200) saving(\"qreg bands.svg\") noopen title(\"Slope as quantile varies\")",
  )
  .unwrap()
  else {
    panic!("qregplot")
  };
  assert_eq!(command.bootstrap.regression.outcome, "lnmed");
  assert_eq!(command.bootstrap.regression.predictors, ["lntotal", "age"]);
  assert_eq!(command.coefficient, "lntotal");
  assert_eq!(command.quantiles, [".05", ".1", ".5", ".9", ".95"]);
  assert_eq!(command.bootstrap.seed, 10101);
  assert_eq!(command.bootstrap.replications, 200);
  assert_eq!(command.saving.as_deref(), Some("qreg bands.svg"));
  assert!(!command.open_artifact);
  assert_eq!(command.title.as_deref(), Some("Slope as quantile varies"));

  for text in [
    "qregplot y x, coefficient(x) quantiles(.5 .9) saving(a.svg) noopen",
    "qregplot y x, coefficient(x) quantiles(.5 .9) seed(7) noopen",
    "qregplot y x, coefficient(x) quantiles(.5 .9) seed(7) saving(a.svg)",
    "qregplot y x, coefficient(z) quantiles(.5 .9) seed(7)",
    "qregplot y x, coefficient(x) quantiles(.5 .9) seed(7) seed(8)",
    "qregplot y x, coefficient(x) quantiles(.5 .9) seed(7) reps(1)",
    "qregplot y x, coefficient(x) quantiles(.5 .9) seed(7) cluster(id)",
    "qregplot y x, coefficient(x) quantiles(.5) seed(7)",
    "qregplot y x, coefficient(x) quantiles(.9 .5) seed(7)",
    "qregplot y x, coefficient(x) quantiles(.5 .95) seed(7) noopen(true)",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}
