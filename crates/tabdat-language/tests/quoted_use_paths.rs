use tabdat_language::{Command, DataSource, ExecutionMode, LazyEngine, parse_command};

#[test]
fn use_paths_are_literals_with_quote_aware_option_boundaries() {
  for (input, path) in [
    ("use \"data.csv\"", "data.csv"),
    ("use 'data space.csv'", "data space.csv"),
    ("use \"目录/é, data.csv\"", "目录/é, data.csv"),
    ("use \"owner's data.csv\"", "owner's data.csv"),
    ("use 'a\"b.csv'", "a\"b.csv"),
    (r#"use "C:\temp\data space.csv""#, r"C:\temp\data space.csv"),
    ("use \"  data.csv  \"", "  data.csv  "),
    ("use data.csv", "data.csv"),
    (r"use C:\temp\data.csv", r"C:\temp\data.csv"),
  ] {
    assert_eq!(
      parse_command(input).unwrap(),
      Command::Use {
        source: DataSource::LocalPath(path.to_owned()),
        execution_mode: ExecutionMode::Eager,
        lazy_engine: None,
        delimiter: None,
        has_header: None,
      },
      "{input}"
    );
  }
  assert_eq!(
    parse_command("use \"目录/a,b.csv\" , delimiter(\";\") has_header(false)").unwrap(),
    Command::Use {
      source: DataSource::LocalPath("目录/a,b.csv".to_owned()),
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: Some(";".to_owned()),
      has_header: Some(false),
    }
  );
  assert_eq!(
    parse_command("use 's3://bucket/data space.parquet', lazy engine=duckdb").unwrap(),
    Command::Use {
      source: DataSource::Uri("s3://bucket/data space.parquet".to_owned()),
      execution_mode: ExecutionMode::Lazy,
      lazy_engine: Some(LazyEngine::DuckDb),
      delimiter: None,
      has_header: None,
    }
  );
}

#[test]
fn malformed_quoted_paths_fail_before_execution_and_options_keep_their_errors() {
  for input in [
    "use \"\"",
    "use ''",
    "use \"a.csv\" b.csv",
    "use 'a.csv' 'b.csv'",
    "use \"a.csv\"junk",
    "use 'a'\"b.csv\"",
    "use , lazy",
  ] {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      "use expects exactly one path: use <path>",
      "{input}"
    );
  }
  for input in ["use \"data.csv", "use '目录,a.csv", "use \"data.csv'"] {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      "unterminated quoted string",
      "{input}"
    );
  }
  for options in [
    ",",
    ", lazy lazy",
    ", unknown",
    ", has_header(1)",
    ", engine=duckdb",
  ] {
    let bare = parse_command(&format!("use data.csv{options}")).unwrap_err();
    let quoted = parse_command(&format!("use \"data,part.csv\"{options}")).unwrap_err();
    assert_eq!(bare.message(), quoted.message());
  }
}
