#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};
static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-json-reshape-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("wide.csv"),
      "id,x_2,x_1,y_1,y_2,z\n2,2,1,2,5,2\n1,4,3,5,9,6\n3,6,5,8,,5\n",
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
fn close(actual: &Value, expected: f64) {
  assert!(
    (actual.as_f64().unwrap() - expected).abs() < 1e-10,
    "{actual} != {expected}"
  );
}

#[test]
fn multistub_order_native_metadata_explicit_refit_prediction_persistence_and_nested_stream() {
  let fixture = Fixture::new();
  let commands = [
    "use wide.csv",
    "sql select * from active into primary",
    "label variable id 'Subject'",
    "label variable x_2 'Old predictor'",
    "regress z x_2",
    "lincom x_2",
    "reshape long x y, i(id) j(t)",
    "lincom x_2",
    "head 10",
    "label list",
    "by t: count",
    "regress y x",
    "predict fitted",
    "head 10",
    "save long.parquet, replace",
    "export long.csv, replace",
    "reshape wide x y, i(id) j(t)",
    "head 10",
    "use primary",
    "head 10",
  ];
  let results = batch_commands(&fixture, &commands);
  assert_eq!(results.len(), commands.len());
  assert_eq!(results[6]["schema_version"], 1);
  assert_eq!(results[6]["result_type"], "ReshapeResult");
  assert_eq!(
    results[6]["data"],
    json!({"dataset":{"path":"primary","row_count":6,"columns":[{"name":"id","data_type":"BIGINT"},{"name":"t","data_type":"VARCHAR"},{"name":"x","data_type":"BIGINT"},{"name":"y","data_type":"BIGINT"}],"execution_mode":"eager","lazy_engine":null,"label_metadata":null,"panel_metadata":null}})
  );
  assert_eq!(results[7], results[5]);
  assert_eq!(
    results[8]["data"]["rows"],
    json!([
      [2, "2", 2, 5],
      [2, "1", 1, 2],
      [1, "2", 4, 9],
      [1, "1", 3, 5],
      [3, "2", 6, null],
      [3, "1", 5, 8]
    ])
  );
  assert_eq!(
    results[9]["data"]["metadata"]["variable_labels"],
    json!([["id", "Subject"]])
  );
  assert_eq!(
    results[10]["data"],
    json!({"headers":["t","Count"],"rows":[["1",3],["2",3]]})
  );
  // Finite long sample x=1..5, y=[2,5,5,9,8]: Sxx=10, Sxy=16.
  // Its slope is 1.6 and intercept 5.8-1.6*3=1; y-NULL row still predicts.
  assert_eq!(results[11]["data"]["observation_count"], 5);
  close(&results[11]["data"]["coefficients"][0]["value"], 1.0);
  close(&results[11]["data"]["coefficients"][1]["value"], 1.6);
  for row in results[13]["data"]["rows"].as_array().unwrap() {
    close(&row[4], 1.0 + 1.6 * row[2].as_f64().unwrap());
  }
  for path in ["long.csv", "long.parquet"] {
    let reloaded = batch_commands(&fixture, &[&format!("use {path}"), "head 10"]);
    let mut expected = results[13]["data"].clone();
    if path.ends_with("csv") {
      for row in expected["rows"].as_array_mut().unwrap() {
        row[1] = json!(row[1].as_str().unwrap().parse::<i64>().unwrap());
      }
    }
    assert_eq!(reloaded[1]["data"], expected);
  }
  assert_eq!(results[16]["data"]["dataset"]["row_count"], 3);
  assert_eq!(
    results[17]["data"],
    json!({"columns":["id","x_1","x_2","y_1","y_2"],"rows":[[2,1,2,2,5],[1,3,4,5,9],[3,5,6,8,null]]})
  );
  assert_eq!(
    results[19]["data"],
    json!({"columns":["id","x_2","x_1","y_1","y_2","z"],"rows":[[2,2,1,2,5,2],[1,4,3,5,9,6],[3,6,5,8,null,5]]})
  );
  fs::write(fixture.0.join("child.td"), commands[6..].join("\n")).unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    format!("{}\nrun child.td\n", commands[..6].join("\n")),
  )
  .unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
}

