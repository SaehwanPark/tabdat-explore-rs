#![forbid(unsafe_code)]
use serde_json::{Value, json};
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::{ExecutionResult, RuntimeError, Session};
static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-key-guards-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "id,g,x,y\n1,A,1,2\n1,B,2,5\n2,A,3,5\n3,B,4,9\n,A,5,8\n,,6,\n",
    )
    .unwrap();
    fs::write(root.join("lookup.csv"), "id,z\n1,10\n2,20\n3,30\n").unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn session(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", self.0.join("data.csv").display()),
    );
    session
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn execute(session: &mut Session, source: &str) -> ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}
fn value(result: &ExecutionResult) -> Value {
  serde_json::from_str(&tabdat_reporting::json_results::serialize_execution_result(result).unwrap())
    .unwrap()
}
fn lines(bytes: &[u8]) -> Vec<Value> {
  std::str::from_utf8(bytes)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn success(output: &Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  lines(&output.stdout)
}
fn batch(fixture: &Fixture, commands: &[&str]) -> Vec<Value> {
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  success(&fixture.batch(&args))
}
fn duplicate_data(
  variables: Value,
  total: u64,
  groups: u64,
  dup_groups: u64,
  dup_rows: u64,
  extra: u64,
  max: u64,
) -> Value {
  json!({"variables":variables,"total_rows":total,"unique_groups":groups,"duplicate_groups":dup_groups,"duplicate_rows":dup_rows,"extra_rows":extra,"max_copies":max})
}
fn close(actual: &Value, expected: f64) {
  assert!(
    (actual.as_f64().unwrap() - expected).abs() < 1e-10,
    "{actual} != {expected}"
  );
}

#[test]
fn exact_single_multiple_repeated_default_key_counts_and_missok_preserve_state_and_cached_fit() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(&mut session, "sql select * from active into primary");
  execute(&mut session, "label variable x 'Predictor'");
  execute(
    &mut session,
    "label define groups 'A' 'Group A' 'B' 'Group B'",
  );
  execute(&mut session, "label values g groups");
  execute(&mut session, "regress y x");
  let state = session.state();
  let labels = session.active_label_metadata().unwrap().clone();
  let preview = execute(&mut session, "head 10");
  let inference = execute(&mut session, "lincom x");
  for (source, expected) in [
    (
      "duplicates id",
      duplicate_data(json!(["id"]), 6, 4, 2, 4, 2, 2),
    ),
    (
      "duplicates report id id",
      duplicate_data(json!(["id", "id"]), 6, 4, 2, 4, 2, 2),
    ),
    (
      "duplicates g",
      duplicate_data(json!(["g"]), 6, 3, 2, 5, 3, 3),
    ),
    (
      "duplicates g id",
      duplicate_data(json!(["g", "id"]), 6, 6, 0, 0, 0, 1),
    ),
    (
      "duplicates",
      duplicate_data(json!(["id", "g", "x", "y"]), 6, 6, 0, 0, 0, 1),
    ),
    (
      "isid id g, missok",
      json!({"variables":["id","g"],"total_rows":6,"unique_groups":6,"missing_key_rows":2,"missok":true}),
    ),
    (
      "isid x x",
      json!({"variables":["x","x"],"total_rows":6,"unique_groups":6,"missing_key_rows":0,"missok":false}),
    ),
  ] {
    let result = execute(&mut session, source);
    let encoded = value(&result);
    assert_eq!(encoded["schema_version"], 1);
    assert_eq!(encoded["data"], expected);
    assert_eq!(session.state(), state);
    assert_eq!(session.active_label_metadata().unwrap(), &labels);
    assert_eq!(execute(&mut session, "head 10"), preview);
    assert_eq!(execute(&mut session, "lincom x"), inference);
  }
  for (source, expected) in [
    (
      "isid id",
      RuntimeError::IsidSemanticFailure {
        missing_key_rows: 2,
        duplicate_rows: 4,
        duplicate_groups: 2,
        missok: false,
      },
    ),
    (
      "isid id g",
      RuntimeError::IsidSemanticFailure {
        missing_key_rows: 2,
        duplicate_rows: 0,
        duplicate_groups: 0,
        missok: false,
      },
    ),
    (
      "isid id, missok",
      RuntimeError::IsidSemanticFailure {
        missing_key_rows: 2,
        duplicate_rows: 4,
        duplicate_groups: 2,
        missok: true,
      },
    ),
  ] {
    assert_eq!(
      session.execute(parse_command(source).unwrap()).unwrap_err(),
      expected
    );
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "head 10"), preview);
    assert_eq!(execute(&mut session, "lincom x"), inference);
  }
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 2, 3, 4]
  );
  execute(&mut session, "use primary");
  assert_eq!(execute(&mut session, "head 10"), preview);
}

