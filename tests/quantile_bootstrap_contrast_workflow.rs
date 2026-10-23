mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::fs;

#[test]
fn contrast_cli_matches_shared_draw_r_inference_and_preserves_retained_predictions() {
  let d = TestDir::new();
  fs::write(
    d.path().join("data.csv"),
    include_str!("../crates/tabdat-stats/tests/fixtures/quantile-bootstrap/inputs.csv"),
  )
  .unwrap();
  let mut compared = 0;
  for (case, parameter, seed, option, prepare) in [
    ("joint", "x", "18446744073709551615", "", ""),
    ("joint", "intercept", "18446744073709551615", "", ""),
    ("joint", "z", "18446744073709551615", "", ""),
    ("noconstant", "x", "7", " noconstant", ""),
    ("shifted", "x", "7", "", "replace y = y + 100\n"),
    ("rescaled", "x", "7", "", "replace x = x * 100\n"),
  ] {
    fs::write(d.path().join("fit.td"),format!("use data.csv\n{prepare}regress y x z\npredict original\nbootstrap y x z, quantiles(.25 .75) contrast({parameter}) seed({seed}) reps(40){option}\npredict preserved\nexport predictions.csv, replace\n")).unwrap();
    let run = |mode| {
      tabdat_bin()
        .current_dir(d.path())
        .args([mode, "-f", "fit.td"])
        .output()
        .unwrap()
    };
    let o = run("--json");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let events: Vec<Value> = String::from_utf8(o.stdout)
      .unwrap()
      .lines()
      .map(|s| serde_json::from_str(s).unwrap())
      .collect();
    let r = &events
      .iter()
      .find(|e| e["result_type"] == "QuantileBootstrapResult")
      .unwrap()["data"];
    assert_eq!(r["seed"], seed);
    assert_eq!(r["contrast"]["parameter"], parameter);
    assert_eq!(r["contrast"]["lower_quantile"], 0.25);
    assert_eq!(r["contrast"]["upper_quantile"], 0.75);
    assert_eq!(
      r["contrast"]["reference_distribution"],
      "asymptotic_standard_normal"
    );
    assert_eq!(r["contrast"]["interval_method"], "normal_wald");
    assert!(r["contrast"]["inference_unavailable_reason"].is_null());
    assert_eq!(r["contrast"]["confidence_level"], 95.);
    assert_eq!(r["interval_method"], "percentile_r_type7");
    for line in include_str!(
      "../crates/tabdat-stats/tests/fixtures/quantile-bootstrap/contrast_reference.csv"
    )
    .lines()
    .skip(1)
    {
      let row: Vec<_> = line.split(',').collect();
      if row[0] != format!("{case}:{parameter}") {
        continue;
      }
      let a = r.pointer(row[1]).unwrap().as_f64().unwrap();
      let e: f64 = row[2].parse().unwrap();
      let tol = if row[1].ends_with("p_value") {
        e.abs() * 1e-7 + 1e-300
      } else {
        1e-8
      };
      assert!((a - e).abs() <= tol, "{row:?}: {a} != {e}");
      compared += 1;
    }
    let csv = fs::read_to_string(d.path().join("predictions.csv")).unwrap();
    let header: Vec<_> = csv.lines().next().unwrap().split(',').collect();
    let old = header.iter().position(|s| *s == "original").unwrap();
    let new = header.iter().position(|s| *s == "preserved").unwrap();
    for line in csv.lines().skip(1) {
      let row: Vec<_> = line.split(',').collect();
      assert_eq!(row[old], row[new]);
    }
    let t = run("--text");
    assert!(t.status.success(), "{}", String::from_utf8_lossy(&t.stderr));
    let text = String::from_utf8(t.stdout).unwrap();
    assert!(text.contains("H0: difference = 0; asymptotic normal z"));
    assert!(text.contains("not Student-t degrees of freedom"));
    assert_eq!(
      fs::read_to_string(d.path().join("predictions.csv")).unwrap(),
      csv
    );
  }
  assert_eq!(compared, 36);
}

#[test]
fn zero_variance_contrast_reports_null_normal_inference_and_explicit_reason() {
  let d = TestDir::new();
  let mut csv = String::from("y,x\n");
  for i in 0..120 {
    csv.push_str(&format!("{},{}\n", 1 + 2 * i, i));
  }
  fs::write(d.path().join("exact.csv"), csv).unwrap();
  let o = tabdat_bin()
    .current_dir(d.path())
    .args([
      "--json",
      "-c",
      "use exact.csv",
      "-c",
      "bootstrap y x, quantiles(.25 .75) contrast(x) seed(7) reps(40)",
    ])
    .output()
    .unwrap();
  assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
  let e: Vec<Value> = String::from_utf8(o.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let r = &e
    .iter()
    .find(|e| e["result_type"] == "QuantileBootstrapResult")
    .unwrap()["data"];
  assert_eq!(
    r["contrast"]["inference_unavailable_reason"],
    "zero_contrast_variance"
  );
  assert_eq!(r["contrast"]["standard_error"], 0.);
  for name in [
    "p_value",
    "statistic",
    "ci_lower",
    "ci_upper",
    "reference_distribution",
    "interval_method",
  ] {
    assert!(r["contrast"][name].is_null());
  }
  assert_eq!(r["coefficients"].as_array().unwrap().len(), 4);
}
