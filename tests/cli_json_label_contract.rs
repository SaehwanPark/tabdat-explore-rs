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
      "tabdat-json-label-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "x,y\n1,2\n2,5\n3,5\n4,9\n").unwrap();
    fs::write(root.join("empty.csv"), "x,y\n").unwrap();
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
fn execute(session: &mut Session, source: &str) -> tabdat_runtime::ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}
fn metadata(result: &Value) -> &Value {
  assert_eq!(result["result_type"], "LabelResult");
  &result["data"]["metadata"]
}

#[test]
fn category_dictionary_reconciles_transforms_subsets_and_nested_persistence() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "recode x (min/2=1) (else=2), generate(group)",
    "label variable group 'Band'",
    "label define bands 2 'High' 1 'Low'",
    "label define other 9 'Nine'",
    "label values group bands",
    "label list",
    "label list bands",
    "tabulate group",
    "rename group category",
    "label list",
    "recode category (1=1), replace",
    "label list",
    "label values category bands",
    "keep category y",
    "label list",
    "drop category",
    "label list",
    "export after.csv, replace",
    "save after.parquet, replace",
    "use after.parquet",
    "label list",
  ];
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  let results = success(&fixture.batch(&args));
  assert_eq!(results.len(), commands.len());
  let band = json!({"name":"bands","mappings":[[{"kind":"integer","payload":2},"High"],[{"kind":"integer","payload":1},"Low"]]});
  let other = json!({"name":"other","mappings":[[{"kind":"integer","payload":9},"Nine"]]});
  assert_eq!(
    metadata(&results[6]),
    &json!({"variable_labels":[["group","Band"]],"value_sets":[band.clone(),other.clone()],"attachments":[["group","bands"]]})
  );
  assert_eq!(
    metadata(&results[7]),
    &json!({"variable_labels":[["group","Band"]],"value_sets":[band.clone()],"attachments":[["group","bands"]]})
  );
  assert_eq!(results[8]["data"]["rows"][0][0], "Low");
  assert_eq!(results[8]["data"]["rows"][1][0], "High");
  assert_eq!(results[8]["data"]["rows"][0][1], 2);
  assert_eq!(results[8]["data"]["rows"][1][1], 2);
  assert_eq!(
    metadata(&results[10])["variable_labels"],
    json!([["category", "Band"]])
  );
  assert_eq!(
    metadata(&results[10])["attachments"],
    json!([["category", "bands"]])
  );
  // Even a no-op successful replacement removes the selected attachment.
  assert_eq!(metadata(&results[12])["attachments"], json!([]));
  assert_eq!(
    metadata(&results[12])["value_sets"],
    json!([band.clone(), other.clone()])
  );
  assert_eq!(metadata(&results[15]), metadata(&results[10]));
  assert_eq!(
    metadata(&results[17]),
    &json!({"variable_labels":[],"value_sets":[band,other],"attachments":[]})
  );
  assert!(metadata(&results[21]).is_null());
  assert!(results[20]["data"]["dataset"]["label_metadata"].is_null());
  let reload = success(&fixture.batch(&[
    "--json",
    "-c",
    "use after.csv",
    "-c",
    "head 10",
    "-c",
    "label list",
  ]));
  assert_eq!(reload[1]["data"]["rows"], json!([[2], [5], [5], [9]]));
  assert!(metadata(&reload[2]).is_null());
  fs::write(fixture.0.join("child.td"), commands[7..].join("\n")).unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    format!("{}\nrun child.td\n", commands[..7].join("\n")),
  )
  .unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
}

#[test]
fn exact_key_types_source_order_clear_replace_and_quoted_empty_data() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "label define mixed 9223372036854775807 'wide' 9007199254740993.001 'exact' '1' 'text' -9223372036854775808 'negative'",
    "label define alpha 1 'A'",
    "label variable y 'Outcome'",
    "label variable x '列\tX'",
    "label values x mixed",
    "label list mixed",
    "label define mixed 2 'New', replace",
    "label values x, clear",
    "label variable x, clear",
    "label variable y, clear",
    "label drop mixed alpha",
    "describe",
    "head 10",
  ];
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  let results = success(&fixture.batch(&args));
  let mappings = &metadata(&results[1])["value_sets"][0]["mappings"];
  assert_eq!(
    mappings,
    &json!([
      [{"kind":"integer","payload":i64::MAX},"wide"],
      [{"kind":"number","payload":"9007199254740993.001"},"exact"],
      [{"kind":"text","payload":"1"},"text"],
      [{"kind":"integer","payload":i64::MIN},"negative"]
    ])
  );
  assert_eq!(
    metadata(&results[6])["variable_labels"],
    json!([["x", "列\tX"], ["y", "Outcome"]])
  );
  assert_eq!(
    metadata(&results[6])["value_sets"]
      .as_array()
      .unwrap()
      .len(),
    1
  );
  assert_eq!(
    metadata(&results[7])["attachments"],
    json!([["x", "mixed"]])
  );
  assert_eq!(
    metadata(&results[7])["value_sets"][1]["mappings"],
    json!([[{"kind":"integer","payload":2},"New"]])
  );
  assert_eq!(metadata(&results[8])["attachments"], json!([]));
  assert!(metadata(&results[11]).is_null());
  assert_eq!(
    results[12]["data"]["dataset"],
    results[0]["data"]["dataset"]
  );
  assert_eq!(
    results[13]["data"]["rows"],
    json!([[1, 2], [2, 5], [3, 5], [4, 9]])
  );
  let empty = success(&fixture.batch(&[
    "--json",
    "-c",
    "use empty.csv",
    "-c",
    "rename x `标\"列`",
    "-c",
    "label variable `标\"列` 'Empty'",
    "-c",
    "label define codes 'A' 'Alpha'",
    "-c",
    "label values `标\"列` codes",
    "-c",
    "label list",
    "-c",
    "head 10",
  ]));
  assert_eq!(
    metadata(&empty[5])["variable_labels"],
    json!([["标\"列", "Empty"]])
  );
  assert_eq!(
    metadata(&empty[5])["attachments"],
    json!([["标\"列", "codes"]])
  );
  assert_eq!(empty[6]["data"]["rows"], json!([]));
}

