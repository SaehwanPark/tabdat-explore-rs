mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
};
const INPUT: &str = include_str!("fixtures/duration_exponential/input.csv");
fn example(name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma17p4duration")
    .join(name)
}
fn setup() -> TestDir {
  let d = TestDir::new();
  fs::write(d.path().join("duration.csv"), INPUT).unwrap();
  fs::create_dir(d.path().join("reference")).unwrap();
  for (file, text) in [
    ("robust.csv", include_str!("fixtures/cox/robust.csv")),
    (
      "predictions.csv",
      include_str!("fixtures/cox/predictions.csv"),
    ),
    (
      "parameter_names.txt",
      include_str!("fixtures/cox/parameter_names.txt"),
    ),
    (
      "score_residuals.csv",
      include_str!("fixtures/cox/score_residuals.csv"),
    ),
    (
      "source_hex.csv",
      include_str!("fixtures/duration_exponential/reference/source_hex.csv"),
    ),
  ] {
    fs::write(d.path().join("reference").join(file), text).unwrap();
  }
  d
}
fn run(d: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(example("cox.td"))
    .output()
    .unwrap()
}
fn audit(d: &TestDir) -> Output {
  Command::new("python3")
    .arg(example("check_cox.py"))
    .args([
      d.path().join("duration.csv"),
      d.path().join("native.json"),
      d.path().join("cox_predictions.csv"),
      d.path().join("reference"),
    ])
    .output()
    .unwrap()
}
fn success(r: &Output) {
  assert!(
    r.status.success(),
    "stdout={}\nstderr={}",
    String::from_utf8_lossy(&r.stdout),
    String::from_utf8_lossy(&r.stderr)
  );
}
fn change_csv(text: &str, column: &str, row: usize, value: &str) -> String {
  let mut lines = text.lines();
  let header = lines.next().unwrap();
  let index = header.split(',').position(|h| h == column).unwrap();
  let mut result = format!("{header}\n");
  for (i, line) in lines.enumerate() {
    let mut cells: Vec<_> = line.split(',').collect();
    if i == row {
      cells[index] = value;
    }
    result.push_str(&cells.join(","));
    result.push('\n');
  }
  result
}

#[test]
fn actual_cox_script_matches_r_full_inference_scores_baseline_points_and_both_cli_modes() {
  let d = setup();
  let result = run(&d, "--json");
  success(&result);
  fs::write(d.path().join("native.json"), &result.stdout).unwrap();
  let events: Vec<Value> = String::from_utf8_lossy(&result.stdout)
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  let fit = &events
    .iter()
    .find(|e| e["result_type"] == "CoxResult")
    .unwrap()["data"];
  assert_eq!(fit["observation_count"], 400);
  assert_eq!(fit["baseline"].as_array().unwrap().len(), 28);
  assert_eq!(fit["covariance_matrix"].as_array().unwrap().len(), 40);
  assert_eq!(fit["baseline_uncertainty"], "unavailable");
  assert_eq!(fit["ties"], "breslow");
  assert!(fit["likelihood_ratio_reference_distribution"].is_null());
  assert_eq!(
    fit["omnibus_wald_reference_distribution"],
    "asymptotic_chi_square"
  );
  let checked = audit(&d);
  success(&checked);
  assert!(
    String::from_utf8_lossy(&checked.stdout)
      .contains("2116 R gates/all1600 covariance entries, 22400 source/design/projection cells")
  );
  let text = setup();
  let result = run(&text, "--text");
  success(&result);
  let report = String::from_utf8_lossy(&result.stdout);
  for fragment in [
    "Cox proportional hazards (Breslow ties",
    "robust LR reference withheld",
    "full efficient martingale-score sandwich N/(N-1)",
    "Baseline reference (all retained covariate means",
    "Baseline uncertainty unavailable",
    "do not identify instantaneous hazard",
  ] {
    assert!(report.contains(fragment), "{fragment}");
  }
  assert_eq!(
    fs::read(d.path().join("cox_predictions.csv")).unwrap(),
    fs::read(text.path().join("cox_predictions.csv")).unwrap()
  );
  for (file, key) in [
    ("robust.csv", "/covariance_matrix/0/39"),
    ("robust.csv", "/baseline/27/cumulative_hazard"),
  ] {
    let path = d.path().join("reference").join(file);
    let before = fs::read_to_string(&path).unwrap();
    let changed = before
      .lines()
      .map(|l| {
        if l.starts_with(&format!("{key},")) {
          format!("{key},123456")
        } else {
          l.to_owned()
        }
      })
      .collect::<Vec<_>>()
      .join("\n")
      + "\n";
    fs::write(&path, changed).unwrap();
    assert!(!audit(&d).status.success(), "audit must detect {key}");
    fs::write(&path, before).unwrap();
  }
  let path = d.path().join("cox_predictions.csv");
  let before = fs::read_to_string(&path).unwrap();
  fs::write(
    &path,
    change_csv(&before, "ui1_cumulative_hazard", 399, "123456"),
  )
  .unwrap();
  assert!(!audit(&d).status.success());
}