#[test]
fn wide_max_duplicates_null_ids_text_suffix_order_and_quoted_multi_identifiers() {
  let fixture = Fixture::new();
  let results = batch_commands(
    &fixture,
    &[
      "use wide.csv",
      "sql select \"person id\",site,t,x,y from (values (1,2,'A','2',1,10),(2,2,'A','2',5,null),(3,2,'A','10',3,30),(4,1,'B',null,99,99),(5,null,'C','2',8,80),(6,1,'B','2',4,40),(7,4,'D',null,99,99)) as source(ord,\"person id\",site,t,x,y) order by ord into longdata",
      "reshape wide x y, i(`person id` site) j(t)",
      "head 10",
    ],
  );
  assert_eq!(results[2]["data"]["dataset"]["row_count"], 4);
  assert_eq!(
    results[3]["data"],
    json!({"columns":["person id","site","x_10","x_2","y_10","y_2"],"rows":[[2,"A",3,5,30,10],[1,"B",null,4,null,40],[null,"C",null,8,null,80],[4,"D",null,null,null,null]]})
  );
  let quoted = batch_commands(
    &fixture,
    &[
      "use wide.csv",
      "sql select id as \"person id\", cast(x_1 as decimal(38,1)) as \"val name_quo'te\",cast(x_2 as decimal(38,1)) as \"val name_\"\"two\" from active into quoted",
      "reshape long `val name`, i(`person id`) j(`time name`)",
      "head 10",
      "reshape wide `val name`, i(`person id`) j(`time name`)",
      "head 10",
    ],
  );
  assert_eq!(
    quoted[3]["data"],
    json!({"columns":["person id","time name","val name"],"rows":[[2,"quo'te","1.0"],[2,"\"two","2.0"],[1,"quo'te","3.0"],[1,"\"two","4.0"],[3,"quo'te","5.0"],[3,"\"two","6.0"]]})
  );
  assert_eq!(
    quoted[5]["data"],
    json!({"columns":["person id","val name_\"two","val name_quo'te"],"rows":[[2,"2.0","1.0"],[1,"4.0","3.0"],[3,"6.0","5.0"]]})
  );
}

#[test]
fn reshape_labels_named_detachment_cached_sample_and_failure_state_are_explicit() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("wide.csv").display()),
  );
  execute(&mut session, "sql select * from active into primary");
  for source in [
    "label variable id 'Subject'",
    "label variable x_2 'Old predictor'",
    "label define ids 1 'One' 2 'Two' 3 'Three'",
    "label values id ids",
    "label values x_2 ids",
    "regress z x_2",
  ] {
    execute(&mut session, source);
  }
  let old = execute(&mut session, "lincom x_2");
  let original = execute(&mut session, "head 10");
  let state = session.state();
  for source in [
    "reshape long x, i(absent) j(t)",
    "reshape long missing, i(id) j(t)",
    "reshape long x, i(id) j(z)",
    "reshape wide missing, i(id) j(z)",
  ] {
    assert!(session.execute(parse_command(source).unwrap()).is_err());
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "head 10"), original);
    assert_eq!(execute(&mut session, "lincom x_2"), old);
  }
  let result = execute(&mut session, "reshape long x y, i(id) j(t)");
  let state = session.state();
  let preview = execute(&mut session, "head 10");
  assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
  assert_eq!(session.state(), state);
  assert_eq!(execute(&mut session, "head 10"), preview);
  assert_eq!(session.active_table_name(), None);
  assert_eq!(execute(&mut session, "lincom x_2"), old);
  assert_eq!(
    session.active_label_metadata().unwrap().variable_labels,
    [("id".into(), "Subject".into())]
  );
  assert_eq!(
    session.active_label_metadata().unwrap().attachments,
    [("id".into(), "ids".into())]
  );
  assert_eq!(session.active_label_metadata().unwrap().value_sets.len(), 1);
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 2]
  );
  assert_eq!(
    session
      .execute(parse_command("predict stale").unwrap())
      .unwrap_err(),
    RuntimeError::PredictUnknownVariable {
      variables: vec!["x_2".into()]
    }
  );
  assert_eq!(session.state(), state);
  assert_eq!(execute(&mut session, "head 10"), preview);
  assert_eq!(execute(&mut session, "lincom x_2"), old);
  execute(&mut session, "regress y x");
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 2, 3, 5]
  );
  execute(&mut session, "reshape wide x y, i(id) j(t)");
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 2, 3, 5]
  );
  assert_eq!(session.active_table_name(), None);
  execute(&mut session, "use primary");
  assert_eq!(execute(&mut session, "head 10"), original);
}

