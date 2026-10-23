mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::fs;

#[test]
fn native_joint_quantile_bootstrap_reports_r_covariance_and_preserves_predictions() {
  let d = TestDir::new();
  fs::write(
    d.path().join("data.csv"),
    include_str!("../crates/tabdat-stats/tests/fixtures/quantile-bootstrap/inputs.csv"),
  )
  .unwrap();
  let mut comparisons = 0;
  for (case, seed, option, prepare) in [
    ("joint", "18446744073709551615", "", ""),
    ("noconstant", "7", " noconstant", ""),
    ("shifted", "7", "", "replace y = y + 100\n"),
    ("rescaled", "7", "", "replace x = x * 100\n"),
  ] {
    fs::write(d.path().join("fit.td"),format!("use data.csv\n{prepare}regress y x z\nestimates store original\nbootstrap y x z, quantiles(.2 .5 .8) seed({seed}) reps(40){option}\npredict preserved\nexport predictions.csv, replace\n")).unwrap();
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
      .map(|l| serde_json::from_str(l).unwrap())
      .collect();
    let r = &events
      .iter()
      .find(|e| e["result_type"] == "QuantileBootstrapResult")
      .unwrap()["data"];
    assert_eq!(r["seed"], seed);
    assert_eq!(r["estimator"], "iid_pairs_bootstrap_joint_quantiles");
    assert_eq!(r["covariance_convention"], "common_iid_pairs_b_minus_1");
    assert_eq!(r["simultaneous_coverage"], false);
    assert_eq!(
      r["sample"],
      serde_json::json!({"total_observations":120,"dropped_observations":0})
    );
    assert_eq!(
      r["coefficients"][0]["name"],
      if case == "noconstant" {
        "q0.2:x"
      } else {
        "q0.2:intercept"
      }
    );
    for line in
      include_str!("../crates/tabdat-stats/tests/fixtures/quantile-bootstrap/reference.csv")
        .lines()
        .skip(1)
    {
      let row: Vec<_> = line.split(',').collect();
      if row[0] != case {
        continue;
      }
      let a = r.pointer(row[1]).unwrap().as_f64().unwrap();
      let e: f64 = row[2].parse().unwrap();
      assert!((a - e).abs() <= 1e-8, "{row:?}: {a} != {e}");
      comparisons += 1;
    }
    let csv = fs::read(d.path().join("predictions.csv")).unwrap();
    let t = run("--text");
    assert!(t.status.success(), "{}", String::from_utf8_lossy(&t.stderr));
    let text = String::from_utf8(t.stdout).unwrap();
    assert!(text.contains("no simultaneous coverage"));
    assert!(text.contains(seed));
    assert_eq!(fs::read(d.path().join("predictions.csv")).unwrap(), csv);
  }
  assert_eq!(comparisons, 477);
}
