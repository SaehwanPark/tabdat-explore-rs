use tabdat_language::{Command, EstatSubcommand, parse_command};

#[test]
fn countprob_defaults_and_explicit_bounded_cutoffs_have_typed_semantics() {
  for (input, expected) in [
    ("estat countprob", 10),
    ("estat countprob, max(1)", 1),
    ("ESTAT COUNTPROB, max(100)", 100),
  ] {
    let Command::Estat { command } = parse_command(input).unwrap() else {
      panic!("estat")
    };
    assert_eq!(
      command.subcommand,
      EstatSubcommand::CountProbabilities {
        tail_from: expected
      }
    );
  }
}

#[test]
fn countprob_rejects_invalid_cutoffs_duplicates_wrong_options_and_extra_syntax() {
  for input in [
    "estat countprob, max(0)",
    "estat countprob, max(101)",
    "estat countprob, max(2.5)",
    "estat countprob, max(-1)",
    "estat countprob, max(Inf)",
    "estat countprob, max(NaN)",
    "estat countprob, max(1e100)",
    "estat countprob, max(999999999999999999999999999999999999)",
    "estat countprob, max()",
    "estat countprob, max",
    "estat countprob, max(1) max(2)",
    "estat countprob, max(2) robust",
    "estat countprob, level(95)",
    "estat countprob if y>0",
    "estat countprob extra",
    "estat `countprob`",
  ] {
    assert!(parse_command(input).is_err(), "{input}");
  }
}