#[test]
fn diagnose_validate_lookup_join_refit_and_persistence_match_nested_stream() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "sql select * from active into primary",
    "regress y x",
    "use lookup.csv",
    "duplicates id",
    "isid id",
    "sql select * from active order by id into lookup",
    "use primary",
    "label variable x 'Predictor'",
    "duplicates id",
    "isid x",
    "join lookup on id",
    "lincom x",
    "head 10",
    "regress y x",
    "predict fitted",
    "head 10",
    "save guarded.parquet, replace",
    "export guarded.csv, replace",
  ];
  let results = batch(&fixture, &commands);
  assert_eq!(results.len(), commands.len());
  assert_eq!(results[4]["result_type"], "DuplicatesResult");
  assert_eq!(results[5]["result_type"], "IsidResult");
  assert_eq!(
    results[4]["data"],
    duplicate_data(json!(["id"]), 3, 3, 0, 0, 0, 1)
  );
  assert_eq!(
    results[5]["data"],
    json!({"variables":["id"],"total_rows":3,"unique_groups":3,"missing_key_rows":0,"missok":false})
  );
  close(&results[12]["data"]["estimate"], 1.6);
  assert_eq!(
    results[13]["data"]["rows"],
    json!([
      [1, "A", 1, 2, 10],
      [1, "B", 2, 5, 10],
      [2, "A", 3, 5, 20],
      [3, "B", 4, 9, 30]
    ])
  );
  // Explicit joined x=1..4,y=[2,5,5,9]: Sxx=5,Sxy=10.5 => slope2.1/intercept0.
  assert_eq!(results[14]["data"]["observation_count"], 4);
  close(&results[14]["data"]["coefficients"][0]["value"], 0.0);
  close(&results[14]["data"]["coefficients"][1]["value"], 2.1);
  for row in results[16]["data"]["rows"].as_array().unwrap() {
    close(&row[5], 2.1 * row[2].as_f64().unwrap());
  }
  for path in ["guarded.csv", "guarded.parquet"] {
    let reloaded = batch(&fixture, &[&format!("use {path}"), "head 10"]);
    assert_eq!(reloaded[1]["data"], results[16]["data"]);
  }
  fs::write(fixture.0.join("child.td"), commands[4..].join("\n")).unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    format!("{}\nrun child.td\n", commands[..4].join("\n")),
  )
  .unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
}

#[test]
fn native_null_nonfinite_empty_quoted_wide_keys_and_internal_aliases_are_not_normalized() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(
    &mut session,
    "sql select * from (values (1,cast('NaN' as double),''),(2,cast('NaN' as double),''),(3,cast('Infinity' as double),''),(4,cast('-Infinity' as double),cast(null as varchar)),(5,cast(null as double),cast(null as varchar))) as s(id,x,g) order by id into special",
  );
  assert_eq!(
    value(&execute(&mut session, "duplicates x"))["data"],
    duplicate_data(json!(["x"]), 5, 4, 1, 2, 1, 2)
  );
  assert!(matches!(
    session.execute(parse_command("isid x, missok").unwrap()),
    Err(RuntimeError::IsidSemanticFailure {
      missing_key_rows: 1,
      duplicate_groups: 1,
      duplicate_rows: 2,
      missok: true
    })
  ));
  execute(&mut session, "drop if id == 2");
  assert_eq!(
    value(&execute(&mut session, "isid x, missok"))["data"],
    json!({"variables":["x"],"total_rows":4,"unique_groups":4,"missing_key_rows":1,"missok":true})
  );
  assert!(matches!(
    session.execute(parse_command("isid x").unwrap()),
    Err(RuntimeError::IsidSemanticFailure {
      missing_key_rows: 1,
      duplicate_groups: 0,
      ..
    })
  ));
  execute(
    &mut session,
    "sql select * from (values (null::bigint,'A'),(null,'A'),(null,'B')) as s(k,g) into repeatednull",
  );
  assert_eq!(
    value(&execute(&mut session, "duplicates k g"))["data"],
    duplicate_data(json!(["k", "g"]), 3, 2, 1, 2, 1, 2)
  );
  assert!(matches!(
    session.execute(parse_command("isid k g, missok").unwrap()),
    Err(RuntimeError::IsidSemanticFailure {
      duplicate_groups: 1,
      duplicate_rows: 2,
      ..
    })
  ));
  execute(
    &mut session,
    "sql select cast(9007199254740992+row_number() over () as bigint) as \"key\"\"名\",row_number() over () as __tabdat_isid_count,row_number() over () as __tabdat_duplicate_count from active into wide",
  );
  assert_eq!(
    value(&execute(&mut session, "isid `key\"名`"))["data"]["unique_groups"],
    3
  );
  let result = value(&execute(
    &mut session,
    "isid `key\"名` __tabdat_isid_count __tabdat_duplicate_count",
  ));
  assert_eq!(result["data"]["unique_groups"], 3);
  assert_eq!(
    result["data"]["variables"],
    json!(["key\"名", "__tabdat_isid_count", "__tabdat_duplicate_count"])
  );
  assert_eq!(
    value(&execute(
      &mut session,
      "duplicates `key\"名` __tabdat_duplicate_count"
    ))["data"]["duplicate_groups"],
    0
  );
  execute(
    &mut session,
    "sql select * from active where false into empty",
  );
  assert_eq!(
    value(&execute(&mut session, "duplicates"))["data"],
    duplicate_data(
      json!(["key\"名", "__tabdat_isid_count", "__tabdat_duplicate_count"]),
      0,
      0,
      0,
      0,
      0,
      0
    )
  );
  assert_eq!(
    value(&execute(&mut session, "isid `key\"名`"))["data"],
    json!({"variables":["key\"名"],"total_rows":0,"unique_groups":0,"missing_key_rows":0,"missok":false})
  );
}

