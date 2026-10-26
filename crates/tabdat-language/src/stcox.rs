use super::*;

pub(super) fn parse(body: &str) -> Result<Command, ParseError> {
  let (arguments, options) = match first_unquoted_comma(body) {
    Some(i) => (&body[..i], parse_use_options(&body[i + 1..])?),
    None => (body, Vec::new()),
  };
  let parts = parse_simple_body(arguments, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.is_empty()
  {
    return Err(ParseError::new(
      "stcox expects syntax: stcox <time_var> [xvars], failure(<event>) [robust]",
    ));
  }
  for option in &options {
    if !matches!(option.name.as_str(), "failure" | "robust") {
      return Err(ParseError::new(format!(
        "stcox unsupported option: {}",
        option.name
      )));
    }
    if option.name == "robust" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "stcox option robust does not accept a value",
      ));
    }
  }
  let failures: Vec<_> = options.iter().filter(|o| o.name == "failure").collect();
  if failures.len() != 1 {
    return Err(ParseError::new(
      "stcox requires exactly one failure(variable) option",
    ));
  }
  let UseOptionValue::Identifiers(names) = &failures[0].value else {
    return Err(ParseError::new("stcox option failure expects one variable"));
  };
  if names.len() != 1 {
    return Err(ParseError::new("stcox option failure expects one variable"));
  }
  if options.iter().filter(|o| o.name == "robust").count() > 1 {
    return Err(ParseError::new(
      "stcox option robust may only be supplied once",
    ));
  }
  Ok(Command::Stcox {
    command: StcoxCommand {
      time_variable: parts.arguments[0].text.clone(),
      predictors: parts.arguments[1..]
        .iter()
        .map(|a| a.text.clone())
        .collect(),
      failure_variable: names[0].clone(),
      robust: options.iter().any(|o| o.name == "robust"),
    },
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn parses_cox_breslow_baseline_and_explicit_design() {
    for text in [
      "stcox t, failure(d)",
      "STCOX `t` `x one` z, robust failure(`d`)",
    ] {
      let Command::Stcox { command } = parse_command(text).unwrap() else {
        panic!("stcox")
      };
      assert_eq!(command.time_variable, "t");
      assert_eq!(command.failure_variable, "d");
      if command.robust {
        assert_eq!(command.predictors, ["x one", "z"]);
      } else {
        assert!(command.predictors.is_empty());
      }
    }
  }
  #[test]
  fn rejects_unsupported_or_ambiguous_cox_requests() {
    for text in [
      "stcox",
      "stcox t",
      "stcox t, failure()",
      "stcox t, failure(d e)",
      "stcox t, failure(d) failure(e)",
      "stcox t, failure(d) robust robust",
      "stcox t, failure(d) robust=true",
      "stcox t, failure(d) noconstant",
      "stcox t, failure(d) cluster(id)",
      "stcox t, failure(d) ties(efron)",
      "stcox t x if x>0, failure(d)",
      "stcox t = x, failure(d)",
      "stcox t, failure(d) dist(weibull)",
    ] {
      assert!(parse_command(text).is_err(), "{text}");
    }
  }
}
