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
      "tabdat-json-persistence-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "x,y,label\n1.5,2,A\n2.5,5,\n3.5,5,\"quoted, \"\"text\"\"\"\n4.5,9,A\n5.5,,B\n,8,C\n",
    )
    .unwrap();
    fs::write(root.join("empty.csv"), "x,label\n").unwrap();
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
fn lines(output: &[u8]) -> Vec<Value> {
  std::str::from_utf8(output)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn assert_success(output: &Output) {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
}

#[test]
fn persistence_json_reports_destination_not_active_source_and_roundtrips_predictions() {
  let fixture = Fixture::new();
  let saved_path = "out dir/模型, data.parquet";
  let csv_path = "out dir/模型, data.csv";
  let output = fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "regress y x",
    "-c",
    "predict fitted",
    "-c",
    "head 10",
    "-c",
    &format!("save \"{saved_path}\""),
    "-c",
    &format!("export '{csv_path}'"),
    "-c",
    "describe",
    "-c",
    "lincom x",
  ]);
  assert_success(&output);
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 8);
  for (index, result_type, path) in [(4, "SaveResult", saved_path), (5, "ExportResult", csv_path)] {
    assert_eq!(results[index]["schema_version"], 1);
    assert_eq!(results[index]["result_type"], result_type);
    assert_eq!(results[index]["data"]["path"], path);
    let mut dataset = results[2]["data"]["dataset"].clone();
    dataset["path"] = json!(path);
    assert_eq!(results[index]["data"]["dataset"], dataset);
    assert!(fixture.0.join(path).is_file());
  }
  assert_eq!(results[6]["data"], results[2]["data"]);
  assert_eq!(results[6]["data"]["dataset"]["path"], "data.csv");
  assert_eq!(results[7]["result_type"], "LincomResult");
  for path in [saved_path, csv_path] {
    let reload = fixture.batch(&["--json", "-c", &format!("use \"{path}\""), "-c", "head 10"]);
    assert_success(&reload);
    assert_eq!(lines(&reload.stdout)[1]["data"], results[3]["data"]);
  }
  fs::write(
    fixture.0.join("child.td"),
    format!(
      "head 10\nsave \"{saved_path}\", replace\nexport '{csv_path}', replace\ndescribe\nlincom x\n"
    ),
  )
  .unwrap();
  fs::write(
    fixture.0.join("middle.td"),
    "regress y x\npredict fitted\nrun child.td\n",
  )
  .unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun middle.td\n").unwrap();
  let nested = fixture.batch(&["--json", "-f", "parent.td"]);
  assert_success(&nested);
  assert_eq!(lines(&nested.stdout), results);
}

#[test]
fn refused_overwrites_keep_existing_bytes_and_replace_writes_current_data() {
  let fixture = Fixture::new();
  for (command, path, result_type) in [
    ("save", "existing.parquet", "SaveResult"),
    ("export", "existing.csv", "ExportResult"),
  ] {
    fs::write(fixture.0.join(path), b"existing bytes").unwrap();
    let refused = fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      &format!("{command} {path}"),
      "-c",
      "export never.csv",
    ]);
    assert_eq!(refused.status.code(), Some(1));
    let results = lines(&refused.stdout);
    assert_eq!(results.len(), 2);
    assert_eq!(results[1]["error"]["type"], "ExecutionError");
    assert_eq!(fs::read(fixture.0.join(path)).unwrap(), b"existing bytes");
    assert!(!fixture.0.join("never.csv").exists());
    let replaced = fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      &format!("{command} {path}, replace"),
      "-c",
      "describe",
    ]);
    assert_success(&replaced);
    assert_eq!(lines(&replaced.stdout)[1]["result_type"], result_type);
    assert_ne!(fs::read(fixture.0.join(path)).unwrap(), b"existing bytes");
    let reloaded = fixture.batch(&["--json", "-c", &format!("use {path}"), "-c", "count"]);
    assert_success(&reloaded);
    assert_eq!(lines(&reloaded.stdout)[1]["data"], json!({"row_count": 6}));
  }
}

