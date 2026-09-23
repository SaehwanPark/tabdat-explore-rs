use tabdat_language::{
  AssertBinaryOperator, AssertExpression, BayesPrefixCommand, CfRegressCommand, CollapseCommand,
  CollapseStatistic, Command, DataSource, DidCommand, DmlCommand, DrDidCommand, DrDidMethod,
  EstatCommand, EstatSubcommand, ExecutionMode, GenerateBinaryOperator, GenerateExpression,
  IvEstimator, IvRegressCommand, JoinCommand, JoinHow, LabelCommand, LabelValue, LazyEngine,
  LincomCommand, LogitCommand, LowessCommand, PanelAction, PanelCommand, ProbitCommand,
  RecodeInput, RecodeRangeEndpoint, RecodeRule, RecodeTarget, RecodeValue, RegressCommand,
  RegressEstimator, ReshapeCommand, ReshapeDirection, RowLimit, SettingName, SortKey,
  TabulateCommand, XtAbondCommand, XtDataCommand, XtDataTransform, XtLogitCommand, XtRegCommand,
  XtRegEstimator, parse_command,
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
fn panel_parses_bounded_declaration_forms() {
  assert_eq!(
    parse_command("panel").unwrap(),
    Command::Panel {
      command: PanelCommand {
        action: PanelAction::Report,
      },
    }
  );
  assert_eq!(
    parse_command("PANEL firm_id year").unwrap(),
    Command::Panel {
      command: PanelCommand {
        action: PanelAction::Set {
          id_variable: "firm_id".to_owned(),
          time_variable: "year".to_owned(),
        },
      },
    }
  );
  assert_eq!(
    parse_command("panel clear").unwrap(),
    Command::Panel {
      command: PanelCommand {
        action: PanelAction::Clear,
      },
    }
  );
  assert_eq!(
    parse_command(r#"panel "clear""#).unwrap(),
    Command::Panel {
      command: PanelCommand {
        action: PanelAction::Clear,
      },
    }
  );
  assert_eq!(
    parse_command("panel `clear` year").unwrap(),
    Command::Panel {
      command: PanelCommand {
        action: PanelAction::Set {
          id_variable: "clear".to_owned(),
          time_variable: "year".to_owned(),
        },
      },
    }
  );
}

#[test]
fn panel_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "panel firm_id year extra",
      "panel expects syntax: panel [<id_var> <time_var>|clear]",
    ),
    (
      "panel firm_id if year > 2020",
      "panel expects syntax: panel [<id_var> <time_var>|clear]",
    ),
    (
      "panel firm_id year, replace",
      "panel expects syntax: panel [<id_var> <time_var>|clear]",
    ),
    (
      "panel firm_id = year",
      "panel expects syntax: panel [<id_var> <time_var>|clear]",
    ),
    (
      "panel firm_id firm_id",
      "panel id and time variables must be distinct",
    ),
    (
      "panel clear now",
      "panel expects syntax: panel [<id_var> <time_var>|clear]",
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
fn xtdata_parses_bounded_transform_forms() {
  assert_eq!(
    parse_command("xtdata wage exper, within").unwrap(),
    Command::XtData {
      command: XtDataCommand {
        variables: vec!["wage".to_owned(), "exper".to_owned()],
        transform: XtDataTransform::Within,
      },
    }
  );
  assert_eq!(
    parse_command("XTDATA `wage value`, between").unwrap(),
    Command::XtData {
      command: XtDataCommand {
        variables: vec!["wage value".to_owned()],
        transform: XtDataTransform::Between,
      },
    }
  );
  assert_eq!(
    parse_command("xtdata wage, within within").unwrap(),
    Command::XtData {
      command: XtDataCommand {
        variables: vec!["wage".to_owned()],
        transform: XtDataTransform::Within,
      },
    }
  );
}

#[test]
fn xtdata_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "xtdata",
      "xtdata expects syntax: xtdata <varlist>, within|between",
    ),
    (
      "xtdata wage",
      "xtdata requires exactly one of within or between",
    ),
    (
      "xtdata wage, within between",
      "xtdata requires exactly one of within or between",
    ),
    ("xtdata wage, detail", "xtdata unsupported option: detail"),
    (
      "xtdata wage, within=true",
      "xtdata option within does not accept a value",
    ),
    (
      "xtdata wage if year > 2020, within",
      "xtdata expects syntax: xtdata <varlist>, within|between",
    ),
    ("xtdata wage, `within`", "option names must be identifiers"),
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
fn ivregress_parses_bounded_estimator_forms() {
  assert_eq!(
    parse_command("ivregress 2sls cost age bmi, endog(hours) iv(distance policy)").unwrap(),
    Command::IvRegress {
      command: IvRegressCommand {
        outcome: "cost".to_owned(),
        exogenous: vec!["age".to_owned(), "bmi".to_owned()],
        endogenous: "hours".to_owned(),
        instruments: vec!["distance".to_owned(), "policy".to_owned()],
        robust: false,
        cluster_variable: None,
        include_intercept: true,
        estimator: IvEstimator::TwoStageLeastSquares,
      },
    }
  );
  assert_eq!(
    parse_command(
      "IVREGRESS gmm `total cost` `age value`, endog(`hours value`) iv(`distance value`) cluster(group_id) noconstant"
    )
    .unwrap(),
    Command::IvRegress {
      command: IvRegressCommand {
        outcome: "total cost".to_owned(),
        exogenous: vec!["age value".to_owned()],
        endogenous: "hours value".to_owned(),
        instruments: vec!["distance value".to_owned()],
        robust: false,
        cluster_variable: Some("group_id".to_owned()),
        include_intercept: false,
        estimator: IvEstimator::GeneralizedMethodOfMoments,
      },
    }
  );
  assert_eq!(
    parse_command("ivregress 2sls cost, endog(hours) iv(distance) robust").unwrap(),
    Command::IvRegress {
      command: IvRegressCommand {
        outcome: "cost".to_owned(),
        exogenous: vec![],
        endogenous: "hours".to_owned(),
        instruments: vec!["distance".to_owned()],
        robust: true,
        cluster_variable: None,
        include_intercept: true,
        estimator: IvEstimator::TwoStageLeastSquares,
      },
    }
  );
}

#[test]
fn ivregress_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "ivregress",
      "ivregress expects syntax: ivregress 2sls|gmm <y> [exog_vars], endog(<var>) iv(<vars>)",
    ),
    (
      "ivregress liml y x, endog(z) iv(w)",
      "ivregress estimator must be 2sls or gmm",
    ),
    (
      "ivregress 2sls y x",
      "ivregress option endog expects one variable",
    ),
    (
      "ivregress 2sls y x, endog(z)",
      "ivregress option iv expects at least one variable",
    ),
    (
      "ivregress 2sls y x, endog(z w) iv(q)",
      "ivregress option endog expects one variable",
    ),
    (
      "ivregress 2sls y x, endog(z) iv(w) robust=true",
      "ivregress option robust does not accept a value",
    ),
    (
      "ivregress 2sls y x, endog(z) iv(w) robust cluster(g)",
      "ivregress cannot combine robust and cluster",
    ),
    (
      "ivregress 2sls y z, endog(z) iv(w)",
      "ivregress endog variable must not appear in exogenous variables",
    ),
    (
      "ivregress 2sls y x, endog(z) iv(w) foo",
      "ivregress unsupported option: foo",
    ),
    (
      "ivregress `2sls` y, endog(z) iv(w)",
      "ivregress estimator must be 2sls or gmm",
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
fn xtreg_parses_bounded_panel_estimator_forms() {
  assert_eq!(
    parse_command("xtreg wage exper tenure, fe").unwrap(),
    Command::XtReg {
      command: XtRegCommand {
        outcome: "wage".to_owned(),
        predictors: vec!["exper".to_owned(), "tenure".to_owned()],
        estimator: XtRegEstimator::FixedEffects,
        robust: false,
        cluster_variable: None,
      },
    }
  );
  assert_eq!(
    parse_command("XTREG `wage value` `exper value`, re").unwrap(),
    Command::XtReg {
      command: XtRegCommand {
        outcome: "wage value".to_owned(),
        predictors: vec!["exper value".to_owned()],
        estimator: XtRegEstimator::RandomEffects,
        robust: false,
        cluster_variable: None,
      },
    }
  );
  assert_eq!(
    parse_command("xtreg wage exper, re robust").unwrap(),
    Command::XtReg {
      command: XtRegCommand {
        outcome: "wage".to_owned(),
        predictors: vec!["exper".to_owned()],
        estimator: XtRegEstimator::RandomEffects,
        robust: true,
        cluster_variable: None,
      },
    }
  );
  assert_eq!(
    parse_command("xtreg wage exper, fe cluster(firm_id)").unwrap(),
    Command::XtReg {
      command: XtRegCommand {
        outcome: "wage".to_owned(),
        predictors: vec!["exper".to_owned()],
        estimator: XtRegEstimator::FixedEffects,
        robust: false,
        cluster_variable: Some("firm_id".to_owned()),
      },
    }
  );
}

#[test]
fn xtreg_preserves_bounded_parser_diagnostics() {
  let cases = [
    ("xtreg", "xtreg expects syntax: xtreg <y> <xvars>, fe|re"),
    (
      "xtreg wage",
      "xtreg expects syntax: xtreg <y> <xvars>, fe|re",
    ),
    ("xtreg wage exper", "xtreg requires exactly one of fe or re"),
    (
      "xtreg wage exper, fe re",
      "xtreg requires exactly one of fe or re",
    ),
    (
      "xtreg wage exper, fe=true",
      "xtreg option fe does not accept a value",
    ),
    (
      "xtreg wage exper, fe cluster(firm year)",
      "xtreg option cluster expects one variable",
    ),
    (
      "xtreg wage exper, fe cluster(firm) robust",
      "xtreg cannot combine robust and cluster",
    ),
    (
      "xtreg wage exper, detail",
      "xtreg unsupported option: detail",
    ),
    (
      "xtreg wage if year > 2020, fe",
      "xtreg expects syntax: xtreg <y> <xvars>, fe|re",
    ),
    ("xtreg wage exper, FE", "xtreg unsupported option: FE"),
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
fn xtabond_parses_bounded_dynamic_panel_forms() {
  assert_eq!(
    parse_command("xtabond wage").unwrap(),
    Command::XtAbond {
      command: XtAbondCommand {
        outcome: "wage".to_owned(),
        predictors: vec![],
        robust: false,
        lag_depth: 1,
        instrument_lag_start: 2,
      },
    }
  );
  assert_eq!(
    parse_command("xtabond wage exposure, robust").unwrap(),
    Command::XtAbond {
      command: XtAbondCommand {
        outcome: "wage".to_owned(),
        predictors: vec!["exposure".to_owned()],
        robust: true,
        lag_depth: 1,
        instrument_lag_start: 2,
      },
    }
  );
  assert_eq!(
    parse_command("xtabond wage exposure, lags(2) instlag(3)").unwrap(),
    Command::XtAbond {
      command: XtAbondCommand {
        outcome: "wage".to_owned(),
        predictors: vec!["exposure".to_owned()],
        robust: false,
        lag_depth: 2,
        instrument_lag_start: 3,
      },
    }
  );
  assert_eq!(
    parse_command("xtabond `wage value` `exposure value`, lags(2) instlag(3)").unwrap(),
    Command::XtAbond {
      command: XtAbondCommand {
        outcome: "wage value".to_owned(),
        predictors: vec!["exposure value".to_owned()],
        robust: false,
        lag_depth: 2,
        instrument_lag_start: 3,
      },
    }
  );
  assert_eq!(
    parse_command("xtabond \"wage value\" \"exposure value\", lags(2) instlag(3)").unwrap(),
    Command::XtAbond {
      command: XtAbondCommand {
        outcome: "wage value".to_owned(),
        predictors: vec!["exposure value".to_owned()],
        robust: false,
        lag_depth: 2,
        instrument_lag_start: 3,
      },
    }
  );
  assert_eq!(
    parse_command("xtabond wage, instlag(3)").unwrap(),
    Command::XtAbond {
      command: XtAbondCommand {
        outcome: "wage".to_owned(),
        predictors: vec![],
        robust: false,
        lag_depth: 1,
        instrument_lag_start: 3,
      },
    }
  );
  assert_eq!(
    parse_command("xtabond wage, robust robust lags(1) instlag(2)").unwrap(),
    Command::XtAbond {
      command: XtAbondCommand {
        outcome: "wage".to_owned(),
        predictors: vec![],
        robust: true,
        lag_depth: 1,
        instrument_lag_start: 2,
      },
    }
  );
}

#[test]
fn xtabond_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "xtabond",
      "xtabond expects syntax: xtabond <y> [xvars] [, robust lags(#) instlag(#)]",
    ),
    (
      "xtabond wage if x > 0",
      "xtabond expects syntax: xtabond <y> [xvars] [, robust lags(#) instlag(#)]",
    ),
    (
      "xtabond wage, robust=true",
      "xtabond option robust does not accept a value",
    ),
    (
      "xtabond wage, cluster(firm_id)",
      "xtabond unsupported option: cluster",
    ),
    (
      "xtabond wage, lags(0)",
      "xtabond option lags must be at least 1",
    ),
    (
      "xtabond wage, instlag(1)",
      "xtabond option instlag must be at least 2",
    ),
    (
      "xtabond wage, lags(2) instlag(2)",
      "xtabond option instlag must be greater than option lags",
    ),
    (
      "xtabond wage, lags(2) lags(3)",
      "xtabond option lags may only be supplied once",
    ),
    ("xtabond wage, detail", "xtabond unsupported option: detail"),
    (
      "xtabond wage, lags(foo)",
      "option lags expects a numeric value",
    ),
    (
      "xtabond wage, instlag(foo)",
      "option instlag expects a numeric value",
    ),
    (
      "xtabond wage, lags(1.5) instlag(3)",
      "xtabond option lags expects an integer value",
    ),
    (
      "xtabond wage, lags(1) instlag(3) foo bar",
      "xtabond unsupported option: bar, foo",
    ),
    (
      "XTABOND wage, LAGS(2)",
      "option LAGS values must be identifiers",
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
fn xtlogit_parses_bounded_panel_fixed_effects_forms() {
  assert_eq!(
    parse_command("xtlogit y x, fe").unwrap(),
    Command::XtLogit {
      command: XtLogitCommand {
        outcome: "y".to_owned(),
        predictors: vec!["x".to_owned()],
        robust: false,
      },
    }
  );
  assert_eq!(
    parse_command("xtlogit y x1 x2, fe").unwrap(),
    Command::XtLogit {
      command: XtLogitCommand {
        outcome: "y".to_owned(),
        predictors: vec!["x1".to_owned(), "x2".to_owned()],
        robust: false,
      },
    }
  );
  assert_eq!(
    parse_command("xtlogit y x, fe robust").unwrap(),
    Command::XtLogit {
      command: XtLogitCommand {
        outcome: "y".to_owned(),
        predictors: vec!["x".to_owned()],
        robust: true,
      },
    }
  );
  assert_eq!(
    parse_command("xtlogit y x, robust fe").unwrap(),
    Command::XtLogit {
      command: XtLogitCommand {
        outcome: "y".to_owned(),
        predictors: vec!["x".to_owned()],
        robust: true,
      },
    }
  );
  assert_eq!(
    parse_command("xtlogit `y var` `x var`, fe").unwrap(),
    Command::XtLogit {
      command: XtLogitCommand {
        outcome: "y var".to_owned(),
        predictors: vec!["x var".to_owned()],
        robust: false,
      },
    }
  );
  assert_eq!(
    parse_command("xtlogit \"y var\" \"x var\", fe").unwrap(),
    Command::XtLogit {
      command: XtLogitCommand {
        outcome: "y var".to_owned(),
        predictors: vec!["x var".to_owned()],
        robust: false,
      },
    }
  );
  assert_eq!(
    parse_command("xtlogit y x, fe fe").unwrap(),
    Command::XtLogit {
      command: XtLogitCommand {
        outcome: "y".to_owned(),
        predictors: vec!["x".to_owned()],
        robust: false,
      },
    }
  );
  assert_eq!(
    parse_command("XTLOGIT y x, fe robust").unwrap(),
    Command::XtLogit {
      command: XtLogitCommand {
        outcome: "y".to_owned(),
        predictors: vec!["x".to_owned()],
        robust: true,
      },
    }
  );
}

#[test]
fn xtlogit_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "xtlogit",
      "xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]",
    ),
    (
      "xtlogit, fe",
      "xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]",
    ),
    (
      "xtlogit y, fe",
      "xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]",
    ),
    ("xtlogit y x", "xtlogit requires option fe"),
    ("xtlogit y x, robust", "xtlogit requires option fe"),
    (
      "xtlogit y x,",
      "comma must be followed by at least one option",
    ),
    (
      "xtlogit y x if y > 0, fe",
      "xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]",
    ),
    ("xtlogit y x, fe extra", "xtlogit unsupported option: extra"),
    (
      "xtlogit y x, fe foo bar",
      "xtlogit unsupported option: bar, foo",
    ),
    (
      "xtlogit y x, FE ROBUST",
      "xtlogit unsupported option: FE, ROBUST",
    ),
    ("xtlogit y x, fe(1)", "option fe values must be identifiers"),
    (
      "xtlogit y x, fe(a)",
      "xtlogit option fe does not accept a value",
    ),
    (
      "xtlogit y x, fe=1",
      "xtlogit option fe does not accept a value",
    ),
    (
      "xtlogit y x, fe=a",
      "xtlogit option fe does not accept a value",
    ),
    (
      "xtlogit y x, fe robust(1)",
      "option robust values must be identifiers",
    ),
    (
      "xtlogit y x, fe robust(a)",
      "xtlogit option robust does not accept a value",
    ),
    (
      "xtlogit y x, fe robust=1",
      "xtlogit option robust does not accept a value",
    ),
    ("xtlogit=", "xtlogit assignment requires a target before ="),
    (
      "xtlogit = 1",
      "xtlogit assignment requires a target before =",
    ),
    ("xtlogit==", "unsupported token in command: =="),
    ("xtlogit == 1", "unsupported token in command: =="),
    ("xtlogit:", "unsupported token in command: :"),
    ("xtlogit: regress y x", "unsupported token in command: :"),
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
fn lowess_parses_bounded_smoother_forms() {
  assert_eq!(
    parse_command("lowess y x, gen(y_hat)").unwrap(),
    Command::Lowess {
      command: LowessCommand {
        outcome: "y".to_owned(),
        predictor: "x".to_owned(),
        target_variable: "y_hat".to_owned(),
        bandwidth: (2.0f64 / 3.0f64).to_string(),
      },
    }
  );
  assert_eq!(
    parse_command("lowess y x, gen(y_hat) bandwidth=0.5").unwrap(),
    Command::Lowess {
      command: LowessCommand {
        outcome: "y".to_owned(),
        predictor: "x".to_owned(),
        target_variable: "y_hat".to_owned(),
        bandwidth: "0.5".to_owned(),
      },
    }
  );
  assert_eq!(
    parse_command("LOWESS y x, gen(y_hat) bandwidth=0.8").unwrap(),
    Command::Lowess {
      command: LowessCommand {
        outcome: "y".to_owned(),
        predictor: "x".to_owned(),
        target_variable: "y_hat".to_owned(),
        bandwidth: "0.8".to_owned(),
      },
    }
  );
  assert_eq!(
    parse_command("lowess `y var` `x var`, gen(y_hat)").unwrap(),
    Command::Lowess {
      command: LowessCommand {
        outcome: "y var".to_owned(),
        predictor: "x var".to_owned(),
        target_variable: "y_hat".to_owned(),
        bandwidth: (2.0f64 / 3.0f64).to_string(),
      },
    }
  );
  assert_eq!(
    parse_command("lowess \"y var\" \"x var\", gen(y_hat)").unwrap(),
    Command::Lowess {
      command: LowessCommand {
        outcome: "y var".to_owned(),
        predictor: "x var".to_owned(),
        target_variable: "y_hat".to_owned(),
        bandwidth: (2.0f64 / 3.0f64).to_string(),
      },
    }
  );
}

#[test]
fn lowess_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "lowess",
      "lowess expects syntax: lowess <y> <x>, gen(<newvar>) [bandwidth=<0,1>]",
    ),
    (
      "lowess y",
      "lowess expects syntax: lowess <y> <x>, gen(<newvar>) [bandwidth=<0,1>]",
    ),
    (
      "lowess y x z, gen(y_hat)",
      "lowess expects syntax: lowess <y> <x>, gen(<newvar>) [bandwidth=<0,1>]",
    ),
    (
      "lowess y x if y > 0, gen(y_hat)",
      "lowess expects syntax: lowess <y> <x>, gen(<newvar>) [bandwidth=<0,1>]",
    ),
    ("lowess y x", "lowess option gen expects one variable"),
    (
      "lowess y x,",
      "comma must be followed by at least one option",
    ),
    ("lowess y x, gen", "lowess option gen expects variables"),
    ("lowess y x, gen()", "option gen expects at least one value"),
    (
      "lowess y x, gen(y1 y2)",
      "lowess option gen expects one variable",
    ),
    (
      "lowess y x, gen(y1) gen(y2)",
      "lowess option gen may only be supplied once",
    ),
    (
      "lowess y x, gen(y_hat) extra",
      "lowess unsupported option: extra",
    ),
    (
      "lowess y x, GEN(y_hat) BANDWIDTH=0.8",
      "lowess unsupported option: BANDWIDTH, GEN",
    ),
    (
      "lowess y x, gen(y_hat) bandwidth",
      "lowess option bandwidth expects a numeric value",
    ),
    (
      "lowess y x, gen(y_hat) bandwidth=0",
      "lowess option bandwidth must be between 0 and 1",
    ),
    (
      "lowess y x, gen(y_hat) bandwidth=1",
      "lowess option bandwidth must be between 0 and 1",
    ),
    (
      "lowess y x, gen(y_hat) bandwidth=1.5",
      "lowess option bandwidth must be between 0 and 1",
    ),
    (
      "lowess y x, gen(y_hat) bandwidth=abc",
      "lowess option bandwidth expects a numeric value",
    ),
    (
      "lowess y x, gen(y_hat) bandwidth(0.5)",
      "option bandwidth values must be identifiers",
    ),
    (
      "lowess y x, gen(y_hat) bandwidth(abc)",
      "lowess option bandwidth expects a numeric value",
    ),
    (
      "lowess y x, gen(y_hat) bandwidth(a b)",
      "lowess option bandwidth expects one value",
    ),
    (
      "lowess y x, gen(y_hat) bandwidth=0.5 bandwidth=0.6",
      "lowess option bandwidth may only be supplied once",
    ),
    ("lowess=", "lowess assignment requires a target before ="),
    ("lowess = 1", "lowess assignment requires a target before ="),
    ("lowess==", "unsupported token in command: =="),
    ("lowess == 1", "unsupported token in command: =="),
    ("lowess:", "unsupported token in command: :"),
    ("lowess: regress y x", "unsupported token in command: :"),
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
fn did_parses_bounded_estimator_forms() {
  assert_eq!(
    parse_command("did y, treat(d) post(t)").unwrap(),
    Command::Did {
      command: DidCommand {
        outcome: "y".to_owned(),
        controls: vec![],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        robust: false,
      },
    }
  );
  assert_eq!(
    parse_command("did y x1 x2, treat(d) post(t) robust").unwrap(),
    Command::Did {
      command: DidCommand {
        outcome: "y".to_owned(),
        controls: vec!["x1".to_owned(), "x2".to_owned()],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        robust: true,
      },
    }
  );
  assert_eq!(
    parse_command("did y, post(t) treat(d)").unwrap(),
    Command::Did {
      command: DidCommand {
        outcome: "y".to_owned(),
        controls: vec![],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        robust: false,
      },
    }
  );
  assert_eq!(
    parse_command("DID y, treat(d) post(t)").unwrap(),
    Command::Did {
      command: DidCommand {
        outcome: "y".to_owned(),
        controls: vec![],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        robust: false,
      },
    }
  );
  assert_eq!(
    parse_command("did `y var` `x var`, treat(`d var`) post(`t var`)").unwrap(),
    Command::Did {
      command: DidCommand {
        outcome: "y var".to_owned(),
        controls: vec!["x var".to_owned()],
        treatment_variable: "d var".to_owned(),
        post_variable: "t var".to_owned(),
        robust: false,
      },
    }
  );
  assert_eq!(
    parse_command("did \"y var\" \"x var\", treat(d) post(t)").unwrap(),
    Command::Did {
      command: DidCommand {
        outcome: "y var".to_owned(),
        controls: vec!["x var".to_owned()],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        robust: false,
      },
    }
  );
}

#[test]
fn did_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "did",
      "did expects syntax: did <y> [controls], treat(<var>) post(<var>)",
    ),
    (
      "did if x > 0, treat(d) post(t)",
      "did expects syntax: did <y> [controls], treat(<var>) post(<var>)",
    ),
    (
      "did = 1, treat(d) post(t)",
      "did assignment requires a target before =",
    ),
    (
      "did y = 1, treat(d) post(t)",
      "did expects syntax: did <y> [controls], treat(<var>) post(<var>)",
    ),
    (
      "did y if x > 0, treat(d) post(t)",
      "did expects syntax: did <y> [controls], treat(<var>) post(<var>)",
    ),
    ("did y", "did option treat expects one variable"),
    ("did y,", "comma must be followed by at least one option"),
    ("did y, treat", "did option treat expects variables"),
    ("did y, treat()", "option treat expects at least one value"),
    (
      "did y, treat(d1 d2) post(t)",
      "did option treat expects one variable",
    ),
    (
      "did y, treat(d) treat(d2) post(t)",
      "did option treat may only be supplied once",
    ),
    ("did y, treat(d)", "did option post expects one variable"),
    ("did y, treat(d) post", "did option post expects variables"),
    (
      "did y, treat(d) post()",
      "option post expects at least one value",
    ),
    (
      "did y, treat(d) post(t1 t2)",
      "did option post expects one variable",
    ),
    (
      "did y, treat(d) post(t) post(t2)",
      "did option post may only be supplied once",
    ),
    (
      "did y, treat(d) post(t) extra",
      "did unsupported option: extra",
    ),
    (
      "did y, TREAT(d) POST(t)",
      "did unsupported option: POST, TREAT",
    ),
    (
      "did y, treat(d) post(t) robust=1",
      "did option robust does not accept a value",
    ),
    (
      "did y, treat(d) post(t) robust(1)",
      "option robust values must be identifiers",
    ),
    (
      "did y, treat(d) post(t) robust(foo)",
      "did option robust does not accept a value",
    ),
    (
      "did y, treat(d) post(d)",
      "did treatment and post variables must be distinct",
    ),
    (
      "did y, treat(y) post(t)",
      "did treatment and post variables must differ from outcome",
    ),
    (
      "did y, treat(d) post(y)",
      "did treatment and post variables must differ from outcome",
    ),
    (
      "did y d, treat(d) post(t)",
      "did treatment and post variables must not appear in controls",
    ),
    (
      "did y t, treat(d) post(t)",
      "did treatment and post variables must not appear in controls",
    ),
    ("did=", "did assignment requires a target before ="),
    ("did = 1", "did assignment requires a target before ="),
    ("did==", "unsupported token in command: =="),
    ("did == 1", "unsupported token in command: =="),
    ("did:", "unsupported token in command: :"),
    (
      "did: y, treat(d) post(t)",
      "unsupported token in command: :",
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
fn drdid_parses_bounded_estimator_forms() {
  assert_eq!(
    parse_command("drdid y, treat(d) post(t)").unwrap(),
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y".to_owned(),
        covariates: vec![],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        method: DrDidMethod::Aipw,
        robust: false,
        bootstrap: None,
        seed: None,
      },
    }
  );
  assert_eq!(
    parse_command("drdid y x1 x2, treat(d) post(t)").unwrap(),
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y".to_owned(),
        covariates: vec!["x1".to_owned(), "x2".to_owned()],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        method: DrDidMethod::Aipw,
        robust: false,
        bootstrap: None,
        seed: None,
      },
    }
  );
  assert_eq!(
    parse_command("drdid y, treat(d) post(t) method(or)").unwrap(),
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y".to_owned(),
        covariates: vec![],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        method: DrDidMethod::Or,
        robust: false,
        bootstrap: None,
        seed: None,
      },
    }
  );
  assert_eq!(
    parse_command("drdid y, treat(d) post(t) method(ipw)").unwrap(),
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y".to_owned(),
        covariates: vec![],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        method: DrDidMethod::Ipw,
        robust: false,
        bootstrap: None,
        seed: None,
      },
    }
  );
  assert_eq!(
    parse_command("drdid y, treat(d) post(t) method(aipw)").unwrap(),
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y".to_owned(),
        covariates: vec![],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        method: DrDidMethod::Aipw,
        robust: false,
        bootstrap: None,
        seed: None,
      },
    }
  );
  assert_eq!(
    parse_command("drdid y, treat(d) post(t) method=or").unwrap(),
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y".to_owned(),
        covariates: vec![],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        method: DrDidMethod::Or,
        robust: false,
        bootstrap: None,
        seed: None,
      },
    }
  );
  assert_eq!(
    parse_command("drdid y, treat(d) post(t) robust").unwrap(),
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y".to_owned(),
        covariates: vec![],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        method: DrDidMethod::Aipw,
        robust: true,
        bootstrap: None,
        seed: None,
      },
    }
  );
  assert_eq!(
    parse_command("drdid y, treat(d) post(t) bootstrap(100)").unwrap(),
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y".to_owned(),
        covariates: vec![],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        method: DrDidMethod::Aipw,
        robust: false,
        bootstrap: Some(100),
        seed: None,
      },
    }
  );
  assert_eq!(
    parse_command("drdid y, treat(d) post(t) bootstrap(100) seed(42)").unwrap(),
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y".to_owned(),
        covariates: vec![],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        method: DrDidMethod::Aipw,
        robust: false,
        bootstrap: Some(100),
        seed: Some(42),
      },
    }
  );
  assert_eq!(
    parse_command("drdid `y var` `x var`, treat(`d var`) post(`t var`)").unwrap(),
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y var".to_owned(),
        covariates: vec!["x var".to_owned()],
        treatment_variable: "d var".to_owned(),
        post_variable: "t var".to_owned(),
        method: DrDidMethod::Aipw,
        robust: false,
        bootstrap: None,
        seed: None,
      },
    }
  );
  assert_eq!(
    parse_command("drdid \"y var\" \"x var\", treat(d) post(t)").unwrap(),
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y var".to_owned(),
        covariates: vec!["x var".to_owned()],
        treatment_variable: "d".to_owned(),
        post_variable: "t".to_owned(),
        method: DrDidMethod::Aipw,
        robust: false,
        bootstrap: None,
        seed: None,
      },
    }
  );
}

