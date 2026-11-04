mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};

const INPUT: &str = include_str!("fixtures/mma21_fe_level_bootstrap/heterogeneous_inputs.csv");
const OUTPUT: &str = "panel_classical_fe_intercept_predictions.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma21p1panel/classical_intercept.td"),
    )
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let header: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|l| {
      let v: Vec<_> = l.split(',').collect();
      assert_eq!(v.len(), header.len());
      header.iter().copied().zip(v).collect()
    })
    .collect()
}
#[test]
fn classical_intercept_script_matches_full_dummy_qr_and_text_predictions() {
  let dir = TestDir::new();
  fs::write(dir.path().join("MOM.csv"), INPUT).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  let fit = &events
    .iter()
    .find(|e| e["result_type"] == "FixedEffectsResult")
    .unwrap()["data"];
  let combinations: Vec<_> = events
    .iter()
    .filter(|e| e["result_type"] == "LincomResult")
    .map(|e| &e["data"])
    .collect();
  assert_eq!(combinations.len(), 2);
  let joint = &events
    .iter()
    .find(|e| e["result_type"] == "TestResult")
    .unwrap()["data"];
  assert_eq!(fit["common_intercept_inference"], "classical_student_t");
  assert_eq!(
    fit["covariance_parameter_names"],
    serde_json::json!(["lnwg"])
  );
  assert_eq!(
    fit["normalized_level"]["parameter_names"],
    serde_json::json!(["_cons", "lnwg"])
  );
  assert_eq!(fit["coefficients"].as_array().unwrap().len(), 1);
  assert_eq!(joint["is_chi2"], false);
  let reference = rows(include_str!(
    "fixtures/mma21_classical_intercept/reference.csv"
  ));
  assert_eq!(reference.len(), 39);
  for row in reference {
    let data = match row["kind"] {
      "fit" => fit,
      "common" => combinations[0],
      "mean" => combinations[1],
      "joint" => joint,
      _ => panic!(),
    };
    let actual = data.pointer(row["path"]).unwrap().as_f64().unwrap();
    let expected: f64 = row["value"].parse().unwrap();
    let tolerance = if row["path"].ends_with("p_value") {
      expected.abs() * 1e-8
    } else if row["kind"] == "joint" && row["path"] == "/statistic" {
      1e-9 + 1e-10 * expected.abs()
    } else {
      1e-9
    };
    assert!(
      actual.is_finite() && (actual - expected).abs() <= tolerance,
      "{row:?}: {actual} != {expected}"
    );
  }
  let json = fs::read_to_string(dir.path().join(OUTPUT)).unwrap();
  let actual = rows(&json);
  let expected = rows(include_str!(
    "fixtures/mma21_classical_intercept/predictions.csv"
  ));
  assert_eq!(actual.len(), 160);
  assert_eq!(actual.len(), expected.len());
  for (a, b) in actual.iter().zip(expected) {
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for (key, expected) in b {
      let x: f64 = a[key].parse().unwrap();
      let y: f64 = expected.parse().unwrap();
      if key.starts_with("fe_") {
        assert!((x - y).abs() < 1e-9, "{key}: {x} != {y}");
      } else {
        assert_eq!(x, y, "raw source {key}");
      }
    }
  }
  fs::remove_file(dir.path().join(OUTPUT)).unwrap();
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let stdout = String::from_utf8(text.stdout).unwrap();
  assert!(
    stdout.contains("Common intercept _cons (classical Student-t")
      && stdout.contains("95% CI lower")
  );
  assert_eq!(fs::read_to_string(dir.path().join(OUTPUT)).unwrap(), json);
}
#[test]
fn classical_common_inference_is_discoverable_in_embedded_help() {
  use std::{io::Write, process::Stdio};
  for topic in ["xtreg", "lincom", "test"] {
    let mut child = tabdat_bin()
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    child
      .stdin
      .take()
      .unwrap()
      .write_all(format!("help {topic}\nexit\n").as_bytes())
      .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
      output.status.success(),
      "{topic}: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(
      text.contains("_cons") && text.contains("N-G-p"),
      "{topic}: {text}"
    );
  }
}

#[test]
fn degenerate_classical_common_report_has_null_tests_and_point_interval() {
  let dir = TestDir::new();
  let mut input = String::from("id,year,y,x\n");
  for id in 1..=3 {
    for t in 0..4 {
      input.push_str(&format!("{id},{},{},{t}\n", 1979 + t, 5 * id + 2 * t));
    }
  }
  fs::write(dir.path().join("data.csv"), input).unwrap();
  let path = dir.path().join("zero.td");
  fs::write(
    &path,
    "use data.csv\nxtreg y x, fe id(id) time(year)\nlincom _cons\n",
  )
  .unwrap();
  let output = tabdat_bin()
    .current_dir(dir.path())
    .args(["--json", "-f"])
    .arg(path)
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  let common = &events
    .iter()
    .find(|e| e["result_type"] == "FixedEffectsResult")
    .unwrap()["data"]["common_intercept_statistics"];
  let lincom = &events
    .iter()
    .find(|e| e["result_type"] == "LincomResult")
    .unwrap()["data"];
  for value in [common, lincom] {
    assert_eq!(value["standard_error"], 0.0);
    assert!(value["statistic"].is_null() && value["p_value"].is_null());
    assert_eq!(value["ci_lower"], 10.0);
    assert_eq!(value["ci_upper"], 10.0);
  }
}

#[test]
fn classical_intercept_script_preserves_source_and_collision_owner() {
  let lines: Vec<_> = INPUT.lines().collect();
  let mut missing: Vec<_> = lines[1].split(',').collect();
  missing[1] = "";
  for input in [
    format!("{INPUT}{}\n", lines[1]),
    format!("{}\n", lines[..lines.len() - 1].join("\n")),
    format!(
      "{}\n{}\n{}\n",
      lines[0],
      missing.join(","),
      lines[2..].join("\n")
    ),
  ] {
    let dir = TestDir::new();
    fs::write(dir.path().join("MOM.csv"), &input).unwrap();
    let output = run(&dir, "--json");
    assert_eq!(output.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("FixedEffectsResult"));
    assert!(!dir.path().join(OUTPUT).exists());
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      input
    );
  }
  let dir = TestDir::new();
  fs::write(dir.path().join("MOM.csv"), INPUT).unwrap();
  fs::write(dir.path().join(OUTPUT), "original owner\n").unwrap();
  let output = run(&dir, "--json");
  assert_eq!(output.status.code(), Some(1));
  assert_eq!(
    fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
    "original owner\n"
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    INPUT
  );
}
