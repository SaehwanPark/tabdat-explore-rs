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
  for text in [
    "bootstrap y x, quantiles(.5) cluster(id) seed(7)",
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
