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
      "tabdat-json-coding-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "x,y,category\n1,2,b\n2,5,a\n3,5,b\n4,9,c\n5,8,\n6,,z\n",
    )
    .unwrap();
    fs::write(root.join("empty.csv"), "category\n").unwrap();
    fs::write(root.join("numbers.csv"), "x\n1.0\n1.5\n9.0\n").unwrap();
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
fn close(actual: &Value, expected: f64) {
  assert!(
    (actual.as_f64().unwrap() - expected).abs() < 1e-10,
    "{actual} != {expected}"
  );
}
fn execute(session: &mut Session, source: &str) -> tabdat_runtime::ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}

#[test]
fn sorted_category_codes_dictionary_numeric_workflow_decode_and_nested_persistence() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "recode category ('z'=''), replace",
    "label variable category 'Category'",
    "encode category, generate(code) label(codes)",
    "label list",
    "head 10",
    "tabulate code, missing",
    "generate centered = code - 1",
    "regress y centered",
    "predict fitted",
    "decode code, generate(readable)",
    "head 10",
    "save coded.parquet, replace",
    "export coded.csv, replace",
  ];
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  let results = success(&fixture.batch(&args));
  assert_eq!(results.len(), commands.len());
  for (index, name) in [(3, "EncodeResult"), (10, "DecodeResult")] {
    assert_eq!(results[index]["schema_version"], 1);
    assert_eq!(results[index]["result_type"], name);
    assert_eq!(results[index]["data"].as_object().unwrap().len(), 1);
    assert_eq!(results[index]["data"]["dataset"]["path"], "data.csv");
    assert_eq!(results[index]["data"]["dataset"]["row_count"], 6);
    assert!(results[index]["data"]["dataset"]["label_metadata"].is_null());
  }
  assert_eq!(
    results[3]["data"]["dataset"]["columns"][3],
    json!({"name":"code","data_type":"INTEGER"})
  );
  assert_eq!(
    results[10]["data"]["dataset"]["columns"][6],
    json!({"name":"readable","data_type":"VARCHAR"})
  );
  assert_eq!(
    results[4]["data"]["metadata"],
    json!({"variable_labels":[["category","Category"],["code","Category"]],"value_sets":[{"name":"codes","mappings":[[{"kind":"integer","payload":1},""],[{"kind":"integer","payload":2},"a"],[{"kind":"integer","payload":3},"b"],[{"kind":"integer","payload":4},"c"]]}],"attachments":[["code","codes"]]})
  );
  assert_eq!(
    results[5]["data"]["rows"],
    json!([
      [1, 2, "b", 3],
      [2, 5, "a", 2],
      [3, 5, "b", 3],
      [4, 9, "c", 4],
      [5, 8, null, null],
      [6, null, "", 1]
    ])
  );
  let table = results[6]["data"]["rows"].as_array().unwrap();
  assert_eq!(
    table.iter().map(|row| row[0].clone()).collect::<Vec<_>>(),
    vec![json!(""), json!("a"), json!("b"), json!("c"), Value::Null]
  );
  assert_eq!(
    table
      .iter()
      .map(|row| row[1].as_u64().unwrap())
      .collect::<Vec<_>>(),
    [1, 1, 2, 1, 1]
  );
  // Explicit numeric interpretation only: centered=[2,1,2,3], y=[2,5,5,9],
  // Sxx=2 and Sxy=4 yield slope=2 and intercept=5.25-2*2=1.25.
  assert_eq!(results[8]["data"]["observation_count"], 4);
  close(&results[8]["data"]["coefficients"][0]["value"], 1.25);
  close(&results[8]["data"]["coefficients"][1]["value"], 2.0);
  assert_eq!(
    results[7]["data"]["dataset"]["columns"][4]["data_type"],
    "DECIMAL(38,0)"
  );
  for (row, centered) in results[11]["data"]["rows"].as_array().unwrap().iter().zip([
    Some(2.0),
    Some(1.0),
    Some(2.0),
    Some(3.0),
    None,
    Some(0.0),
  ]) {
    assert_eq!(row[6], row[2]);
    if let Some(centered) = centered {
      assert_eq!(row[4], json!(centered.to_string()));
      close(&row[5], 1.25 + 2.0 * centered);
    } else {
      assert!(row[4].is_null());
      assert!(row[5].is_null());
    }
  }
  let reload = success(&fixture.batch(&[
    "--json",
    "-c",
    "use coded.parquet",
    "-c",
    "head 10",
    "-c",
    "label list",
  ]));
  assert_eq!(reload[1]["data"], results[11]["data"]);
  assert!(reload[2]["data"]["metadata"].is_null());
  let reload = success(&fixture.batch(&["--json", "-c", "use coded.csv", "-c", "head 10"]));
  let mut expected = results[11]["data"].clone();
  expected["rows"][5][2] = Value::Null;
  expected["rows"][5][6] = Value::Null;
  // CSV reload infers these small exact DECIMAL values as BIGINT, not strings.
  for row in expected["rows"].as_array_mut().unwrap() {
    if let Some(spelling) = row[4].as_str() {
      row[4] = json!(spelling.parse::<i64>().unwrap());
    }
  }
  assert_eq!(reload[1]["data"], expected);
  fs::write(fixture.0.join("child.td"), commands[3..].join("\n")).unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    format!("{}\nrun child.td\n", commands[..3].join("\n")),
  )
  .unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
}

