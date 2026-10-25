mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
};

const INPUT: &str = include_str!("fixtures/duration_exponential/input.csv");
const REFERENCES: &[(&str, &str)] = &[
  (
    "reference.csv",
    include_str!("fixtures/duration_exponential/reference/reference.csv"),
  ),
  (
    "predictions.csv",
    include_str!("fixtures/duration_exponential/reference/predictions.csv"),
  ),
  (
    "source_hex.csv",
    include_str!("fixtures/duration_exponential/reference/source_hex.csv"),
  ),
  (
    "parameter_names.txt",
    include_str!("fixtures/duration_exponential/reference/parameter_names.txt"),
  ),
];
fn example(name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma17p4duration")
    .join(name)
}
fn run(d: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(example("exponential.td"))
    .output()
    .unwrap()
}
fn setup() -> TestDir {
  let d = TestDir::new();
  fs::write(d.path().join("duration.csv"), INPUT).unwrap();
  fs::create_dir(d.path().join("reference")).unwrap();
  for (file, text) in REFERENCES {
    fs::write(d.path().join("reference").join(file), text).unwrap();
  }
  d
}
fn fit(output: &Output) -> Value {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(|line| serde_json::from_str::<Value>(line).unwrap())
    .find(|e| e["result_type"] == "PoissonResult")
    .unwrap()["data"]
    .clone()
}
fn audit(d: &TestDir) -> Output {
  Command::new("python3")
    .arg(example("check_reference.py"))
    .arg(d.path().join("duration.csv"))
    .arg(d.path().join("native.json"))
    .arg(d.path().join("exponential_predictions.csv"))
    .arg(d.path().join("reference"))
    .output()
    .unwrap()
}
fn map_csv(text: &str, column: &str, mut change: impl FnMut(usize, &str) -> String) -> String {
  let mut lines = text.lines();
  let header = lines.next().unwrap();
  let index = header.split(',').position(|name| name == column).unwrap();
  let mut result = format!("{header}\n");
  for (i, line) in lines.enumerate() {
    let mut values: Vec<String> = line.split(',').map(str::to_owned).collect();
    values[index] = change(i, &values[index]);
    result.push_str(&values.join(","));
    result.push('\n');
  }
  result
}
fn reference_change(text: &str, path: &str, value: f64) -> String {
  text
    .lines()
    .map(|line| match line.split_once(',') {
      Some((key, _)) if key == path => format!("{key},{value}"),
      _ => line.to_owned(),
    })
    .collect::<Vec<_>>()
    .join("\n")
    + "\n"
}

