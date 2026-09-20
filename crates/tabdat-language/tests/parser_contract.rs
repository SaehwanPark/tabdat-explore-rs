use tabdat_language::{
  AssertBinaryOperator, AssertExpression, CollapseCommand, CollapseStatistic, Command, DataSource,
  ExecutionMode, GenerateBinaryOperator, GenerateExpression, JoinCommand, JoinHow, LabelCommand,
  LabelValue, LazyEngine, RecodeInput, RecodeRangeEndpoint, RecodeRule, RecodeTarget, RecodeValue,
  ReshapeCommand, ReshapeDirection, RowLimit, SettingName, SortKey, TabulateCommand, parse_command,
};

#[test]
fn public_parser_returns_owned_typed_commands() {
  let command = parse_command("help summarize").expect("valid help command");

  assert_eq!(
    command,
    Command::Help {
      topic: Some("summarize".to_owned()),
    }
  );
}

#[test]
fn parse_error_exposes_a_stable_message() {
  let error = parse_command("status now").expect_err("status has no arguments");

  assert_eq!(
    error.message(),
    "status does not accept arguments, if clauses, options, or assignment syntax"
  );
  assert_eq!(error.to_string(), error.message());
}

#[test]
fn tabulate_parses_bounded_frequency_forms() {
  assert_eq!(
    parse_command("TABULATE `sex value`, MISSING NOLABEL").unwrap(),
    Command::Tabulate {
      command: TabulateCommand {
        row_variables: vec!["sex value".to_owned()],
        column_variables: vec![],
        row_percent: false,
        column_percent: false,
        include_missing: true,
        nolabel: true,
      },
    }
  );
  assert_eq!(
    parse_command("tabulate sex age, row col missing").unwrap(),
    Command::Tabulate {
      command: TabulateCommand {
        row_variables: vec!["sex".to_owned()],
        column_variables: vec!["age".to_owned()],
        row_percent: true,
        column_percent: true,
        include_missing: true,
        nolabel: false,
      },
    }
  );
}

#[test]
fn append_parses_bounded_named_table_forms() {
  assert_eq!(
    parse_command("append followup").unwrap(),
    Command::Append {
      table_name: "followup".to_owned(),
    }
  );
  assert_eq!(
    parse_command("APPEND `followup`").unwrap(),
    Command::Append {
      table_name: "followup".to_owned(),
    }
  );
  assert_eq!(
    parse_command("append \"followup\"").unwrap(),
    Command::Append {
      table_name: "followup".to_owned(),
    }
  );
}