#[test]
fn validation_and_semantic_error_streams_preserve_state_and_stop_later_files() {
  let fixture = Fixture::new();
  for source in ["duplicates id", "isid x"] {
    assert_eq!(
      fixture.batch(&["--json", "-c", source]).status.code(),
      Some(1)
    );
  }
  let mut session = fixture.session();
  let state = session.state();
  let preview = execute(&mut session, "head 10");
  for source in ["duplicates absent", "isid absent"] {
    assert!(session.execute(parse_command(source).unwrap()).is_err());
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "head 10"), preview);
  }
  assert_eq!(
    session
      .execute(tabdat_language::Command::Isid {
        variables: vec![],
        missok: false
      })
      .unwrap_err(),
    RuntimeError::IsidNoVariables
  );
  assert_eq!(session.state(), state);
  for (source, code, kind) in [
    ("isid id", 1, "ExecutionError"),
    ("isid id g", 1, "ExecutionError"),
    ("duplicates absent", 1, "ExecutionError"),
    ("isid absent", 1, "ExecutionError"),
    ("isid", 2, "ParseError"),
    ("duplicates id if x > 0", 2, "ParseError"),
    ("isid x, missok=true", 2, "ParseError"),
    ("by g: isid x", 2, "JsonResultUnsupported"),
  ] {
    let output = fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      "duplicates id",
      "-c",
      source,
      "-c",
      "export never.csv",
    ]);
    assert_eq!(output.status.code(), Some(code));
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 3);
    assert_eq!(results[2]["error"]["type"], kind);
    fs::write(
      fixture.0.join("child.td"),
      format!("duplicates id\n{source}\nexport never.csv\n"),
    )
    .unwrap();
    fs::write(
      fixture.0.join("parent.td"),
      "use data.csv\nrun child.td\nexport never.csv\n",
    )
    .unwrap();
    let output = fixture.batch(&["--json", "-f", "parent.td"]);
    assert_eq!(
      output.status.code(),
      Some(if kind == "JsonResultUnsupported" {
        1
      } else {
        code
      })
    );
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 3);
    assert_eq!(results[2]["error"]["line"], 2);
    assert_eq!(
      results[2]["error"]["path"],
      fixture
        .0
        .join("child.td")
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .as_ref()
    );
    assert!(!fixture.0.join("never.csv").exists());
  }
}

#[test]
fn terminal_guards_show_owned_diagnostics_recover_and_stream_nested_results() {
  let fixture = Fixture::new();
  let commands = "use data.csv\nduplicates id\nisid id\nisid id g, missok\nexport terminal.csv\n";
  fs::write(
    fixture.0.join("child.td"),
    "duplicates id\nisid id g, missok\n",
  )
  .unwrap();
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&fixture.0)
    .env("TERM", "dumb")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child
    .stdin
    .take()
    .unwrap()
    .write_all(format!("{commands}run child.td\nexit\n").as_bytes())
    .unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(output.status.success());
  let text = String::from_utf8_lossy(&output.stdout);
  assert_eq!(text.matches("Duplicate key report").count(), 2);
  assert_eq!(text.matches("isid passed").count(), 2);
  assert!(text.contains(
    "Duplicate groups: 2\nRows in duplicate groups: 4\nSurplus rows: 2\nMaximum copies: 2\n"
  ));
  assert!(text.contains("Rows with SQL NULL keys: 2\nMissing keys allowed: yes\n"));
  assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate key groups"));
  let reloaded = batch(&fixture, &["use terminal.csv", "head 10"]);
  let plain = batch(&fixture, &["use data.csv", "head 10"]);
  assert_eq!(reloaded[1]["data"], plain[1]["data"]);
}
