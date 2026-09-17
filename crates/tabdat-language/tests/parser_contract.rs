use tabdat_language::{
  Command, DataSource, ExecutionMode, LazyEngine, RowLimit, SettingName, parse_command,
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
