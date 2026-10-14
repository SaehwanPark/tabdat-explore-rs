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
use tabdat_runtime::{ExecutionResult, Session};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-terminal-cohort-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "id,g,x,y\n1,A,1,2\n2,A,2,5\n3,B,3,5\n4,B,4,9\n5,,5,8\n",
    )
    .unwrap();
    fs::write(
      root.join("wide.csv"),
      "id,x_a,x_b,y_a,y_b\n1,1,2,2,5\n2,3,4,5,9\n",
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
fn owned_named_and_transform_outcomes_are_exact_control_safe_without_mutating_metadata() {
  let fixture = Fixture::new();
  let mut dataset = fixture.session().active_dataset().unwrap().clone();
  dataset.row_count = u64::MAX;
  let cases = [
    (
      ExecutionResult::SqlCreate(tabdat_domain::SqlCreateResult {
        table_name: "雪\n\u{1b}".into(),
        dataset: dataset.clone(),
      }),
      format!(
        "Created and activated table \"雪\\n\\u{{1b}}\": {} rows, 4 columns\n",
        u64::MAX
      ),
    ),
    (
      ExecutionResult::Activate(tabdat_domain::ActivateResult {
        table_name: "雪\t".into(),
        dataset: dataset.clone(),
      }),
      format!("Activated table \"雪\\t\": {} rows, 4 columns\n", u64::MAX),
    ),
    (
      ExecutionResult::Join(tabdat_domain::JoinResult {
        dataset: dataset.clone(),
      }),
      format!("Joined: {} rows, 4 columns\n", u64::MAX),
    ),
    (
      ExecutionResult::Append(tabdat_domain::AppendResult {
        dataset: dataset.clone(),
      }),
      format!("Appended: {} rows, 4 columns\n", u64::MAX),
    ),
    (
      ExecutionResult::Reshape(tabdat_domain::ReshapeResult {
        dataset: dataset.clone(),
      }),
      format!("Reshaped: {} rows, 4 columns\n", u64::MAX),
    ),
    (
      ExecutionResult::Recode(tabdat_domain::RecodeResult { dataset }),
      format!("Recoded: {} rows, 4 columns\n", u64::MAX),
    ),
  ];
  for (result, expected) in cases {
    let before = value(&result);
    assert_eq!(render(&result), expected);
    assert_eq!(value(&result), before);
  }
}
#[test]
fn grouped_owned_counts_native_means_scalars_and_empty_tables_keep_state_and_json_limits() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(&mut session, "sql select * from active into cohort");
  execute(&mut session, "label variable g 'Group'");
  execute(&mut session, "regress y x");
  let state = session.state();
  let labels = session.active_label_metadata().unwrap().clone();
  let preview = execute(&mut session, "head 10");
  let inference = execute(&mut session, "lincom x");
  let counted = execute(&mut session, "by g: count");
  assert_eq!(
    render(&counted),
    "Grouped results (native; no finite-sample reinterpretation)\ng\tCount\n\"A\"\t2\n\"B\"\t2\nNA\t1\n(3 rows shown)\n"
  );
  let means = execute(&mut session, "by g: summarize x y");
  assert_eq!(
    render(&means),
    "Grouped results (native; no finite-sample reinterpretation)\ng\tmean_x\tmean_y\n\"A\"\t1.5\t3.5\n\"B\"\t3.5\t7\nNA\t5\t8\n(3 rows shown)\n"
  );
  assert_eq!(session.state(), state);
  assert_eq!(session.active_label_metadata().unwrap(), &labels);
  assert_eq!(execute(&mut session, "head 10"), preview);
  assert_eq!(execute(&mut session, "lincom x"), inference);
  assert!(!tabdat_reporting::json_results::supports_batch_command(
    &parse_command("by g: summarize x").unwrap()
  ));
  assert!(!tabdat_reporting::json_results::supports_script_command(
    &parse_command("by g: summarize x").unwrap()
  ));
  execute(
    &mut session,
    "sql select * from (values ('A',1.0::double),('A',3.0::double),('B',cast('Infinity' as double)),('C',NULL),('D',cast('NaN' as double))) as t(g,x) into edge",
  );
  let text = render(&execute(&mut session, "by g: summarize x"));
  assert!(text.contains("\"A\"\t2\n"));
  assert!(text.contains("\"B\"\tinf\n"));
  assert!(text.contains("\"C\"\tNA\n"));
  assert!(text.contains("\"D\"\tNaN\n"));
  execute(
    &mut session,
    "sql select * from active where false into empty",
  );
  assert!(render(&execute(&mut session, "by g: count")).ends_with("g\tCount\n(0 rows shown)\n"));
  assert!(
    render(&execute(&mut session, "by g: summarize x")).ends_with("g\tmean_x\n(0 rows shown)\n")
  );
  let scalar = ExecutionResult::By(tabdat_domain::ByResult {
    headers: vec!["key\n\u{1b}".into(), "Value".into()],
    rows: vec![
      vec![
        tabdat_domain::CellValue::Text("雪\t".into()),
        tabdat_domain::CellValue::UnsignedInteger(u128::MAX),
      ],
      vec![
        tabdat_domain::CellValue::Null,
        tabdat_domain::CellValue::Decimal {
          width: 38,
          scale: 2,
          value: 120,
        },
      ],
    ],
  });
  let text = render(&scalar);
  assert!(!text.contains('\u{1b}'));
  assert!(text.contains("\"雪\\t\"\t340282366920938463463374607431768211455"));
  assert!(text.contains("NA\t1.20"));
}
#[test]
fn named_cohort_enrichment_stacking_cleaning_and_analysis_match_nested_terminal_streams() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "sql select id,g,x,y from active order by id into base",
    "sql select id, x*10 as z from active order by id into lookup",
    "use base",
    "join lookup on id",
    "sql select * from active where id <= 2 order by id into followup",
    "use base",
    "append followup",
    "recode x (1=10), replace",
    "by g: count",
    "regress y x",
    "predict fitted",
    "save prepared.parquet",
    "export prepared.csv",
  ];
  let mut session = fixture.session();
  let mut expected = String::new();
  for source in &commands[1..commands.len() - 2] {
    expected.push_str(&render(&execute(&mut session, source)));
  }
  let prepared = value(&execute(&mut session, "head 20"))["data"].clone();
  assert_eq!(prepared["rows"].as_array().unwrap().len(), 7);
  assert_eq!(
    prepared["columns"],
    json!(["id", "g", "x", "y", "z", "fitted"])
  );
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
  for file in ["prepared.parquet", "prepared.csv"] {
    execute(
      &mut session,
      &format!("use {}", fixture.0.join(file).display()),
    );
    assert_eq!(value(&execute(&mut session, "head 20"))["data"], prepared);
    fs::remove_file(fixture.0.join(file)).unwrap();
  }
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
}
#[test]
fn reshape_layouts_and_recovery_report_owned_effects_with_child_failure_boundaries() {
  let fixture = Fixture::new();
  let commands = [
    "use wide.csv",
    "reshape long x y, i(id) j(period)",
    "by period: count",
    "reshape wide x y, i(id) j(period)",
    "head 10",
    "save shaped.parquet",
  ];
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("wide.csv").display()),
  );
  let mut expected = String::new();
  for source in &commands[1..commands.len() - 1] {
    expected.push_str(&render(&execute(&mut session, source)));
  }
  let before = execute(&mut session, "head 10");
  let output = fixture.shell(&format!("{}\nexit\n", commands.join("\n")));
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  assert!(
    String::from_utf8(output.stdout)
      .unwrap()
      .contains(&expected)
  );
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("shaped.parquet").display()),
  );
  assert_eq!(execute(&mut session, "head 10"), before);
  let output=fixture.shell("use data.csv\nuse missing\njoin missing on id\nappend missing\nrecode absent (1=2), replace\nreshape long absent, i(id) j(period)\nby missing: count\nsql select * from active into good\nuse good\nhead 10\nexit\n");
  assert!(output.status.success());
  assert_eq!(
    String::from_utf8(output.stderr)
      .unwrap()
      .matches("Error:")
      .count(),
    6
  );
  assert!(
    String::from_utf8(output.stdout)
      .unwrap()
      .contains("Activated table \"good\": 5 rows, 4 columns")
  );
  fs::write(
    fixture.0.join("child.td"),
    "join missing on id\nexport forbidden.csv\n",
  )
  .unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  let output = fixture.shell("run parent.td\ncount\nexit\n");
  assert!(output.status.success());
  assert!(
    String::from_utf8(output.stderr)
      .unwrap()
      .contains("child.td:1:")
  );
  assert!(
    String::from_utf8(output.stdout)
      .unwrap()
      .ends_with("5 rows\n")
  );
  assert!(!fixture.0.join("forbidden.csv").exists());
}