#[test]
fn repeated_named_default_coding_and_exact_decode_keys_cover_empty_and_unknown_values() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "label define shared 1 'Old'",
    "label values x shared",
    "encode category, generate(code) label(shared)",
    "encode category, generate(second)",
    "label list",
    "decode x, generate(xtext)",
    "head 10",
  ];
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  let results = success(&fixture.batch(&args));
  assert_eq!(
    results[5]["data"]["metadata"]["attachments"],
    json!([["code", "shared"], ["second", "second"], ["x", "shared"]])
  );
  assert_eq!(
    results[5]["data"]["metadata"]["value_sets"][0]["name"],
    "second"
  );
  assert_eq!(
    results[5]["data"]["metadata"]["value_sets"][1]["mappings"],
    results[5]["data"]["metadata"]["value_sets"][0]["mappings"]
  );
  assert_eq!(
    results[7]["data"]["rows"]
      .as_array()
      .unwrap()
      .iter()
      .map(|row| row[5].clone())
      .collect::<Vec<_>>(),
    vec![
      json!("a"),
      json!("b"),
      json!("c"),
      json!("z"),
      Value::Null,
      Value::Null
    ]
  );
  let numbers = success(&fixture.batch(&[
    "--json",
    "-c",
    "use numbers.csv",
    "-c",
    "label define keys 1 'One'",
    "-c",
    "label values x keys",
    "-c",
    "decode x, generate(text)",
    "-c",
    "head 10",
  ]));
  assert_eq!(
    numbers[4]["data"]["rows"],
    json!([[1.0, "One"], [1.5, null], [9.0, null]])
  );
  let empty = success(&fixture.batch(&[
    "--json",
    "-c",
    "use empty.csv",
    "-c",
    "rename category `标\"列`",
    "-c",
    "encode `标\"列`, generate(`编\"码`)",
    "-c",
    "decode `编\"码`, generate(`文\"本`)",
    "-c",
    "label list",
    "-c",
    "head 10",
  ]));
  assert_eq!(
    empty[2]["data"]["dataset"]["columns"][1],
    json!({"name":"编\"码","data_type":"BIGINT"})
  );
  assert_eq!(
    empty[3]["data"]["dataset"]["columns"][2],
    json!({"name":"文\"本","data_type":"VARCHAR"})
  );
  assert_eq!(
    empty[4]["data"]["metadata"]["value_sets"][0]["mappings"],
    json!([])
  );
  assert_eq!(empty[5]["data"]["rows"], json!([]));
  fs::write(fixture.0.join("nulls.csv"), "x,category\n1,\n2,\n").unwrap();
  let nulls = success(&fixture.batch(&[
    "--json",
    "-c",
    "use nulls.csv",
    "-c",
    "encode category, generate(code)",
    "-c",
    "decode code, generate(text)",
    "-c",
    "head 10",
  ]));
  assert_eq!(
    nulls[1]["data"]["dataset"]["columns"][2]["data_type"],
    "BIGINT"
  );
  assert_eq!(
    nulls[3]["data"]["rows"],
    json!([[1, null, null, null], [2, null, null, null]])
  );
}