#[test]
fn append_preserves_bounded_parser_diagnostics() {
  let cases = [
    ("append", "append expects syntax: append <table>"),
    (
      "append followup extra",
      "append expects syntax: append <table>",
    ),
    (
      "append followup, replace",
      "append expects syntax: append <table>",
    ),
    (
      "append followup if age > 18",
      "append expects syntax: append <table>",
    ),
    (
      "append active",
      "sql into cannot use reserved table name: active",
    ),
    ("append 123", "sql into table name must be an identifier"),
  ];

  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn reshape_parses_bounded_layout_forms() {
  assert_eq!(
    parse_command("reshape long income cost, i(id) j(year)").unwrap(),
    Command::Reshape {
      command: ReshapeCommand {
        direction: ReshapeDirection::Long,
        variables: vec!["income".to_owned(), "cost".to_owned()],
        identifiers: vec!["id".to_owned()],
        j_variable: "year".to_owned(),
      },
    }
  );
  assert_eq!(
    parse_command("reshape WIDE income cost, i(firm_id `person id`) j(year)").unwrap(),
    Command::Reshape {
      command: ReshapeCommand {
        direction: ReshapeDirection::Wide,
        variables: vec!["income".to_owned(), "cost".to_owned()],
        identifiers: vec!["firm_id".to_owned(), "person id".to_owned()],
        j_variable: "year".to_owned(),
      },
    }
  );
  assert_eq!(
    parse_command("reshape \"long\" \"income value\", i(id) j(year)").unwrap(),
    Command::Reshape {
      command: ReshapeCommand {
        direction: ReshapeDirection::Long,
        variables: vec!["income value".to_owned()],
        identifiers: vec!["id".to_owned()],
        j_variable: "year".to_owned(),
      },
    }
  );
}

#[test]
fn reshape_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "reshape",
      "reshape expects syntax: reshape long|wide varlist, i(id_vars) j(name)",
    ),
    (
      "reshape wider income, i(id) j(year)",
      "reshape direction must be long or wide",
    ),
    (
      "reshape long, i(id) j(year)",
      "reshape expects syntax: reshape long|wide varlist, i(id_vars) j(name)",
    ),
    (
      "reshape `long` income, i(id) j(year)",
      "reshape direction must be long or wide",
    ),
    (
      "reshape long income if age > 18, i(id) j(year)",
      "reshape expects syntax: reshape long|wide varlist, i(id_vars) j(name)",
    ),
    (
      "reshape long income = cost, i(id) j(year)",
      "reshape expects syntax: reshape long|wide varlist, i(id_vars) j(name)",
    ),
    (
      "reshape long income, i(id)",
      "reshape expects exactly one j(name) option",
    ),
    (
      "reshape long income, j(year)",
      "reshape expects exactly one i(id_vars) option",
    ),
    (
      "reshape long income, i() j(year)",
      "reshape expects exactly one i(id_vars) option",
    ),
    (
      "reshape long income, i(id) j()",
      "reshape expects exactly one j(name) option",
    ),
    (
      "reshape long income, i(id) j(year month)",
      "reshape expects exactly one j(name) option",
    ),
    (
      "reshape long income income, i(id) j(year)",
      "reshape variable list contains duplicates",
    ),
    (
      "reshape long income, i(id id) j(year)",
      "reshape variable list contains duplicates",
    ),
    (
      "reshape long income, i(id) i(group) j(year)",
      "reshape option i may only be supplied once",
    ),
    (
      "reshape long income, i(id) j(income)",
      "reshape variables, i(), and j() names must be distinct",
    ),
    (
      "reshape long income, i(income) j(year)",
      "reshape variables, i(), and j() names must be distinct",
    ),
    (
      "reshape long income, i(id) j(year) replace",
      "reshape unsupported option: replace",
    ),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn tabulate_preserves_bounded_parser_diagnostics() {
  let cases = [
    ("tabulate", "tabulate expects one or two variables"),
    (
      "tabulate age bmi sex",
      "tabulate expects one or two variables",
    ),
    ("tabulate sex sex", "tabulate duplicate variable: sex"),
    (
      "tabulate sex, row",
      "tabulate one-way tables do not accept row or col options",
    ),
    (
      "tabulate sex age, row row",
      "tabulate option row can only be specified once",
    ),
    (
      "tabulate sex, values(cost)",
      "tabulate unsupported option: values",
    ),
    (
      "tabulate sex age, row=true",
      "tabulate option row does not accept a value",
    ),
    (
      "tabulate sex = age",
      "tabulate expects one or two variables",
    ),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn collapse_parses_grouped_aggregate_forms() {
  assert_eq!(
    parse_command("COLLAPSE MEAN age cost, BY(sex region)").unwrap(),
    Command::Collapse {
      command: CollapseCommand {
        statistic: CollapseStatistic::Mean,
        variables: vec!["age".to_owned(), "cost".to_owned()],
        groups: vec!["sex".to_owned(), "region".to_owned()],
      },
    }
  );
  assert_eq!(
    parse_command("collapse count `value col`, by(`group key`)").unwrap(),
    Command::Collapse {
      command: CollapseCommand {
        statistic: CollapseStatistic::Count,
        variables: vec!["value col".to_owned()],
        groups: vec!["group key".to_owned()],
      },
    }
  );
}

#[test]
fn collapse_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "collapse mean age",
      "collapse expects exactly one by(group_vars) option",
    ),
    (
      "collapse median age, by(sex)",
      "collapse unsupported statistic: median",
    ),
    (
      "collapse mean age, by(sex) by(region)",
      "collapse expects exactly one by(group_vars) option",
    ),
    (
      "collapse mean age, by()",
      "collapse by() expects at least one grouping variable",
    ),
    (
      "collapse mean age if age > 0, by(sex)",
      "collapse does not accept if clauses or assignment syntax",
    ),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn join_parses_bounded_named_table_forms() {
  assert_eq!(
    parse_command("JOIN lookup ON firm_id year").unwrap(),
    Command::Join {
      command: JoinCommand {
        table_name: "lookup".to_owned(),
        keys: vec!["firm_id".to_owned(), "year".to_owned()],
        how: JoinHow::Inner,
        suffix: "_right".to_owned(),
      },
    }
  );
  assert_eq!(
    parse_command("join `lookup` on `firm id` year, how=left suffix(_lookup)").unwrap(),
    Command::Join {
      command: JoinCommand {
        table_name: "lookup".to_owned(),
        keys: vec!["firm id".to_owned(), "year".to_owned()],
        how: JoinHow::Left,
        suffix: "_lookup".to_owned(),
      },
    }
  );
}

#[test]
fn join_preserves_bounded_parser_diagnostics() {
  let cases = [
    ("join", "join expects syntax: join <table> on <keylist>"),
    (
      "join lookup id",
      "join expects syntax: join <table> on <keylist>",
    ),
    ("join lookup on id id", "join key list contains duplicates"),
    (
      "join lookup `on` id",
      "join expects syntax: join <table> on <keylist>",
    ),
    (
      "join lookup on id, how=right",
      "join how must be inner or left",
    ),
    (
      "join lookup on id, suffix(_x) suffix(_y)",
      "join option suffix may only be supplied once",
    ),
    (
      "join lookup on id, replace",
      "join unsupported option: replace",
    ),
    (
      "join active on id",
      "sql into cannot use reserved table name: active",
    ),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn status_preserves_pinned_sign_and_empty_condition_diagnostics() {
  let cases = [
    ("status -1", "unsupported token in command: -"),
    ("status +1", "unsupported token in command: +"),
    ("status-1", "unsupported token in command: -"),
    ("status+1", "unsupported token in command: +"),
    ("status -", "unsupported token in command: -"),
    ("status +", "unsupported token in command: +"),
    ("status --1", "unsupported token in command: -"),
    ("status ++1", "unsupported token in command: +"),
    ("status -1,", "unsupported token in command: -"),
    ("status +1,", "unsupported token in command: +"),
    ("status if", "missing expression after if"),
    (
      "status if x",
      "status does not accept arguments, if clauses, options, or assignment syntax",
    ),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn inspection_commands_are_public_syntax_only_values() {
  assert_eq!(parse_command("count").unwrap(), Command::Count);
  assert_eq!(
    parse_command("head").unwrap(),
    Command::Head {
      limit: RowLimit::default(),
    }
  );
  let command = parse_command("tail 00042").expect("valid tail limit");
  match command {
    Command::Tail { limit } => assert_eq!(limit.as_decimal(), "42"),
    other => panic!("unexpected command: {other:?}"),
  }
}

#[test]
fn describe_is_a_public_syntax_only_command() {
  assert_eq!(parse_command(" DESCRIBE ").unwrap(), Command::Describe);
  assert_eq!(
    parse_command("describe age").unwrap_err().message(),
    "describe does not accept arguments, if clauses, or options"
  );
}

#[test]
fn doctor_is_a_public_syntax_only_command() {
  assert_eq!(parse_command(" DOCTOR ").unwrap(), Command::Doctor);
  assert_eq!(
    parse_command("doctor if age > 18").unwrap_err().message(),
    "doctor does not accept arguments, if clauses, options, or assignment syntax"
  );
}

#[test]
fn summarize_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command(" SUMMARIZE `bmi-zscore` \"value col\" ").unwrap(),
    Command::Summarize {
      variables: vec!["bmi-zscore".to_owned(), "value col".to_owned()],
    }
  );
  assert_eq!(
    parse_command("summarize").unwrap(),
    Command::Summarize { variables: vec![] }
  );
  assert_eq!(
    parse_command("summarize `x``y` \"report\"").unwrap(),
    Command::Summarize {
      variables: vec!["x`y".to_owned(), "report".to_owned()],
    }
  );
}

#[test]
fn summarize_preserves_exact_public_diagnostics() {
  let cases = [
    (
      "summarize age if age > 18",
      "summarize does not accept if clauses or options",
    ),
    (
      "summarize age, detail",
      "summarize does not accept if clauses or options",
    ),
    (
      "summarize age = other",
      "summarize does not accept assignment syntax",
    ),
    (
      "summarize = age",
      "summarize assignment requires a target before =",
    ),
    (
      "summarize age,",
      "comma must be followed by at least one option",
    ),
    (
      "summarize,",
      "comma must be followed by at least one option",
    ),
    ("summarize if", "missing expression after if"),
    ("summarize age if", "missing expression after if"),
    ("summarize age==x", "unsupported token in command: =="),
    ("summarize age-1", "unsupported token in command: -"),
    ("summarize age+1", "unsupported token in command: +"),
    ("summarize age!x", "unsupported token in command: !"),
    ("summarize age@x", "unsupported token in command: @"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn datasignature_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command(" DATASIGNATURE ").unwrap(),
    Command::Datasignature
  );
  assert_eq!(
    parse_command("datasignature if age > 0")
      .unwrap_err()
      .message(),
    "datasignature does not accept arguments, if clauses, options, or assignment syntax"
  );
}

#[test]
fn assert_parses_the_bounded_typed_expression_form() {
  assert_eq!(
    parse_command(" ASSERT `age` >= 18 ").unwrap(),
    Command::Assert {
      expression: AssertExpression::Binary {
        left: Box::new(AssertExpression::Identifier("age".to_owned())),
        operator: AssertBinaryOperator::GreaterOrEqual,
        right: Box::new(AssertExpression::Number("18".to_owned())),
      },
    }
  );
  assert_eq!(
    parse_command("assert (bmi + 1) > 20").unwrap(),
    Command::Assert {
      expression: AssertExpression::Binary {
        left: Box::new(AssertExpression::Binary {
          left: Box::new(AssertExpression::Identifier("bmi".to_owned())),
          operator: AssertBinaryOperator::Add,
          right: Box::new(AssertExpression::Number("1".to_owned())),
        }),
        operator: AssertBinaryOperator::Greater,
        right: Box::new(AssertExpression::Number("20".to_owned())),
      },
    }
  );
  assert_eq!(
    parse_command("assert cost == null").unwrap(),
    Command::Assert {
      expression: AssertExpression::Binary {
        left: Box::new(AssertExpression::Identifier("cost".to_owned())),
        operator: AssertBinaryOperator::Equal,
        right: Box::new(AssertExpression::Null),
      },
    }
  );
  assert_eq!(
    parse_command("assert `null` == null").unwrap(),
    Command::Assert {
      expression: AssertExpression::Binary {
        left: Box::new(AssertExpression::Identifier("null".to_owned())),
        operator: AssertBinaryOperator::Equal,
        right: Box::new(AssertExpression::Null),
      },
    }
  );
}

#[test]
fn assert_preserves_exact_bounded_diagnostics() {
  let cases = [
    ("assert", "assert expects a boolean expression"),
    ("assert age > 0, strict", "assert does not accept options"),
    (
      "assert age > 0 if sex == 'F'",
      "assert does not accept if clauses",
    ),
    ("assert age = 0", "assert does not accept assignment syntax"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn generate_preserves_typed_expression_structure() {
  assert_eq!(
    parse_command("GENERATE `age group` = `body mass` + 1").unwrap(),
    Command::Generate {
      variable: "age group".to_owned(),
      expression: GenerateExpression::Binary {
        left: Box::new(GenerateExpression::Identifier("body mass".to_owned())),
        operator: GenerateBinaryOperator::Add,
        right: Box::new(GenerateExpression::Number("1".to_owned())),
      },
    }
  );
  assert_eq!(
    parse_command("generate total = round(sqrt(age + 1), 2)").unwrap(),
    Command::Generate {
      variable: "total".to_owned(),
      expression: GenerateExpression::FunctionCall {
        name: "round".to_owned(),
        arguments: vec![
          GenerateExpression::FunctionCall {
            name: "sqrt".to_owned(),
            arguments: vec![GenerateExpression::Binary {
              left: Box::new(GenerateExpression::Identifier("age".to_owned())),
              operator: GenerateBinaryOperator::Add,
              right: Box::new(GenerateExpression::Number("1".to_owned())),
            }],
          },
          GenerateExpression::Number("2".to_owned()),
        ],
      },
    }
  );
  assert_eq!(
    parse_command("generate `if` = `a``b` == null").unwrap(),
    Command::Generate {
      variable: "if".to_owned(),
      expression: GenerateExpression::Binary {
        left: Box::new(GenerateExpression::Identifier("a`b".to_owned())),
        operator: GenerateBinaryOperator::Equal,
        right: Box::new(GenerateExpression::Null),
      },
    }
  );
}

#[test]
fn replace_preserves_typed_expression_and_condition_structure() {
  assert_eq!(
    parse_command("REPLACE `cost value` = round(`cost value` * 2, 1) if `sex value` == 'F'")
      .unwrap(),
    Command::Replace {
      variable: "cost value".to_owned(),
      expression: GenerateExpression::FunctionCall {
        name: "round".to_owned(),
        arguments: vec![
          GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("cost value".to_owned())),
            operator: GenerateBinaryOperator::Multiply,
            right: Box::new(GenerateExpression::Number("2".to_owned())),
          },
          GenerateExpression::Number("1".to_owned()),
        ],
      },
      condition: Some(GenerateExpression::Binary {
        left: Box::new(GenerateExpression::Identifier("sex value".to_owned())),
        operator: GenerateBinaryOperator::Equal,
        right: Box::new(GenerateExpression::String("F".to_owned())),
      }),
    }
  );
  assert_eq!(
    parse_command("replace amount = null").unwrap(),
    Command::Replace {
      variable: "amount".to_owned(),
      expression: GenerateExpression::Null,
      condition: None,
    }
  );
}

#[test]
fn replace_preserves_exact_bounded_diagnostics_and_nested_boundaries() {
  let cases = [
    (
      "replace",
      "replace expects syntax: replace existing = expression",
    ),
    (
      "replace cost",
      "replace expects syntax: replace existing = expression",
    ),
    (
      "replace = cost",
      "replace assignment requires a target before =",
    ),
    (
      "replace cost =",
      "replace assignment requires an expression after =",
    ),
    (
      "replace cost = cost, force",
      "replace does not accept options",
    ),
    (
      "replace cost = cost if sex >",
      "incomplete expression after >",
    ),
    (
      "replace cost = cost if sex == 'F' if x",
      "duplicate if clause",
    ),
    ("replace cost == 1", "unsupported token in command: =="),
    ("replace cost + 1", "unsupported token in command: +"),
    ("replace if = 1", "unsupported token in expression: ="),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }

  assert_eq!(
    parse_command("replace value = combine(if, (value + 1)) if (if == 1)").unwrap(),
    Command::Replace {
      variable: "value".to_owned(),
      expression: GenerateExpression::FunctionCall {
        name: "combine".to_owned(),
        arguments: vec![
          GenerateExpression::Identifier("if".to_owned()),
          GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("value".to_owned())),
            operator: GenerateBinaryOperator::Add,
            right: Box::new(GenerateExpression::Number("1".to_owned())),
          },
        ],
      },
      condition: Some(GenerateExpression::Binary {
        left: Box::new(GenerateExpression::Identifier("if".to_owned())),
        operator: GenerateBinaryOperator::Equal,
        right: Box::new(GenerateExpression::Number("1".to_owned())),
      }),
    }
  );
}

#[test]
fn generate_preserves_exact_bounded_diagnostics() {
  let cases = [
    (
      "generate",
      "generate expects syntax: generate new = expression",
    ),
    (
      "generate new",
      "generate expects syntax: generate new = expression",
    ),
    (
      "generate = age",
      "generate assignment requires a target before =",
    ),
    (
      "generate new =",
      "generate assignment requires an expression after =",
    ),
    ("generate new = age +", "incomplete expression after +"),
    ("generate new = age IF age > 18", "duplicate if clause"),
    (
      "generate new = age, force",
      "generate does not accept if clauses or options",
    ),
    (
      "generate new = age + 1)",
      "unsupported token in expression: )",
    ),
    ("generate if = 1", "unsupported token in expression: ="),
    ("generate if", "missing expression after if"),
    (
      "generate if age",
      "generate does not accept if clauses or options",
    ),
    (
      "generate new =, force",
      "generate assignment requires an expression after =",
    ),
    ("generate new == age", "unsupported token in command: =="),
    ("generate new + age", "unsupported token in command: +"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }

  assert_eq!(
    parse_command("generate chained = age == 1 == 2").unwrap(),
    Command::Generate {
      variable: "chained".to_owned(),
      expression: GenerateExpression::Binary {
        left: Box::new(GenerateExpression::Binary {
          left: Box::new(GenerateExpression::Identifier("age".to_owned())),
          operator: GenerateBinaryOperator::Equal,
          right: Box::new(GenerateExpression::Number("1".to_owned())),
        }),
        operator: GenerateBinaryOperator::Equal,
        right: Box::new(GenerateExpression::Number("2".to_owned())),
      },
    }
  );
}

#[test]
fn codebook_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command(" CODEBOOK `bmi-zscore` sex ").unwrap(),
    Command::Codebook {
      variables: vec!["bmi-zscore".to_owned(), "sex".to_owned()],
    }
  );
  assert_eq!(
    parse_command("codebook").unwrap(),
    Command::Codebook { variables: vec![] }
  );
  assert_eq!(
    parse_command("codebook foo\"bar\"").unwrap(),
    Command::Codebook {
      variables: vec!["foo".to_owned(), "bar".to_owned()],
    }
  );
}

#[test]
fn codebook_preserves_exact_public_diagnostics() {
  let cases = [
    (
      "codebook age if age > 18",
      "codebook does not accept if clauses or options",
    ),
    (
      "codebook age, detail",
      "codebook does not accept if clauses or options",
    ),
    (
      "codebook age = 1",
      "codebook does not accept assignment syntax",
    ),
    (
      "codebook = 1",
      "codebook assignment requires a target before =",
    ),
    (
      "codebook age,",
      "comma must be followed by at least one option",
    ),
    ("codebook if", "missing expression after if"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn missing_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command(" MISSING `bmi-zscore` sex ").unwrap(),
    Command::Missing {
      variables: vec!["bmi-zscore".to_owned(), "sex".to_owned()],
    }
  );
  assert_eq!(
    parse_command("missing").unwrap(),
    Command::Missing { variables: vec![] }
  );
}

#[test]
fn missing_preserves_exact_public_diagnostics() {
  let cases = [
    (
      "missing age if age > 0",
      "missing does not accept if clauses or options",
    ),
    (
      "missing age, detail",
      "missing does not accept if clauses or options",
    ),
    (
      "missing age = other",
      "missing does not accept assignment syntax",
    ),
    (
      "missing = age",
      "missing assignment requires a target before =",
    ),
    (
      "missing age,",
      "comma must be followed by at least one option",
    ),
    ("missing,", "comma must be followed by at least one option"),
    ("missing if", "missing expression after if"),
    ("missing age==x", "unsupported token in command: =="),
    ("missing age-1", "unsupported token in command: -"),
    ("missing age+1", "unsupported token in command: +"),
    ("missing age!x", "unsupported token in command: !"),
    ("missing age@x", "unsupported token in command: @"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn duplicates_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command(" DUPLICATES report id label ").unwrap(),
    Command::Duplicates {
      variables: vec!["id".to_owned(), "label".to_owned()],
    }
  );
  assert_eq!(
    parse_command("duplicates id label").unwrap(),
    Command::Duplicates {
      variables: vec!["id".to_owned(), "label".to_owned()],
    }
  );
  assert_eq!(
    parse_command("duplicates `report`").unwrap(),
    Command::Duplicates {
      variables: vec!["report".to_owned()],
    }
  );
  assert_eq!(
    parse_command("duplicates \"report\"").unwrap(),
    Command::Duplicates { variables: vec![] }
  );
}

#[test]
fn duplicates_preserves_exact_public_diagnostics() {
  let cases = [
    (
      "duplicates id if id > 0",
      "duplicates does not accept if clauses or options",
    ),
    (
      "duplicates id, missing",
      "duplicates does not accept if clauses or options",
    ),
    (
      "duplicates id = other",
      "duplicates does not accept assignment syntax",
    ),
    (
      "duplicates = id",
      "duplicates assignment requires a target before =",
    ),
    (
      "duplicates id,",
      "comma must be followed by at least one option",
    ),
    (
      "duplicates,",
      "comma must be followed by at least one option",
    ),
    ("duplicates if", "missing expression after if"),
    ("duplicates id if", "missing expression after if"),
    ("duplicates id==x", "unsupported token in command: =="),
    ("duplicates id-1", "unsupported token in command: -"),
    ("duplicates id+1", "unsupported token in command: +"),
    ("duplicates id!x", "unsupported token in command: !"),
    ("duplicates id@x", "unsupported token in command: @"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn isid_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command(" ISID patient_id visit ").unwrap(),
    Command::Isid {
      variables: vec!["patient_id".to_owned(), "visit".to_owned()],
      missok: false,
    }
  );
  assert_eq!(
    parse_command("isid `patient_id` visit, missok").unwrap(),
    Command::Isid {
      variables: vec!["patient_id".to_owned(), "visit".to_owned()],
      missok: true,
    }
  );
  assert_eq!(
    parse_command("isid `a,b`, missok").unwrap(),
    Command::Isid {
      variables: vec!["a,b".to_owned()],
      missok: true,
    }
  );
  assert_eq!(
    parse_command("isid a a").unwrap(),
    Command::Isid {
      variables: vec!["a".to_owned(), "a".to_owned()],
      missok: false,
    }
  );
  assert_eq!(
    parse_command("isid `a``b`").unwrap(),
    Command::Isid {
      variables: vec!["a`b".to_owned()],
      missok: false,
    }
  );
}

#[test]
fn isid_preserves_exact_public_diagnostics() {
  let cases = [
    ("isid", "isid expects at least one key variable"),
    ("isid, missok", "isid expects at least one key variable"),
    ("isid,", "comma must be followed by at least one option"),
    (
      "isid patient_id if visit > 0",
      "isid only accepts a variable list and missok option",
    ),
    (
      "isid patient_id = other",
      "isid only accepts a variable list and missok option",
    ),
    (
      "isid patient_id =",
      "isid assignment requires an expression after =",
    ),
    (
      "isid = patient_id",
      "isid assignment requires a target before =",
    ),
    ("isid patient_id, report", "isid unsupported option: report"),
    (
      "isid patient_id, foo bar",
      "isid unsupported option: bar, foo",
    ),
    (
      "isid patient_id, missok(true)",
      "isid option missok does not accept a value",
    ),
    (
      "isid patient_id, missok 1",
      "option missok value must use option=value syntax",
    ),
    ("isid patient_id if", "missing expression after if"),
    ("isid patient_id==x", "unsupported token in command: =="),
    ("isid patient_id-x", "unsupported token in command: -"),
    ("isid patient_id+x", "unsupported token in command: +"),
    ("isid patient_id!x", "unsupported token in command: !"),
    ("isid patient_id@x", "unsupported token in command: @"),
    ("isid patient_id, MISSOK", "isid unsupported option: MISSOK"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn select_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command(" SELECT age sex ").unwrap(),
    Command::Select {
      variables: vec!["age".to_owned(), "sex".to_owned()],
    }
  );
  assert_eq!(
    parse_command("select\u{1c}`a,b`\u{1d}\"old name\"").unwrap(),
    Command::Select {
      variables: vec!["a,b".to_owned(), "old name".to_owned()],
    }
  );
  assert_eq!(
    parse_command("select age age").unwrap(),
    Command::Select {
      variables: vec!["age".to_owned(), "age".to_owned()],
    }
  );
}

#[test]
fn select_preserves_exact_public_diagnostics() {
  let cases = [
    ("select", "select expects at least one variable"),
    (
      "select age if age > 0",
      "select only accepts a variable list",
    ),
    ("select age, stable", "select only accepts a variable list"),
    ("select age = x", "select only accepts a variable list"),
    ("select = x", "select assignment requires a target before ="),
    (
      "select age =",
      "select assignment requires an expression after =",
    ),
    (
      "select age,",
      "comma must be followed by at least one option",
    ),
    ("select if", "missing expression after if"),
    ("select age==x", "unsupported token in command: =="),
    ("select age-1", "unsupported token in command: -"),
    ("select age+1", "unsupported token in command: +"),
    ("select age!x", "unsupported token in command: !"),
    ("select age@x", "unsupported token in command: @"),
    ("select:age", "unsupported token in command: :"),
    ("select if age >= ", "incomplete expression after >="),
    ("select age if age > 0 if age > 1", "duplicate if clause"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn keep_parses_an_ordered_projection() {
  assert_eq!(
    parse_command(" KEEP sex `age years` sex ").unwrap(),
    Command::Keep {
      variables: vec!["sex".to_owned(), "age years".to_owned(), "sex".to_owned()],
    }
  );
}

#[test]
fn keep_preserves_bounded_projection_diagnostics() {
  let cases = [
    ("keep", "keep expects a variable list or if clause"),
    ("keep if", "missing expression after if"),
    (
      "keep if age > 0",
      "keep if execution is deferred in the bounded runtime",
    ),
    (
      "keep age if age > 0",
      "keep cannot combine a variable list with an if clause",
    ),
    (
      "keep age, stable",
      "keep does not accept options or assignment syntax",
    ),
    (
      "keep age = other",
      "keep does not accept options or assignment syntax",
    ),
    ("keep = other", "keep assignment requires a target before ="),
    (
      "keep age =",
      "keep assignment requires an expression after =",
    ),
    (
      "keep if age > 0, stable",
      "keep does not accept options or assignment syntax",
    ),
    (
      "keep if age > 0,",
      "keep if execution is deferred in the bounded runtime",
    ),
    (
      "keep age if age > 0, stable",
      "keep does not accept options or assignment syntax",
    ),
    (
      "keep age if age > 0,",
      "keep cannot combine a variable list with an if clause",
    ),
    ("keep age==x", "unsupported token in command: =="),
    ("keep age-1", "unsupported token in command: -"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn drop_parses_an_explicit_projection() {
  assert_eq!(
    parse_command(" DROP sex `age years` sex ").unwrap(),
    Command::Drop {
      variables: vec!["sex".to_owned(), "age years".to_owned(), "sex".to_owned()],
    }
  );
}

#[test]
fn drop_preserves_bounded_projection_diagnostics() {
  let cases = [
    ("drop", "drop expects a variable list or if clause"),
    ("drop if", "missing expression after if"),
    (
      "drop if age > 0",
      "drop if execution is deferred in the bounded runtime",
    ),
    (
      "drop age if age > 0",
      "drop cannot combine a variable list with an if clause",
    ),
    (
      "drop age, stable",
      "drop does not accept options or assignment syntax",
    ),
    (
      "drop age = other",
      "drop does not accept options or assignment syntax",
    ),
    ("drop = other", "drop assignment requires a target before ="),
    (
      "drop age =",
      "drop assignment requires an expression after =",
    ),
    (
      "drop if age > 0, stable",
      "drop does not accept options or assignment syntax",
    ),
    (
      "drop if age > 0,",
      "drop if execution is deferred in the bounded runtime",
    ),
    (
      "drop age if age > 0, stable",
      "drop does not accept options or assignment syntax",
    ),
    (
      "drop age if age > 0,",
      "drop cannot combine a variable list with an if clause",
    ),
    ("drop age==x", "unsupported token in command: =="),
    ("drop age-1", "unsupported token in command: -"),
    ("drop age+1", "unsupported token in command: +"),
    ("drop age!x", "unsupported token in command: !"),
    ("drop age@x", "unsupported token in command: @"),
    ("drop age:x", "unsupported token in command: :"),
    ("drop if age > @", "unsupported token in command: @"),
    ("drop if age >= ", "incomplete expression after >="),
    ("drop if age > 0 if age > 1", "duplicate if clause"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn sort_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command(" SORT age label ").unwrap(),
    Command::Sort {
      variables: vec!["age".to_owned(), "label".to_owned()],
    }
  );
  assert_eq!(
    parse_command("sort\u{1c}`a,b`\u{1d}\"old name\"").unwrap(),
    Command::Sort {
      variables: vec!["a,b".to_owned(), "old name".to_owned()],
    }
  );
  assert_eq!(
    parse_command("sort age age").unwrap(),
    Command::Sort {
      variables: vec!["age".to_owned(), "age".to_owned()],
    }
  );
  assert_eq!(
    parse_command("sort age label now").unwrap(),
    Command::Sort {
      variables: vec!["age".to_owned(), "label".to_owned(), "now".to_owned()],
    }
  );
}

#[test]
fn sort_preserves_exact_public_diagnostics() {
  let cases = [
    ("sort", "sort expects at least one variable"),
    ("sort age if age > 0", "sort only accepts a variable list"),
    ("sort age, stable", "sort only accepts a variable list"),
    ("sort age = x", "sort only accepts a variable list"),
    ("sort = x", "sort assignment requires a target before ="),
    (
      "sort age =",
      "sort assignment requires an expression after =",
    ),
    ("sort age,", "comma must be followed by at least one option"),
    ("sort,", "comma must be followed by at least one option"),
    ("sort if", "missing expression after if"),
    ("sort age==x", "unsupported token in command: =="),
    ("sort age-1", "unsupported token in command: -"),
    ("sort age+1", "unsupported token in command: +"),
    ("sort age!x", "unsupported token in command: !"),
    ("sort age@x", "unsupported token in command: @"),
    ("sort:age", "unsupported token in command: :"),
    ("sort age:label", "unsupported token in command: :"),
    ("sort age/label", "unsupported token in command: /"),
    ("sort age.label", "unsupported token in command: ."),
    ("sort +age", "unsupported token in command: +"),
    ("sort -age", "unsupported token in command: -"),
    ("sort !age", "unsupported token in command: !"),
    ("sort @age", "unsupported token in command: @"),
    ("sort ``", "quoted identifier cannot be empty"),
    ("sort \"unterminated", "unterminated quoted string"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn gsort_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command(" GSORT +group_id -label ").unwrap(),
    Command::Gsort {
      keys: vec![
        SortKey {
          variable: "group_id".to_owned(),
          descending: false,
        },
        SortKey {
          variable: "label".to_owned(),
          descending: true,
        },
      ],
    }
  );
  assert_eq!(
    parse_command("gsort\u{1c}`-score`\u{1d}\"old name\"").unwrap(),
    Command::Gsort {
      keys: vec![
        SortKey {
          variable: "-score".to_owned(),
          descending: false,
        },
        SortKey {
          variable: "old name".to_owned(),
          descending: false,
        },
      ],
    }
  );
  assert_eq!(
    parse_command("gsort \"-score\"").unwrap(),
    Command::Gsort {
      keys: vec![SortKey {
        variable: "score".to_owned(),
        descending: true,
      }],
    }
  );
  assert_eq!(
    parse_command("gsort+age").unwrap(),
    Command::Gsort {
      keys: vec![SortKey {
        variable: "age".to_owned(),
        descending: false,
      }],
    }
  );
  assert_eq!(
    parse_command("gsort-age").unwrap(),
    Command::Gsort {
      keys: vec![SortKey {
        variable: "age".to_owned(),
        descending: true,
      }],
    }
  );
}

#[test]
fn gsort_preserves_exact_public_diagnostics() {
  let cases = [
    ("gsort", "gsort expects at least one variable"),
    (
      "gsort group_id if x > 0",
      "gsort only accepts a signed variable list",
    ),
    (
      "gsort group_id, stable",
      "gsort only accepts a signed variable list",
    ),
    (
      "gsort group_id = x",
      "gsort only accepts a signed variable list",
    ),
    ("gsort = x", "gsort assignment requires a target before ="),
    (
      "gsort group_id =",
      "gsort assignment requires an expression after =",
    ),
    (
      "gsort group_id,",
      "comma must be followed by at least one option",
    ),
    (
      "gsort --group_id",
      "gsort keys must use at most one + or - prefix",
    ),
    (
      "gsort -",
      "gsort expects a variable after each direction prefix",
    ),
    ("gsort age!x", "unsupported token in command: !"),
    ("gsort age@x", "unsupported token in command: @"),
    ("gsort ``", "quoted identifier cannot be empty"),
    ("gsort \"unterminated", "unterminated quoted string"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn recode_preserves_typed_rules_and_target_modes() {
  assert_eq!(
    parse_command("recode age (min/17 = 0) (18 19 20 = 1) (else = -1), generate(age_group)")
      .unwrap(),
    Command::Recode {
      variables: vec!["age".to_owned()],
      rules: vec![
        RecodeRule {
          inputs: vec![RecodeInput::Range {
            start: RecodeRangeEndpoint::Min,
            end: RecodeRangeEndpoint::Number("17".to_owned()),
          }],
          output: RecodeValue::Number("0".to_owned()),
        },
        RecodeRule {
          inputs: vec![
            RecodeInput::Value(RecodeValue::Number("18".to_owned())),
            RecodeInput::Value(RecodeValue::Number("19".to_owned())),
            RecodeInput::Value(RecodeValue::Number("20".to_owned())),
          ],
          output: RecodeValue::Number("1".to_owned()),
        },
        RecodeRule {
          inputs: vec![RecodeInput::Else],
          output: RecodeValue::Number("-1".to_owned()),
        },
      ],
      target: RecodeTarget::Generate {
        variables: vec!["age_group".to_owned()],
      },
    }
  );

  assert_eq!(
    parse_command("recode cost (missing = 999) (nonmissing = 100), replace").unwrap(),
    Command::Recode {
      variables: vec!["cost".to_owned()],
      rules: vec![
        RecodeRule {
          inputs: vec![RecodeInput::Missing],
          output: RecodeValue::Number("999".to_owned()),
        },
        RecodeRule {
          inputs: vec![RecodeInput::NonMissing],
          output: RecodeValue::Number("100".to_owned()),
        },
      ],
      target: RecodeTarget::Replace,
    }
  );

  assert_eq!(
    parse_command(r#"recode `sex value` ('F' = 1), generate(`sex code`)"#).unwrap(),
    Command::Recode {
      variables: vec!["sex value".to_owned()],
      rules: vec![RecodeRule {
        inputs: vec![RecodeInput::Value(RecodeValue::Text("F".to_owned()))],
        output: RecodeValue::Number("1".to_owned()),
      }],
      target: RecodeTarget::Generate {
        variables: vec!["sex code".to_owned()],
      },
    }
  );
}

#[test]
fn recode_preserves_exact_bounded_diagnostics() {
  let cases = [
    ("recode", "recode command: missing variable list and rules"),
    (
      "recode age (1 = 0)",
      "recode command requires either generate() or replace option",
    ),
    (
      "recode age (1 = 0), generate(age_group) replace",
      "recode command: cannot specify both generate() and replace",
    ),
    ("recode age, replace", "recode command: no rules specified"),
    (
      "recode (1 = 0), replace",
      "recode command: no variables specified",
    ),
    (
      "recode age (else 1 = 0), replace",
      "else rule must not be combined with other inputs",
    ),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn encode_preserves_source_target_and_optional_label() {
  assert_eq!(
    parse_command("encode sex, generate(sex_n)").unwrap(),
    Command::Encode {
      source: "sex".to_owned(),
      generate: "sex_n".to_owned(),
      label: None,
    }
  );
  assert_eq!(
    parse_command(r#"encode `region value`, generate(`region code`) label(regionlbl)"#).unwrap(),
    Command::Encode {
      source: "region value".to_owned(),
      generate: "region code".to_owned(),
      label: Some("regionlbl".to_owned()),
    }
  );
}

#[test]
fn encode_preserves_exact_bounded_diagnostics() {
  let cases = [
    (
      "encode",
      "encode expects syntax: encode <strvar>, generate(<newvar>)",
    ),
    ("encode sex", "encode requires generate(<newvar>)"),
    (
      "encode sex, label(sex_label)",
      "encode requires generate(<newvar>)",
    ),
    (
      "encode sex, generate(sex_n) unknown",
      "encode unsupported option: unknown",
    ),
    (
      "encode sex, GENERATE(sex_n)",
      "encode unsupported option: GENERATE",
    ),
    (
      "encode sex, generate(sex_n) generate(other)",
      "encode option generate can only be specified once",
    ),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn run_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command(" RUN analysis.td ").unwrap(),
    Command::Run {
      path: "analysis.td".to_owned(),
    }
  );
  assert_eq!(
    parse_command("run\u{1c}analysis.td").unwrap(),
    Command::Run {
      path: "analysis.td".to_owned(),
    }
  );
  assert_eq!(
    parse_command("run \"analysis.td\"").unwrap(),
    Command::Run {
      path: "\"analysis.td\"".to_owned(),
    }
  );
  assert_eq!(
    parse_command("run `analysis.td`").unwrap(),
    Command::Run {
      path: "`analysis.td`".to_owned(),
    }
  );
  assert_eq!(
    parse_command("run analysis.td,").unwrap(),
    Command::Run {
      path: "analysis.td,".to_owned(),
    }
  );
}

#[test]
fn run_preserves_exact_public_diagnostics() {
  let cases = [
    ("run", "run expects exactly one path: run <script>"),
    ("run   ", "run expects exactly one path: run <script>"),
    (
      "run a.td b.td",
      "run expects exactly one path: run <script>",
    ),
    (
      "run \"a b.td\"",
      "run expects exactly one path: run <script>",
    ),
    (
      "run a.td if x > 0",
      "run expects exactly one path: run <script>",
    ),
    ("run,", "comma must be followed by at least one option"),
    ("run,foo", "unknown command: run"),
    ("run=foo", "run assignment requires a target before ="),
    ("run==foo", "unsupported token in command: =="),
    ("run:foo", "unsupported token in command: :"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn save_and_export_are_public_syntax_only_commands() {
  assert_eq!(
    parse_command(" SAVE output.parquet ").unwrap(),
    Command::Save {
      path: "output.parquet".to_owned(),
      replace: false,
    }
  );
  assert_eq!(
    parse_command("export \"my output.parquet\", replace").unwrap(),
    Command::Export {
      path: "my output.parquet".to_owned(),
      replace: true,
    }
  );
  assert_eq!(
    parse_command("save `a,b`, replace replace").unwrap(),
    Command::Save {
      path: "a,b".to_owned(),
      replace: true,
    }
  );
  assert_eq!(
    parse_command("save:out.parquet").unwrap(),
    Command::Save {
      path: ":out.parquet".to_owned(),
      replace: false,
    }
  );
  assert_eq!(
    parse_command("export/out.csv").unwrap(),
    Command::Export {
      path: "/out.csv".to_owned(),
      replace: false,
    }
  );
}

#[test]
fn save_and_export_preserve_exact_public_diagnostics() {
  let cases = [
    ("save", "save expects exactly one path"),
    ("save one two", "save expects exactly one path"),
    ("export", "export expects exactly one path"),
    ("export one two", "export expects exactly one path"),
    (
      "save out if x > 0",
      "save does not accept if clauses or assignment syntax",
    ),
    (
      "export out = x",
      "export does not accept if clauses or assignment syntax",
    ),
    ("save if", "missing expression after if"),
    ("save = out", "save assignment requires a target before ="),
    (
      "export out =",
      "export assignment requires an expression after =",
    ),
    ("save out,", "comma must be followed by at least one option"),
    ("save out, force", "save unsupported option: force"),
    ("export out, REPLACE", "export unsupported option: REPLACE"),
    (
      "save out, replace=true",
      "save option replace does not accept a value",
    ),
    (
      "export out, replace(foo)",
      "export option replace does not accept a value",
    ),
    ("save out@x", "unsupported token in command: @"),
    ("export@out", "unsupported token in command: @"),
    (
      "save out, replace, replace",
      "option names must be identifiers",
    ),
    ("save ``, replace", "quoted identifier cannot be empty"),
    ("export \"unterminated", "unterminated quoted string"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn decode_preserves_source_and_target() {
  assert_eq!(
    parse_command("decode sex_n, generate(sex_str)").unwrap(),
    Command::Decode {
      source: "sex_n".to_owned(),
      generate: "sex_str".to_owned(),
    }
  );
  assert_eq!(
    parse_command(r#"decode `region code`, generate(`region value`)"#).unwrap(),
    Command::Decode {
      source: "region code".to_owned(),
      generate: "region value".to_owned(),
    }
  );
}

#[test]
fn decode_preserves_exact_bounded_diagnostics() {
  let cases = [
    (
      "decode",
      "decode expects syntax: decode <numvar>, generate(<newvar>)",
    ),
    ("decode sex_n", "decode requires generate(<newvar>)"),
    (
      "decode sex_n, label(sex_label)",
      "decode unsupported option: label",
    ),
    (
      "decode sex_n, GENERATE(sex_str)",
      "decode unsupported option: GENERATE",
    ),
    (
      "decode sex_n, generate(sex_str) generate(other)",
      "decode option generate can only be specified once",
    ),
    (
      "decode sex_n, generate(sex_str other)",
      "decode option generate expects exactly one variable",
    ),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn label_preserves_bounded_session_local_commands() {
  assert_eq!(
    parse_command("label variable age \"Age in years\"").unwrap(),
    Command::Label {
      command: LabelCommand::Variable {
        variable: "age".to_owned(),
        text: Some("Age in years".to_owned()),
      },
    }
  );
  assert_eq!(
    parse_command("label variable age, clear").unwrap(),
    Command::Label {
      command: LabelCommand::Variable {
        variable: "age".to_owned(),
        text: None,
      },
    }
  );
  assert_eq!(
    parse_command("label define sexlbl 0 \"Male\" 1 \"Female\", replace").unwrap(),
    Command::Label {
      command: LabelCommand::Define {
        name: "sexlbl".to_owned(),
        mappings: vec![
          (LabelValue::Integer(0), "Male".to_owned()),
          (LabelValue::Integer(1), "Female".to_owned()),
        ],
        replace: true,
      },
    }
  );
  assert_eq!(
    parse_command("label define status -1 \"Missing\" 1.5 \"Fraction\"").unwrap(),
    Command::Label {
      command: LabelCommand::Define {
        name: "status".to_owned(),
        mappings: vec![
          (LabelValue::Integer(-1), "Missing".to_owned()),
          (LabelValue::Number("1.5".to_owned()), "Fraction".to_owned()),
        ],
        replace: false,
      },
    }
  );
  assert_eq!(
    parse_command("label values sex sexlbl").unwrap(),
    Command::Label {
      command: LabelCommand::Values {
        variable: "sex".to_owned(),
        set_name: Some("sexlbl".to_owned()),
      },
    }
  );
  assert_eq!(
    parse_command("label list sexlbl other").unwrap(),
    Command::Label {
      command: LabelCommand::List {
        names: vec!["sexlbl".to_owned(), "other".to_owned()],
      },
    }
  );
  assert_eq!(
    parse_command("label drop sexlbl").unwrap(),
    Command::Label {
      command: LabelCommand::Drop {
        names: vec!["sexlbl".to_owned()],
      },
    }
  );
}

#[test]
fn label_preserves_bounded_diagnostics_and_persistence_deferral() {
  let cases = [
    (
      "label variable age Age",
      "label variable expects syntax: label variable <varname> \"text\"",
    ),
    (
      "label define sexlbl 0 Male",
      "label define text must be a quoted string",
    ),
    (
      "label drop",
      "label drop expects at least one label set name",
    ),
    (
      "label save labels.json",
      "label expects syntax: label variable|define|values|list|drop|save|use ...",
    ),
    (
      "label define sexlbl 0 \"Male\" 0 \"Duplicate\"",
      "label define duplicate value: 0",
    ),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn rename_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command(" rename sex gender ").unwrap(),
    Command::Rename {
      old_name: "sex".to_owned(),
      new_name: "gender".to_owned(),
    }
  );
  assert_eq!(
    parse_command("RENAME\u{1c}  `old-name`\u{1d}\"new name\"").unwrap(),
    Command::Rename {
      old_name: "old-name".to_owned(),
      new_name: "new name".to_owned(),
    }
  );
  assert_eq!(
    parse_command("rename old old").unwrap(),
    Command::Rename {
      old_name: "old".to_owned(),
      new_name: "old".to_owned(),
    }
  );
}

#[test]
fn rename_preserves_exact_public_diagnostics() {
  let cases = [
    (
      "rename",
      "rename expects exactly two variables: rename old new",
    ),
    (
      "rename old",
      "rename expects exactly two variables: rename old new",
    ),
    (
      "rename old new now",
      "rename expects exactly two variables: rename old new",
    ),
    ("rename if", "missing expression after if"),
    ("rename old if", "missing expression after if"),
    ("rename old new if", "missing expression after if"),
    (
      "rename old if x > 0",
      "rename expects exactly two variables: rename old new",
    ),
    (
      "rename old new if x > 0",
      "rename expects exactly two variables: rename old new",
    ),
    (
      "rename old new, replace",
      "rename expects exactly two variables: rename old new",
    ),
    (
      "rename old new,",
      "comma must be followed by at least one option",
    ),
    (
      "rename=old new",
      "rename assignment requires a target before =",
    ),
    (
      "rename = old",
      "rename assignment requires a target before =",
    ),
    ("rename==old new", "unsupported token in command: =="),
    ("rename:old new", "unsupported token in command: :"),
    ("rename old-new new", "unsupported token in command: -"),
    ("rename old+new new", "unsupported token in command: +"),
    ("rename old@new new", "unsupported token in command: @"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn datasignature_preserves_exact_public_diagnostics() {
  let cases = [
    ("datasignature age if", "missing expression after if"),
    ("datasignature if,", "missing expression after if"),
    (
      "datasignature,",
      "comma must be followed by at least one option",
    ),
    (
      "datasignature = value",
      "datasignature assignment requires a target before =",
    ),
    ("datasignature age-1", "unsupported token in command: -"),
    ("datasignature age==x", "unsupported token in command: =="),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn set_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command("set graph_format png").unwrap(),
    Command::Set {
      name: SettingName::GraphFormat,
      value: "png".to_owned(),
    }
  );
  assert_eq!(
    parse_command("set artifact_dir \"my plots\"").unwrap(),
    Command::Set {
      name: SettingName::ArtifactDir,
      value: "my plots".to_owned(),
    }
  );
}

#[test]
fn use_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command("use data.parquet").unwrap(),
    Command::Use {
      source: DataSource::LocalPath("data.parquet".to_owned()),
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: None,
      has_header: None,
    }
  );
  assert_eq!(
    parse_command("USE s3://bucket/data.parquet, lazy engine=polars").unwrap(),
    Command::Use {
      source: DataSource::Uri("s3://bucket/data.parquet".to_owned()),
      execution_mode: ExecutionMode::Lazy,
      lazy_engine: Some(LazyEngine::Polars),
      delimiter: None,
      has_header: None,
    }
  );
  assert_eq!(
    parse_command("use survey.csv, delimiter(\",\") has_header(true)").unwrap(),
    Command::Use {
      source: DataSource::LocalPath("survey.csv".to_owned()),
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: Some(",".to_owned()),
      has_header: Some(true),
    }
  );
  assert_eq!(
    parse_command("use survey.csv, has_header").unwrap(),
    Command::Use {
      source: DataSource::LocalPath("survey.csv".to_owned()),
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: None,
      has_header: Some(true),
    }
  );
}
