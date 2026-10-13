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
      "tabdat-json-join-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("left.csv"),"id,key,year,x,y,x_right\n1,b,2020,1,2,100\n2,a,2020,2,5,101\n3,b,2020,3,5,102\n4,c,2021,4,9,103\n5,,2020,5,8,104\n").unwrap();
    fs::write(root.join("right.csv"),"rid,key,year,x,z\n1,b,2020,20,2\n2,a,2020,10,1\n3,b,2020,21,3\n4,c,2021,30,4\n5,,2020,99,9\n").unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn setup(&self) -> String {
    "use left.csv\nsql select * from active order by id into primary\nuse right.csv\nsql select key,year,x,z from active order by rid into lookup\nuse primary\n".into()
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
fn close(actual: &Value, expected: f64) {
  assert!(
    (actual.as_f64().unwrap() - expected).abs() < 1e-10,
    "{actual} != {expected}"
  );
}
fn execute(session: &mut Session, source: &str) -> tabdat_runtime::ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}
fn batch_commands(fixture: &Fixture, commands: &[&str]) -> Vec<Value> {
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  success(&fixture.batch(&args))
}
fn joined_rows() -> Value {
  json!([
    [1, "b", 2020, 1, 2, 100, 20, 2],
    [1, "b", 2020, 1, 2, 100, 21, 3],
    [2, "a", 2020, 2, 5, 101, 10, 1],
    [3, "b", 2020, 3, 5, 102, 20, 2],
    [3, "b", 2020, 3, 5, 102, 21, 3],
    [4, "c", 2021, 4, 9, 103, 30, 4]
  ])
}

