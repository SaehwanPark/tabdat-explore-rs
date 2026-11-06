use tabdat_domain::Command;
use tabdat_language::parse_command;

#[test]
fn gamma_poisson_re_requires_seed_cluster_and_joint_intercept() {
  let Command::GammaPoissonBootstrap { command } = parse_command(
    "bootstrap y x z, model(gamma_poisson_re) cluster(group) seed(18446744073709551615) reps(500)",
  )
  .unwrap() else {
    panic!("wrong command")
  };
  assert_eq!(command.predictors, ["x", "z"]);
  assert_eq!(command.cluster_variable, "group");
  assert_eq!(command.replications, 500);
  assert_eq!(command.seed, u64::MAX);
  for input in [
    "bootstrap y x, model(gamma_poisson_re) seed(1)",
    "bootstrap y x, model(gamma_poisson_re) cluster(g)",
    "bootstrap y x, model(gamma_poisson_re) cluster(g) seed(1) noconstant",
    "bootstrap y x, model(gamma_poisson_re) cluster(g) seed(1) quantiles(.5)",
    "bootstrap y x, model(gamma_poisson_re) cluster(g) seed(1) contrast(x)",
    "bootstrap y x, model(gamma_poisson_re) cluster(g) seed(1) reps(501)",
    "bootstrap y x, model(gamma_poisson_re gamma_poisson_re) cluster(g) seed(1)",
  ] {
    assert!(parse_command(input).is_err(), "{input}");
  }
}
