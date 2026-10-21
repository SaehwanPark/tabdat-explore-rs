mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};

fn script() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma21p1panel/random_effects_gls_bootstrap.td")
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(script())
    .output()
    .unwrap()
}
const INPUT: &str = include_str!("../crates/tabdat-stats/tests/fixtures/panel_hausman/inputs.csv");
#[test]
fn fixed_theta_bootstrap_matches_independent_plm_r_and_text_json_exports() {
  let d = TestDir::new();
  fs::write(d.path().join("MOM.csv"), INPUT).unwrap();
  let o = run(&d, "--json");
  assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
  let events: Vec<Value> = String::from_utf8(o.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  assert!(events.iter().all(|e| e.get("error").is_none()));
  let results: BTreeMap<_, _> = events
    .iter()
    .map(|e| (e["result_type"].as_str().unwrap(), &e["data"]))
    .collect();
  assert_eq!(
    results["RegressionResult"]["sample"],
    serde_json::json!({"total_observations":24,"dropped_observations":0})
  );
  assert_eq!(results["ClusterBootstrapOlsResult"]["cluster_count"], 6);
  assert_eq!(results["ClusterBootstrapOlsResult"]["replications"], 500);
  assert_eq!(
    results["ClusterBootstrapOlsResult"]["cluster_sizes"],
    serde_json::json!(vec![4; 6])
  );
  assert_eq!(
    results["ClusterBootstrapOlsResult"]["replicate_observation_counts"],
    serde_json::json!(vec![24; 500])
  );
  let mut compared = 0;
  for line in
    include_str!("../crates/tabdat-stats/tests/fixtures/panel_regls_bootstrap/reference.csv")
      .lines()
      .skip(1)
  {
    let r: Vec<_> = line.split(',').collect();
    if r[0] == "/theta" {
      continue;
    }
    let actual = results["ClusterBootstrapOlsResult"]
      .pointer(r[0])
      .and_then(Value::as_f64)
      .unwrap();
    let expected: f64 = r[1].parse().unwrap();
    let tol = 1e-9;
    assert!(
      (actual - expected).abs() <= tol,
      "{r:?}: {actual} != {expected}"
    );
    compared += 1;
  }
  assert_eq!(compared, 18);
  let target = d.path().join("panel_regls_bootstrap_predictions.csv");
  let csv = fs::read(&target).unwrap();
  let actual = String::from_utf8(csv.clone()).unwrap();
  let expected =
    include_str!("../crates/tabdat-stats/tests/fixtures/panel_regls_bootstrap/predictions.csv");
  assert_eq!(actual.lines().next(), expected.lines().next());
  let actual: Vec<_> = actual.lines().skip(1).collect();
  let expected: Vec<_> = expected.lines().skip(1).collect();
  assert_eq!(actual.len(), 24);
  assert_eq!(expected.len(), 24);
  let mut cells = 0;
  for (a, e) in actual.iter().zip(&expected) {
    for (a, e) in a.split(',').zip(e.split(',')) {
      let a: f64 = a.parse().unwrap();
      let e: f64 = e.parse().unwrap();
      assert!((a - e).abs() <= 1e-9, "{a} != {e}");
      cells += 1;
    }
  }
  assert_eq!(cells, 432);
  let bytes = fs::read(&target).unwrap();
  let refused = run(&d, "--json");
  assert!(!refused.status.success());
  assert_eq!(fs::read(&target).unwrap(), bytes);
  fs::remove_file(&target).unwrap();
  let text = run(&d, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert_eq!(fs::read(&target).unwrap(), csv);
}

#[test]
fn invalid_raw_keys_imbalanced_joint_cases_and_degenerate_designs_publish_no_export() {
  let lines: Vec<_> = INPUT.lines().collect();
  let duplicate = format!("{INPUT}{}\n", lines[1]);
  let empty = format!("{}\n", lines[0]);
  let imbalance = lines
    .iter()
    .enumerate()
    .filter(|(i, _)| *i != 1)
    .map(|(_, s)| *s)
    .collect::<Vec<_>>()
    .join("\n");
  let mut no_within = String::from(lines[0]);
  no_within.push('\n');
  let mut no_between = no_within.clone();
  let mut extreme = no_within.clone();
  let mut bad_key = no_within.clone();
  for (i, line) in lines.iter().skip(1).enumerate() {
    let mut r: Vec<_> = line.split(',').map(str::to_owned).collect();
    r[2] = r[7].clone();
    no_within.push_str(&r.join(","));
    no_within.push('\n');
    let mut r: Vec<_> = line.split(',').map(str::to_owned).collect();
    r[2] = r[8].clone();
    no_between.push_str(&r.join(","));
    no_between.push('\n');
    let mut r: Vec<_> = line.split(',').map(str::to_owned).collect();
    if r[7] == "1" {
      r[1] = "1e150".into();
    }
    extreme.push_str(&r.join(","));
    extreme.push('\n');
    let mut r: Vec<_> = line.split(',').map(str::to_owned).collect();
    if i == 24 {
      r[7] = "".into();
    } // An excluded missing-y unit still requires valid keys.
    bad_key.push_str(&r.join(","));
    bad_key.push('\n');
  }
  // Set the first retained y missing without depending on its numeric spelling.
  let mut r: Vec<_> = lines[1].split(',').collect();
  r[1] = "";
  let missing_outcome = INPUT.replace(lines[1], &r.join(","));
  for source in [
    duplicate,
    empty,
    imbalance,
    missing_outcome,
    no_within,
    no_between,
    extreme,
    bad_key,
  ] {
    let d = TestDir::new();
    fs::write(d.path().join("MOM.csv"), source).unwrap();
    let o = run(&d, "--json");
    assert!(
      !o.status.success(),
      "{}",
      String::from_utf8_lossy(&o.stdout)
    );
    assert!(!String::from_utf8_lossy(&o.stdout).contains("RegressionResult"));
    assert!(
      !d.path()
        .join("panel_regls_bootstrap_predictions.csv")
        .exists()
    );
  }
}
