use tabdat_language::parse_command;

#[test]
fn richer_tabulate_forms_parse_without_a_backend() {
  for input in ["tabulate sex if age >= 18",
    "tabulate, rows(region sex) columns(outcome year) values(cost) stat(mean)"] {
    assert!(parse_command(input).is_ok(), "{input}");
  }
}
