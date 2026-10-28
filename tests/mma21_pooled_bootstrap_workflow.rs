mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf};

const INPUT: &str = include_str!("../crates/tabdat-stats/tests/fixtures/panel_hausman/inputs.csv");
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma21p1panel/pooled_bootstrap.td"),
    )
    .output()
    .unwrap()
}

#[test]
fn pooled_whole_person_bootstrap_matches_r_and_text_json_exports() {
  let d = TestDir::new();
  fs::write(d.path().join("MOM.csv"), INPUT).unwrap();
  let o = run(&d, "--json");
  assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
  let events: Vec<Value> = String::from_utf8(o.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let sample_table = events
    .iter()
    .find(|e| {
      e["result_type"] == "TableResult"
        && e["data"]["headers"] == serde_json::json!(["raw_rows", "retained_rows", "excluded_rows"])
    })
    .unwrap();
  assert_eq!(
    sample_table["data"]["rows"][0],
    serde_json::json!([28, 24, 4])
  );
  let fit = &events
    .iter()
    .find(|e| e["result_type"] == "ClusterBootstrapOlsResult")
    .unwrap()["data"];
  assert_eq!(
    fit["sample"],
    serde_json::json!({"total_observations":24,"dropped_observations":0})
  );
  assert_eq!(fit["seed"], "10001");
  assert_eq!(fit["cluster_count"], 6);
  assert_eq!(fit["cluster_sizes"], serde_json::json!(vec![4; 6]));
  assert_eq!(
    fit["replicate_observation_counts"],
    serde_json::json!(vec![24; 500])
  );
  assert_eq!(fit["cluster_order"], "first_seen_complete_case");
  let mut checked = 0;
  for line in
    include_str!("../crates/tabdat-stats/tests/fixtures/panel_pooled_bootstrap/reference.csv")
      .lines()
      .skip(1)
  {
    let (path, expected) = line.split_once(',').unwrap();
    let actual = fit.pointer(path).and_then(Value::as_f64).unwrap();
    let expected: f64 = expected.parse().unwrap();
    assert!(
      (actual - expected).abs() <= 1e-9,
      "{path}: {actual} != {expected}"
    );
    checked += 1;
  }
  assert_eq!(checked, 18);
  let path = d.path().join("panel_pooled_bootstrap_predictions.csv");
  let csv = fs::read(&path).unwrap();
  let actual = String::from_utf8(csv.clone()).unwrap();
  let expected =
    include_str!("../crates/tabdat-stats/tests/fixtures/panel_pooled_bootstrap/predictions.csv");
  assert_eq!(actual.lines().next(), expected.lines().next());
  let a: Vec<_> = actual.lines().skip(1).collect();
  let b: Vec<_> = expected.lines().skip(1).collect();
  assert_eq!(a.len(), 24);
  assert_eq!(b.len(), 24);
  for (a, b) in a.iter().zip(b) {
    for (a, b) in a.split(',').zip(b.split(',')) {
      let a: f64 = a.parse().unwrap();
      let b: f64 = b.parse().unwrap();
      assert!((a - b).abs() <= 1e-9, "{a} != {b}");
    }
  }
  assert!(!run(&d, "--json").status.success());
  assert_eq!(fs::read(&path).unwrap(), csv);
  fs::remove_file(&path).unwrap();
  let text = run(&d, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert_eq!(fs::read(&path).unwrap(), csv);
}

#[test]
fn panel_keys_and_original_or_replicate_rank_failures_do_not_export() {
  let lines: Vec<_> = INPUT.lines().collect();
  let duplicate = format!("{INPUT}{}\n", lines[1]);
  let mut rank_failure = String::from(lines[0]);
  rank_failure.push('\n');
  let mut bad_key = rank_failure.clone();
  for (i, line) in lines.iter().skip(1).enumerate() {
    let mut r: Vec<_> = line.split(',').collect();
    r[2] = "0";
    rank_failure.push_str(&r.join(","));
    rank_failure.push('\n');
    let mut r: Vec<_> = line.split(',').collect();
    if i == 24 {
      r[7] = "";
    }
    bad_key.push_str(&r.join(","));
    bad_key.push('\n');
  }
  let replicate_failure = "source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year\n1,1,0,0,30,900,0,1,1979\n2,2,0,0,30,900,0,1,1980\n3,3,1,0,30,900,0,2,1979\n4,4,1,0,30,900,0,2,1980\n";
  for (source, expect_replicate) in [
    (duplicate.as_str(), false),
    (bad_key.as_str(), false),
    (rank_failure.as_str(), false),
    (replicate_failure, true),
  ] {
    let d = TestDir::new();
    fs::write(d.path().join("MOM.csv"), source).unwrap();
    let output = run(&d, "--json");
    assert!(!output.status.success());
    if expect_replicate {
      let stderr = String::from_utf8_lossy(&output.stderr);
      assert!(stderr.contains("replicate"), "{stderr}");
    }
    assert!(
      !d.path()
        .join("panel_pooled_bootstrap_predictions.csv")
        .exists()
    );
  }
}
