use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{
  Command, DataSource, ExecutionMode, LabelCommand, LabelValue, parse_command,
};
use tabdat_runtime::{ExecutionResult, LabelResultAction, RuntimeError, Session, ValueLabelSet};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
  root: PathBuf,
  parquet: PathBuf,
}

impl Fixture {
  fn new() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after the Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-label-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir(&root).expect("fixture directory should be created");
    let parquet = root.join("label.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES (30, 'b'), (42, 'a'), (54, CAST(NULL AS VARCHAR)), (60, 'b')) AS patients(age, sex)) TO ? (FORMAT PARQUET)",
        [&parquet_string],
      )
      .expect("fixture Parquet should be written");
    Self { root, parquet }
  }

  fn command(&self) -> Command {
    Command::Use {
      source: DataSource::LocalPath(self.parquet.to_string_lossy().into_owned()),
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: None,
      has_header: None,
    }
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

#[test]
fn label_requires_an_active_dataset_without_initializing_a_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("label variable age \"Age\"").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "label" }
  );
  assert!(session.active_dataset().is_none());
  assert!(session.active_label_metadata().is_none());
}

#[test]
fn label_mutations_list_and_drop_publish_normalized_metadata() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  session
    .execute(parse_command("label variable age \"Age years\"").unwrap())
    .expect("variable label should execute");
  session
    .execute(parse_command("label define sexlbl 0 \"Male\" 1 \"Female\"").unwrap())
    .expect("value label definition should execute");
  session
    .execute(parse_command("label values sex sexlbl").unwrap())
    .expect("value label attachment should execute");

  let result = session
    .execute(parse_command("label list").unwrap())
    .expect("label list should execute");
  let ExecutionResult::Label(result) = result else {
    panic!("label list should return a Label result");
  };
  assert_eq!(result.action, LabelResultAction::List);
  assert_eq!(result.message, "Label dictionary");
  assert_eq!(
    result.metadata,
    Some(tabdat_runtime::LabelMetadata {
      variable_labels: vec![("age".to_owned(), "Age years".to_owned())],
      value_sets: vec![ValueLabelSet {
        name: "sexlbl".to_owned(),
        mappings: vec![
          (LabelValue::Integer(0), "Male".to_owned()),
          (LabelValue::Integer(1), "Female".to_owned()),
        ],
      }],
      attachments: vec![("sex".to_owned(), "sexlbl".to_owned())],
    })
  );

  session
    .execute(parse_command("label variable age, clear").unwrap())
    .expect("variable label clear should execute");
  session
    .execute(parse_command("label values sex, clear").unwrap())
    .expect("value label clear should execute");
  session
    .execute(parse_command("label drop sexlbl").unwrap())
    .expect("value label drop should execute");
  assert!(session.active_label_metadata().is_none());
}

