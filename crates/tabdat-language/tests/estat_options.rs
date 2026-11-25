use tabdat_language::{Command, EstatCommand, EstatSubcommand, SpregressContiguity, parse_command};

#[test]
fn option_fields_and_defaults_are_retained() {
  for (input, subcommand) in [
    (
      "estat report, saving(my_report.html) noopen",
      EstatSubcommand::Report {
        saving: Some("my_report.html".into()),
        open_artifact: false,
      },
    ),
    (
      "estat spatial, coord(`latitude x` lon)",
      EstatSubcommand::SpatialCoordinates {
        latitude: "latitude x".into(),
        longitude: "lon".into(),
        knn: 5,
      },
    ),
    (
      "estat spatial, weights(columbus.gal) id(neighborhood)",
      EstatSubcommand::SpatialWeights {
        path: "columbus.gal".into(),
        id_variable: "neighborhood".into(),
        contiguity: SpregressContiguity::Queen,
      },
    ),
  ] {
    assert_eq!(
      parse_command(input).unwrap(),
      Command::Estat {
        command: EstatCommand { subcommand }
      }
    );
  }
}

#[test]
fn report_and_spatial_options_are_syntax_only() {
  for input in [
    "estat report",
    "estat report, saving(my_report.html)",
    "estat report, noopen",
    "estat spatial, coord(lat lon) knn(5)",
    "estat spatial, weights(columbus.gal) id(neighborhood) contiguity(rook)",
  ] {
    assert!(matches!(
      parse_command(input).unwrap(),
      Command::Estat { .. }
    ));
  }
}

#[test]
fn grouped_binary_gof_parses_only_the_bounded_quantile_option() {
  assert_eq!(
    parse_command("estat gof, group(10)").unwrap(),
    Command::Estat {
      command: EstatCommand {
        subcommand: EstatSubcommand::BinaryGoodnessOfFit { group_count: 10 },
      },
    }
  );
  assert_eq!(
    parse_command("estat gof").unwrap(),
    Command::Estat {
      command: EstatCommand {
        subcommand: EstatSubcommand::Gof,
      },
    }
  );
  for input in [
    "estat gof, group(2)",
    "estat gof, group(21)",
    "estat gof, group(10.5)",
    "estat gof, table",
    "estat gof, group(10) table",
  ] {
    assert!(parse_command(input).is_err(), "accepted {input:?}");
  }
}
