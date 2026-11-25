use tabdat_domain::Command;
use tabdat_language::parse_command;

#[test]
fn logit_and_probit_accept_the_bounded_factor_main_effect_syntax() {
  match parse_command("logit y x i.region ib(West).district, robust").unwrap() {
    Command::Logit { command } => {
      assert_eq!(command.predictors, ["x", "i.region", "ib(West).district"]);
      assert!(command.robust);
      assert!(command.include_intercept);
    }
    other => panic!("expected logit, got {other:?}"),
  }

  match parse_command("probit y ib(0).group x, noconstant").unwrap() {
    Command::Probit { command } => {
      assert_eq!(command.predictors, ["ib(0).group", "x"]);
      assert!(!command.include_intercept);
    }
    other => panic!("expected probit, got {other:?}"),
  }
}

#[test]
fn count_models_and_factor_interactions_remain_outside_the_parser_slice() {
  assert!(parse_command("poisson y i.region").is_err());
  assert!(parse_command("nbreg y i.region").is_err());
  assert!(parse_command("logit y x i.region#i.year").is_err());
}
