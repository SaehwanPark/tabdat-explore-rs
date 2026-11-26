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
const BOOTSTRAP_INPUT: &str =
  include_str!("../crates/tabdat-stats/tests/fixtures/quantile-bootstrap/inputs.csv");
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

#[test]
fn joint_quantile_slope_plot_json_and_text_match_r_reference_and_publish_svg() {
  let d = TestDir::new();
  fs::write(d.path().join("data.csv"), BOOTSTRAP_INPUT).unwrap();
  let make_script = |artifact: &str| {
    format!(
      "use data.csv\nregress y x z\nqregplot y x z, coefficient(x) quantiles(.2 .5 .8) seed(18446744073709551615) reps(40) saving(\"{artifact}\") noopen title(\"Slope & bands\")\n"
    )
  };
  fs::write(d.path().join("fit.td"), make_script("bands.svg")).unwrap();
  let run = |mode: &str| {
    tabdat_bin()
      .current_dir(d.path())
      .args([mode, "-f", "fit.td"])
      .output()
      .unwrap()
  };
  let json = run("--json");
  assert!(
    json.status.success(),
    "{}",
    String::from_utf8_lossy(&json.stderr)
  );
  let events: Vec<Value> = String::from_utf8(json.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let ols = &events
    .iter()
    .find(|event| event["result_type"] == "RegressionResult")
    .unwrap()["data"];
  let result = &events
    .iter()
    .find(|event| event["result_type"] == "QuantileSlopePlotResult")
    .unwrap()["data"];
  let analysis = &result["analysis"];
  assert_eq!(
    analysis["estimator"],
    "iid_pairs_bootstrap_quantile_slope_bands"
  );
  assert_eq!(analysis["seed"], "18446744073709551615");
  assert_eq!(analysis["replications"], 40);
  assert_eq!(analysis["observation_count"], 120);
  assert_eq!(
    analysis["sample"],
    serde_json::json!({"total_observations":120,"dropped_observations":0})
  );
  assert_eq!(
    analysis["covariance_convention"],
    "common_iid_pairs_b_minus_1"
  );
  assert_eq!(analysis["band_method"], "pointwise_normal_fixed_1.96");
  assert_eq!(analysis["simultaneous_coverage"], false);
  let ols_slope = ols["coefficients"]
    .as_array()
    .unwrap()
    .iter()
    .find(|coefficient| coefficient["name"] == "x")
    .unwrap()["value"]
    .as_f64()
    .unwrap();
  assert_eq!(analysis["ols_coefficient"].as_f64().unwrap(), ols_slope);

  let mut reference = std::collections::HashMap::new();
  for line in include_str!("../crates/tabdat-stats/tests/fixtures/quantile-bootstrap/reference.csv")
    .lines()
    .skip(1)
  {
    let row: Vec<_> = line.split(',').collect();
    if row[0] == "joint" {
      reference.insert(row[1].to_owned(), row[2].parse::<f64>().unwrap());
    }
  }
  let points = analysis["points"].as_array().unwrap();
  assert_eq!(points.len(), 3);
  for (point_index, coefficient_index) in [1, 4, 7].into_iter().enumerate() {
    let point = &points[point_index];
    let estimate = reference[&format!("/coefficients/{coefficient_index}/estimate")];
    let standard_error = reference[&format!("/coefficients/{coefficient_index}/standard_error")];
    assert!((point["estimate"].as_f64().unwrap() - estimate).abs() < 1e-8);
    assert!((point["standard_error"].as_f64().unwrap() - standard_error).abs() < 1e-8);
    assert!((point["lower"].as_f64().unwrap() - (estimate - 1.96 * standard_error)).abs() < 1e-8);
    assert!((point["upper"].as_f64().unwrap() - (estimate + 1.96 * standard_error)).abs() < 1e-8);
  }

  let svg_path = d.path().join("bands.svg");
  let svg = fs::read_to_string(&svg_path).unwrap();
  assert!(svg.contains("Slope &amp; bands"));
  assert!(svg.contains("No simultaneous-coverage guarantee"));
  assert_eq!(svg.matches("class=\"quantile-slope-series\"").count(), 4);
  fs::write(d.path().join("fit.td"), make_script("bands-text.svg")).unwrap();
  let text = run("--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let text_output = String::from_utf8_lossy(&text.stdout);
  assert!(text_output.contains("no simultaneous-coverage guarantee"));
  assert!(
    text_output.contains("N = 120; candidate rows = 120; excluded joint missing/nonfinite = 0")
  );
  assert_eq!(
    fs::read(&svg_path).unwrap(),
    fs::read(d.path().join("bands-text.svg")).unwrap()
  );

  let help = tabdat_bin()
    .current_dir(d.path())
    .args(["--json", "--help-topic", "qregplot"])
    .output()
    .unwrap();
  assert!(help.status.success());
  assert!(String::from_utf8_lossy(&help.stdout).contains("pointwise"));
}
