use tabdat_domain::Command;
use tabdat_language::parse_command;

#[test]
fn explicit_poisson_model_requires_cluster_seed_and_intercept() {
  let Command::PoissonBootstrap { command } = parse_command(
    "bootstrap y x z, model(poisson) cluster(group) seed(18446744073709551615) reps(500)",
  )
  .unwrap() else {
    panic!("wrong command")
  };
  assert_eq!(command.outcome, "y");
  assert_eq!(command.predictors, ["x", "z"]);
  assert_eq!(command.cluster_variable, "group");
  assert_eq!(command.replications, 500);
  assert_eq!(command.seed, u64::MAX);
  assert!(matches!(
    parse_command("bootstrap y x, seed(1) cluster(g)").unwrap(),
    Command::Bootstrap { .. }
  ));
  for input in [
    "bootstrap y x, model(poisson) seed(1)",
    "bootstrap y x, model(poisson) cluster(g)",
    "bootstrap y x, model(poisson) cluster(g) seed(1) noconstant",
    "bootstrap y x, model(poisson) cluster(g) seed(1) quantiles(.5)",
    "bootstrap y x, model(poisson) cluster(g) seed(1) contrast(x)",
    "bootstrap y x, model(poisson) cluster(g) seed(1) reps(501)",
    "bootstrap y x, model(nb2) cluster(g) seed(1)",
    "bootstrap y x, model(poisson poisson) cluster(g) seed(1)",
    "bootstrap y x, model(poisson) model(poisson) cluster(g) seed(1)",
    "bootstrap y x, model(poisson) cluster(g g2) seed(1)",
  ] {
    assert!(parse_command(input).is_err(), "{input}");
  }
}
