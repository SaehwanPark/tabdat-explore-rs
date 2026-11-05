mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
  process::{Command, Output},
};

const SCRIPT: &str = include_str!("../examples/mma27p1milinear/baseline.td");
const EXPORT: &str = "missing_comparison_predictions.csv";
fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn fixture(kind: &str, name: &str) -> String {
  let stem = if name == "asc" {
    format!("{kind}.asc")
  } else {
    format!("{kind}_{name}.csv")
  };
  fs::read_to_string(root().join("tests/fixtures/mma27p1milinear").join(stem)).unwrap()
}
fn prepare(dir: &Path, input: &str) -> Output {
  fs::write(dir.join("input.asc"), input).unwrap();
  Command::new("sh")
    .arg(root().join("examples/mma27p1milinear/prepare.sh"))
    .arg(dir.join("input.asc"))
    .arg(dir.join("missing.csv"))
    .output()
    .unwrap()
}
fn execute(dir: &Path, script: &str, mode: &str) -> Output {
  fs::write(dir.join("script.td"), script).unwrap();
  tabdat_bin()
    .current_dir(dir)
    .args([mode, "-f", "script.td"])
    .output()
    .unwrap()
}
fn reference(dir: &Path, kind: &str) {
  fs::create_dir(dir.join("reference")).unwrap();
  for name in ["fit", "diagnostics", "predictions"] {
    fs::write(
      dir.join("reference").join(format!("{name}.csv")),
      fixture(kind, name),
    )
    .unwrap();
  }
}
fn audit(dir: &Path) -> Output {
  Command::new("python3")
    .arg(root().join("examples/mma27p1milinear/check_reference.py"))
    .arg(dir)
    .arg(dir.join("reference"))
    .arg(dir.join("input.asc"))
    .arg("--synthetic")
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let keys: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let values: Vec<_> = line.split(',').collect();
      assert_eq!(keys.len(), values.len());
      keys.iter().copied().zip(values).collect()
    })
    .collect()
}
fn check_success(kind: &str) {
  let d = TestDir::new();
  let source = fixture(kind, "asc");
  let p = prepare(d.path(), &source);
  assert!(p.status.success(), "{}", String::from_utf8_lossy(&p.stderr));
  reference(d.path(), kind);
  let script = format!("{SCRIPT}\npredict retained_mean, xb\nexport retained.csv\n");
  let json = execute(d.path(), &script, "--json");
  assert!(
    json.status.success(),
    "{}",
    String::from_utf8_lossy(&json.stderr)
  );
  fs::write(d.path().join("out.jsonl"), &json.stdout).unwrap();
  let check = audit(d.path());
  assert!(
    check.status.success(),
    "{}",
    String::from_utf8_lossy(&check.stderr)
  );
  let events: Vec<Value> = String::from_utf8(json.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  assert_eq!(
    events
      .iter()
      .filter(|e| e["result_type"] == "RegressionResult")
      .count(),
    2
  );
  let data = fs::read_to_string(d.path().join(EXPORT)).unwrap();
  let retained = fs::read_to_string(d.path().join("retained.csv")).unwrap();
  let all = rows(&retained);
  assert_eq!(all.len(), 20);
  for row in &all {
    let model: f64 = row["retained_mean"].parse().unwrap();
    let fitted: f64 = row["filled_mean"].parse().unwrap();
    assert!((model - fitted).abs() < 1e-12);
    if kind == "complete" {
      assert!((row["complete_mean"].parse::<f64>().unwrap() - fitted).abs() < 1e-12);
    }
  }
  if kind == "mixed" {
    let complete: Vec<_> = all
      .iter()
      .filter(|r| r["x1_missing"] == "0" && r["x2_missing"] == "0")
      .collect();
    for (variable, missing, output) in [
      ("x1", "x1_missing", "x1_filled"),
      ("x2", "x2_missing", "x2_filled"),
    ] {
      let complete_mean = complete
        .iter()
        .map(|r| r[variable].parse::<f64>().unwrap())
        .sum::<f64>()
        / complete.len() as f64;
      let filled = all.iter().find(|r| r[missing] == "1").unwrap()[output]
        .parse::<f64>()
        .unwrap();
      assert!(
        (filled - complete_mean).abs() > 1e-3,
        "{variable} must use its observed sample, not complete cases"
      );
    }
  }
  fs::remove_file(d.path().join(EXPORT)).unwrap();
  fs::remove_file(d.path().join("retained.csv")).unwrap();
  let text = execute(d.path(), &script, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(
    String::from_utf8(text.stdout)
      .unwrap()
      .contains("not multiple imputation")
  );
  assert_eq!(fs::read_to_string(d.path().join(EXPORT)).unwrap(), data);
  assert_eq!(
    fs::read_to_string(d.path().join("retained.csv")).unwrap(),
    retained
  );
  assert_eq!(
    fs::read_to_string(d.path().join("input.asc")).unwrap(),
    source
  );
}
#[test]
fn mixed_missingness_matches_r_and_retains_filled_fit() {
  check_success("mixed");
}
#[test]
fn fully_observed_models_coincide() {
  check_success("complete");
}

#[test]
fn converter_rejects_bad_input_without_publishing_or_clobbering() {
  for input in [
    "",
    "1 2\n",
    "1 2 3 4\n",
    ". 1 2\n",
    "1 NaN 2\n",
    "1 1e999 2\n",
    "1 1000001 2\n",
    "1 2 3\n2 nope 4\n",
  ] {
    let d = TestDir::new();
    let out = prepare(d.path(), input);
    assert!(!out.status.success(), "accepted {input:?}");
    assert!(!d.path().join("missing.csv").exists());
    assert_eq!(
      fs::read_to_string(d.path().join("input.asc")).unwrap(),
      input
    );
    assert!(
      fs::read_dir(d.path()).unwrap().all(|e| !e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .contains(".tmp."))
    );
  }
  let d = TestDir::new();
  fs::write(d.path().join("missing.csv"), "owner\n").unwrap();
  assert!(!prepare(d.path(), "1 2 3\n").status.success());
  assert_eq!(
    fs::read_to_string(d.path().join("missing.csv")).unwrap(),
    "owner\n"
  );
}

#[test]
fn rejected_panels_and_export_collision_preserve_source_and_owners() {
  let input = fixture("mixed", "asc");
  let d = TestDir::new();
  assert!(prepare(d.path(), &input).status.success());
  let csv = fs::read_to_string(d.path().join("missing.csv")).unwrap();
  let lines: Vec<_> = csv.lines().collect();
  let mut cases = Vec::new();
  cases.push(format!("{}\n{}\n", lines.join("\n"), lines[1]));
  cases.push("source_row,y,x1,x2\n1,,1,2\n2,2,2,3\n3,3,3,4\n4,4,4,5\n".to_owned());
  cases.push("source_row,y,x1,x2\n1,1,,2\n2,2,,3\n3,3,,4\n4,4,,5\n".to_owned());
  cases.push("source_row,y,x1,x2\n1,1,1,2\n2,2,2,3\n3,3,3,4\n4,4,,5\n".to_owned());
  cases.push("source_row,y,x1,x2\n1,1,1,2\n2,2,2,4\n3,4,3,6\n4,3,4,8\n5,7,5,10\n".to_owned());
  cases
    .push("source_row,y,x1,x2\n1,1,0,0\n2,3,1,0\n3,4,0,1\n4,6,1,1\n5,8,2,1\n6,10,0,3\n".to_owned());
  cases.push(
    "source_row,y,x1,x2\n1,1,0,0\n2,3,1,0\n3,4,0,1\n4,6,1,1\n5,8,2,1\n6,10.00000001,0,3\n"
      .to_owned(),
  );
  cases.push("source_row,y,x1,x2\n1,1,1,2.0000001\n2,2,2,4\n3,4,3,6.0000002\n4,3,4,8\n5,7,5,9.9999998\n6,5,6,12\n".to_owned());
  cases.push(csv.replacen(lines[1], "1,1000001,1,2", 1));
  cases.push(csv.replacen(lines[1], "1,NaN,1,2", 1));
  cases.push(csv.replacen(lines[1], "1,1,NaN,2", 1));
  for (i, bad) in cases.iter().enumerate() {
    let d = TestDir::new();
    fs::write(d.path().join("missing.csv"), bad).unwrap();
    let out = execute(d.path(), SCRIPT, "--json");
    assert!(
      !out.status.success(),
      "bad case {i} succeeded: {}",
      String::from_utf8_lossy(&out.stdout)
    );
    assert!(!d.path().join(EXPORT).exists());
    assert!(
      !String::from_utf8_lossy(&out.stdout).contains("RegressionResult"),
      "case {i} printed inference before rejection"
    );
    assert_eq!(
      fs::read_to_string(d.path().join("missing.csv")).unwrap(),
      *bad
    );
  }
  let d = TestDir::new();
  assert!(prepare(d.path(), &input).status.success());
  fs::write(d.path().join(EXPORT), "owner\n").unwrap();
  let out = execute(d.path(), SCRIPT, "--json");
  assert!(!out.status.success());
  assert_eq!(
    fs::read_to_string(d.path().join(EXPORT)).unwrap(),
    "owner\n"
  );
  assert_eq!(
    String::from_utf8(out.stdout)
      .unwrap()
      .matches("RegressionResult")
      .count(),
    2
  );
  assert_eq!(
    fs::read_to_string(d.path().join("input.asc")).unwrap(),
    input
  );
}

#[test]
fn source_bound_audit_rejects_forged_models_and_rows() {
  let d = TestDir::new();
  assert!(prepare(d.path(), &fixture("mixed", "asc")).status.success());
  reference(d.path(), "mixed");
  let out = execute(d.path(), SCRIPT, "--json");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let original = String::from_utf8(out.stdout).unwrap();
  fs::write(d.path().join("out.jsonl"), &original).unwrap();
  assert!(audit(d.path()).status.success());
  for field in ["value", "p_value", "covariance"] {
    let mut events: Vec<Value> = original
      .lines()
      .map(|s| serde_json::from_str(s).unwrap())
      .collect();
    let fit = events
      .iter_mut()
      .find(|e| e["result_type"] == "RegressionResult")
      .unwrap();
    if field == "covariance" {
      fit["data"]["covariance_matrix"][0][1] = Value::from(0.123456789);
    } else {
      fit["data"]["coefficients"][1][field] = Value::from(0.123456789);
    }
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
      !audit(d.path()).status.success(),
      "accepted forged fit {field}"
    );
  }
  fs::write(d.path().join("out.jsonl"), original).unwrap();
  let path = d.path().join(EXPORT);
  let baseline = fs::read_to_string(&path).unwrap();
  for column in [
    "y",
    "x1_missing",
    "x1_filled",
    "complete_mean",
    "filled_residual",
  ] {
    let mut lines: Vec<String> = baseline.lines().map(str::to_owned).collect();
    let col = lines[0].split(',').position(|s| s == column).unwrap();
    let mut values: Vec<String> = lines[1].split(',').map(str::to_owned).collect();
    values[col] = "0.123456789".to_owned();
    lines[1] = values.join(",");
    fs::write(&path, lines.join("\n") + "\n").unwrap();
    assert!(
      !audit(d.path()).status.success(),
      "accepted forged {column}"
    );
  }
  fs::write(&path, &baseline).unwrap();
  let reference = d.path().join("reference/predictions.csv");
  for p in [&path, &reference] {
    let text = fs::read_to_string(p).unwrap();
    let mut lines: Vec<_> = text.lines().map(str::to_owned).collect();
    let col = lines[0]
      .split(',')
      .position(|s| s.trim_matches('"') == "y")
      .unwrap();
    let mut vals: Vec<_> = lines[1].split(',').map(str::to_owned).collect();
    vals[col] = "0.123456789".to_owned();
    lines[1] = vals.join(",");
    fs::write(p, lines.join("\n") + "\n").unwrap();
  }
  assert!(
    !audit(d.path()).status.success(),
    "accepted source forgery shared by native and R files"
  );
}
