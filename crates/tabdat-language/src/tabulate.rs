//! Validation of extended tabulate syntax, independent of frequency execution.
use super::*;

pub(super) fn parse(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.has_assignment {
    return Err(ParseError::new(
      "tabulate does not accept assignment syntax",
    ));
  }
  let mut options = first_top_level_comma(body)
    .map(|index| parse_use_options(&body[index + 1..]))
    .transpose()?
    .unwrap_or_default();
  for option in &mut options {
    option.name.make_ascii_lowercase();
  }
  let supported = [
    "row", "col", "missing", "nolabel", "rows", "columns", "values", "stat",
  ];
  let mut unsupported = options
    .iter()
    .map(|option| option.name.as_str())
    .filter(|name| !supported.contains(name))
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "tabulate unsupported option: {}",
      unsupported.join(", ")
    )));
  }
  for option in &options {
    if ["row", "col", "missing", "nolabel"].contains(&option.name.as_str())
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "tabulate option {} does not accept a value",
        option.name
      )));
    }
  }
  let one = |name: &str| -> Result<Option<&UseOptionValue>, ParseError> {
    let matches = options
      .iter()
      .filter(|option| option.name == name)
      .collect::<Vec<_>>();
    if matches.len() > 1 {
      return Err(ParseError::new(format!(
        "tabulate option {name} can only be specified once"
      )));
    }
    Ok(matches.first().map(|option| &option.value))
  };
  let identifiers = |name: &str| -> Result<Option<Vec<String>>, ParseError> {
    match one(name)? {
      None => Ok(None),
      Some(UseOptionValue::Identifiers(values)) if !values.is_empty() => Ok(Some(values.clone())),
      Some(UseOptionValue::Identifiers(_)) => Err(ParseError::new(format!(
        "tabulate option {name} expects at least one variable"
      ))),
      _ => Err(ParseError::new(format!(
        "tabulate option {name} expects identifiers in parentheses"
      ))),
    }
  };
  let rows = identifiers("rows")?;
  let columns = identifiers("columns")?;
  let values = identifiers("values")?;
  let value_variable = match values {
    None => None,
    Some(values) if values.len() == 1 => Some(values[0].clone()),
    _ => {
      return Err(ParseError::new(
        "tabulate option values expects exactly one variable",
      ));
    }
  };
  let statistic = match one("stat")? {
    None => None,
    Some(UseOptionValue::Identifiers(values)) if values.len() == 1 => {
      Some(match values[0].to_ascii_lowercase().as_str() {
        "count" => CollapseStatistic::Count,
        "mean" => CollapseStatistic::Mean,
        "sum" => CollapseStatistic::Sum,
        "min" => CollapseStatistic::Min,
        "max" => CollapseStatistic::Max,
        other => {
          return Err(ParseError::new(format!(
            "tabulate unsupported stat: {other}"
          )));
        }
      })
    }
    _ => {
      return Err(ParseError::new(
        "tabulate option stat expects exactly one statistic",
      ));
    }
  };
  let explicit = rows.is_some() || columns.is_some();
  let (row_variables, column_variables) = if explicit {
    if !parts.arguments.is_empty() {
      return Err(ParseError::new(
        "tabulate cannot combine positional variables with rows() or columns()",
      ));
    }
    (
      rows.ok_or_else(|| ParseError::new("tabulate rows() expects at least one variable"))?,
      columns.unwrap_or_default(),
    )
  } else {
    if !matches!(parts.arguments.len(), 1 | 2) {
      return Err(ParseError::new(
        "tabulate expects one or two variables or rows()/columns() options",
      ));
    }
    (
      vec![parts.arguments[0].text.clone()],
      parts
        .arguments
        .get(1)
        .map(|arg| arg.text.clone())
        .into_iter()
        .collect(),
    )
  };
  if value_variable.is_some() != statistic.is_some() {
    return Err(ParseError::new(
      "tabulate values() and stat() must be specified together",
    ));
  }
  let has = |name: &str| options.iter().any(|option| option.name == name);
  if value_variable.is_some() && (has("row") || has("col")) {
    return Err(ParseError::new(
      "tabulate row and col percentages are only supported for frequencies",
    ));
  }
  if column_variables.is_empty() && (has("row") || has("col")) {
    return Err(ParseError::new(
      "one-way tabulate does not accept row or col options",
    ));
  }
  let dimensions = row_variables
    .iter()
    .chain(&column_variables)
    .collect::<Vec<_>>();
  for (index, name) in dimensions.iter().enumerate() {
    if dimensions[..index].contains(name) {
      return Err(ParseError::new(format!(
        "tabulate duplicate variable: {name}"
      )));
    }
  }
  if value_variable
    .as_ref()
    .is_some_and(|value| dimensions.contains(&value))
  {
    return Err(ParseError::new(
      "tabulate values() variable cannot also be a row or column variable",
    ));
  }
  let condition = if parts.has_condition {
    Some(parse_row_condition(body)?)
  } else {
    None
  };
  let aggregation =
    value_variable
      .zip(statistic)
      .map(|(variable, statistic)| TabulateAggregation {
        variable,
        statistic,
      });
  if !explicit && condition.is_none() && aggregation.is_none() {
    return parse_frequency_tabulate(body);
  }
  Ok(Command::TabulateQuery {
    command: TabulateCommand {
      row_variables,
      column_variables,
      row_percent: has("row"),
      column_percent: has("col"),
      include_missing: has("missing"),
      nolabel: has("nolabel"),
    },
    condition,
    aggregation,
  })
}
