mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::fs;
#[test]
fn cluster_cli_matches_r_reports_and_preserves_prediction_exports() {
  let d = TestDir::new();
  fs::write(
    d.path().join("data.csv"),
    include_str!("../crates/tabdat-stats/tests/fixtures/cluster-bootstrap/inputs.csv"),
  )
  .unwrap();
  let mut compared = 0;
  for (case, seed, option, prepare) in [
    ("balanced", "18446744073709551615", "", ""),
    ("unbalanced", "7", "", ""),
    ("noconstant", "7", " noconstant", ""),
    ("shifted", "7", "", "replace y = y + 100\n"),
  ] {
    let group = if case == "balanced" {
      "balanced"
    } else {
      "unbalanced"
    };
    fs::write(d.path().join("fit.td"),format!("use data.csv\n{prepare}regress y x z\nbootstrap y x z, cluster({group}) seed({seed}) reps(40){option}\npredict fitted\nexport predictions.csv, replace\n")).unwrap();
    let o = tabdat_bin()
      .current_dir(d.path())
      .args(["--json", "-f", "fit.td"])
      .output()
      .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let events: Vec<Value> = String::from_utf8(o.stdout)
      .unwrap()
      .lines()
      .map(|l| serde_json::from_str(l).unwrap())
      .collect();
    let r = &events
      .iter()
      .find(|e| e["result_type"] == "ClusterBootstrapOlsResult")
      .unwrap()["data"];
    assert_eq!(r["seed"], seed);
    assert_eq!(r["interval_method"], "percentile_r_type7");
    assert_eq!(
      r["sample"],
      serde_json::json!({"total_observations":20,"dropped_observations":0})
    );
    assert!(r.get("p_value").is_none());
    for line in
      include_str!("../crates/tabdat-stats/tests/fixtures/cluster-bootstrap/reference.csv")
        .lines()
        .skip(1)
    {
      let row: Vec<_> = line.split(',').collect();
      if row[0] != case {
        continue;
      }
      let actual = r.pointer(row[1]).and_then(Value::as_f64).unwrap();
      let expected: f64 = row[2].parse().unwrap();
      assert!(
        (actual - expected).abs() <= 1e-9,
        "{row:?}: {actual} != {expected}"
      );
      compared += 1;
    }
    let csv = fs::read(d.path().join("predictions.csv")).unwrap();
    let text = tabdat_bin()
      .current_dir(d.path())
      .args(["--text", "-f", "fit.td"])
      .output()
      .unwrap();
    assert!(text.status.success());
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(text.contains("95% percentile intervals (R type7)"));
    assert!(text.contains(seed));
    assert!(text.contains("Whole-cluster pairs-bootstrap"));
    assert!(text.contains("Clusters = 4"));
    assert_eq!(fs::read(d.path().join("predictions.csv")).unwrap(), csv);
  }
  assert_eq!(compared, 285);
}
