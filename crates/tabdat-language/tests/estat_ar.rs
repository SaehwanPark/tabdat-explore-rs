use tabdat_language::{Command, EstatCommand, EstatSubcommand, parse_command};

#[test]
fn ar_preserves_finite_null_values_in_order() {
  for (input, values) in [
    ("estat ar, null(1.7 -0.6)", vec!["1.7", "-0.6"]),
    ("estat AR, null(0)", vec!["0"]),
    ("estat ar, null(1e-3)", vec!["1e-3"]),
  ] {
    assert_eq!(
      parse_command(input).unwrap(),
      Command::Estat {
        command: EstatCommand {
          subcommand: EstatSubcommand::AndersonRubin {
            null_values: values.into_iter().map(String::from).collect(),
          }
        },
      }
    );
  }
}

#[test]
fn scalar_confidence_sets_require_explicit_ci_and_a_valid_optional_level() {
  for (input, level) in [
    ("estat ar, ci", "95"),
    ("estat ar, ci level(90)", "90"),
    ("estat ar, level(99) ci", "99"),
  ] {
    assert_eq!(
      parse_command(input).unwrap(),
      Command::Estat {
        command: EstatCommand {
          subcommand: EstatSubcommand::AndersonRubinConfidenceSet {
            level: level.into()
          }
        },
      }
    );
  }
  for input in [
    "estat ar, level(95)",
    "estat ar, ci ci",
    "estat ar, ci(true)",
    "estat ar, ci null(0)",
    "estat ar, ci robust",
    "estat ar, ci level(100)",
    "estat ar, ci level(0)",
    "estat ar, ci level(NaN)",
    "estat ar, ci level(1e999)",
    "estat ar, ci level(90) level(95)",
    "estat ar, ci level(95) if x > 0",
  ] {
    assert!(parse_command(input).is_err(), "accepted {input}");
  }
}

#[test]
fn ar_requires_one_numeric_null_and_rejects_unimplemented_options() {
  for input in [
    "estat ar",
    "estat ar, null",
    "estat ar, null()",
    "estat ar, null(x)",
    "estat ar, null(NaN)",
    "estat ar, null(1e999)",
    "estat ar, null(0) null(1)",
    "estat ar, null(0) robust",
    "estat ar, null(0) level(95)",
    "estat ar, null(0) if x > 0",
    "estat ar x, null(0)",
    "estat ar, null(\"0\")",
  ] {
    assert!(parse_command(input).is_err(), "accepted {input}");
  }
}
