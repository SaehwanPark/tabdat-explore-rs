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
    "bootstrap y x, seed(1) cluster(id)",
    "bootstrap y x if x>0, seed(1)",
    "bootstrap y x, seed(1) noconstant(true)",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}
