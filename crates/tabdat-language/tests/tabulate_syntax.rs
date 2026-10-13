use tabdat_language::{CollapseStatistic, Command, parse_command};

#[test]
fn extended_fields_and_flag_repetition_are_retained() {
  let Command::TabulateQuery {
    command,
    condition,
    aggregation,
  } = parse_command("tabulate, rows(region sex) columns(outcome year) values(cost) stat(MEAN)")
    .unwrap()
  else {
    panic!("query expected")
  };
  assert_eq!(command.row_variables, ["region", "sex"]);
  assert_eq!(command.column_variables, ["outcome", "year"]);
  assert!(condition.is_none());
  let aggregation = aggregation.unwrap();
  assert_eq!(aggregation.variable, "cost");
  assert_eq!(aggregation.statistic, CollapseStatistic::Mean);
  assert_eq!(
    parse_command("tabulate sex age, row row").unwrap(),
    parse_command("tabulate sex age, row").unwrap()
  );
}

#[test]
fn richer_tabulate_forms_parse_without_a_backend() {
  for input in [
    "tabulate sex if age >= 18",
    "tabulate, rows(region sex) columns(outcome year) values(cost) stat(mean)",
  ] {
    assert!(parse_command(input).is_ok(), "{input}");
  }
}
