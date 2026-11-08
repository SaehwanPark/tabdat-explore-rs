use tabdat_domain::Command;
use tabdat_language::parse_command;
#[test]
fn normal_bootstrap_explicit_selector_and_failure_contracts() {
  let Command::NormalPoissonBootstrap { command } = parse_command(
    "bootstrap y x z, model(NORMAL_POISSON_RE) cluster(group) seed(18446744073709551615) reps(500)",
  )
  .unwrap() else {
    panic!("wrong model")
  };
  assert_eq!(command.outcome, "y");
  assert_eq!(command.predictors, ["x", "z"]);
  assert_eq!(command.cluster_variable, "group");
  assert_eq!(command.seed, u64::MAX);
  assert_eq!(command.replications, 500);
  let Command::NormalPoissonBootstrap { command } =
    parse_command("bootstrap y x, model(normal_poisson_re) cluster(g) seed(0)").unwrap()
  else {
    panic!()
  };
  assert_eq!(command.replications, 200);
  for input in [
    "bootstrap y x, model(normal_poisson_re) seed(1)",
    "bootstrap y x, model(normal_poisson_re) cluster(g)",
    "bootstrap y x, model(normal_poisson_re) cluster(g h) seed(1)",
    "bootstrap y x, model(normal_poisson_re) cluster(g) seed(-1)",
    "bootstrap y x, model(normal_poisson_re) cluster(g) seed(1) reps(1)",
    "bootstrap y x, model(normal_poisson_re) cluster(g) seed(1) reps(501)",
    "bootstrap y x, model(normal_poisson_re) cluster(g) seed(1) noconstant",
    "bootstrap y x, model(normal_poisson_re) cluster(g) seed(1) quantiles(.5)",
    "bootstrap y x, model(normal_poisson_re) cluster(g) seed(1) contrast(x)",
    "bootstrap y x, model(normal_poisson_re) cluster(g) seed(1) robust",
    "bootstrap y x, model(normal_poisson_re) cluster(g) seed(1) distribution(normal)",
    "bootstrap y x, model(normal_poisson_re normal_poisson_re) cluster(g) seed(1)",
  ] {
    assert!(parse_command(input).is_err(), "{input}");
  }
  assert!(matches!(
    parse_command("bootstrap y x, model(gamma_poisson_re) cluster(g) seed(1)").unwrap(),
    Command::GammaPoissonBootstrap { .. }
  ));
  assert!(matches!(
    parse_command("bootstrap y x, model(poisson) cluster(g) seed(1)").unwrap(),
    Command::PoissonBootstrap { .. }
  ));
  assert!(matches!(
    parse_command("bootstrap y x, model(conditional_poisson) cluster(g) seed(1)").unwrap(),
    Command::ConditionalPoissonBootstrap { .. }
  ));
}
