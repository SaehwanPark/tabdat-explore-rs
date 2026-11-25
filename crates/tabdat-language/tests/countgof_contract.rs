use tabdat_domain::{Command, EstatSubcommand};
use tabdat_language::parse_command;

#[test]
fn countgof_defaults_and_explicit_bounded_cutoffs_have_typed_semantics() {
  for (input, expected) in [
    ("estat countgof", 10),
    ("estat countgof, max(1)", 1),
    ("ESTAT COUNTGOF, max(10)", 10),
  ] {
    let Command::Estat { command } = parse_command(input).unwrap() else {
      panic!("estat")
    };
    assert_eq!(
      command.subcommand,
      EstatSubcommand::CountGoodnessOfFit {
        tail_from: expected
      }
    );
  }
}

#[test]
fn countgof_rejects_invalid_cutoffs_duplicate_options_and_extra_syntax() {
  for input in [
    "estat countgof, max(0)",
    "estat countgof, max(11)",
    "estat countgof, max(2.5)",
    "estat countgof, max(-1)",
    "estat countgof, max(Inf)",
    "estat countgof, max(NaN)",
    "estat countgof, max(1e100)",
    "estat countgof, max()",
    "estat countgof, max",
    "estat countgof, max(1) max(2)",
    "estat countgof, robust",
    "estat countgof, level(95)",
    "estat countgof if y>0",
    "estat countgof extra",
    "estat `countgof`",
  ] {
    assert!(parse_command(input).is_err(), "{input}");
  }
}