#[test]
fn drdid_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "drdid",
      "drdid expects syntax: drdid <y> [covariates], treat(<var>) post(<var>) [method(or|ipw|aipw) robust bootstrap(<n>) seed(<n>)]",
    ),
    (
      "drdid if x > 0, treat(d) post(t)",
      "drdid expects syntax: drdid <y> [covariates], treat(<var>) post(<var>) [method(or|ipw|aipw) robust bootstrap(<n>) seed(<n>)]",
    ),
    (
      "drdid = 1, treat(d) post(t)",
      "drdid assignment requires a target before =",
    ),
    (
      "drdid y = 1, treat(d) post(t)",
      "drdid expects syntax: drdid <y> [covariates], treat(<var>) post(<var>) [method(or|ipw|aipw) robust bootstrap(<n>) seed(<n>)]",
    ),
    (
      "drdid y if x > 0, treat(d) post(t)",
      "drdid expects syntax: drdid <y> [covariates], treat(<var>) post(<var>) [method(or|ipw|aipw) robust bootstrap(<n>) seed(<n>)]",
    ),
    ("drdid y", "drdid option treat expects one variable"),
    ("drdid y,", "comma must be followed by at least one option"),
    ("drdid y, treat", "drdid option treat expects variables"),
    (
      "drdid y, treat()",
      "option treat expects at least one value",
    ),
    (
      "drdid y, treat(d1 d2) post(t)",
      "drdid option treat expects one variable",
    ),
    (
      "drdid y, treat(d) treat(d2) post(t)",
      "drdid option treat may only be supplied once",
    ),
    (
      "drdid y, treat(d)",
      "drdid option post expects one variable",
    ),
    (
      "drdid y, treat(d) post",
      "drdid option post expects variables",
    ),
    (
      "drdid y, treat(d) post()",
      "option post expects at least one value",
    ),
    (
      "drdid y, treat(d) post(t1 t2)",
      "drdid option post expects one variable",
    ),
    (
      "drdid y, treat(d) post(t) post(t2)",
      "drdid option post may only be supplied once",
    ),
    (
      "drdid y, treat(d) post(t) method(bad)",
      "drdid option method must be one of: or, ipw, aipw",
    ),
    (
      "drdid y, treat(d) post(t) method",
      "drdid option method expects a value",
    ),
    (
      "drdid y, treat(d) post(t) method()",
      "option method expects at least one value",
    ),
    (
      "drdid y, treat(d) post(t) method(or ipw)",
      "drdid option method expects one value",
    ),
    (
      "drdid y, treat(d) post(t) method(or) method(ipw)",
      "drdid option method may only be supplied once",
    ),
    (
      "drdid y, treat(d) post(t) bootstrap(0)",
      "drdid option bootstrap must be at least 1",
    ),
    (
      "drdid y, treat(d) post(t) bootstrap(-1)",
      "drdid option bootstrap must be at least 1",
    ),
    (
      "drdid y, treat(d) post(t) bootstrap(abc)",
      "option bootstrap expects a numeric value",
    ),
    (
      "drdid y, treat(d) post(t) bootstrap",
      "drdid option bootstrap expects an integer value",
    ),
    (
      "drdid y, treat(d) post(t) bootstrap()",
      "option bootstrap expects at least one value",
    ),
    (
      "drdid y, treat(d) post(t) bootstrap(10) bootstrap(20)",
      "drdid option bootstrap may only be supplied once",
    ),
    (
      "drdid y, treat(d) post(t) seed(42)",
      "drdid option seed requires option bootstrap",
    ),
    (
      "drdid y, treat(d) post(t) bootstrap(10) seed(-1)",
      "drdid option seed must be at least 0",
    ),
    (
      "drdid y, treat(d) post(t) bootstrap(10) seed(abc)",
      "option seed expects a numeric value",
    ),
    (
      "drdid y, treat(d) post(t) bootstrap(10) seed",
      "drdid option seed expects an integer value",
    ),
    (
      "drdid y, treat(d) post(t) bootstrap(10) seed()",
      "option seed expects at least one value",
    ),
    (
      "drdid y, treat(d) post(t) bootstrap(10) seed(10) seed(20)",
      "drdid option seed may only be supplied once",
    ),
    (
      "drdid y, treat(d) post(t) robust=1",
      "drdid option robust does not accept a value",
    ),
    (
      "drdid y, treat(d) post(t) robust(1)",
      "option robust values must be identifiers",
    ),
    (
      "drdid y, treat(d) post(t) robust(foo)",
      "drdid option robust does not accept a value",
    ),
    (
      "drdid y, treat(d) post(t) extra",
      "drdid unsupported option: extra",
    ),
    (
      "drdid y, TREAT(d) POST(t)",
      "drdid unsupported option: POST, TREAT",
    ),
    (
      "drdid y, treat(d) post(d)",
      "drdid treatment and post variables must be distinct",
    ),
    (
      "drdid y, treat(y) post(t)",
      "drdid treatment and post variables must differ from outcome",
    ),
    (
      "drdid y, treat(d) post(y)",
      "drdid treatment and post variables must differ from outcome",
    ),
    (
      "drdid y d, treat(d) post(t)",
      "drdid treatment and post variables must not appear in covariates",
    ),
    (
      "drdid y t, treat(d) post(t)",
      "drdid treatment and post variables must not appear in covariates",
    ),
    ("drdid=", "drdid assignment requires a target before ="),
    ("drdid = 1", "drdid assignment requires a target before ="),
    ("drdid==", "unsupported token in command: =="),
    ("drdid == 1", "unsupported token in command: =="),
    ("drdid:", "unsupported token in command: :"),
    (
      "drdid: y, treat(d) post(t)",
      "unsupported token in command: :",
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
fn dml_parses_supported_options_and_controls() {
  assert_eq!(
    parse_command("dml linear y x1 x2, treat(d)").unwrap(),
    Command::Dml {
      command: DmlCommand {
        outcome: "y".to_owned(),
        controls: vec!["x1".to_owned(), "x2".to_owned()],
        treatment_variable: "d".to_owned(),
        folds: 5,
        alpha: "1.0".to_owned(),
        robust: false,
        seed: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("dml linear y x1, treat(d) folds(3)").unwrap(),
    Command::Dml {
      command: DmlCommand {
        outcome: "y".to_owned(),
        controls: vec!["x1".to_owned()],
        treatment_variable: "d".to_owned(),
        folds: 3,
        alpha: "1.0".to_owned(),
        robust: false,
        seed: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("dml linear y x1 x2, treat(d) alpha(0.5)").unwrap(),
    Command::Dml {
      command: DmlCommand {
        outcome: "y".to_owned(),
        controls: vec!["x1".to_owned(), "x2".to_owned()],
        treatment_variable: "d".to_owned(),
        folds: 5,
        alpha: "0.5".to_owned(),
        robust: false,
        seed: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("dml linear y x1, treat(d) robust").unwrap(),
    Command::Dml {
      command: DmlCommand {
        outcome: "y".to_owned(),
        controls: vec!["x1".to_owned()],
        treatment_variable: "d".to_owned(),
        folds: 5,
        alpha: "1.0".to_owned(),
        robust: true,
        seed: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("dml linear y x1, treat(d) seed(42)").unwrap(),
    Command::Dml {
      command: DmlCommand {
        outcome: "y".to_owned(),
        controls: vec!["x1".to_owned()],
        treatment_variable: "d".to_owned(),
        folds: 5,
        alpha: "1.0".to_owned(),
        robust: false,
        seed: Some(42),
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("dml linear y x1, treat(d) noconstant").unwrap(),
    Command::Dml {
      command: DmlCommand {
        outcome: "y".to_owned(),
        controls: vec!["x1".to_owned()],
        treatment_variable: "d".to_owned(),
        folds: 5,
        alpha: "1.0".to_owned(),
        robust: false,
        seed: None,
        include_intercept: false,
      },
    }
  );
  assert_eq!(
    parse_command("dml LINEAR y x1 x2, treat(d) folds(10) alpha(0.1) robust seed(123) noconstant")
      .unwrap(),
    Command::Dml {
      command: DmlCommand {
        outcome: "y".to_owned(),
        controls: vec!["x1".to_owned(), "x2".to_owned()],
        treatment_variable: "d".to_owned(),
        folds: 10,
        alpha: "0.1".to_owned(),
        robust: true,
        seed: Some(123),
        include_intercept: false,
      },
    }
  );
  assert_eq!(
    parse_command("dml linear `y var` `x var`, treat(`d var`)").unwrap(),
    Command::Dml {
      command: DmlCommand {
        outcome: "y var".to_owned(),
        controls: vec!["x var".to_owned()],
        treatment_variable: "d var".to_owned(),
        folds: 5,
        alpha: "1.0".to_owned(),
        robust: false,
        seed: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("dml linear \"y var\" \"x var\", treat(d)").unwrap(),
    Command::Dml {
      command: DmlCommand {
        outcome: "y var".to_owned(),
        controls: vec!["x var".to_owned()],
        treatment_variable: "d".to_owned(),
        folds: 5,
        alpha: "1.0".to_owned(),
        robust: false,
        seed: None,
        include_intercept: true,
      },
    }
  );
}

#[test]
fn dml_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "dml",
      "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]",
    ),
    (
      "dml linear",
      "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]",
    ),
    (
      "dml linear y",
      "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]",
    ),
    (
      "dml linear y, treat(d)",
      "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]",
    ),
    (
      "dml linear y x if y > 0, treat(d)",
      "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]",
    ),
    ("dml logistic y x, treat(d)", "dml model must be linear"),
    ("dml `linear` y x, treat(d)", "dml model must be linear"),
    (
      "dml linear y x = 1, treat(d)",
      "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]",
    ),
    (
      "dml linear y x == 1, treat(d)",
      "unsupported token in command: ==",
    ),
    (
      "dml linear y x,",
      "comma must be followed by at least one option",
    ),
    ("dml linear y x", "dml option treat expects one variable"),
    (
      "dml linear y x, treat",
      "dml option treat expects variables",
    ),
    (
      "dml linear y x, treat()",
      "option treat expects at least one value",
    ),
    (
      "dml linear y x, treat(d1 d2)",
      "dml option treat expects one variable",
    ),
    (
      "dml linear y x, treat(d) treat(d2)",
      "dml option treat may only be supplied once",
    ),
    (
      "dml linear y x, treat(y)",
      "dml treatment variable must differ from outcome",
    ),
    (
      "dml linear y x, treat(x)",
      "dml treatment variable must not appear in controls",
    ),
    (
      "dml linear y x, treat(d) folds(1)",
      "dml option folds must be at least 2",
    ),
    (
      "dml linear y x, treat(d) folds(0)",
      "dml option folds must be at least 2",
    ),
    (
      "dml linear y x, treat(d) folds(-1)",
      "dml option folds must be at least 2",
    ),
    (
      "dml linear y x, treat(d) folds(1.5)",
      "dml option folds expects an integer value",
    ),
    (
      "dml linear y x, treat(d) folds(abc)",
      "option folds expects a numeric value",
    ),
    (
      "dml linear y x, treat(d) folds",
      "dml option folds expects an integer value",
    ),
    (
      "dml linear y x, treat(d) folds()",
      "option folds expects at least one value",
    ),
    (
      "dml linear y x, treat(d) folds(5) folds(10)",
      "dml option folds may only be supplied once",
    ),
    (
      "dml linear y x, treat(d) alpha(-1)",
      "dml option alpha must be positive",
    ),
    (
      "dml linear y x, treat(d) alpha(0)",
      "dml option alpha must be positive",
    ),
    (
      "dml linear y x, treat(d) alpha(abc)",
      "option alpha expects a numeric value",
    ),
    (
      "dml linear y x, treat(d) alpha",
      "dml option alpha expects a numeric value",
    ),
    (
      "dml linear y x, treat(d) alpha()",
      "option alpha expects at least one value",
    ),
    (
      "dml linear y x, treat(d) alpha(1) alpha(2)",
      "dml option alpha may only be supplied once",
    ),
    (
      "dml linear y x, treat(d) seed(-1)",
      "dml option seed must be at least 0",
    ),
    (
      "dml linear y x, treat(d) seed(1.5)",
      "dml option seed expects an integer value",
    ),
    (
      "dml linear y x, treat(d) seed(abc)",
      "option seed expects a numeric value",
    ),
    (
      "dml linear y x, treat(d) seed",
      "dml option seed expects an integer value",
    ),
    (
      "dml linear y x, treat(d) seed()",
      "option seed expects at least one value",
    ),
    (
      "dml linear y x, treat(d) seed(1) seed(2)",
      "dml option seed may only be supplied once",
    ),
    (
      "dml linear y x, treat(d) robust=true",
      "dml option robust does not accept a value",
    ),
    (
      "dml linear y x, treat(d) robust(foo)",
      "dml option robust does not accept a value",
    ),
    (
      "dml linear y x, treat(d) noconstant=true",
      "dml option noconstant does not accept a value",
    ),
    (
      "dml linear y x, treat(d) noconstant(foo)",
      "dml option noconstant does not accept a value",
    ),
    (
      "dml linear y x, treat(d) extra",
      "dml unsupported option: extra",
    ),
    (
      "dml linear y x, treat(d) extra2 extra1",
      "dml unsupported option: extra1, extra2",
    ),
    ("dml linear y x, TREAT(d)", "dml unsupported option: TREAT"),
    ("dml=", "dml assignment requires a target before ="),
    ("dml = 1", "dml assignment requires a target before ="),
    ("dml==", "unsupported token in command: =="),
    ("dml == 1", "unsupported token in command: =="),
    ("dml:", "unsupported token in command: :"),
    (
      "dml: linear y x, treat(d)",
      "unsupported token in command: :",
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
fn cfregress_parses_supported_options_and_controls() {
  assert_eq!(
    parse_command("cfregress cost age bmi, endog(hours) iv(distance policy)").unwrap(),
    Command::CfRegress {
      command: CfRegressCommand {
        outcome: "cost".to_owned(),
        exogenous: vec!["age".to_owned(), "bmi".to_owned()],
        endogenous: "hours".to_owned(),
        instruments: vec!["distance".to_owned(), "policy".to_owned()],
        robust: false,
        cluster_variable: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("cfregress cost, endog(hours) iv(distance) robust").unwrap(),
    Command::CfRegress {
      command: CfRegressCommand {
        outcome: "cost".to_owned(),
        exogenous: vec![],
        endogenous: "hours".to_owned(),
        instruments: vec!["distance".to_owned()],
        robust: true,
        cluster_variable: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("cfregress cost age, endog(hours) iv(distance) cluster(group_id) noconstant")
      .unwrap(),
    Command::CfRegress {
      command: CfRegressCommand {
        outcome: "cost".to_owned(),
        exogenous: vec!["age".to_owned()],
        endogenous: "hours".to_owned(),
        instruments: vec!["distance".to_owned()],
        robust: false,
        cluster_variable: Some("group_id".to_owned()),
        include_intercept: false,
      },
    }
  );
  assert_eq!(
    parse_command("cfregress `cost var` `age var`, endog(`hours var`) iv(`dist var`)").unwrap(),
    Command::CfRegress {
      command: CfRegressCommand {
        outcome: "cost var".to_owned(),
        exogenous: vec!["age var".to_owned()],
        endogenous: "hours var".to_owned(),
        instruments: vec!["dist var".to_owned()],
        robust: false,
        cluster_variable: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("cfregress \"cost var\" \"age var\", endog(hours) iv(distance)").unwrap(),
    Command::CfRegress {
      command: CfRegressCommand {
        outcome: "cost var".to_owned(),
        exogenous: vec!["age var".to_owned()],
        endogenous: "hours".to_owned(),
        instruments: vec!["distance".to_owned()],
        robust: false,
        cluster_variable: None,
        include_intercept: true,
      },
    }
  );
}

#[test]
fn cfregress_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "cfregress",
      "cfregress expects syntax: cfregress <y> [exog_vars], endog(<var>) iv(<vars>)",
    ),
    (
      "cfregress if x > 0, endog(d) iv(z)",
      "cfregress expects syntax: cfregress <y> [exog_vars], endog(<var>) iv(<vars>)",
    ),
    (
      "cfregress y = 1, endog(d) iv(z)",
      "cfregress expects syntax: cfregress <y> [exog_vars], endog(<var>) iv(<vars>)",
    ),
    (
      "cfregress y == 1, endog(d) iv(z)",
      "unsupported token in command: ==",
    ),
    (
      "cfregress y,",
      "comma must be followed by at least one option",
    ),
    ("cfregress y", "cfregress option endog expects one variable"),
    (
      "cfregress y, endog(d)",
      "cfregress option iv expects at least one variable",
    ),
    (
      "cfregress y, iv(z)",
      "cfregress option endog expects one variable",
    ),
    (
      "cfregress y, endog() iv(z)",
      "option endog expects at least one value",
    ),
    (
      "cfregress y, endog(d1 d2) iv(z)",
      "cfregress option endog expects one variable",
    ),
    (
      "cfregress y, endog iv(z)",
      "cfregress option endog expects variables",
    ),
    (
      "cfregress y, endog(d) iv()",
      "option iv expects at least one value",
    ),
    (
      "cfregress y, endog(d) iv",
      "cfregress option iv expects variables",
    ),
    (
      "cfregress y, endog(d) iv(z) cluster",
      "cfregress option cluster expects variables",
    ),
    (
      "cfregress y, endog(d) iv(z) cluster()",
      "option cluster expects at least one value",
    ),
    (
      "cfregress y, endog(d) iv(z) cluster(c1 c2)",
      "cfregress option cluster expects one variable",
    ),
    (
      "cfregress y, endog(d) iv(z) robust cluster(c)",
      "cfregress cannot combine robust and cluster",
    ),
    (
      "cfregress y, endog(d) endog(d2) iv(z)",
      "cfregress option endog may only be supplied once",
    ),
    (
      "cfregress y, endog(d) iv(z) iv(z2)",
      "cfregress option iv may only be supplied once",
    ),
    (
      "cfregress y, endog(d) iv(z) cluster(c1) cluster(c2)",
      "cfregress option cluster may only be supplied once",
    ),
    (
      "cfregress y d, endog(d) iv(z)",
      "cfregress endog variable must not appear in exogenous variables",
    ),
    (
      "cfregress y, endog(d) iv(z) robust=true",
      "cfregress option robust does not accept a value",
    ),
    (
      "cfregress y, endog(d) iv(z) robust(foo)",
      "cfregress option robust does not accept a value",
    ),
    (
      "cfregress y, endog(d) iv(z) noconstant=true",
      "cfregress option noconstant does not accept a value",
    ),
    (
      "cfregress y, endog(d) iv(z) noconstant(foo)",
      "cfregress option noconstant does not accept a value",
    ),
    (
      "cfregress y, endog(d) iv(z) extra",
      "cfregress unsupported option: extra",
    ),
    (
      "cfregress y, endog(d) iv(z) extra2 extra1",
      "cfregress unsupported option: extra1, extra2",
    ),
    (
      "cfregress y, ENDOG(d) IV(z)",
      "cfregress unsupported option: ENDOG, IV",
    ),
    (
      "cfregress=",
      "cfregress assignment requires a target before =",
    ),
    (
      "cfregress = 1",
      "cfregress assignment requires a target before =",
    ),
    ("cfregress==", "unsupported token in command: =="),
    ("cfregress == 1", "unsupported token in command: =="),
    ("cfregress:", "unsupported token in command: :"),
    (
      "cfregress: y, endog(d) iv(z)",
      "unsupported token in command: :",
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
fn lincom_parses_supported_linear_expressions() {
  assert_eq!(
    parse_command("lincom x1 - x2").unwrap(),
    Command::Lincom {
      command: LincomCommand {
        expression: GenerateExpression::Binary {
          left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
          operator: GenerateBinaryOperator::Subtract,
          right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
        },
      },
    }
  );

  assert_eq!(
    parse_command("lincom x1 + 2 * x2").unwrap(),
    Command::Lincom {
      command: LincomCommand {
        expression: GenerateExpression::Binary {
          left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
          operator: GenerateBinaryOperator::Add,
          right: Box::new(GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Number("2".to_owned())),
            operator: GenerateBinaryOperator::Multiply,
            right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
          }),
        },
      },
    }
  );

  assert_eq!(
    parse_command("lincom (x1 + x2) * 3").unwrap(),
    Command::Lincom {
      command: LincomCommand {
        expression: GenerateExpression::Binary {
          left: Box::new(GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
            operator: GenerateBinaryOperator::Add,
            right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
          }),
          operator: GenerateBinaryOperator::Multiply,
          right: Box::new(GenerateExpression::Number("3".to_owned())),
        },
      },
    }
  );

  assert_eq!(
    parse_command("lincom `wage rate` + 2 * `hours worked`").unwrap(),
    Command::Lincom {
      command: LincomCommand {
        expression: GenerateExpression::Binary {
          left: Box::new(GenerateExpression::Identifier("wage rate".to_owned())),
          operator: GenerateBinaryOperator::Add,
          right: Box::new(GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Number("2".to_owned())),
            operator: GenerateBinaryOperator::Multiply,
            right: Box::new(GenerateExpression::Identifier("hours worked".to_owned())),
          }),
        },
      },
    }
  );
}

#[test]
fn lincom_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "lincom",
      "lincom command expects a linear combination expression",
    ),
    (
      "lincom   ",
      "lincom command expects a linear combination expression",
    ),
    ("lincom:", "unsupported token in command: :"),
    ("lincom: x1 + x2", "unsupported token in command: :"),
    ("lincom;", "unknown command: lincom;"),
    ("lincom x1;", "unsupported token in command: ;"),
    ("lincom=", "lincom assignment requires a target before ="),
    ("lincom=1", "lincom assignment requires a target before ="),
    ("lincom==", "unsupported token in command: =="),
    ("lincom==1", "unsupported token in command: =="),
    ("lincom,", "comma must be followed by at least one option"),
    ("lincom, level(95)", "unknown command: lincom"),
    ("lincom ,", "unsupported token in expression: ,"),
    ("lincom , level(95)", "unsupported token in expression: ,"),
    ("lincom = 1", "unsupported token in expression: ="),
    ("lincom == 1", "unsupported token in expression: =="),
    ("lincom x1 +", "incomplete expression after +"),
    ("lincom x1 *", "incomplete expression after *"),
    ("lincom (x1 + x2", "missing closing ) in expression"),
    ("lincom x1 + x2)", "unsupported token in expression: )"),
    ("lincom x1 + + x2", "unsupported token in expression: +"),
    ("lincom x1.x2", "unsupported token in expression: ."),
    ("lincom x1[0]", "unsupported token in command: ["),
    ("lincom x1 if x2 > 0", "unsupported token in expression: if"),
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
fn estat_parses_bounded_diagnostic_subcommands() {
  assert_eq!(
    parse_command("estat firststage").unwrap(),
    Command::Estat {
      command: EstatCommand {
        subcommand: EstatSubcommand::FirstStage,
      },
    }
  );
  assert_eq!(
    parse_command("ESTAT OVERID").unwrap(),
    Command::Estat {
      command: EstatCommand {
        subcommand: EstatSubcommand::Overid,
      },
    }
  );
  assert_eq!(
    parse_command("estat 'endogenous'").unwrap(),
    Command::Estat {
      command: EstatCommand {
        subcommand: EstatSubcommand::Endogenous,
      },
    }
  );
  assert_eq!(
    parse_command("estat \"hausman\"").unwrap(),
    Command::Estat {
      command: EstatCommand {
        subcommand: EstatSubcommand::Hausman,
      },
    }
  );
}

#[test]
fn estat_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "estat",
      "estat expects syntax: estat <residuals|ovtest|vif|firststage|overid|hausman|endogenous|margins|gof|did|drdid|dml|bayes|spatial|report>",
    ),
    (
      "estat firststage extra",
      "estat expects syntax: estat <residuals|ovtest|vif|firststage|overid|hausman|endogenous|margins|gof|did|drdid|dml|bayes|spatial|report>",
    ),
    (
      "estat, firststage",
      "estat expects syntax: estat <residuals|ovtest|vif|firststage|overid|hausman|endogenous|margins|gof|did|drdid|dml|bayes|spatial|report>",
    ),
    (
      "estat detail",
      "estat subcommand must be residuals, ovtest, vif, firststage, overid, hausman, endogenous, margins, gof, did, drdid, dml, bayes, spatial, or report",
    ),
    (
      "estat firststage, robust",
      "estat firststage does not support options",
    ),
    ("estat firststage if", "missing expression after if"),
    (
      "estat firststage=",
      "estat assignment requires an expression after =",
    ),
    ("estat firststage -x", "unsupported token in command: -"),
    (
      "estat `firststage`",
      "estat subcommand must be residuals, ovtest, vif, firststage, overid, hausman, endogenous, margins, gof, did, drdid, dml, bayes, spatial, or report",
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

#[test]
fn regress_parses_bounded_estimator_forms() {
  assert_eq!(
    parse_command("regress cost age bmi").unwrap(),
    Command::Regress {
      command: RegressCommand {
        outcome: "cost".to_owned(),
        predictors: vec!["age".to_owned(), "bmi".to_owned()],
        estimator: RegressEstimator::Ols,
        weight_variable: None,
        robust: false,
        cluster_variable: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("regress cost age, robust").unwrap(),
    Command::Regress {
      command: RegressCommand {
        outcome: "cost".to_owned(),
        predictors: vec!["age".to_owned()],
        estimator: RegressEstimator::Ols,
        weight_variable: None,
        robust: true,
        cluster_variable: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("regress cost age, cluster(sex)").unwrap(),
    Command::Regress {
      command: RegressCommand {
        outcome: "cost".to_owned(),
        predictors: vec!["age".to_owned()],
        estimator: RegressEstimator::Ols,
        weight_variable: None,
        robust: false,
        cluster_variable: Some("sex".to_owned()),
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("regress cost age, noconstant").unwrap(),
    Command::Regress {
      command: RegressCommand {
        outcome: "cost".to_owned(),
        predictors: vec!["age".to_owned()],
        estimator: RegressEstimator::Ols,
        weight_variable: None,
        robust: false,
        cluster_variable: None,
        include_intercept: false,
      },
    }
  );
  assert_eq!(
    parse_command("regress cost age, wls(weight) cluster(firm)").unwrap(),
    Command::Regress {
      command: RegressCommand {
        outcome: "cost".to_owned(),
        predictors: vec!["age".to_owned()],
        estimator: RegressEstimator::Wls,
        weight_variable: Some("weight".to_owned()),
        robust: false,
        cluster_variable: Some("firm".to_owned()),
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("regress cost age, gls(sigma) robust").unwrap(),
    Command::Regress {
      command: RegressCommand {
        outcome: "cost".to_owned(),
        predictors: vec!["age".to_owned()],
        estimator: RegressEstimator::Gls,
        weight_variable: Some("sigma".to_owned()),
        robust: true,
        cluster_variable: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("REGRESS `total cost` `age value` 'bmi value', noconstant robust").unwrap(),
    Command::Regress {
      command: RegressCommand {
        outcome: "total cost".to_owned(),
        predictors: vec!["age value".to_owned(), "bmi value".to_owned()],
        estimator: RegressEstimator::Ols,
        weight_variable: None,
        robust: true,
        cluster_variable: None,
        include_intercept: false,
      },
    }
  );
}

#[test]
fn regress_preserves_bounded_parser_diagnostics() {
  let cases = [
    ("regress", "regress expects syntax: regress <y> <xvars>"),
    (
      "regress cost",
      "regress expects syntax: regress <y> <xvars>",
    ),
    (
      "regress cost age if age > 18",
      "regress expects syntax: regress <y> <xvars>",
    ),
    (
      "regress cost age, robust cluster(sex)",
      "regress cannot combine robust and cluster",
    ),
    (
      "regress cost age, cluster",
      "regress option cluster expects variables",
    ),
    (
      "regress cost age, cluster()",
      "option cluster expects at least one value",
    ),
    (
      "regress cost age, cluster(sex firm)",
      "regress option cluster expects one variable",
    ),
    (
      "regress cost age, cluster(a) cluster(b)",
      "regress option cluster may only be supplied once",
    ),
    (
      "regress cost age, wls",
      "regress option wls expects variables",
    ),
    (
      "regress cost age, wls()",
      "option wls expects at least one value",
    ),
    (
      "regress cost age, wls(age bmi)",
      "regress option wls expects one variable",
    ),
    (
      "regress cost age, wls(a) wls(b)",
      "regress option wls may only be supplied once",
    ),
    (
      "regress cost age, gls",
      "regress option gls expects variables",
    ),
    (
      "regress cost age, gls()",
      "option gls expects at least one value",
    ),
    (
      "regress cost age, gls(age bmi)",
      "regress option gls expects one variable",
    ),
    (
      "regress cost age, gls(a) gls(b)",
      "regress option gls may only be supplied once",
    ),
    (
      "regress cost age, wls(age) gls(sigma)",
      "regress cannot combine wls and gls",
    ),
    (
      "regress cost age, robust=true",
      "regress option robust does not accept a value",
    ),
    (
      "regress cost age, noconstant=true",
      "regress option noconstant does not accept a value",
    ),
    (
      "regress cost age, invalid",
      "regress unsupported option: invalid",
    ),
    (
      "regress cost age,",
      "comma must be followed by at least one option",
    ),
    ("regress,", "comma must be followed by at least one option"),
    ("regress=", "regress assignment requires a target before ="),
    ("regress==", "unsupported token in command: =="),
    ("regress:cost age", "unsupported token in command: :"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().to_string(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn logit_and_probit_parse_bounded_estimator_forms() {
  assert_eq!(
    parse_command("logit outcome x1 x2").unwrap(),
    Command::Logit {
      command: LogitCommand {
        outcome: "outcome".to_owned(),
        predictors: vec!["x1".to_owned(), "x2".to_owned()],
        robust: false,
        cluster_variable: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("logit outcome x1, robust").unwrap(),
    Command::Logit {
      command: LogitCommand {
        outcome: "outcome".to_owned(),
        predictors: vec!["x1".to_owned()],
        robust: true,
        cluster_variable: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("LOGIT outcome x1, cluster(group_id)").unwrap(),
    Command::Logit {
      command: LogitCommand {
        outcome: "outcome".to_owned(),
        predictors: vec!["x1".to_owned()],
        robust: false,
        cluster_variable: Some("group_id".to_owned()),
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("logit outcome x1, noconstant").unwrap(),
    Command::Logit {
      command: LogitCommand {
        outcome: "outcome".to_owned(),
        predictors: vec!["x1".to_owned()],
        robust: false,
        cluster_variable: None,
        include_intercept: false,
      },
    }
  );

  assert_eq!(
    parse_command("probit outcome x1 x2").unwrap(),
    Command::Probit {
      command: ProbitCommand {
        outcome: "outcome".to_owned(),
        predictors: vec!["x1".to_owned(), "x2".to_owned()],
        robust: false,
        cluster_variable: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("probit outcome x1, robust").unwrap(),
    Command::Probit {
      command: ProbitCommand {
        outcome: "outcome".to_owned(),
        predictors: vec!["x1".to_owned()],
        robust: true,
        cluster_variable: None,
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("PROBIT outcome x1, cluster(group_id)").unwrap(),
    Command::Probit {
      command: ProbitCommand {
        outcome: "outcome".to_owned(),
        predictors: vec!["x1".to_owned()],
        robust: false,
        cluster_variable: Some("group_id".to_owned()),
        include_intercept: true,
      },
    }
  );
  assert_eq!(
    parse_command("probit outcome x1, noconstant").unwrap(),
    Command::Probit {
      command: ProbitCommand {
        outcome: "outcome".to_owned(),
        predictors: vec!["x1".to_owned()],
        robust: false,
        cluster_variable: None,
        include_intercept: false,
      },
    }
  );
}

#[test]
fn logit_and_probit_preserve_bounded_parser_diagnostics() {
  let cases = [
    ("logit", "logit expects syntax: logit <y> <xvars>"),
    ("logit y", "logit expects syntax: logit <y> <xvars>"),
    (
      "logit y x if y == 1",
      "logit expects syntax: logit <y> <xvars>",
    ),
    (
      "logit y x, robust cluster(group)",
      "logit cannot combine robust and cluster",
    ),
    (
      "logit y x, cluster",
      "logit option cluster expects variables",
    ),
    (
      "logit y x, cluster()",
      "option cluster expects at least one value",
    ),
    (
      "logit y x, cluster(group firm)",
      "logit option cluster expects one variable",
    ),
    (
      "logit y x, cluster(a) cluster(b)",
      "logit option cluster may only be supplied once",
    ),
    (
      "logit y x, robust=true",
      "logit option robust does not accept a value",
    ),
    (
      "logit y x, noconstant=true",
      "logit option noconstant does not accept a value",
    ),
    ("logit y x, invalid", "logit unsupported option: invalid"),
    (
      "logit y x,",
      "comma must be followed by at least one option",
    ),
    ("logit,", "comma must be followed by at least one option"),
    ("logit=", "logit assignment requires a target before ="),
    ("logit==", "unsupported token in command: =="),
    ("logit:y x", "unsupported token in command: :"),
    ("probit", "probit expects syntax: probit <y> <xvars>"),
    ("probit y", "probit expects syntax: probit <y> <xvars>"),
    (
      "probit y x if y == 1",
      "probit expects syntax: probit <y> <xvars>",
    ),
    (
      "probit y x, robust cluster(group)",
      "probit cannot combine robust and cluster",
    ),
    (
      "probit y x, cluster",
      "probit option cluster expects variables",
    ),
    (
      "probit y x, cluster()",
      "option cluster expects at least one value",
    ),
    (
      "probit y x, cluster(group firm)",
      "probit option cluster expects one variable",
    ),
    (
      "probit y x, cluster(a) cluster(b)",
      "probit option cluster may only be supplied once",
    ),
    (
      "probit y x, robust=true",
      "probit option robust does not accept a value",
    ),
    (
      "probit y x, noconstant=true",
      "probit option noconstant does not accept a value",
    ),
    ("probit y x, invalid", "probit unsupported option: invalid"),
    (
      "probit y x,",
      "comma must be followed by at least one option",
    ),
    ("probit,", "comma must be followed by at least one option"),
    ("probit=", "probit assignment requires a target before ="),
    ("probit==", "unsupported token in command: =="),
    ("probit:y x", "unsupported token in command: :"),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().to_string(),
      expected,
      "{input:?}"
    );
  }
}

#[test]
fn bayes_prefix_parses_supported_commands_and_options() {
  assert_eq!(
    parse_command("bayes: regress y x").unwrap(),
    Command::BayesPrefix {
      command: BayesPrefixCommand {
        command: Box::new(Command::Regress {
          command: RegressCommand {
            outcome: "y".to_string(),
            predictors: vec!["x".to_string()],
            estimator: RegressEstimator::Ols,
            weight_variable: None,
            robust: false,
            cluster_variable: None,
            include_intercept: true,
          },
        }),
        draws: None,
        burnin: None,
        chains: None,
        thin: None,
        seed: None,
        priors: Vec::new(),
      },
    }
  );

  let cmd1 = "bayes, draws(500) burnin(200) chains(2) thin(2) seed(123): regress y x";
  assert_eq!(
    parse_command(cmd1).unwrap(),
    Command::BayesPrefix {
      command: BayesPrefixCommand {
        command: Box::new(Command::Regress {
          command: RegressCommand {
            outcome: "y".to_string(),
            predictors: vec!["x".to_string()],
            estimator: RegressEstimator::Ols,
            weight_variable: None,
            robust: false,
            cluster_variable: None,
            include_intercept: true,
          },
        }),
        draws: Some(500),
        burnin: Some(200),
        chains: Some(2),
        thin: Some(2),
        seed: Some(123),
        priors: Vec::new(),
      },
    }
  );

  let cmd_rseed = "bayes, rseed(42): regress y x";
  assert_eq!(
    parse_command(cmd_rseed).unwrap(),
    Command::BayesPrefix {
      command: BayesPrefixCommand {
        command: Box::new(Command::Regress {
          command: RegressCommand {
            outcome: "y".to_string(),
            predictors: vec!["x".to_string()],
            estimator: RegressEstimator::Ols,
            weight_variable: None,
            robust: false,
            cluster_variable: None,
            include_intercept: true,
          },
        }),
        draws: None,
        burnin: None,
        chains: None,
        thin: None,
        seed: Some(42),
        priors: Vec::new(),
      },
    }
  );

  let cmd_tune = "bayes, tune(100): regress y x";
  assert_eq!(
    parse_command(cmd_tune).unwrap(),
    Command::BayesPrefix {
      command: BayesPrefixCommand {
        command: Box::new(Command::Regress {
          command: RegressCommand {
            outcome: "y".to_string(),
            predictors: vec!["x".to_string()],
            estimator: RegressEstimator::Ols,
            weight_variable: None,
            robust: false,
            cluster_variable: None,
            include_intercept: true,
          },
        }),
        draws: None,
        burnin: Some(100),
        chains: None,
        thin: None,
        seed: None,
        priors: Vec::new(),
      },
    }
  );

  let cmd2 = "bayes, prior(x, normal(0, 10)) prior(intercept, uniform(-5, 5)): logit y x";
  assert_eq!(
    parse_command(cmd2).unwrap(),
    Command::BayesPrefix {
      command: BayesPrefixCommand {
        command: Box::new(Command::Logit {
          command: LogitCommand {
            outcome: "y".to_string(),
            predictors: vec!["x".to_string()],
            robust: false,
            cluster_variable: None,
            include_intercept: true,
          },
        }),
        draws: None,
        burnin: None,
        chains: None,
        thin: None,
        seed: None,
        priors: vec![
          ("x".to_string(), "normal(0,10)".to_string()),
          ("intercept".to_string(), "uniform(-5,5)".to_string()),
        ],
      },
    }
  );

  let cmd_noconst = "bayes: regress y x, noconstant";
  assert_eq!(
    parse_command(cmd_noconst).unwrap(),
    Command::BayesPrefix {
      command: BayesPrefixCommand {
        command: Box::new(Command::Regress {
          command: RegressCommand {
            outcome: "y".to_string(),
            predictors: vec!["x".to_string()],
            estimator: RegressEstimator::Ols,
            weight_variable: None,
            robust: false,
            cluster_variable: None,
            include_intercept: false,
          },
        }),
        draws: None,
        burnin: None,
        chains: None,
        thin: None,
        seed: None,
        priors: Vec::new(),
      },
    }
  );

  let cmd_quoted = "bayes, prior(`x:y`, normal): regress `outcome col` `x:y`";
  assert_eq!(
    parse_command(cmd_quoted).unwrap(),
    Command::BayesPrefix {
      command: BayesPrefixCommand {
        command: Box::new(Command::Regress {
          command: RegressCommand {
            outcome: "outcome col".to_string(),
            predictors: vec!["x:y".to_string()],
            estimator: RegressEstimator::Ols,
            weight_variable: None,
            robust: false,
            cluster_variable: None,
            include_intercept: true,
          },
        }),
        draws: None,
        burnin: None,
        chains: None,
        thin: None,
        seed: None,
        priors: vec![("x:y".to_string(), "normal".to_string())],
      },
    }
  );
}

#[test]
fn bayes_prefix_rejects_malformed_syntax_and_unsupported_commands() {
  let cases = [
    (
      "bayes: codebook",
      "bayes prefix only supports regress and logit commands",
    ),
    (
      "bayes: summarize",
      "bayes prefix only supports regress and logit commands",
    ),
    (
      "bayes: probit y x",
      "bayes prefix only supports regress and logit commands",
    ),
    ("bayes:", "bayes expects a command after :"),
    ("bayes: ", "bayes expects a command after :"),
    (
      "bayes, draws(100)",
      "bayes prefix expects syntax: bayes [, options]: command",
    ),
    (
      "bayes draws(100): regress y x",
      "bayes prefix options must start with a comma",
    ),
    (
      "bayes, invalid(1): regress y x",
      "option invalid values must be identifiers",
    ),
    (
      "bayes, invalid: regress y x",
      "unsupported bayes option: invalid",
    ),
    (
      "bayes, draws(abc): regress y x",
      "option draws expects a numeric value",
    ),
    (
      "bayes, draws=abc: regress y x",
      "draws must be a numeric value",
    ),
    (
      "bayes, prior(x): regress y x",
      "prior option expects prior(variable, distribution) syntax",
    ),
    (
      "bayes, prior: regress y x",
      "prior expects (variable, distribution)",
    ),
    (
      "bayes, prior(x, normal): codebook",
      "bayes prefix only supports regress and logit commands",
    ),
    (
      "bayes,",
      "bayes prefix expects syntax: bayes [, options]: command",
    ),
    (
      "bayes, : regress y x",
      "comma must be followed by at least one option",
    ),
    ("bayes", "bayes expects syntax: bayes linear <y> <xvars>"),
    (
      "bayes linear",
      "bayes expects syntax: bayes linear <y> <xvars>",
    ),
    ("bayes = 1", "bayes assignment requires a target before ="),
    ("bayes == 1", "unsupported token in command: =="),
  ];
  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().to_string(),
      expected,
      "{input:?}"
    );
  }
}
