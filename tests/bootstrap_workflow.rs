mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::fs;
#[test]
fn bootstrap_cli_matches_r_reports_exact_u64_seed_and_preserves_prediction_exports() {
  let d = TestDir::new();
  fs::write(
    d.path().join("data.csv"),
    include_str!("../crates/tabdat-stats/tests/fixtures/bootstrap/inputs.csv"),
  )
  .unwrap();
  let mut compared = 0;
  for (case, seed, option, prepare) in [
    ("multiple", "18446744073709551615", "", ""),
    ("noconstant", "7", " noconstant", ""),
    ("shifted", "7", "", "replace y = y + 100\n"),
  ] {
    fs::write(d.path().join("fit.td"),format!("use data.csv\n{prepare}regress y x z\nbootstrap y x z, seed({seed}) reps(40){option}\npredict fitted\nexport predictions.csv, replace\n")).unwrap();
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
      .find(|e| e["result_type"] == "BootstrapOlsResult")
      .unwrap()["data"];
    assert_eq!(r["seed"], seed);
    assert_eq!(r["interval_method"], "percentile_r_type7");
    assert_eq!(
      r["sample"],
      serde_json::json!({"total_observations":12,"dropped_observations":0})
    );
    assert!(r.get("p_value").is_none());
    for line in include_str!("../crates/tabdat-stats/tests/fixtures/bootstrap/reference.csv")
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
    assert_eq!(fs::read(d.path().join("predictions.csv")).unwrap(), csv);
  }
  assert_eq!(compared, 76);
}
#[test]
fn discovery_exposes_the_new_read_only_seeded_report_without_rewriting_pinned_commands() {
  let o = tabdat_bin()
    .args(["--json", "--describe-command", "bootstrap"])
    .output()
    .unwrap();
  assert!(o.status.success());
  let v: Value = serde_json::from_slice(&o.stdout).unwrap();
  assert_eq!(
    v["data"]["syntax"],
    "bootstrap y xvars, seed(integer) [reps(200) noconstant cluster(unit) | quantiles(values)]"
  );
  let o = tabdat_bin()
    .args(["--json", "--help-topic", "bootstrap"])
    .output()
    .unwrap();
  assert!(o.status.success());
  let v: Value = serde_json::from_slice(&o.stdout).unwrap();
  assert!(
    v["data"]["text"]
      .as_str()
      .unwrap()
      .contains("no discarded draws")
  );
}

#[test]
fn actual_between_bootstrap_script_resamples_people_and_preserves_existing_exports() {
  let d = TestDir::new();
  let mut input = String::from("source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year\n");
  let mut index = 0;
  for t in (0..4).rev() {
    for id in (1..=20).rev() {
      index += 1;
      let x = 2.0 + 0.15 * id as f64 + 0.1 * t as f64 + 0.03 * ((id * t + 2) % 5) as f64;
      let y = 7.0 + 0.2 * id as f64 + 0.04 * t as f64 + 0.005 * ((id * t + 1) % 7) as f64;
      input.push_str(&format!("{index},{y},{x},0,30,900,0,{id},{}\n", 1979 + t));
    }
  }
  fs::write(d.path().join("MOM.csv"), input).unwrap();
  let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma21p1panel/between_bootstrap.td");
  let run = |mode| {
    tabdat_bin()
      .current_dir(d.path())
      .args([mode, "-f"])
      .arg(&script)
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
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  let r = &events
    .iter()
    .find(|e| e["result_type"] == "BootstrapOlsResult")
    .unwrap()["data"];
  assert_eq!(r["observation_count"], 20);
  assert_eq!(r["replications"], 200);
  assert_eq!(r["seed"], "10001");
  let target = d.path().join("panel_between_bootstrap_predictions.csv");
  let bytes = fs::read(&target).unwrap();
  let text = String::from_utf8(bytes.clone()).unwrap();
  assert_eq!(text.lines().skip(1).count(), 20);
  for (index, line) in text.lines().skip(1).enumerate() {
    let row: Vec<f64> = line.split(',').map(|v| v.parse().unwrap()).collect();
    assert_eq!(row[0], (index + 1) as f64);
    assert_eq!(row[1], 4.0);
    assert_eq!(row[2], 1979.0);
    assert_eq!(row[3], 1982.0);
    assert!((row[4] - row[6] - row[7]).abs() < 1e-12);
  }
  let refused = run("--json");
  assert!(!refused.status.success());
  assert_eq!(fs::read(&target).unwrap(), bytes);
  fs::remove_file(&target).unwrap();
  let text = run("--text");
  assert!(text.status.success());
  assert_eq!(fs::read(&target).unwrap(), bytes);
}