#[test]
fn label_errors_preserve_stream_categories_locations_and_later_file_guards() {
  let fixture = Fixture::new();
  for (source, code) in [
    ("label variable absent 'A'", 1),
    ("label define codes 2 'two'", 1),
    ("label values x absent", 1),
    ("label list absent", 1),
    ("label drop absent codes", 1),
    ("label define dup 1 'a' 1 'b'", 2),
    ("label variable x unquoted", 2),
    ("label save dictionary.json", 2),
    ("label use dictionary.json", 2),
  ] {
    let output = fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      "label define codes 1 'one'",
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
    assert_eq!(results.len(), 3);
    assert_eq!(
      results[2]["error"]["type"],
      if code == 2 {
        "ParseError"
      } else {
        "ExecutionError"
      }
    );
    fs::write(
      fixture.0.join("child.td"),
      format!("label list\n{source}\nexport never.csv\n"),
    )
    .unwrap();
    fs::write(
      fixture.0.join("parent.td"),
      "use data.csv\nlabel define codes 1 'one'\nrun child.td\nexport never.csv\n",
    )
    .unwrap();
    let output = fixture.batch(&["--json", "-f", "parent.td"]);
    assert_eq!(output.status.code(), Some(code));
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 4);
    assert_eq!(results[3]["error"]["line"], 2);
    assert_eq!(
      results[3]["error"]["path"],
      fixture
        .0
        .join("child.td")
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .as_ref()
    );
    assert!(!fixture.0.join("never.csv").exists());
    assert!(!fixture.0.join("dictionary.json").exists());
  }
  let output = fixture.batch(&["--json", "-c", "label list"]);
  assert_eq!(output.status.code(), Some(1));
}

#[test]
fn labels_and_reporting_do_not_change_data_or_cached_estimation_and_failures_are_atomic() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(&mut session, "regress y x");
  let original = session.state();
  let preview = execute(&mut session, "head 10");
  let inference = execute(&mut session, "lincom x");
  for source in [
    "label variable x 'Predictor'",
    "label define codes 1 'one'",
    "label values x codes",
    "label list codes",
    "label define codes 2 'two', replace",
    "label values x, clear",
    "label values x codes",
  ] {
    let result = execute(&mut session, source);
    let before = session.state();
    let value: Value = serde_json::from_str(
      &tabdat_reporting::json_results::serialize_execution_result(&result).unwrap(),
    )
    .unwrap();
    assert_eq!(value["result_type"], "LabelResult");
    assert_eq!(session.state(), before);
    assert_eq!(before.estimation, original.estimation);
    assert_eq!(before.active_dataset, original.active_dataset);
    assert_eq!(execute(&mut session, "head 10"), preview);
    assert_eq!(execute(&mut session, "lincom x"), inference);
  }
  let state = session.state();
  for source in [
    "label variable absent 'A'",
    "label define codes 3 'three'",
    "label values x absent",
    "label list absent",
    "label drop codes absent",
  ] {
    assert!(session.execute(parse_command(source).unwrap()).is_err());
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "head 10"), preview);
  }
  let duplicate = tabdat_domain::Command::Label {
    command: tabdat_domain::LabelCommand::Define {
      name: "codes".into(),
      mappings: vec![
        (tabdat_domain::LabelValue::Integer(1), "one".into()),
        (tabdat_domain::LabelValue::Integer(1), "duplicate".into()),
      ],
      replace: true,
    },
  };
  assert!(session.execute(duplicate).is_err());
  assert_eq!(session.state(), state);
  execute(&mut session, "label drop codes");
  assert!(session.state().labels.attachments.is_empty());
  execute(&mut session, "label variable x, clear");
  assert!(session.active_label_metadata().is_none());
}