#[test]
fn label_validation_failures_are_atomic_and_references_reconcile() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  session
    .execute(parse_command("label variable age \"Age\"").unwrap())
    .expect("variable label should execute");
  session
    .execute(parse_command("label define sexlbl 0 \"Male\"").unwrap())
    .expect("value label definition should execute");
  session
    .execute(parse_command("label values sex sexlbl").unwrap())
    .expect("value label attachment should execute");
  let before = session
    .active_label_metadata()
    .expect("metadata should be published")
    .clone();

  assert_eq!(
    session
      .execute(parse_command("label variable missing \"Missing\"").unwrap())
      .unwrap_err(),
    RuntimeError::LabelUnknownVariable {
      variable: "missing".to_owned(),
    }
  );
  assert_eq!(
    session
      .execute(parse_command("label define sexlbl 1 \"Female\"").unwrap())
      .unwrap_err(),
    RuntimeError::LabelDefineSetExists {
      set_name: "sexlbl".to_owned(),
    }
  );
  assert_eq!(
    session
      .execute(parse_command("label values sex other").unwrap())
      .unwrap_err(),
    RuntimeError::LabelValuesUnknownSet {
      set_name: "other".to_owned(),
    }
  );
  assert_eq!(
    session
      .execute(parse_command("label list other missing").unwrap())
      .unwrap_err(),
    RuntimeError::LabelListUnknownSet {
      names: vec!["missing".to_owned(), "other".to_owned()],
    }
  );
  assert_eq!(
    session
      .execute(parse_command("label drop other").unwrap())
      .unwrap_err(),
    RuntimeError::LabelDropUnknownSet {
      names: vec!["other".to_owned()],
    }
  );
  assert_eq!(
    session
      .execute(Command::Label {
        command: LabelCommand::Define {
          name: "sexlbl".to_owned(),
          mappings: vec![
            (LabelValue::Integer(0), "Male".to_owned()),
            (LabelValue::Integer(0), "Duplicate".to_owned()),
          ],
          replace: true,
        },
      })
      .unwrap_err(),
    RuntimeError::LabelDefineDuplicateValue {
      set_name: "sexlbl".to_owned(),
      value: LabelValue::Integer(0),
    }
  );
  assert_eq!(session.active_label_metadata(), Some(&before));

  session
    .execute(parse_command("rename age years").unwrap())
    .expect("rename should move variable labels");
  session
    .execute(parse_command("keep years sex").unwrap())
    .expect("keep should retain surviving metadata");
  let metadata = session
    .active_label_metadata()
    .expect("metadata should survive schema changes");
  assert_eq!(
    metadata.variable_labels,
    vec![("years".to_owned(), "Age".to_owned())]
  );
  assert_eq!(
    metadata.attachments,
    vec![("sex".to_owned(), "sexlbl".to_owned())]
  );
}

#[test]
fn label_metadata_reconciles_value_changes_and_projections() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  session
    .execute(parse_command("label variable age \"Age\"").unwrap())
    .expect("variable label should execute");
  session
    .execute(parse_command("label define agelbl 30 \"Initial\"").unwrap())
    .expect("value label definition should execute");
  session
    .execute(parse_command("label values age agelbl").unwrap())
    .expect("value label attachment should execute");

  session
    .execute(parse_command("replace age = age + 1").unwrap())
    .expect("replace should execute");
  assert_eq!(
    session
      .active_label_metadata()
      .expect("variable label and set should survive replace")
      .attachments,
    Vec::<(String, String)>::new()
  );

  session
    .execute(parse_command("label values age agelbl").unwrap())
    .expect("value label should reattach");
  session
    .execute(parse_command("recode age (31 = 99), replace").unwrap())
    .expect("recode should execute");
  assert_eq!(
    session
      .active_label_metadata()
      .expect("variable label and set should survive recode")
      .attachments,
    Vec::<(String, String)>::new()
  );

  session
    .execute(parse_command("label values age agelbl").unwrap())
    .expect("value label should reattach after recode");
  session
    .execute(parse_command("select age sex").unwrap())
    .expect("select should preserve surviving metadata");
  assert_eq!(
    session
      .active_label_metadata()
      .expect("metadata should survive select")
      .attachments,
    vec![("age".to_owned(), "agelbl".to_owned())]
  );
  session
    .execute(parse_command("drop age").unwrap())
    .expect("drop should execute with another column remaining");
  let metadata = session
    .active_label_metadata()
    .expect("value set metadata should survive dropping its variable");
  assert!(metadata.variable_labels.is_empty());
  assert!(metadata.attachments.is_empty());
  assert_eq!(metadata.value_sets.len(), 1);
}

#[test]
fn label_decode_rejects_non_integer_attached_values_and_use_clears_metadata() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  session
    .execute(parse_command("label define textlbl \"a\" \"Alpha\"").unwrap())
    .expect("text label set should execute");
  session
    .execute(parse_command("label values sex textlbl").unwrap())
    .expect("text label attachment should execute");

  let command = Command::Decode {
    source: "sex".to_owned(),
    generate: "sex_text".to_owned(),
  };
  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::DecodeRequiresIntegerLabels {
      variable: "sex".to_owned(),
    }
  );
  assert!(session.active_label_metadata().is_some());

  session
    .execute(fixture.command())
    .expect("reload should succeed");
  assert!(session.active_label_metadata().is_none());
}
