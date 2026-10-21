mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf};
fn run_script(d: &TestDir, mode: &str, script: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma04p2qreg")
        .join(script),
    )
    .output()
    .unwrap()
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  run_script(d, mode, "quantiles.td")
}
const INPUT: &str =
  include_str!("../crates/tabdat-stats/tests/fixtures/quantile_workflow/inputs.csv");
#[test]
fn native_script_inference_and_current_row_exports_match_independent_quantreg() {
  let d = TestDir::new();
  fs::write(d.path().join("qreg.csv"), INPUT).unwrap();
  let o = run(&d, "--json");
  assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
  let events: Vec<Value> = String::from_utf8(o.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  assert!(events.iter().all(|e| e.get("error").is_none()));
  let fits: Vec<_> = events
    .iter()
    .filter(|e| e["result_type"] == "QuantileRegressionResult")
    .map(|e| &e["data"])
    .collect();
  assert_eq!(fits.len(), 3);
  for fit in &fits {
    assert_eq!(
      fit["sample"],
      serde_json::json!({"total_observations":120,"dropped_observations":0})
    );
    assert_eq!(
      fit["covariance"],
      "iid_location_hall_sheather_residual_spacing"
    );
    assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
    assert_eq!(fit["confidence_level"], 95);
  }
  let mut compared = 0;
  for l in include_str!("../crates/tabdat-stats/tests/fixtures/quantile_workflow/reference.csv")
    .lines()
    .skip(1)
  {
    let r: Vec<_> = l.split(',').collect();
    let i = match r[0] {
      "q10" => 0,
      "q50" => 1,
      "q90" => 2,
      _ => panic!("case"),
    };
    let actual = fits[i].pointer(r[1]).and_then(Value::as_f64).unwrap();
    let expected: f64 = r[2].parse().unwrap();
    let tol = if r[1].ends_with("p_value") {
      expected.abs() * 1e-7 + 1e-300
    } else {
      1e-8
    };
    assert!(
      (actual - expected).abs() <= tol,
      "{r:?}: {actual} != {expected}"
    );
    compared += 1;
  }
  assert_eq!(compared, 66);
  let path = d.path().join("quantile_predictions.csv");
  let csv = fs::read(&path).unwrap();
  let actual = String::from_utf8(csv.clone()).unwrap();
  let expected =
    include_str!("../crates/tabdat-stats/tests/fixtures/quantile_workflow/predictions.csv");
  let header: Vec<_> = actual.lines().next().unwrap().split(',').collect();
  let keys: Vec<_> = expected.lines().next().unwrap().split(',').collect();
  let indices: Vec<_> = keys
    .iter()
    .map(|k| header.iter().position(|h| h == k).unwrap())
    .collect();
  let mut cells = 0;
  assert_eq!(actual.lines().count(), 121);
  for (a, e) in actual.lines().skip(1).zip(expected.lines().skip(1)) {
    let a: Vec<_> = a.split(',').collect();
    for (&i, e) in indices.iter().zip(e.split(',')) {
      let a: f64 = a[i].parse().unwrap();
      let e: f64 = e.parse().unwrap();
      assert!((a - e).abs() < 1e-8);
      cells += 1;
    }
  }
  assert_eq!(cells, 840);
  assert!(!run(&d, "--json").status.success());
  assert_eq!(fs::read(&path).unwrap(), csv);
  fs::remove_file(&path).unwrap();
  let text = run(&d, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(String::from_utf8_lossy(&text.stdout).contains("IID"));
  assert_eq!(fs::read(&path).unwrap(), csv);
}
#[test]
fn duplicate_source_identity_and_unresolved_designs_publish_no_export() {
  for input in [
    format!("{INPUT}1,0,0\n"),
    INPUT
      .lines()
      .enumerate()
      .map(|(i, l)| {
        if i == 0 {
          l.to_owned()
        } else {
          format!("{i},1,{i}")
        }
      })
      .collect::<Vec<_>>()
      .join("\n"),
  ] {
    let d = TestDir::new();
    fs::write(d.path().join("qreg.csv"), input).unwrap();
    assert!(!run(&d, "--json").status.success());
    assert!(!d.path().join("quantile_predictions.csv").exists());
  }
}

#[test]
fn quantile_figure_preserves_reference_vertices_legends_and_refuses_existing_artifacts() {
  let d = TestDir::new();
  fs::write(d.path().join("qreg.csv"), INPUT).unwrap();
  let json = run_script(&d, "--json", "figure.td");
  assert!(
    json.status.success(),
    "{}",
    String::from_utf8_lossy(&json.stderr)
  );
  let events: Vec<Value> = String::from_utf8(json.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  let plot = events
    .iter()
    .find(|e| e["result_type"] == "PlotArtifactResult")
    .unwrap();
  assert_eq!(plot["data"]["candidate_rows"], 120);
  assert_eq!(plot["data"]["jitter_y"], 0.0);
  assert_eq!(
    plot["data"]["lines"],
    serde_json::json!([
      {"variable":"q90","point_count":120,"omitted_rows":0},
      {"variable":"q50","point_count":120,"omitted_rows":0},
      {"variable":"q10","point_count":120,"omitted_rows":0}
    ])
  );
  let path = d.path().join("quantile_lines.svg");
  let svg = fs::read_to_string(&path).unwrap();
  assert_eq!(svg.matches("class=\"scatter-point\"").count(), 120);
  for label in [
    "Log total household expenditure",
    "Log household medical expenditure",
    "90th percentile",
    "Median",
    "10th percentile",
  ] {
    assert!(svg.contains(label));
  }
  let lines: Vec<_> = svg
    .lines()
    .filter(|l| l.contains("class=\"line-series\""))
    .collect();
  assert_eq!(lines.len(), 3);
  let mut reference: Vec<Vec<f64>> =
    include_str!("../crates/tabdat-stats/tests/fixtures/quantile_workflow/predictions.csv")
      .lines()
      .skip(1)
      .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
      .collect();
  reference.sort_by(|a, b| a[1].total_cmp(&b[1]).then(a[0].total_cmp(&b[0])));
  for (line, index) in lines.iter().zip([6, 4, 3]) {
    let values = line
      .split("data-y-values=\"")
      .nth(1)
      .unwrap()
      .split('"')
      .next()
      .unwrap();
    let values: Vec<f64> = values.split(',').map(|v| v.parse().unwrap()).collect();
    assert_eq!(values.len(), 120);
    for (value, row) in values.iter().zip(&reference) {
      assert!((value - row[index]).abs() < 1e-8);
    }
  }
  // Reach the plot's own no-clobber path rather than stopping at the earlier CSV export.
  let csv_path = d.path().join("quantile_predictions.csv");
  fs::remove_file(&csv_path).unwrap();
  let refused = run_script(&d, "--json", "figure.td");
  assert!(!refused.status.success());
  let error = String::from_utf8_lossy(&refused.stderr);
  assert!(
    error.contains("already exists") && error.contains("quantile_lines.svg"),
    "{error}"
  );
  assert_eq!(fs::read_to_string(&path).unwrap(), svg);
  fs::remove_file(&path).unwrap();
  fs::remove_file(&csv_path).unwrap();
  assert!(run_script(&d, "--text", "figure.td").status.success());
  assert_eq!(fs::read_to_string(&path).unwrap(), svg);
}
