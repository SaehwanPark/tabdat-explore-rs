mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  collections::BTreeMap,
  fs,
  path::PathBuf,
  process::{Command, Output},
};

const EXPORT: &str = "native_gamma_re_bootstrap_predictions.csv";
const RAW: &str = include_str!("fixtures/mma23_gamma_re_bootstrap/source.asc");
fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn prepare(dir: &TestDir, raw: &str) {
  fs::write(dir.path().join("source.asc"), raw).unwrap();
  let out = Command::new("sh")
    .arg(root().join("examples/mma23p1pannonlin/prepare.sh"))
    .arg(dir.path().join("source.asc"))
    .arg(dir.path().join("patents.csv"))
    .output()
    .unwrap();
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
}
fn run(dir: &TestDir, mode: &str, script: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(root().join("examples/mma23p1pannonlin").join(script))
    .output()
    .unwrap()
}
fn events(out: &Output) -> Vec<Value> {
  assert!(
    out.status.success(),
    "{}\n{}",
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  );
  String::from_utf8_lossy(&out.stdout)
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn report(events: &[Value]) -> &Value {
  let values: Vec<_> = events
    .iter()
    .filter(|e| e["result_type"] == "GammaPoissonBootstrapResult")
    .collect();
  assert_eq!(values.len(), 1);
  &values[0]["data"]
}
fn close(actual: f64, expected: f64) {
  assert!(
    actual.is_finite()
      && expected.is_finite()
      && (actual - expected).abs() <= 1e-9 + 1e-10 * expected.abs(),
    "{actual} vs {expected}"
  );
}
fn check_control(dir: &TestDir, events: &[Value]) {
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let d = report(events);
  assert_eq!(d["cluster_count"], 40);
  assert_eq!(d["observation_count"], 200);
  assert_eq!(d["zero_total_group_count"], 8);
  assert_eq!(d["sample"]["dropped_observations"], 0);
  assert_eq!(d["replications"], 500);
  assert_eq!(d["seed"], "10001");
  assert_eq!(
    d["covariance_parameter_names"],
    serde_json::json!(["_cons", "logr", "lnalpha"])
  );
  let coefficients = d["coefficients"].as_array().unwrap();
  assert_eq!(coefficients.len(), 3);
  let fits = d["replicate_coefficients"].as_array().unwrap();
  assert_eq!(fits.len(), 500);
  let references: Vec<_> = include_str!("fixtures/mma23_native_gamma_re_bootstrap/fits.csv")
    .lines()
    .skip(1)
    .collect();
  assert_eq!(references.len(), 500);
  for (i, line) in references.iter().enumerate() {
    let cells: Vec<_> = line.split(',').collect();
    assert_eq!(cells[0].parse::<usize>().unwrap(), i + 1);
    assert_eq!(fits[i].as_array().unwrap().len(), 3);
    for j in 0..3 {
      close(fits[i][j].as_f64().unwrap(), cells[j + 1].parse().unwrap());
    }
  }
  let mut actual = BTreeMap::from([
    ("candidate_rows".to_owned(), 200.),
    ("firms_per_draw".to_owned(), 40.),
    ("replications".to_owned(), 500.),
  ]);
  for (i, name) in ["intercept", "slope", "lnalpha"].iter().enumerate() {
    for (prefix, key) in [
      ("estimate", "estimate"),
      ("mean", "bootstrap_mean"),
      ("bias", "bias"),
      ("se", "standard_error"),
      ("lower", "ci_lower"),
      ("upper", "ci_upper"),
    ] {
      actual.insert(
        format!("{prefix}_{name}"),
        coefficients[i][key].as_f64().unwrap(),
      );
    }
  }
  for (i, a) in ['a', 'b', 't'].iter().enumerate() {
    for (j, b) in ['a', 'b', 't'].iter().enumerate() {
      if i <= j {
        actual.insert(
          format!("covariance_{a}{b}"),
          d["covariance_matrix"][i][j].as_f64().unwrap(),
        );
      }
    }
  }
  let text = fs::read_to_string(dir.path().join(EXPORT)).unwrap();
  let mut lines = text.lines();
  let header: Vec<_> = lines.next().unwrap().split(',').collect();
  let mean_index = header
    .iter()
    .position(|&name| name == "native_re_mean")
    .unwrap();
  let residual_index = header
    .iter()
    .position(|&name| name == "native_re_residual")
    .unwrap();
  let mut means = vec![];
  let mut residuals = vec![];
  for line in lines {
    let cells: Vec<_> = line.split(',').collect();
    means.push(cells[mean_index].parse::<f64>().unwrap());
    residuals.push(cells[residual_index].parse::<f64>().unwrap());
  }
  assert_eq!(means.len(), 200);
  actual.insert("original_mu_mean".into(), means.iter().sum::<f64>() / 200.);
  actual.insert(
    "original_mu_min".into(),
    means.iter().copied().fold(f64::INFINITY, f64::min),
  );
  actual.insert(
    "original_mu_max".into(),
    means.iter().copied().fold(f64::NEG_INFINITY, f64::max),
  );
  actual.insert(
    "original_residual_mean".into(),
    residuals.iter().sum::<f64>() / 200.,
  );
  let oracle: Vec<_> = include_str!("fixtures/mma23_native_gamma_re_bootstrap/reference.csv")
    .lines()
    .skip(1)
    .collect();
  assert_eq!(oracle.len(), 31);
  assert_eq!(actual.len(), 31);
  for line in oracle {
    let (field, value) = line.split_once(',').unwrap();
    close(actual[field], value.parse().unwrap());
  }
  assert_eq!(
    d["replicate_cluster_indices"].as_array().unwrap().len(),
    500
  );
  for draw in d["replicate_cluster_indices"].as_array().unwrap() {
    assert_eq!(draw.as_array().unwrap().len(), 40);
  }
  assert!(
    d["replicate_observation_counts"]
      .as_array()
      .unwrap()
      .iter()
      .all(|n| *n == 200)
  );
}

#[test]
fn all_joint_r_refits_and_reports_preserve_original_predictions_text_and_files() {
  let dir = TestDir::new();
  prepare(&dir, RAW);
  check_control(
    &dir,
    &events(&run(&dir, "--json", "native_gamma_re_bootstrap.td")),
  );
  let exported = fs::read(dir.path().join(EXPORT)).unwrap();
  let original = TestDir::new();
  prepare(&original, RAW);
  events(&run(&original, "--json", "native_gamma_re.td"));
  assert_eq!(
    exported,
    fs::read(original.path().join("native_gamma_re_predictions.csv")).unwrap()
  );
  let text = TestDir::new();
  prepare(&text, RAW);
  let out = run(&text, "--text", "native_gamma_re_bootstrap.td");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  assert_eq!(exported, fs::read(text.path().join(EXPORT)).unwrap());
  assert!(
    !run(&dir, "--json", "native_gamma_re_bootstrap.td")
      .status
      .success()
  );
  assert_eq!(exported, fs::read(dir.path().join(EXPORT)).unwrap());
}

#[test]
fn reversed_source_and_missing_nonmodel_metadata_keep_all_joint_references() {
  let dir = TestDir::new();
  prepare(&dir, RAW);
  let csv = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let mut lines = csv.lines();
  let header = lines.next().unwrap();
  let mut records: Vec<_> = lines
    .map(|line| {
      let mut cells: Vec<_> = line.split(',').collect();
      cells[3] = "";
      cells.join(",")
    })
    .collect();
  records.reverse();
  fs::write(
    dir.path().join("patents.csv"),
    format!("{header}\n{}\n", records.join("\n")),
  )
  .unwrap();
  check_control(
    &dir,
    &events(&run(&dir, "--json", "native_gamma_re_bootstrap.td")),
  );
}

#[test]
fn actual_257_firm_bootstrap_keeps_zero_total_firms_and_all_500_joint_vectors() {
  let source: Vec<_> = RAW.lines().filter(|line| !line.trim().is_empty()).collect();
  let chosen: Vec<_> = (0..257).map(|i| source[i % source.len()]).collect();
  let zeros = chosen
    .iter()
    .filter(|line| {
      line
        .split_whitespace()
        .skip(20)
        .all(|token| token.parse::<f64>().unwrap() == 0.)
    })
    .count();
  assert!(zeros > 0);
  let dir = TestDir::new();
  prepare(&dir, &format!("{}\n", chosen.join("\n")));
  let es = events(&run(&dir, "--json", "native_gamma_re_bootstrap.td"));
  let d = report(&es);
  assert_eq!(d["cluster_count"], 257);
  assert_eq!(d["observation_count"], 1285);
  assert_eq!(d["zero_total_group_count"], zeros);
  assert_eq!(d["replicate_coefficients"].as_array().unwrap().len(), 500);
  for values in d["replicate_coefficients"].as_array().unwrap() {
    assert_eq!(values.as_array().unwrap().len(), 3);
    assert!(
      values
        .as_array()
        .unwrap()
        .iter()
        .all(|v| v.as_f64().unwrap().is_finite())
    );
  }
  assert_eq!(
    d["replicate_cluster_indices"].as_array().unwrap().len(),
    500
  );
  for draw in d["replicate_cluster_indices"].as_array().unwrap() {
    assert_eq!(draw.as_array().unwrap().len(), 257);
    assert!(
      draw
        .as_array()
        .unwrap()
        .iter()
        .all(|i| i.as_u64().unwrap() < 257)
    );
  }
}
