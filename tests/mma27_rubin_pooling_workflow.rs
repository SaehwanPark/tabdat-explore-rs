mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  fs,
  path::{Path, PathBuf},
  process::{Command, Output},
};

const SCRIPT: &str = include_str!("../examples/mma27p1milinear/rubin_pooling.td");
const EXPORT: &str = "rubin_marginal_report.csv";
fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn fixture(kind: &str, name: &str) -> String {
  fs::read_to_string(
    root()
      .join("tests/fixtures/mma27_rubin")
      .join(format!("{kind}_{name}.csv")),
  )
  .unwrap()
}
fn run(dir: &Path, script: &str, mode: &str) -> Output {
  fs::write(dir.join("script.td"), script).unwrap();
  tabdat_bin()
    .current_dir(dir)
    .args([mode, "-f", "script.td"])
    .output()
    .unwrap()
}
fn audit(dir: &Path, extra: &[&str]) -> Output {
  Command::new("python3")
    .arg(root().join("examples/mma27p1milinear/check_rubin.py"))
    .arg(dir)
    .arg(dir.join("reference"))
    .args(extra)
    .output()
    .unwrap()
}
fn setup(kind: &str, dir: &Path) {
  fs::write(dir.join("estimates.csv"), fixture(kind, "inputs")).unwrap();
  fs::create_dir(dir.join("reference")).unwrap();
  fs::write(dir.join("reference").join(EXPORT), fixture(kind, "report")).unwrap();
}
fn success(kind: &str) {
  let d = TestDir::new();
  setup(kind, d.path());
  let source = fixture(kind, "inputs");
  let output = run(d.path(), SCRIPT, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  fs::write(d.path().join("out.jsonl"), &output.stdout).unwrap();
  let checked = audit(d.path(), &["--synthetic"]);
  assert!(
    checked.status.success(),
    "{}",
    String::from_utf8_lossy(&checked.stderr)
  );
  let first = fs::read_to_string(d.path().join(EXPORT)).unwrap();
  if kind == "mixed" {
    assert!(
      first.contains("normal_infinite_df_limit")
        && first.contains("rubin_student_t_large_complete_df")
    );
    let row = first.lines().find(|s| s.starts_with("x2missing,")).unwrap();
    assert_eq!(row.split(',').nth(10).unwrap().parse::<f64>().unwrap(), 0.0);
  }
  fs::remove_file(d.path().join(EXPORT)).unwrap();
  let text = run(d.path(), SCRIPT, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(
    String::from_utf8(text.stdout)
      .unwrap()
      .contains("no imputation")
  );
  assert_eq!(fs::read_to_string(d.path().join(EXPORT)).unwrap(), first);
  assert_eq!(
    fs::read_to_string(d.path().join("estimates.csv")).unwrap(),
    source
  );
}
#[test]
fn mixed_zero_and_positive_between_match_r() {
  success("mixed");
}
#[test]
fn two_imputations_use_small_fractional_student_df() {
  success("small");
}

#[test]
fn existing_linear_model_is_retained_but_not_a_pooled_model() {
  let d = TestDir::new();
  setup("small", d.path());
  fs::write(
    d.path().join("seed.csv"),
    "y,x\n1,1\n2.2,2\n3.1,3\n4.4,4\n5.2,5\n",
  )
  .unwrap();
  let script = format!(
    "use seed.csv\nregress y x\n{SCRIPT}\nuse seed.csv\npredict still_fit, xb\nexport retained.csv\n"
  );
  let output = run(d.path(), &script, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let status = &events
    .iter()
    .find(|e| e["result_type"] == "StatusResult")
    .unwrap()["data"];
  assert_eq!(status["active_table_name"], "rubin_marginal_report");
  assert_eq!(
    status["linear_model"]["predictors"],
    serde_json::json!(["x"])
  );
  let fit = &events
    .iter()
    .find(|e| e["result_type"] == "RegressionResult")
    .unwrap()["data"];
  let a = fit["coefficients"][0]["value"].as_f64().unwrap();
  let b = fit["coefficients"][1]["value"].as_f64().unwrap();
  let csv = fs::read_to_string(d.path().join("retained.csv")).unwrap();
  for line in csv.lines().skip(1) {
    let values: Vec<_> = line.split(',').map(|s| s.parse::<f64>().unwrap()).collect();
    assert!((values[2] - (a + b * values[1])).abs() < 1e-12);
  }
}

#[test]
fn audit_rejects_forged_source_json_and_pooled_results() {
  let d = TestDir::new();
  setup("small", d.path());
  let out = run(d.path(), SCRIPT, "--json");
  assert!(out.status.success());
  let original = String::from_utf8(out.stdout).unwrap();
  fs::write(d.path().join("out.jsonl"), &original).unwrap();
  assert!(audit(d.path(), &["--synthetic"]).status.success());
  let path = d.path().join(EXPORT);
  let csv = fs::read_to_string(&path).unwrap();
  for field in ["estimate", "degrees_freedom", "p_value", "between_variance"] {
    let mut lines: Vec<_> = csv.lines().map(str::to_owned).collect();
    let index = lines[0].split(',').position(|v| v == field).unwrap();
    let mut values: Vec<_> = lines[1].split(',').map(str::to_owned).collect();
    values[index] = "0.123456789".to_owned();
    lines[1] = values.join(",");
    fs::write(&path, lines.join("\n") + "\n").unwrap();
    assert!(
      !audit(d.path(), &["--synthetic"]).status.success(),
      "accepted forged {field}"
    );
  }
  fs::write(&path, &csv).unwrap();
  let mut events: Vec<Value> = original
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let table = events
    .iter_mut()
    .find(|e| e["result_type"] == "TableResult" && e["data"]["headers"][0] == "parameter")
    .unwrap();
  table["data"]["rows"][0][2] = Value::from(0.123456789);
  fs::write(
    d.path().join("out.jsonl"),
    events
      .iter()
      .map(Value::to_string)
      .collect::<Vec<_>>()
      .join("\n")
      + "\n",
  )
  .unwrap();
  assert!(
    !audit(d.path(), &["--synthetic"]).status.success(),
    "accepted forged JSON table"
  );
  fs::write(d.path().join("out.jsonl"), original).unwrap();
  let input = d.path().join("estimates.csv");
  let source = fs::read_to_string(&input).unwrap();
  fs::write(&input, source.replacen("-1,0.1", "-2,0.1", 1)).unwrap();
  assert!(
    !audit(d.path(), &["--synthetic"]).status.success(),
    "accepted forged input"
  );
  fs::write(d.path().join("fake.lst"), synthetic_log()).unwrap();
  assert!(
    !audit(
      d.path(),
      &["--author-log", d.path().join("fake.lst").to_str().unwrap()]
    )
    .status
    .success(),
    "accepted unpinned author log"
  );
}

fn prepare(dir: &Path, text: &str) -> Output {
  fs::write(dir.join("author.lst"), text).unwrap();
  Command::new("sh")
    .arg(root().join("examples/mma27p1milinear/prepare_pooling.sh"))
    .arg(dir.join("author.lst"))
    .arg(dir.join("estimates.csv"))
    .output()
    .unwrap()
}
fn synthetic_log() -> String {
  let mut s = String::new();
  for i in 1..=10 {
    s.push_str(&format!("----- Imputation Number={i} -----\n"));
    for (name, q) in [("Intercept", 1.0), ("x1missing", 2.0), ("x2missing", 3.0)] {
      s.push_str(&format!(" {name} 1 {q:.5} 0.50000 2.00 0.1234\n"));
    }
  }
  s
}
#[test]
fn converter_is_no_clobber_and_rejects_incomplete_or_duplicate_log() {
  let s = synthetic_log();
  let d = TestDir::new();
  let out = prepare(d.path(), &s);
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  assert_eq!(
    fs::read_to_string(d.path().join("estimates.csv"))
      .unwrap()
      .lines()
      .count(),
    31
  );
  fs::write(d.path().join("estimates.csv"), "owner\n").unwrap();
  assert!(!prepare(d.path(), &s).status.success());
  assert_eq!(
    fs::read_to_string(d.path().join("estimates.csv")).unwrap(),
    "owner\n"
  );
  for bad in [
    s.replace(" x2missing 1 3.00000 0.50000 2.00 0.1234\n", ""),
    format!("{s}----- Imputation Number=1 -----\n"),
    s.replacen("0.50000", "0.00000", 1),
  ] {
    let d = TestDir::new();
    let out = prepare(d.path(), &bad);
    assert!(!out.status.success());
    assert!(!d.path().join("estimates.csv").exists());
    assert_eq!(
      fs::read_to_string(d.path().join("author.lst")).unwrap(),
      bad
    );
    assert!(
      fs::read_dir(d.path()).unwrap().all(|e| !e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .contains(".tmp."))
    );
  }
}
#[test]
fn bad_input_and_collisions_do_not_clobber() {
  let source = fixture("small", "inputs");
  let lines: Vec<_> = source.lines().collect();
  let cases=[format!("{source}{}\n",lines[1]),format!("{}\n",lines[..lines.len()-1].join("\n")),source.replace("2,","3,"),
 source.replacen("intercept","wrong",1),source.replacen("-1,0.1","NaN,0.1",1),source.replacen("-1,0.1","-1,0",1),
 "imputation,parameter,estimate,standard_error\n1,intercept,1,1\n1,x1missing,1,1\n1,x2missing,1,1\n2,intercept,1.00001,1\n2,x1missing,2,1\n2,x2missing,2,1\n".to_owned(),
 "imputation,parameter,estimate,standard_error\n1,intercept,100,0.000000000001\n1,x1missing,1,1\n1,x2missing,1,1\n2,intercept,100,0.000000000001\n2,x1missing,2,1\n2,x2missing,2,1\n".to_owned()];
  for (i, bad) in cases.iter().enumerate() {
    let d = TestDir::new();
    fs::write(d.path().join("estimates.csv"), bad).unwrap();
    let out = run(d.path(), SCRIPT, "--json");
    assert!(
      !out.status.success(),
      "case {i} unexpectedly passed: {}",
      String::from_utf8_lossy(&out.stdout)
    );
    assert!(!d.path().join(EXPORT).exists());
    assert_eq!(
      fs::read_to_string(d.path().join("estimates.csv")).unwrap(),
      *bad
    );
  }
  let d = TestDir::new();
  setup("small", d.path());
  fs::write(d.path().join(EXPORT), "owner\n").unwrap();
  let out = run(d.path(), SCRIPT, "--json");
  assert!(!out.status.success());
  assert_eq!(
    fs::read_to_string(d.path().join(EXPORT)).unwrap(),
    "owner\n"
  );
  assert_eq!(
    fs::read_to_string(d.path().join("estimates.csv")).unwrap(),
    source
  );
}
