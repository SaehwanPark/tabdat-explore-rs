mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
};
const INPUT: &str = include_str!("fixtures/duration_exponential/input.csv");
const REF: &str = include_str!("fixtures/weibull/robust.csv");
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
    ("robust.csv", REF),
    (
      "predictions.csv",
      include_str!("fixtures/weibull/predictions.csv"),
    ),
    (
      "parameter_names.txt",
      include_str!("fixtures/weibull/parameter_names.txt"),
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
    .arg(example("weibull.td"))
    .output()
    .unwrap()
}
fn audit(d: &TestDir) -> Output {
  Command::new("python3")
    .arg(example("check_weibull.py"))
    .args([
      d.path().join("duration.csv"),
      d.path().join("native.json"),
      d.path().join("weibull_predictions.csv"),
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
fn change_csv(text: &str, column: &str, mut change: impl FnMut(usize, &str) -> String) -> String {
  let mut lines = text.lines();
  let header = lines.next().unwrap();
  let j = header.split(',').position(|s| s == column).unwrap();
  let mut result = format!("{header}\n");
  for (i, line) in lines.enumerate() {
    let mut cells: Vec<String> = line.split(',').map(str::to_owned).collect();
    cells[j] = change(i, &cells[j]);
    result.push_str(&cells.join(","));
    result.push('\n');
  }
  result
}
fn change_gate(text: &str, key: &str, value: f64) -> String {
  text
    .lines()
    .map(|line| match line.split_once(',') {
      Some((p, _)) if p == key => format!("{key},{value}"),
      _ => line.to_owned(),
    })
    .collect::<Vec<_>>()
    .join("\n")
    + "\n"
}

#[test]
fn actual_weibull_script_matches_full_joint_r_in_both_cli_modes_and_restores_source() {
  let d = setup();
  let result = run(&d, "--json");
  success(&result);
  fs::write(d.path().join("native.json"), &result.stdout).unwrap();
  let fit: Value = String::from_utf8_lossy(&result.stdout)
    .lines()
    .map(|s| serde_json::from_str::<Value>(s).unwrap())
    .find(|e| e["result_type"] == "WeibullResult")
    .unwrap()["data"]
    .clone();
  assert_eq!(fit["observation_count"], 400);
  assert_eq!(
    fit["covariance_parameter_names"].as_array().unwrap().len(),
    42
  );
  assert_eq!(fit["likelihood_ratio_p_value"], Value::Null);
  assert!(fit.get("fitted_survival").is_none());
  assert!(fit.get("predictor_names").is_none());
  let checked = audit(&d);
  success(&checked);
  assert!(
    String::from_utf8_lossy(&checked.stdout)
      .contains("2031 R gates/all1764 joint covariance entries, 23600")
  );
  let bytes = fs::read(d.path().join("weibull_predictions.csv")).unwrap();
  fs::remove_file(d.path().join("weibull_predictions.csv")).unwrap();
  let text = run(&d, "--text");
  success(&text);
  let report = String::from_utf8_lossy(&text.stdout);
  assert!(report.contains("Weibull proportional-hazards maximum likelihood"));
  assert!(report.contains("robust LR reference withheld"));
  assert!(report.contains("No shape=0 Wald test"));
  assert!(report.contains("NOT all-cause joblessness survival"));
  assert_eq!(
    fs::read(d.path().join("weibull_predictions.csv")).unwrap(),
    bytes
  );
  assert_eq!(
    fs::read_to_string(d.path().join("duration.csv")).unwrap(),
    INPUT
  );
}

#[test]
fn weibull_shared_wrong_shape_cross_covariance_tails_and_survival_cannot_pass_by_r_comparison_only()
{
  let d = setup();
  let output = run(&d, "--json");
  success(&output);
  let baseline = String::from_utf8(output.stdout).unwrap();
  for path in [
    "/coefficients/0/p_value",
    "/shape/log_p_value",
    "/omnibus_wald/p_value",
    "/covariance_matrix/0/41",
  ] {
    let mut events: Vec<Value> = baseline
      .lines()
      .map(|s| serde_json::from_str(s).unwrap())
      .collect();
    let data = events
      .iter_mut()
      .find(|e| e["result_type"] == "WeibullResult")
      .unwrap()
      .get_mut("data")
      .unwrap();
    let wrong = if path.contains("covariance_matrix") {
      let old = data.pointer(path).unwrap().as_f64().unwrap();
      assert_ne!(old, 0.0);
      old * 2.0
    } else {
      0.0
    };
    *data.pointer_mut(path).unwrap() = Value::from(wrong);
    let mut reference = change_gate(REF, path, wrong);
    if path.contains("covariance_matrix") {
      // This cross-shape forgery preserves all diagonal SEs/normal and slope Wald.
      data["covariance_matrix"][41][0] = Value::from(wrong);
      reference = change_gate(&reference, "/covariance_matrix/41/0", wrong);
    }
    let text = events
      .iter()
      .map(|e| serde_json::to_string(e).unwrap())
      .collect::<Vec<_>>()
      .join("\n")
      + "\n";
    fs::write(d.path().join("native.json"), text).unwrap();
    fs::write(d.path().join("reference/robust.csv"), reference).unwrap();
    assert!(!audit(&d).status.success(), "shared wrong {path} accepted");
  }
  fs::write(d.path().join("native.json"), baseline).unwrap();
  fs::write(d.path().join("reference/robust.csv"), REF).unwrap();
  for path in [
    d.path().join("weibull_predictions.csv"),
    d.path().join("reference/predictions.csv"),
  ] {
    let text = fs::read_to_string(&path).unwrap();
    fs::write(
      &path,
      change_csv(&text, "net_survival", |i, v| {
        if i == 0 { "0".into() } else { v.into() }
      }),
    )
    .unwrap();
  }
  assert!(
    !audit(&d).status.success(),
    "shared wrong net survival accepted"
  );
}

#[test]
fn weibull_cli_rejects_original_nonbinary_decimals_and_stops_before_prediction_export() {
  let names = include_str!("fixtures/weibull/parameter_names.txt")
    .lines()
    .skip(1)
    .filter(|name| *name != "lnshape")
    .collect::<Vec<_>>()
    .join(" ");
  for mode in ["--json", "--text"] {
    for value in ["1.000000000000000001", "0.999999999999999999"] {
      let d = setup();
      let script = format!(
        "use duration.csv\nsql \"\"\"SELECT *,reprate*ui AS rr_ui,disrate*ui AS dr_ui,CASE WHEN censor1=1 THEN CAST('{value}' AS DECIMAL(38,18)) ELSE CAST(censor1 AS DECIMAL(38,18)) END AS event_decimal FROM active\"\"\" into decimal_events\nstreg spell {names}, failure(event_decimal) dist(weibull) robust\npredict unreachable, survival\nexport must_not_exist.csv\n"
      );
      fs::write(d.path().join("decimal.td"), script).unwrap();
      let output = tabdat_bin()
        .current_dir(d.path())
        .args([mode, "-f", "decimal.td"])
        .output()
        .unwrap();
      assert_eq!(output.status.code(), Some(1), "{mode}, {value}");
      let stdout = String::from_utf8_lossy(&output.stdout);
      let stderr = String::from_utf8_lossy(&output.stderr);
      assert!(stderr.contains("exactly0 or1"), "{stderr}");
      assert!(!stdout.contains("\"result_type\":\"WeibullResult\""));
      assert!(!stdout.contains("Weibull proportional-hazards maximum likelihood"));
      assert!(!d.path().join("must_not_exist.csv").exists());
      assert_eq!(
        fs::read_to_string(d.path().join("duration.csv")).unwrap(),
        INPUT
      );
      if mode == "--json" {
        let events: Vec<Value> = stdout
          .lines()
          .map(|line| serde_json::from_str(line).unwrap())
          .collect();
        assert!(events.iter().any(|e| {
          e["error"]["message"]
            .as_str()
            .is_some_and(|s| s.contains("exactly0 or1"))
        }));
      }
    }
  }
}

#[test]
fn weibull_script_guard_and_export_failure_preserve_inputs_and_existing_files() {
  for (column, value) in [
    ("spell", "0"),
    ("spell", "-1"),
    ("spell", "NaN"),
    ("censor1", "2"),
    ("ui", "2"),
    ("stateur", ""),
    ("source_row", "1"),
  ] {
    let d = setup();
    let text = change_csv(INPUT, column, |i, old| {
      if i == 1 { value.into() } else { old.into() }
    });
    fs::write(d.path().join("duration.csv"), &text).unwrap();
    let output = run(&d, "--json");
    assert!(!output.status.success(), "accepted {column}={value}");
    assert!(!d.path().join("weibull_predictions.csv").exists());
    assert_eq!(
      fs::read_to_string(d.path().join("duration.csv")).unwrap(),
      text
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("\"result_type\":\"WeibullResult\""));
  }
  let d = setup();
  fs::write(
    d.path().join("weibull_predictions.csv"),
    "existing output\n",
  )
  .unwrap();
  let output = run(&d, "--json");
  assert!(!output.status.success());
  assert_eq!(
    fs::read_to_string(d.path().join("weibull_predictions.csv")).unwrap(),
    "existing output\n"
  );
  assert_eq!(
    fs::read_to_string(d.path().join("duration.csv")).unwrap(),
    INPUT
  );
}
