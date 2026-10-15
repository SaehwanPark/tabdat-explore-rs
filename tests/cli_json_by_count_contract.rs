#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-json-by-count-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "x,y,g,h\n1,2,b,1\n2,5,a,1\n3,5,b,2\n4,9,b,1\n5,8,,1\n6,,a,\n",
    )
    .unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
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
fn batch_commands(fixture: &Fixture, commands: &[&str]) -> Vec<Value> {
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  success(&fixture.batch(&args))
}
fn execute(session: &mut Session, source: &str) -> tabdat_runtime::ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}

#[test]
fn raw_keys_empty_strings_nulls_group_order_counts_and_nested_analysis_persistence() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "recode g ('a'=''), replace",
    "label variable g 'Group'",
    "label define gs 'b' 'Beta' '' 'Empty'",
    "label values g gs",
    "regress y x",
    "by g: count",
    "by g h: count",
    "by h g: count",
    "by g g: count",
    "predict fitted",
    "head 10",
    "save counted.parquet, replace",
    "export counted.csv, replace",
  ];
  let results = batch_commands(&fixture, &commands);
  assert_eq!(results.len(), commands.len());
  let expected = [
    json!({"headers":["g","Count"],"rows":[["",2],["b",3],[null,1]]}),
    json!({"headers":["g","h","Count"],"rows":[["",1,1],["",null,1],["b",1,2],["b",2,1],[null,1,1]]}),
    json!({"headers":["h","g","Count"],"rows":[[1,"",1],[1,"b",2],[1,null,1],[2,"b",1],[null,"",1]]}),
    json!({"headers":["g","g","Count"],"rows":[["","",2],["b","b",3],[null,null,1]]}),
  ];
  for (result, expected) in results[6..10].iter().zip(expected) {
    assert_eq!(result["schema_version"], 1);
    assert_eq!(result["result_type"], "ByResult");
    assert_eq!(result["data"], expected);
    let total: u64 = result["data"]["rows"]
      .as_array()
      .unwrap()
      .iter()
      .map(|row| row.as_array().unwrap().last().unwrap().as_u64().unwrap())
      .sum();
    assert_eq!(total, 6);
  }
  assert_eq!(results[5]["data"]["observation_count"], 5);
  assert_eq!(results[11]["data"]["rows"].as_array().unwrap().len(), 6);
  let reload = batch_commands(&fixture, &["use counted.parquet", "head 10"]);
  assert_eq!(reload[1]["data"], results[11]["data"]);
  let reload = batch_commands(&fixture, &["use counted.csv", "head 10"]);
  let mut expected = results[11]["data"].clone();
  expected["rows"][1][2] = Value::Null;
  expected["rows"][5][2] = Value::Null;
  assert_eq!(reload[1]["data"], expected);
  fs::write(fixture.0.join("child.td"), commands[5..].join("\n")).unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    format!("{}\nrun child.td\n", commands[..5].join("\n")),
  )
  .unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
}

#[test]
fn append_then_count_before_fit_matches_the_same_fit_prediction_and_export_without_counting() {
  let fixture = Fixture::new();
  let prefix = [
    "use data.csv",
    "sql select * from active where x<=2 into extra",
    "use data.csv",
    "recode g ('a'=''), replace",
    "append extra",
  ];
  let suffix = [
    "regress y x",
    "predict fitted",
    "head 20",
    "export appended.csv, replace",
  ];
  let mut commands = prefix.to_vec();
  commands.push("by g: count");
  commands.extend(suffix);
  let counted = batch_commands(&fixture, &commands);
  assert_eq!(
    counted[5]["data"],
    json!({"headers":["g","Count"],"rows":[["",2],["a",1],["b",4],[null,1]]})
  );
  assert_eq!(counted[6]["data"]["observation_count"], 7);
  assert_eq!(counted[8]["data"]["rows"].as_array().unwrap().len(), 8);
  let exported = fs::read(fixture.0.join("appended.csv")).unwrap();
  let mut commands = prefix.to_vec();
  commands.extend(suffix);
  let uncounted = batch_commands(&fixture, &commands);
  assert_eq!(counted[..5], uncounted[..5]);
  assert_eq!(counted[6..], uncounted[5..]);
  assert_eq!(fs::read(fixture.0.join("appended.csv")).unwrap(), exported);
  fs::write(
    fixture.0.join("child.td"),
    format!("by g: count\n{}\n", suffix.join("\n")),
  )
  .unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    format!("{}\nrun child.td\n", prefix.join("\n")),
  )
  .unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    counted
  );
}