#[test]
fn multi_key_duplicate_join_cached_fit_refit_prediction_named_state_and_nested_persistence() {
  let fixture = Fixture::new();
  let commands = [
    "use left.csv",
    "sql select * from active order by id into primary",
    "use right.csv",
    "sql select key,year,x,z from active order by rid into lookup",
    "use primary",
    "label variable x 'Predictor'",
    "regress y x",
    "join lookup on key year",
    "lincom x",
    "label list",
    "head 10",
    "regress y x",
    "predict fitted",
    "head 10",
    "save joined.parquet, replace",
    "export joined.csv, replace",
    "use primary",
    "head 10",
  ];
  let results = batch_commands(&fixture, &commands);
  assert_eq!(results.len(), commands.len());
  assert_eq!(results[7]["schema_version"], 1);
  assert_eq!(results[7]["result_type"], "JoinResult");
  assert_eq!(results[7]["data"].as_object().unwrap().len(), 1);
  assert_eq!(results[7]["data"]["dataset"]["path"], "primary");
  assert_eq!(results[7]["data"]["dataset"]["row_count"], 6);
  assert_eq!(
    results[7]["data"]["dataset"]["columns"],
    json!([
     {"name":"id","data_type":"BIGINT"},{"name":"key","data_type":"VARCHAR"},{"name":"year","data_type":"BIGINT"},{"name":"x","data_type":"BIGINT"},{"name":"y","data_type":"BIGINT"},{"name":"x_right","data_type":"BIGINT"},{"name":"x_right_2","data_type":"BIGINT"},{"name":"z","data_type":"BIGINT"}
    ])
  );
  assert!(results[7]["data"]["dataset"]["label_metadata"].is_null());
  assert_eq!(results[10]["data"]["rows"], joined_rows());
  assert_eq!(
    results[9]["data"]["metadata"]["variable_labels"],
    json!([["x", "Predictor"]])
  );
  assert_eq!(results[6]["data"]["observation_count"], 5);
  close(&results[8]["data"]["estimate"], 1.6);
  // New fit uses duplicated joined observations x=[1,1,2,3,3,4],
  // y=[2,2,5,5,5,9]: Sxx=22/3, Sxy=44/3, slope=2/intercept=0.
  assert_eq!(results[11]["data"]["observation_count"], 6);
  close(&results[11]["data"]["coefficients"][0]["value"], 0.0);
  close(&results[11]["data"]["coefficients"][1]["value"], 2.0);
  let rows = results[13]["data"]["rows"].as_array().unwrap();
  assert_eq!(rows.len(), 6);
  for row in rows {
    close(&row[8], 2.0 * row[3].as_f64().unwrap());
  }
  assert_eq!(results[16]["result_type"], "ActivateResult");
  assert_eq!(results[16]["data"]["dataset"]["row_count"], 6);
  assert_eq!(results[17]["data"], results[13]["data"]);
  for path in ["joined.csv", "joined.parquet"] {
    let reload = batch_commands(&fixture, &[&format!("use {path}"), "head 10"]);
    assert_eq!(reload[1]["data"], results[13]["data"]);
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
fn left_null_nonmatches_custom_suffix_quoted_empty_and_self_joins_report_actual_schema() {
  let fixture = Fixture::new();
  let setup = fixture.setup();
  let mut commands = setup.lines().collect::<Vec<_>>();
  commands.extend([
    "join lookup on key year, how=left suffix(_lookup)",
    "head 10",
  ]);
  let results = batch_commands(&fixture, &commands);
  assert_eq!(results[5]["data"]["dataset"]["row_count"], 7);
  assert_eq!(
    results[5]["data"]["dataset"]["columns"][6]["name"],
    "x_lookup"
  );
  let mut expected = joined_rows();
  expected
    .as_array_mut()
    .unwrap()
    .push(json!([5, null, 2020, 5, 8, 104, null, null]));
  assert_eq!(results[6]["data"]["rows"], expected);
  let quoted = batch_commands(
    &fixture,
    &[
      "use left.csv",
      "sql select key as \"a\"\"b\", x from active where false into empty",
      "use left.csv",
      "rename key `a\"b`",
      "join empty on `a\"b`",
      "head 10",
    ],
  );
  assert_eq!(quoted[4]["data"]["dataset"]["row_count"], 0);
  assert_eq!(quoted[5]["data"]["rows"], json!([]));
  assert_eq!(quoted[4]["data"]["dataset"]["columns"][1]["name"], "a\"b");
  let self_join = batch_commands(
    &fixture,
    &[
      "use left.csv",
      "sql select * from active order by id into primary",
      "join primary on id",
      "head 10",
    ],
  );
  assert_eq!(self_join[2]["data"]["dataset"]["row_count"], 5);
  assert_eq!(self_join[3]["data"]["rows"].as_array().unwrap().len(), 5);
  for row in self_join[3]["data"]["rows"].as_array().unwrap() {
    assert_eq!(row[1], row[6]);
    assert_eq!(row[2], row[7]);
    assert_eq!(row[3], row[8]);
    assert_eq!(row[4], row[9]);
    assert_eq!(row[5], row[10]);
  }
}

#[test]
fn join_errors_preserve_batch_script_categories_locations_and_stop_later_files() {
  let fixture = Fixture::new();
  let setup = format!(
    "{}sql select 1 as key,2020 as year into bad\nuse primary\n",
    fixture.setup()
  );
  for (source, code) in [
    ("join absent on key", 1),
    ("join lookup on absent", 1),
    ("join lookup on id", 1),
    ("join bad on key year", 1),
    ("join lookup key", 2),
    ("join lookup on key, how=right", 2),
  ] {
    fs::write(
      fixture.0.join("child.td"),
      format!("count\n{source}\nexport never.csv\n"),
    )
    .unwrap();
    fs::write(
      fixture.0.join("parent.td"),
      format!("{setup}run child.td\nexport never.csv\n"),
    )
    .unwrap();
    let output = fixture.batch(&["--json", "-f", "parent.td"]);
    assert_eq!(
      output.status.code(),
      Some(code),
      "{source}: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 9);
    assert_eq!(results[8]["error"]["line"], 2);
    assert_eq!(
      results[8]["error"]["path"],
      fixture
        .0
        .join("child.td")
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .as_ref()
    );
    let mut args = vec!["--json"];
    for command in setup.lines() {
      args.extend(["-c", command]);
    }
    args.extend(["-c", source, "-c", "export never.csv"]);
    let output = fixture.batch(&args);
    assert_eq!(output.status.code(), Some(code));
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 8);
    assert_eq!(
      results[7]["error"]["type"],
      if code == 2 {
        "ParseError"
      } else {
        "ExecutionError"
      }
    );
    assert!(!fixture.0.join("never.csv").exists());
  }
  assert_eq!(
    fixture
      .batch(&["--json", "-c", "join lookup on key"])
      .status
      .code(),
    Some(1)
  );
}

#[test]
fn joins_and_reporting_preserve_label_definitions_cached_samples_and_atomic_failure_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("left.csv").display()),
  );
  execute(
    &mut session,
    "sql select * from active order by id into primary",
  );
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("right.csv").display()),
  );
  execute(
    &mut session,
    "sql select key,year,x,z from active order by rid into lookup",
  );
  execute(&mut session, "sql select 1 as key,2020 as year into bad");
  execute(&mut session, "use primary");
  execute(&mut session, "regress y x");
  execute(&mut session, "label variable x 'Predictor'");
  execute(&mut session, "label define xs 1 'One'");
  execute(&mut session, "label values x xs");
  let before = session.state();
  let preview = execute(&mut session, "head 10");
  let inference = execute(&mut session, "lincom x");
  for source in [
    "join absent on key",
    "join lookup on absent",
    "join lookup on id",
    "join bad on key year",
  ] {
    assert!(session.execute(parse_command(source).unwrap()).is_err());
    assert_eq!(session.state(), before);
    assert_eq!(execute(&mut session, "head 10"), preview);
  }
  let bad = tabdat_domain::Command::Join {
    command: tabdat_domain::JoinCommand {
      table_name: "lookup".into(),
      keys: vec!["key".into(), "year".into()],
      how: tabdat_domain::JoinHow::Inner,
      suffix: "bad\0suffix".into(),
    },
  };
  assert!(session.execute(bad).is_err());
  assert_eq!(session.state(), before);
  assert_eq!(execute(&mut session, "head 10"), preview);
  let result = execute(&mut session, "join lookup on key year");
  let after = session.state();
  assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
  assert_eq!(session.state(), after);
  assert_eq!(after.estimation, before.estimation);
  assert_eq!(after.labels, before.labels);
  assert_eq!(after.named_tables["primary"].row_count, 6);
  assert_eq!(after.named_tables["lookup"], before.named_tables["lookup"]);
  assert_eq!(execute(&mut session, "lincom x"), inference);
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
  execute(&mut session, "regress y x");
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 2, 3, 4, 5]
  );
}