#[test]
fn persistence_failures_preserve_prior_streams_locations_and_later_targets() {
  let fixture = Fixture::new();
  fs::create_dir(fixture.0.join("directory.parquet")).unwrap();
  fs::create_dir(fixture.0.join("directory.csv")).unwrap();
  fs::write(fixture.0.join("blocked"), b"parent blocker").unwrap();
  let failed_save = format!("save \"{}.parquet\"", "a".repeat(300));
  let failed_export = format!("export \"{}.csv\"", "a".repeat(300));
  for source in [
    "save unsupported.csv",
    "export unsupported.parquet",
    "save directory.parquet, replace",
    "export directory.csv, replace",
    "save blocked/out.parquet",
    "export blocked/out.csv",
    "save out.parquet, unknown",
    "export out.csv, unknown",
    &failed_save,
    &failed_export,
  ] {
    let code = if source.contains("unknown") { 2 } else { 1 };
    let output = fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      source,
      "-c",
      "export never.csv",
    ]);
    assert_eq!(output.status.code(), Some(code), "{source}");
    assert_eq!(lines(&output.stdout).len(), 2);
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
    let nested = fixture.batch(&["--json", "-f", "parent.td"]);
    assert_eq!(nested.status.code(), Some(code), "{source}");
    let results = lines(&nested.stdout);
    assert_eq!(results.len(), 3);
    let error = &results[2]["error"];
    assert_eq!(error["type"], "ScriptError");
    assert_eq!(error["line"], 2);
    assert_eq!(
      error["path"],
      fixture
        .0
        .join("child.td")
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .as_ref()
    );
    assert!(!fixture.0.join("never.csv").exists());
    assert!(!fixture.0.join("out.csv").exists());
    assert!(!fixture.0.join("out.parquet").exists());
    assert_eq!(
      fs::read(fixture.0.join("blocked")).unwrap(),
      b"parent blocker"
    );
  }
  for source in ["save absent.parquet", "export absent.csv"] {
    let output = fixture.batch(&["--json", "-c", source]);
    assert_eq!(output.status.code(), Some(1));
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 1);
    assert!(
      results[0]["error"]["message"]
        .as_str()
        .unwrap()
        .contains("active dataset")
    );
  }
}

#[test]
fn empty_relation_persistence_is_reported_and_reloadable() {
  let fixture = Fixture::new();
  let output = fixture.batch(&[
    "--json",
    "-c",
    "use empty.csv",
    "-c",
    "save empty.parquet",
    "-c",
    "export empty-out.csv",
    "-c",
    "count",
  ]);
  assert_success(&output);
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 4);
  for index in [1, 2] {
    assert_eq!(results[index]["data"]["dataset"]["row_count"], 0);
  }
  for path in ["empty.parquet", "empty-out.csv"] {
    let reloaded = fixture.batch(&["--json", "-c", &format!("use {path}"), "-c", "count"]);
    assert_success(&reloaded);
    assert_eq!(lines(&reloaded.stdout)[1]["data"], json!({"row_count": 0}));
  }
}

#[test]
fn persistence_success_and_preflight_failure_preserve_session_state_and_rows() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  let execute =
    |session: &mut Session, source: &str| session.execute(parse_command(source).unwrap());
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  )
  .unwrap();
  execute(&mut session, "regress y x").unwrap();
  execute(&mut session, "predict fitted").unwrap();
  execute(&mut session, "label variable x \"Predictor label\"").unwrap();
  let state = session.state();
  let rows = format!("{:?}", execute(&mut session, "head 10").unwrap());
  for (command, suffix) in [("save", "parquet"), ("export", "csv")] {
    let path = fixture.0.join(format!("saved.{suffix}"));
    let result = execute(&mut session, &format!("{command} {}", path.display())).unwrap();
    assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
    let bytes = fs::read(&path).unwrap();
    assert!(execute(&mut session, &format!("{command} {}", path.display())).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    let failed_path = fixture.0.join(format!("{}.{suffix}", "a".repeat(300)));
    assert!(
      execute(
        &mut session,
        &format!("{command} \"{}\"", failed_path.display())
      )
      .is_err()
    );
    assert_eq!(session.state(), state);
    assert_eq!(
      format!("{:?}", execute(&mut session, "head 10").unwrap()),
      rows
    );
  }
}