#[test]
fn quoted_empty_typed_groups_use_owned_scalar_policy_without_normalizing_keys() {
  let fixture = Fixture::new();
  let results = batch_commands(
    &fixture,
    &[
      "use data.csv",
      "sql select cast(x as decimal(38,1)) as \"a\"\"b\" from active where x<=2 into typed",
      "by `a\"b`: count",
      "sql select * from active where false into empty",
      "by `a\"b`: count",
    ],
  );
  assert_eq!(
    results[2]["data"],
    json!({"headers":["a\"b","Count"],"rows":[["1.0",1],["2.0",1]]})
  );
  assert_eq!(
    results[4]["data"],
    json!({"headers":["a\"b","Count"],"rows":[]})
  );
  let wide = batch_commands(
    &fixture,
    &[
      "use data.csv",
      "sql select cast(9007199254740993 as bigint) as g from active into wide",
      "by g: count",
    ],
  );
  assert_eq!(
    wide[2]["data"],
    json!({"headers":["g","Count"],"rows":[[9007199254740993_i64,6]]})
  );
  let nonfinite = batch_commands(
    &fixture,
    &[
      "use data.csv",
      "sql select cast('NaN' as double) as g union all select cast(null as double) into groups",
      "by g: count",
    ],
  );
  // Existing scalar policy makes distinct native NaN and SQL NULL keys both null.
  // They are separate counted groups, not a new missingness normalization.
  assert_eq!(
    nonfinite[2]["data"],
    json!({"headers":["g","Count"],"rows":[[null,1],[null,1]]})
  );
}

#[test]
fn narrow_admission_failure_streams_and_count_serialization_preserve_all_session_state() {
  let fixture = Fixture::new();
  for (source, code, kind) in [
    ("by absent: count", 1, "ExecutionError"),
    ("by g: summarize x, detail", 2, "JsonResultUnsupported"),
    ("by g: summarize x if x > 0", 2, "JsonResultUnsupported"),
    ("by absent: summarize x", 1, "ExecutionError"),
    ("by g: summarize absent", 1, "ExecutionError"),
    ("by g: summarize g", 1, "ExecutionError"),
    ("by g: tabulate h", 2, "JsonResultUnsupported"),
    ("by g: sort x", 2, "JsonResultUnsupported"),
    ("by g count", 2, "ParseError"),
  ] {
    let output = fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      source,
      "-c",
      "export never.csv",
    ]);
    assert_eq!(
      output.status.code(),
      Some(code),
      "{source}: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 2);
    assert_eq!(results[1]["error"]["type"], kind);
    fs::write(
      fixture.0.join("child.td"),
      format!("count\n{source}\nexport never.csv\n"),
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
  assert_eq!(
    fixture
      .batch(&["--json", "-c", "by g: count"])
      .status
      .code(),
    Some(1)
  );
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(
    &mut session,
    "sql select * from active order by x into primary",
  );
  execute(&mut session, "regress y x");
  execute(&mut session, "label variable g 'Group'");
  let state = session.state();
  let preview = execute(&mut session, "head 10");
  let inference = execute(&mut session, "lincom x");
  for source in [
    "by g: count",
    "by g h: count",
    "by h g: count",
    "by g g: count",
  ] {
    let result = execute(&mut session, source);
    assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "head 10"), preview);
    assert_eq!(execute(&mut session, "lincom x"), inference);
  }
  for command in [
    parse_command("by absent: count").unwrap(),
    tabdat_domain::Command::By {
      command: tabdat_domain::ByCommand {
        groups: vec![],
        command: Box::new(tabdat_domain::Command::Count),
      },
    },
  ] {
    assert!(session.execute(command).is_err());
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "head 10"), preview);
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
}
