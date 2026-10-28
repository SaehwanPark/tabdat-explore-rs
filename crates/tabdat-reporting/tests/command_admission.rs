use tabdat_language::{CollapseStatistic, Command, TabulateAggregation, parse_command};
use tabdat_reporting::json_results::{supports_batch_command, supports_script_command};

#[test]
fn batch_and_script_share_bounded_command_admission() {
  for (source, expected) in [
    ("status", true),
    ("exit", true),
    ("run child.td", true),
    ("count if x > 0", true),
    ("regress y x if x > 0", true),
    ("ivregress 2sls y x, endog(z) iv(w)", true),
    ("ivregress gmm y x, endog(z) iv(w)", false),
    (
      "ivregress gmm y, endog(x) iv(z w) noconstant cluster(id)",
      true,
    ),
    ("ivregress gmm y, endog(x) iv(z w) noconstant", false),
    (
      "ivregress gmm y x, endog(q) iv(z w) noconstant cluster(id)",
      false,
    ),
    ("summarize x if x > 0", true),
    ("summarize x if x > 0, detail", true),
    ("summarize x, detail detail", false),
    ("by g: count", true),
    ("by g: summarize x", true),
    ("by g: summarize x, detail", false),
    ("collapse count x, by(g)", true),
    ("collapse mean x, by(g)", true),
    ("collapse sum x, by(g)", false),
    ("tabulate x if x > 0", true),
    ("tabulate x g if x > 0", true),
    ("estat vif", true),
    ("estat residuals", false),
    ("help", false),
    ("logit y x", true),
  ] {
    let command = parse_command(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    assert_eq!(
      supports_batch_command(&command),
      expected,
      "batch: {source}"
    );
    assert_eq!(
      supports_script_command(&command),
      expected,
      "script: {source}"
    );
  }
}

#[test]
fn conditional_frequency_admission_checks_raw_command_dimensions_and_aggregation() {
  let Command::TabulateQuery {
    command, condition, ..
  } = parse_command("tabulate x if x > 0").unwrap()
  else {
    panic!("expected conditional frequency command");
  };
  // Typed callers can construct shapes that the command-line parser rejects.
  for (rows, columns, aggregation, expected) in [
    (vec!["x"], vec![], None, true),
    (vec!["x"], vec!["g"], None, true),
    (vec![], vec![], None, false),
    (vec!["x", "g"], vec![], None, false),
    (vec!["x"], vec!["g", "h"], None, false),
    (
      vec!["x"],
      vec![],
      Some(TabulateAggregation {
        variable: "y".into(),
        statistic: CollapseStatistic::Mean,
      }),
      false,
    ),
  ] {
    let mut command = command.clone();
    command.row_variables = rows.into_iter().map(str::to_owned).collect();
    command.column_variables = columns.into_iter().map(str::to_owned).collect();
    let query = Command::TabulateQuery {
      command,
      condition: condition.clone(),
      aggregation,
    };
    assert_eq!(supports_batch_command(&query), expected, "{query:?}");
    assert_eq!(supports_script_command(&query), expected, "{query:?}");
  }
}