#[test]
fn actual_cox_script_rejects_bad_source_before_fit_and_never_replaces_output() {
  for (column, value) in [
    ("source_row", "1"),
    ("spell", "0"),
    ("censor1", "2"),
    ("stateur", ""),
  ] {
    let d = setup();
    fs::write(
      d.path().join("duration.csv"),
      change_csv(INPUT, column, 399, value),
    )
    .unwrap();
    let result = run(&d, "--json");
    assert!(!result.status.success(), "{column}");
    assert!(!String::from_utf8_lossy(&result.stdout).contains("\"result_type\":\"CoxResult\""));
    assert!(!d.path().join("cox_predictions.csv").exists());
  }
  let d = setup();
  fs::write(d.path().join("cox_predictions.csv"), "do not replace\n").unwrap();
  let result = run(&d, "--json");
  assert!(!result.status.success());
  assert_eq!(
    fs::read_to_string(d.path().join("cox_predictions.csv")).unwrap(),
    "do not replace\n"
  );
  assert!(String::from_utf8_lossy(&result.stdout).contains("\"result_type\":\"CoxResult\""));
}

#[test]
fn cox_discovery_baseline_only_json_and_rejected_hazard_are_reachable() {
  let help = tabdat_bin()
    .args(["--json", "--help-topic", "stcox"])
    .output()
    .unwrap();
  success(&help);
  assert!(String::from_utf8_lossy(&help.stdout).contains("Breslow"));
  let schema = tabdat_bin()
    .args(["--json", "--describe-command", "stcox"])
    .output()
    .unwrap();
  success(&schema);
  assert!(String::from_utf8_lossy(&schema.stdout).contains("failure"));
  let d = TestDir::new();
  fs::write(d.path().join("base.csv"), "t,d\n1,1\n1,0\n2,1\n3,0\n").unwrap();
  let result = tabdat_bin()
    .current_dir(d.path())
    .args([
      "--json",
      "-c",
      "use base.csv",
      "-c",
      "stcox t, failure(d)",
      "-c",
      "predict s, survival",
      "-c",
      "predict bad, hazard",
      "-c",
      "generate unreachable = 1",
    ])
    .output()
    .unwrap();
  assert!(!result.status.success());
  let stdout = String::from_utf8_lossy(&result.stdout);
  let events: Vec<Value> = stdout
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  let fit = &events
    .iter()
    .find(|e| e["result_type"] == "CoxResult")
    .unwrap()["data"];
  assert!(fit["coefficients"].as_array().unwrap().is_empty());
  assert_eq!(fit["baseline"][0]["risk_count"], 4);
  assert_eq!(fit["baseline"][1]["cumulative_hazard"], 0.75);
  assert!(fit["likelihood_ratio_reference_distribution"].is_null());
  assert!(!stdout.contains("unreachable"));
  assert!(String::from_utf8_lossy(&result.stderr).contains("instantaneous hazard"));
}
