mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
};

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn prepare(dir: &TestDir, source: &str, dest: &str) -> Output {
  fs::write(dir.path().join("source.asc"), source).unwrap();
  Command::new("sh")
    .current_dir(dir.path())
    .arg(root().join("examples/mma23p1pannonlin/prepare.sh"))
    .args(["source.asc", dest])
    .output()
    .unwrap()
}
fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(root().join("examples/mma23p1pannonlin/pooled_poisson.td"))
    .output()
    .unwrap()
}
fn close(actual: f64, expected: f64, label: &str) {
  let tolerance = if label.ends_with("p_value") {
    (expected.abs() * 1e-8).max(f64::MIN_POSITIVE)
  } else {
    1e-9 + 1e-9 * expected.abs()
  };
  assert!(
    actual.is_finite() && (actual - expected).abs() <= tolerance,
    "{label}: {actual} != {expected}, tolerance {tolerance}"
  );
}

#[test]
fn synthetic_reference_text_json_and_non_clobber() {
  let d = TestDir::new();
  let source = include_str!("fixtures/mma23_pooled_poisson/source.asc");
  let prep = prepare(&d, source, "patents.csv");
  assert!(
    prep.status.success(),
    "{}",
    String::from_utf8_lossy(&prep.stderr)
  );
  let output = run(&d, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fits: Vec<_> = events
    .iter()
    .filter(|e| e["result_type"] == "PoissonResult")
    .map(|e| &e["data"])
    .collect();
  let effects: Vec<_> = events
    .iter()
    .filter(|e| e["result_type"] == "PoissonMarginalEffectsResult")
    .map(|e| &e["data"])
    .collect();
  assert_eq!(fits.len(), 3);
  assert_eq!(effects.len(), 3);
  for (i, mode) in [
    "model_based_poisson_information",
    "robust_sandwich_n_over_n_minus_1",
    "cluster_sandwich_g_over_g_minus_1",
  ]
  .iter()
  .enumerate()
  {
    assert_eq!(fits[i]["covariance"], *mode);
    assert_eq!(effects[i]["covariance"], *mode);
    assert_eq!(fits[i]["observation_count"], 200);
    assert_eq!(fits[i]["sample"]["dropped_observations"], 0);
    assert_eq!(fits[i]["predictors"], serde_json::json!(["logr"]));
  }
  assert_eq!(fits[2]["cluster"]["group_count"], 40);
  let mut count = 0;
  for line in include_str!("fixtures/mma23_pooled_poisson/reference.csv")
    .lines()
    .skip(1)
  {
    let fields: Vec<_> = line.split(',').collect();
    let idx = match fields[0] {
      "classical" => 0,
      "robust" => 1,
      "cluster" => 2,
      _ => panic!(),
    };
    let q = fields[1];
    let i: usize = fields[2].parse().unwrap();
    let j: usize = fields[3].parse().unwrap();
    let actual = if let Some(q) = q.strip_prefix("coefficient_") {
      &fits[idx]["coefficients"][i][q]
    } else if let Some(q) = q.strip_prefix("effect_") {
      &effects[idx]["effects"][i][q]
    } else if q == "covariance" {
      &fits[idx]["covariance_matrix"][i][j]
    } else if let Some(q) = q.strip_prefix("wald_") {
      &fits[idx]["omnibus_wald"][q]
    } else {
      &fits[idx][q]
    };
    close(actual.as_f64().unwrap(), fields[4].parse().unwrap(), q);
    count += 1;
  }
  assert_eq!(count, 103);
  let path = d.path().join("pooled_poisson_predictions.csv");
  let bytes = fs::read(&path).unwrap();
  let actual = String::from_utf8(bytes.clone()).unwrap();
  let expected = include_str!("fixtures/mma23_pooled_poisson/predictions.csv");
  let a: Vec<_> = actual.lines().skip(1).collect();
  let e: Vec<_> = expected.lines().skip(1).collect();
  assert_eq!(a.len(), 200);
  assert_eq!(a.len(), e.len());
  for (a, e) in a.iter().zip(e) {
    let a: Vec<_> = a.split(',').collect();
    let e: Vec<_> = e.split(',').collect();
    assert_eq!(a.len(), 11);
    assert_eq!(a.len(), e.len());
    for j in 0..11 {
      if e[j].is_empty() {
        assert_eq!(a[j], "");
      } else {
        close(a[j].parse().unwrap(), e[j].parse().unwrap(), "prediction");
      }
    }
  }
  assert!(!run(&d, "--json").status.success());
  assert_eq!(fs::read(&path).unwrap(), bytes);
  fs::remove_file(&path).unwrap();
  let text = run(&d, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert_eq!(fs::read(&path).unwrap(), bytes);
}

#[test]
fn preparation_and_workflow_reject_invalid_inputs_without_publication() {
  let source = include_str!("fixtures/mma23_pooled_poisson/source.asc");
  for bad in [
    String::new(),
    "1 2 3".into(),
    source.replacen("1001", "NaN", 1),
    source.replacen("1001", "1001.5", 1),
    format!("{source}\n1"),
    {
      let mut fields: Vec<_> = source.split_whitespace().map(str::to_owned).collect();
      fields[20] = "-1".into();
      fields.join(" ")
    },
    {
      let mut fields: Vec<_> = source.split_whitespace().map(str::to_owned).collect();
      fields[10] = "1e9999".into();
      fields.join(" ")
    },
  ] {
    let d = TestDir::new();
    let out = prepare(&d, &bad, "patents.csv");
    assert!(!out.status.success(), "accepted invalid source");
    assert!(!d.path().join("patents.csv").exists());
    assert_eq!(fs::read_dir(d.path()).unwrap().count(), 1);
  }
  let d = TestDir::new();
  fs::write(d.path().join("patents.csv"), "sentinel").unwrap();
  assert!(!prepare(&d, source, "patents.csv").status.success());
  assert_eq!(
    fs::read_to_string(d.path().join("patents.csv")).unwrap(),
    "sentinel"
  );
  fs::remove_file(d.path().join("patents.csv")).unwrap();
  assert!(prepare(&d, source, "patents.csv").status.success());
  let path = d.path().join("patents.csv");
  let csv = fs::read_to_string(&path).unwrap();
  let mut lines: Vec<_> = csv.lines().map(str::to_owned).collect();
  lines[2] = lines[1].clone();
  fs::write(&path, lines.join("\n") + "\n").unwrap();
  let before = fs::read(&path).unwrap();
  assert!(!run(&d, "--json").status.success());
  assert!(!d.path().join("pooled_poisson_predictions.csv").exists());
  assert_eq!(fs::read(&path).unwrap(), before);
  fs::write(
    &path,
    csv.lines().take(200).collect::<Vec<_>>().join("\n") + "\n",
  )
  .unwrap();
  let before = fs::read(&path).unwrap();
  assert!(!run(&d, "--json").status.success());
  assert!(!d.path().join("pooled_poisson_predictions.csv").exists());
  assert_eq!(fs::read(&path).unwrap(), before);
}