#[test]
fn coding_failures_retain_nested_categories_locations_and_stop_later_files() {
  let fixture = Fixture::new();
  for (source, code) in [
    ("encode absent, generate(new)", 1),
    ("encode x, generate(new)", 1),
    ("encode category, generate(y)", 1),
    ("encode category, generate(Y)", 1),
    ("decode absent, generate(new)", 1),
    ("decode code, generate(category)", 1),
    ("decode code, generate(Y)", 1),
    ("decode x, generate(new)", 1),
    ("decode category, generate(new)", 1),
    ("encode category", 2),
    ("decode code", 2),
  ] {
    let setup = "use data.csv\nencode category, generate(code)\nlabel define fractional 1.0 'One'\nlabel values x fractional\nlabel values category code\n";
    fs::write(
      fixture.0.join("child.td"),
      format!("label list\n{source}\nexport never.csv\n"),
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
    assert_eq!(results.len(), 7);
    assert_eq!(results[6]["error"]["line"], 2);
    assert_eq!(
      results[6]["error"]["path"],
      fixture
        .0
        .join("child.td")
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .as_ref()
    );
    assert!(!fixture.0.join("never.csv").exists());
    let mut args = vec!["--json"];
    for command in setup.lines() {
      args.extend(["-c", command]);
    }
    args.extend(["-c", source, "-c", "export never.csv"]);
    let output = fixture.batch(&args);
    assert_eq!(output.status.code(), Some(code));
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 6);
    assert_eq!(
      results[5]["error"]["type"],
      if code == 2 {
        "ParseError"
      } else {
        "ExecutionError"
      }
    );
    assert!(!fixture.0.join("never.csv").exists());
  }
  for source in [
    "encode category, generate(code)",
    "decode code, generate(text)",
  ] {
    assert_eq!(
      fixture.batch(&["--json", "-c", source]).status.code(),
      Some(1)
    );
  }
}

#[test]
fn coding_reporting_label_inheritance_and_native_failures_preserve_cached_fit_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(&mut session, "regress y x");
  execute(&mut session, "label variable category 'Category'");
  let original = session.state();
  let inference = execute(&mut session, "lincom x");
  for source in [
    "encode category, generate(code) label(codes)",
    "decode code, generate(readable)",
  ] {
    let result = execute(&mut session, source);
    let before = session.state();
    assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
    assert_eq!(session.state(), before);
    assert_eq!(before.estimation, original.estimation);
    assert_eq!(execute(&mut session, "lincom x"), inference);
  }
  assert_eq!(
    session.state().labels.variable_labels,
    vec![
      ("category".into(), "Category".into()),
      ("code".into(), "Category".into()),
      ("readable".into(), "Category".into())
    ]
  );
  let before = session.state();
  let preview = execute(&mut session, "head 10");
  for source in [
    "encode absent, generate(new)",
    "encode x, generate(new)",
    "encode category, generate(y)",
    "encode category, generate(Y)",
    "decode absent, generate(new)",
    "decode code, generate(category)",
    "decode code, generate(Y)",
  ] {
    assert!(session.execute(parse_command(source).unwrap()).is_err());
    assert_eq!(session.state(), before);
    assert_eq!(execute(&mut session, "head 10"), preview);
  }
  for command in [
    tabdat_domain::Command::Encode {
      source: "category".into(),
      generate: "bad\0name".into(),
      label: Some("codes".into()),
    },
    tabdat_domain::Command::Decode {
      source: "code".into(),
      generate: "bad\0name".into(),
    },
  ] {
    assert!(session.execute(command).is_err());
    assert_eq!(session.state(), before);
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