#[test]
fn empty_long_no_j_wide_ragged_collisions_and_incompatible_union_preserve_state() {
  let fixture = Fixture::new();
  let empty = batch_commands(
    &fixture,
    &[
      "use wide.csv",
      "sql select * from active where false into empty",
      "reshape long x y, i(id) j(t)",
      "head 10",
    ],
  );
  assert_eq!(empty[2]["data"]["dataset"]["row_count"], 0);
  assert_eq!(
    empty[3]["data"],
    json!({"columns":["id","t","x","y"],"rows":[]})
  );
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("wide.csv").display()),
  );
  execute(&mut session, "regress z x_2");
  let old = execute(&mut session, "lincom x_2");
  for (setup, source, expected) in [
    (
      "generate cost_1=1",
      "reshape long x cost, i(id) j(t)",
      RuntimeError::ReshapeLongMissingColumn {
        column: "cost_2".into(),
      },
    ),
    (
      "sql select id,x_1 as x,'1' as t,x_2 as x_1 from active into collision",
      "reshape wide x, i(id) j(t)",
      RuntimeError::ReshapeWideOutputColumnExists {
        variable: "x_1".into(),
      },
    ),
    (
      "sql select id,x,cast(null as varchar) as t from active into nullj",
      "reshape wide x, i(id) j(t)",
      RuntimeError::ReshapeWideFoundNoJValues,
    ),
    (
      "sql select * from active where false into empty",
      "reshape wide x, i(id) j(t)",
      RuntimeError::ReshapeWideFoundNoJValues,
    ),
  ] {
    execute(&mut session, setup);
    let state = session.state();
    let preview = execute(&mut session, "head 10");
    assert_eq!(
      session.execute(parse_command(source).unwrap()).unwrap_err(),
      expected
    );
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "head 10"), preview);
    assert_eq!(execute(&mut session, "lincom x_2"), old);
  }
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("wide.csv").display()),
  );
  execute(
    &mut session,
    "sql select id, [x_1] as v_a,struct_pack(x:=x_2) as v_b from active into incompatible",
  );
  let state = session.state();
  let query = "sql select id,cast(v_a as varchar),cast(v_b as varchar) from active order by id";
  let preview = execute(&mut session, query);
  assert_eq!(
    session
      .execute(parse_command("reshape long v, i(id) j(t)").unwrap())
      .unwrap_err(),
    RuntimeError::ReshapeFailed
  );
  assert_eq!(session.state(), state);
  assert_eq!(execute(&mut session, query), preview);
  assert_eq!(
    execute(&mut session, "count"),
    tabdat_runtime::ExecutionResult::Count(tabdat_runtime::CountResult { row_count: 3 })
  );
}

#[test]
fn malformed_runtime_and_unsupported_by_failures_stop_later_files_with_child_locations() {
  let fixture = Fixture::new();
  assert_eq!(
    fixture
      .batch(&["--json", "-c", "reshape long x, i(id) j(t)"])
      .status
      .code(),
    Some(1)
  );
  for (source, code, kind) in [
    ("reshape long missing, i(id) j(t)", 1, "ExecutionError"),
    ("reshape long x, i(absent) j(t)", 1, "ExecutionError"),
    ("reshape wider x, i(id) j(t)", 2, "ParseError"),
    (
      "by id: reshape long x, i(id) j(t)",
      2,
      "JsonResultUnsupported",
    ),
  ] {
    let output = fixture.batch(&[
      "--json",
      "-c",
      "use wide.csv",
      "-c",
      source,
      "-c",
      "export never.csv",
    ]);
    assert_eq!(output.status.code(), Some(code));
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
      "use wide.csv\nrun child.td\nexport never.csv\n",
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
