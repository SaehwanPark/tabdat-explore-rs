use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_spregress_remains_explicitly_deferred_at_runtime() {
  let command =
    parse_command("spregress y x1 x2, coord(lat lon)").expect("spregress parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "spregress" }
  );
}
