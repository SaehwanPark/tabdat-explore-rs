use tabdat_language::{Command, parse_command};

#[test]
fn report_and_spatial_options_are_syntax_only() {
  for input in ["estat report", "estat report, saving(my_report.html)",
    "estat report, noopen", "estat spatial, coord(lat lon) knn(5)",
    "estat spatial, weights(columbus.gal) id(neighborhood) contiguity(rook)"] {
    assert!(matches!(parse_command(input).unwrap(), Command::Estat { .. }));
  }
}
