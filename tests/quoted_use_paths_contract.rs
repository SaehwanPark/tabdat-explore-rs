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
use tabdat_runtime::Session;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-quoted-use-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "x,y\n1,2\n2,5\n3,5\n4,9\n").unwrap();
    for name in ["data space.csv", "目录,é data.csv", "owner's data.csv"] {
      fs::copy(root.join("data.csv"), root.join(name)).unwrap();
    }
    fs::write(root.join("corrupt data.parquet"), "not parquet").unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn shell(&self, input: &str) -> Output {
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
      .write_all(input.as_bytes())
      .unwrap();
    child.wait_with_output().unwrap()
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
fn quoted_csv_and_parquet_load_through_batch_and_nested_json_with_exact_paths() {
  let fixture = Fixture::new();
  let bare = fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "count",
    "-c",
    "head 4",
  ]);
  assert_success(&bare);
  let expected = lines(&bare.stdout);
  for (source, path) in [
    ("use \"data.csv\"".to_owned(), "data.csv".to_owned()),
    (
      "use 'data space.csv'".to_owned(),
      "data space.csv".to_owned(),
    ),
    (
      "use \"目录,é data.csv\"".to_owned(),
      "目录,é data.csv".to_owned(),
    ),
    (
      "use \"owner's data.csv\"".to_owned(),
      "owner's data.csv".to_owned(),
    ),
    (
      format!("use \"{}\"", fixture.0.join("目录,é data.csv").display()),
      fixture
        .0
        .join("目录,é data.csv")
        .to_string_lossy()
        .into_owned(),
    ),
  ] {
    let output = fixture.batch(&["--json", "-c", &source, "-c", "count", "-c", "head 4"]);
    assert_success(&output);
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0]["data"]["dataset"]["path"], path);
    assert_eq!(
      results[0]["data"]["dataset"]["columns"],
      expected[0]["data"]["dataset"]["columns"]
    );
    assert_eq!(&results[1..], &expected[1..]);
    fs::write(
      fixture.0.join("child.td"),
      format!("{source}\ncount\nhead 4\n"),
    )
    .unwrap();
    fs::write(fixture.0.join("middle.td"), "run child.td\n").unwrap();
    fs::write(fixture.0.join("parent.td"), "run middle.td\n").unwrap();
    let nested = fixture.batch(&["--json", "-f", "parent.td"]);
    assert_success(&nested);
    assert_eq!(lines(&nested.stdout), results);
  }
  let saved = fixture.batch(&["-c", "use data.csv", "-c", "save roundtrip.parquet"]);
  assert_success(&saved);
  fs::copy(
    fixture.0.join("roundtrip.parquet"),
    fixture.0.join("目录, data.parquet"),
  )
  .unwrap();
  let parquet = fixture.batch(&[
    "--json",
    "-c",
    "use \"目录, data.parquet\"",
    "-c",
    "count",
    "-c",
    "head 4",
  ]);
  assert_success(&parquet);
  assert_eq!(&lines(&parquet.stdout)[1..], &expected[1..]);
}

#[test]
fn quoted_csv_options_and_shell_recovery_keep_the_existing_workflow() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("semi, data.csv"),
    "x;y\n1;2\n2;5\n3;5\n4;9\n",
  )
  .unwrap();
  let options = fixture.batch(&[
    "--json",
    "-c",
    "use 'semi, data.csv', delimiter(\";\") has_header(true)",
    "-c",
    "head 4",
  ]);
  let ordinary = fixture.batch(&["--json", "-c", "use data.csv", "-c", "head 4"]);
  assert_success(&options);
  assert_success(&ordinary);
  assert_eq!(lines(&options.stdout)[1], lines(&ordinary.stdout)[1]);
  let output = fixture.shell("use \"data space.csv\"\nregress y x\nuse \"missing file.csv\"\nuse \"data.csv\" junk\nuse \"corrupt data.parquet\"\npredict fitted\ncount\nexit\n");
  assert!(output.status.success());
  let errors = String::from_utf8(output.stderr).unwrap();
  assert_eq!(errors.matches("Error:").count(), 3, "{errors}");
  assert!(errors.contains("use expects exactly one path"));
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("Predicted: 4 rows, 3 columns\n"), "{text}");
  assert!(text.ends_with("4 rows\n"));
}

#[test]
fn quoted_path_failures_are_located_and_prevent_later_effects() {
  let fixture = Fixture::new();
  for (source, code, kind) in [
    ("use \"data.csv\" extra.csv", 2, "ParseError"),
    ("use \"目录, missing.csv", 2, "ParseError"),
    ("use ''", 2, "ParseError"),
    ("use \"missing file.csv\"", 1, "ExecutionError"),
    ("use \"corrupt data.parquet\"", 1, "ExecutionError"),
    ("use 'https://example.com/data.csv'", 1, "ExecutionError"),
    ("use 'data space.csv', lazy", 1, "ExecutionError"),
  ] {
    let batch = fixture.batch(&["--json", "-c", "use data.csv", "-c", source, "-c", "count"]);
    assert_eq!(batch.status.code(), Some(code), "{source}");
    let results = lines(&batch.stdout);
    assert_eq!(results.len(), 2, "{source}");
    assert_eq!(results[1]["error"]["type"], kind, "{source}");
    fs::write(
      fixture.0.join("child.td"),
      format!("count\n{source}\ncount\nexport never.csv\n"),
    )
    .unwrap();
    fs::write(
      fixture.0.join("parent.td"),
      "use data.csv\nrun child.td\ncount\nexport never.csv\n",
    )
    .unwrap();
    let nested = fixture.batch(&["--json", "-f", "parent.td"]);
    assert_eq!(
      nested.status.code(),
      Some(if source == "use \"missing file.csv\"" {
        3
      } else {
        code
      }),
      "{source}"
    );
    let plain = fixture.batch(&["-f", "parent.td"]);
    assert_eq!(plain.status.code(), nested.status.code(), "{source}");
    assert!(!fixture.0.join("never.csv").exists(), "{source}");
    let results = lines(&nested.stdout);
    assert_eq!(results.len(), 3, "{source}");
    assert_eq!(results[1]["data"], json!({"row_count": 4}));
    assert_eq!(results[2]["error"]["type"], "ScriptError");
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
  }
}

#[test]
fn quoted_missing_corrupt_and_invalid_loads_preserve_data_and_regression_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(
      parse_command(&format!(
        "use \"{}\"",
        fixture.0.join("目录,é data.csv").display()
      ))
      .unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command("regress y x").unwrap())
    .unwrap();
  let dataset = session.active_dataset().cloned();
  let model = session.last_regression().cloned();
  let before = format!(
    "{:?}",
    session.execute(parse_command("head 4").unwrap()).unwrap()
  );
  for name in ["missing file.csv", "corrupt data.parquet"] {
    assert!(
      session
        .execute(parse_command(&format!("use \"{}\"", fixture.0.join(name).display())).unwrap())
        .is_err()
    );
    assert_eq!(session.active_dataset(), dataset.as_ref());
    assert_eq!(session.last_regression(), model.as_ref());
    assert_eq!(
      format!(
        "{:?}",
        session.execute(parse_command("head 4").unwrap()).unwrap()
      ),
      before
    );
  }
  assert!(parse_command("use \"data space.csv\" other.csv").is_err());
  assert_eq!(session.active_dataset(), dataset.as_ref());
  assert_eq!(session.last_regression(), model.as_ref());
}
