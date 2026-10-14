#![forbid(unsafe_code)]
use serde_json::Value;
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::{ExecutionResult, Session};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-terminal-coding-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "x,y,category\n1,2,b\n2,5,a\n3,5,b\n4,9,c\n5,8,\n6,,z\n",
    )
    .unwrap();
    Self(root)
  }
  fn shell(&self, source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    child
      .stdin
      .take()
      .unwrap()
      .write_all(source.as_bytes())
      .unwrap();
    child.wait_with_output().unwrap()
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
fn render(result: &ExecutionResult) -> String {
  tabdat_reporting::terminal::render_execution_result(result)
}
#[test]
fn owned_label_rendering_preserves_typed_keys_order_spelling_and_escapes_controls() {
  use tabdat_domain::{LabelMetadata, LabelResult, LabelResultAction, LabelValue, ValueLabelSet};
  let result = ExecutionResult::Label(LabelResult {
    action: LabelResultAction::List,
    message: "listed\n\u{1b}".into(),
    metadata: Some(LabelMetadata {
      variable_labels: vec![("z\n".into(), "雪\t".into()), ("a".into(), "".into())],
      attachments: vec![("z\n".into(), "set\r".into())],
      value_sets: vec![ValueLabelSet {
        name: "set\r".into(),
        mappings: vec![
          (LabelValue::Integer(i64::MAX), "Wide".into()),
          (LabelValue::Number("1.200".into()), "Decimal".into()),
          (LabelValue::Text("1.200".into()), "Text\u{1b}".into()),
          (LabelValue::Integer(i64::MIN), "".into()),
        ],
      }],
    }),
  });
  let before = value(&result);
  let text = render(&result);
  assert!(text.starts_with("Label list: listed\\n\\u{1b}\n"));
  assert!(!text.contains('\u{1b}'));
  assert!(!text.contains('\r'));
  assert!(!text.contains('\t'));
  assert!(text.contains("integer 9223372036854775807 -> \"Wide\""));
  assert!(text.contains("integer -9223372036854775808 -> \"\""));
  assert!(text.contains("number \"1.200\" -> \"Decimal\""));
  assert!(text.contains("text \"1.200\" -> \"Text\\u{1b}\""));
  assert!(text.find("\"z\\n\" ->").unwrap() < text.find("\"a\" ->").unwrap());
  assert!(
    text.find("integer 9223372036854775807").unwrap() < text.find("number \"1.200\"").unwrap()
  );
  assert_eq!(value(&result), before);
  for action in [
    LabelResultAction::Variable,
    LabelResultAction::Define,
    LabelResultAction::Values,
    LabelResultAction::List,
    LabelResultAction::Drop,
  ] {
    let empty = ExecutionResult::Label(LabelResult {
      action,
      message: "done".into(),
      metadata: None,
    });
    assert!(render(&empty).ends_with("No label metadata.\n"));
  }
  let empty = ExecutionResult::Label(LabelResult {
    action: LabelResultAction::List,
    message: "listed".into(),
    metadata: Some(LabelMetadata::default()),
  });
  assert_eq!(render(&empty).matches("(none)").count(), 3);
}
#[test]
fn coding_and_filtered_dictionary_rendering_leave_cached_data_and_json_contract_unchanged() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(&mut session, "replace category = '' if x == 6");
  execute(&mut session, "label variable category 'Category 雪'");
  execute(&mut session, "regress y x");
  let cached = session.last_regression().unwrap().clone();
  let inference = execute(&mut session, "lincom x");
  let encoded = execute(&mut session, "encode category, generate(code) label(codes)");
  assert_eq!(render(&encoded), "Encoded: 6 rows, 4 columns\n");
  assert_eq!(value(&encoded)["result_type"], "EncodeResult");
  execute(
    &mut session,
    "label define exact 9007199254740993 'Wide' 1.20 'Decimal' '1.20' 'Text'",
  );
  let listed = execute(&mut session, "label list exact");
  let state = session.state();
  let labels = session.active_label_metadata().unwrap().clone();
  let preview = execute(&mut session, "head 10");
  let signature = execute(&mut session, "datasignature");
  let text = render(&listed);
  assert!(text.contains("integer 9007199254740993"));
  assert!(text.contains("number \"1.20\""));
  assert!(text.contains("text \"1.20\""));
  assert!(text.contains("\"code\" -> \"Category 雪\""));
  assert!(!text.contains("\"codes\":"));
  assert_eq!(session.state(), state);
  assert_eq!(session.active_label_metadata().unwrap(), &labels);
  assert_eq!(execute(&mut session, "head 10"), preview);
  assert_eq!(execute(&mut session, "datasignature"), signature);
  assert_eq!(execute(&mut session, "lincom x"), inference);
  assert_eq!(session.last_regression().unwrap(), &cached);
  let decoded = execute(&mut session, "decode code, generate(readable)");
  assert_eq!(render(&decoded), "Decoded: 6 rows, 5 columns\n");
  assert_eq!(value(&decoded)["result_type"], "DecodeResult");
  let rows = value(&execute(&mut session, "head 10"))["data"]["rows"].clone();
  for row in rows.as_array().unwrap() {
    assert_eq!(row[2], row[4]);
  }
  execute(
    &mut session,
    &format!("save {}", fixture.0.join("coded.parquet").display()),
  );
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("coded.parquet").display()),
  );
  assert_eq!(
    value(&execute(&mut session, "head 10"))["data"]["rows"],
    rows
  );
  assert!(render(&execute(&mut session, "label list")).contains("No label metadata."));
  execute(
    &mut session,
    "sql select category from active where false into empty",
  );
  assert_eq!(
    render(&execute(&mut session, "encode category, generate(code)")),
    "Encoded: 0 rows, 2 columns\n"
  );
  assert!(render(&execute(&mut session, "label list code")).contains("\"code\":\n    (none)\n"));
  assert_eq!(
    render(&execute(&mut session, "decode code, generate(readable)")),
    "Decoded: 0 rows, 3 columns\n"
  );
}
#[test]
fn persistent_terminal_workflow_recovery_clear_drop_and_nested_streams() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "replace category = '' if x == 6",
    "label variable category 'Category 雪'",
    "encode category, generate(code) label(codes)",
    "label list codes",
    "decode code, generate(readable)",
    "head 10",
    "save coded.parquet",
    "export coded.csv",
  ];
  let mut session = fixture.session();
  let mut expected = String::new();
  for source in &commands[1..] {
    // The CLI and runtime use the same owned renderer, but execute independently.
    let actual = if source.starts_with("save ") || source.starts_with("export ") {
      execute(
        &mut session,
        &source.replace("coded.", &format!("{}/expected.", fixture.0.display())),
      )
    } else {
      execute(&mut session, source)
    };
    if !source.starts_with("save ") && !source.starts_with("export ") {
      expected.push_str(&render(&actual));
    }
  }
  let output = fixture.shell(&format!("{}\nexit\n", commands.join("\n")));
  assert!(output.status.success());
  assert!(
    output.stderr.is_empty(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains(&expected));
  assert!(!text.contains("terminal details"));
  let output=fixture.shell("use data.csv\nencode y, generate(bad)\nlabel drop missing\ndecode y, generate(bad)\nencode category, generate(code) label(codes)\nlabel values code, clear\nlabel drop codes\nlabel list\nexit\n");
  assert!(output.status.success());
  assert!(!output.stderr.is_empty());
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("Encoded: 6 rows, 4 columns"));
  assert!(text.contains("Label values:"));
  assert!(text.contains("Label drop:"));
  assert!(text.contains("No label metadata."));
  fs::remove_file(fixture.0.join("coded.parquet")).unwrap();
  fs::remove_file(fixture.0.join("coded.csv")).unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  fs::write(fixture.0.join("child.td"), commands[1..].join("\n")).unwrap();
  let output = fixture.shell("run parent.td\nexit\n");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  assert!(
    String::from_utf8(output.stdout)
      .unwrap()
      .contains(&expected)
  );
  fs::write(
    fixture.0.join("child.td"),
    "encode y, generate(bad)\nexport forbidden.csv\n",
  )
  .unwrap();
  let output = fixture.shell("run parent.td\nlabel list\nexit\n");
  assert!(output.status.success());
  assert!(
    String::from_utf8(output.stderr)
      .unwrap()
      .contains("child.td:1:")
  );
  assert!(!fixture.0.join("forbidden.csv").exists());
}