#[test]
fn full_exponential_script_matches_independent_survival_and_preserves_source_in_both_modes() {
  let d = setup();
  let output = run(&d, "--json");
  let p = fit(&output);
  assert_eq!(p["observation_count"], 400);
  assert_eq!(p["coefficients"].as_array().unwrap().len(), 41);
  assert_eq!(p["sample"]["dropped_observations"], 0);
  fs::write(d.path().join("native.json"), &output.stdout).unwrap();
  let checked = audit(&d);
  assert!(
    checked.status.success(),
    "{}",
    String::from_utf8_lossy(&checked.stderr)
  );
  assert!(
    String::from_utf8_lossy(&checked.stdout)
      .contains("1936 R gates/all1681 covariance entries, 24800")
  );
  let path = d.path().join("exponential_predictions.csv");
  let bytes = fs::read(&path).unwrap();
  fs::remove_file(&path).unwrap();
  let text = run(&d, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(
    String::from_utf8_lossy(&text.stdout).contains("not all-cause survival or causal UI effect")
  );
  assert_eq!(fs::read(&path).unwrap(), bytes);
  assert_eq!(
    fs::read_to_string(d.path().join("duration.csv")).unwrap(),
    INPUT
  );
}

#[test]
fn independent_audit_rejects_shared_wrong_covariance_normal_and_wald_tails_and_projections() {
  let d = setup();
  let output = run(&d, "--json");
  fit(&output);
  let baseline = String::from_utf8(output.stdout).unwrap();
  let reference_path = d.path().join("reference/reference.csv");
  let reference = fs::read_to_string(&reference_path).unwrap();
  for (pointer, path) in [
    ("/coefficients/0/p_value", "/coefficients/0/p_value"),
    ("/omnibus_wald/p_value", "/omnibus_wald/p_value"),
    ("/covariance_matrix/0/0", "/covariance_matrix/0/0"),
  ] {
    let mut events: Vec<Value> = baseline
      .lines()
      .map(|line| serde_json::from_str(line).unwrap())
      .collect();
    let data = events
      .iter_mut()
      .find(|e| e["result_type"] == "PoissonResult")
      .unwrap()
      .get_mut("data")
      .unwrap();
    let wrong = if path.contains("covariance_matrix") {
      data.pointer(pointer).unwrap().as_f64().unwrap() * 2.0
    } else {
      0.0
    };
    *data.pointer_mut(pointer).unwrap() = Value::from(wrong);
    let forged = events
      .iter()
      .map(|e| serde_json::to_string(e).unwrap())
      .collect::<Vec<_>>()
      .join("\n")
      + "\n";
    fs::write(d.path().join("native.json"), forged).unwrap();
    fs::write(&reference_path, reference_change(&reference, path, wrong)).unwrap();
    assert!(!audit(&d).status.success(), "shared wrong {path} accepted");
  }
  fs::write(d.path().join("native.json"), baseline).unwrap();
  fs::write(&reference_path, reference).unwrap();
  for file in [
    d.path().join("exponential_predictions.csv"),
    d.path().join("reference/predictions.csv"),
  ] {
    let text = fs::read_to_string(&file).unwrap();
    fs::write(file, map_csv(&text, "net_survival_10", |_, _| "0".into())).unwrap();
  }
  assert!(
    !audit(&d).status.success(),
    "shared zero net survival accepted"
  );
}

#[test]
fn invalid_sample_design_and_existing_output_fail_without_changing_inputs_or_files() {
  let header = INPUT.lines().next().unwrap();
  let mut cases = vec![
    format!("{header}\n"),
    map_csv(INPUT, "censor1", |_, _| "0".into()),
    map_csv(INPUT, "stateur", |_, _| "0".into()),
    map_csv(INPUT, "source_row", |i, v| {
      if i == 1 { "1".into() } else { v.into() }
    }),
  ];
  for (column, value) in [
    ("spell", "0"),
    ("spell", "-1"),
    ("spell", ""),
    ("spell", "inf"),
    ("censor1", "2"),
    ("stateur", ""),
    ("logwage", "NaN"),
    ("ui", "0.5"),
    ("source_row", "1.5"),
  ] {
    cases.push(map_csv(INPUT, column, |i, v| {
      if i == 0 { value.into() } else { v.into() }
    }));
  }
  let overlap = map_csv(
    &map_csv(
      INPUT,
      "censor1",
      |i, v| if i == 0 { "1".into() } else { v.into() },
    ),
    "censor2",
    |i, v| if i == 0 { "1".into() } else { v.into() },
  );
  cases.push(overlap);
  // The author's ASCII export lacks this necessary regressor: do not fit a
  // reduced design or invent a value when it is absent from user input.
  let index = header
    .split(',')
    .position(|name| name == "stateur")
    .unwrap();
  cases.push(
    INPUT
      .lines()
      .map(|line| {
        line
          .split(',')
          .enumerate()
          .filter_map(|(i, value)| (i != index).then_some(value))
          .collect::<Vec<_>>()
          .join(",")
      })
      .collect::<Vec<_>>()
      .join("\n")
      + "\n",
  );
  for input in cases {
    let d = TestDir::new();
    fs::write(d.path().join("duration.csv"), &input).unwrap();
    let output = run(&d, "--json");
    assert!(!output.status.success(), "invalid full design accepted");
    assert!(!d.path().join("exponential_predictions.csv").exists());
    assert_eq!(
      fs::read_to_string(d.path().join("duration.csv")).unwrap(),
      input
    );
  }
  let d = setup();
  let target = d.path().join("exponential_predictions.csv");
  fs::write(&target, "sentinel").unwrap();
  assert!(!run(&d, "--json").status.success());
  assert_eq!(fs::read_to_string(&target).unwrap(), "sentinel");
  assert_eq!(
    fs::read_to_string(d.path().join("duration.csv")).unwrap(),
    INPUT
  );
}

#[test]
fn duration_unit_rescaling_changes_hazard_units_not_coefficients_covariance_or_observed_survival() {
  let base = setup();
  let scaled = setup();
  let input = map_csv(INPUT, "spell", |_, v| {
    (v.parse::<f64>().unwrap() * 2.0).to_string()
  });
  fs::write(scaled.path().join("duration.csv"), input).unwrap();
  let a = fit(&run(&base, "--json"));
  let b = fit(&run(&scaled, "--json"));
  let close = |a: f64, b: f64| assert!((a - b).abs() < 1e-9 + 1e-8 * b.abs(), "{a} != {b}");
  for i in 0..41 {
    let shift = if i == 0 { 2.0_f64.ln() } else { 0.0 };
    close(
      a["coefficients"][i]["value"].as_f64().unwrap() - shift,
      b["coefficients"][i]["value"].as_f64().unwrap(),
    );
    for j in 0..41 {
      close(
        a["covariance_matrix"][i][j].as_f64().unwrap(),
        b["covariance_matrix"][i][j].as_f64().unwrap(),
      );
    }
  }
  close(
    a["log_likelihood"].as_f64().unwrap(),
    b["log_likelihood"].as_f64().unwrap(),
  );
  let ca = fs::read_to_string(base.path().join("exponential_predictions.csv")).unwrap();
  let cb = fs::read_to_string(scaled.path().join("exponential_predictions.csv")).unwrap();
  let names: Vec<_> = ca.lines().next().unwrap().split(',').collect();
  for (a, b) in ca.lines().skip(1).zip(cb.lines().skip(1)) {
    let av: Vec<f64> = a.split(',').map(|v| v.parse().unwrap()).collect();
    let bv: Vec<f64> = b.split(',').map(|v| v.parse().unwrap()).collect();
    for (field, multiplier) in [
      ("hazard", 0.5),
      ("net_survival", 1.0),
      ("mean_net_duration", 2.0),
      ("ui_hazard_ratio", 1.0),
    ] {
      let i = names.iter().position(|v| *v == field).unwrap();
      close(av[i] * multiplier, bv[i]);
    }
  }
}
