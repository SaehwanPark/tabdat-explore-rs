mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};
use tabdat_domain::EstimationState;
use tabdat_language::parse_command;
use tabdat_runtime::Session;

const INPUT: &str = include_str!("fixtures/mma24_household_gee/negative/input.csv");
const REFERENCE: &str = include_str!("fixtures/mma24_household_ols_bootstrap/reference.csv");
const PREDICTIONS: &str = include_str!("fixtures/mma24_household_ols_bootstrap/predictions.csv");
const GROUPS: &str = include_str!("fixtures/mma24_household_ols_bootstrap/groups.csv");
const COUNTS: &str = include_str!("fixtures/mma24_household_ols_bootstrap/replicate_counts.csv");
const SCRIPT: &str = "examples/mma24p1olscluster/household_ols_bootstrap.td";
const OUTPUT: &str = "household_ols_bootstrap_predictions.csv";
const FIT: &str = "regress lnexp12m lnhhexp age sex hhsize farm educ";
const BOOT: &str =
  "bootstrap lnexp12m lnhhexp age sex hhsize farm educ, cluster(commune) reps(500) seed(10001)";

fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let names = lines
    .next()
    .unwrap()
    .split(',')
    .map(|v| v.trim_matches('"'))
    .collect::<Vec<_>>();
  lines
    .map(|line| {
      let values = line
        .split(',')
        .map(|v| v.trim_matches('"'))
        .collect::<Vec<_>>();
      assert_eq!(names.len(), values.len());
      names.iter().copied().zip(values).collect()
    })
    .collect()
}
fn near(a: f64, b: f64, path: &str) {
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9,
    "{path}: {a} != {b}"
  );
}
fn execute(s: &mut Session, text: &str) -> tabdat_domain::ExecutionResult {
  s.execute(parse_command(text).unwrap())
    .unwrap_or_else(|e| panic!("{text}: {e}"))
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SCRIPT))
    .output()
    .unwrap()
}
fn fit_from_output(out: std::process::Output) -> Value {
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let events = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str::<Value>(l).unwrap())
    .collect::<Vec<_>>();
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  assert_eq!(
    events
      .iter()
      .filter(|e| e["result_type"] == "ClusterBootstrapOlsResult")
      .count(),
    1
  );
  let table = events
    .iter()
    .find(|e| {
      e["result_type"] == "TableResult"
        && e["data"]["headers"] == serde_json::json!(["raw_rows", "retained_rows", "excluded_rows"])
    })
    .unwrap();
  assert_eq!(table["data"]["rows"], serde_json::json!([[170, 168, 2]]));
  events
    .iter()
    .find(|e| e["result_type"] == "ClusterBootstrapOlsResult")
    .unwrap()["data"]
    .clone()
}
#[test]
fn actual_script_matches_r_full_inference_all_rows_and_every_replicate_size() {
  let d = TestDir::new();
  fs::write(d.path().join("vietnam.csv"), INPUT).unwrap();
  let fit = fit_from_output(run(&d, "--json"));
  assert_eq!(fit["estimator"], "cluster_pairs_bootstrap_ols");
  assert_eq!(fit["seed"], "10001");
  assert_eq!(fit["rng"], "xoshiro256++/splitmix64/u64-rejection-v1");
  assert_eq!(fit["cluster_order"], "first_seen_complete_case");
  assert_eq!(fit["interval_method"], "percentile_r_type7");
  assert_eq!(fit["confidence_level"], 95.0);
  assert_eq!(fit["observation_count"], 168);
  assert_eq!(fit["cluster_count"], 48);
  assert_eq!(
    fit["sample"],
    serde_json::json!({"total_observations":168,"dropped_observations":0})
  );
  let expected = rows(REFERENCE);
  assert_eq!(expected.len(), 94);
  for r in expected {
    let value = fit
      .pointer(r["path"])
      .unwrap_or_else(|| panic!("missing {}", r["path"]));
    near(
      value.as_f64().unwrap(),
      r["value"].parse().unwrap(),
      r["path"],
    );
  }
  let groups = rows(GROUPS);
  assert_eq!(groups.len(), 48);
  let sizes = groups
    .iter()
    .map(|r| r["observation_count"].parse::<usize>().unwrap())
    .collect::<Vec<_>>();
  assert_eq!(fit["cluster_sizes"], serde_json::json!(sizes));
  let counts = rows(COUNTS);
  assert_eq!(counts.len(), 500);
  let expected_counts = counts
    .iter()
    .enumerate()
    .map(|(i, r)| {
      assert_eq!(r["replication"].parse::<usize>().unwrap(), i + 1);
      r["observations"].parse::<usize>().unwrap()
    })
    .collect::<Vec<_>>();
  assert_eq!(
    fit["replicate_observation_counts"],
    serde_json::json!(expected_counts)
  );
  assert_ne!(expected_counts.iter().min(), expected_counts.iter().max());
  let csv = fs::read_to_string(d.path().join(OUTPUT)).unwrap();
  let actual = rows(&csv);
  let expected = rows(PREDICTIONS);
  assert_eq!(actual.len(), 168);
  assert_eq!(actual.len(), expected.len());
  for (a, b) in actual.iter().zip(expected) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a["source_row"], b["source_row"]);
    for (key, value) in b {
      near(a[key].parse().unwrap(), value.parse().unwrap(), key);
    }
  }
  assert_eq!(
    fs::read_to_string(d.path().join("vietnam.csv")).unwrap(),
    INPUT
  );
  let t = TestDir::new();
  fs::write(t.path().join("vietnam.csv"), INPUT).unwrap();
  let text = run(&t, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert_eq!(fs::read(t.path().join(OUTPUT)).unwrap(), csv.as_bytes());
  let again = TestDir::new();
  fs::write(again.path().join("vietnam.csv"), INPUT).unwrap();
  assert_eq!(fit_from_output(run(&again, "--json")), fit);
  assert_eq!(fs::read(again.path().join(OUTPUT)).unwrap(), csv.as_bytes());
  let collision = run(&d, "--json");
  assert!(!collision.status.success());
  assert_eq!(fs::read_to_string(d.path().join(OUTPUT)).unwrap(), csv);
}
#[test]
fn bootstrap_success_and_failure_keep_original_model_and_table() {
  let d = TestDir::new();
  let path = d.path().join("vietnam.csv");
  fs::write(&path, INPUT).unwrap();
  let mut s = Session::new();
  execute(&mut s, &format!("use \"{}\"", path.display()));
  execute(&mut s, FIT);
  let before = s.state();
  let before_head = execute(&mut s, "head 3");
  let report = execute(&mut s, BOOT);
  assert!(matches!(
    report,
    tabdat_domain::ExecutionResult::ClusterBootstrap(_)
  ));
  assert_eq!(s.state(), before);
  assert_eq!(execute(&mut s, "head 3"), before_head);
  execute(&mut s, "predict before_bootstrap, xb");
  assert_eq!(s.state().estimation, before.estimation);
  // The bootstrap command fails on the first singular resample instead of redrawing it.
  execute(
    &mut s,
    "sql \"\"\"SELECT * FROM (VALUES (1.0,0.0,'a'),(2.0,0.0,'a'),(3.0,0.0,'b'),(8.0,1.0,'b')) AS t(y,x,g)\"\"\" into rank_draw",
  );
  let state = s.state();
  let data = execute(&mut s, "head 3");
  let err = s
    .execute(parse_command("bootstrap y x, cluster(g) reps(100) seed(1)").unwrap())
    .unwrap_err();
  assert!(
    err
      .to_string()
      .contains("no draws were discarded or retried"),
    "{err}"
  );
  assert_eq!(s.state(), state);
  assert_eq!(execute(&mut s, "head 3"), data);
  assert!(matches!(
    s.state().estimation,
    Some(EstimationState::Linear(_))
  ));
  execute(
    &mut s,
    "sql \"\"\"SELECT y, 0.0 AS x, g FROM active\"\"\" into rank_original",
  );
  let state = s.state();
  let data = execute(&mut s, "head 3");
  let err = s
    .execute(parse_command("bootstrap y x, cluster(g) reps(20) seed(1)").unwrap())
    .unwrap_err();
  assert!(
    err.to_string().contains("singular") || err.to_string().contains("collinear"),
    "{err}"
  );
  assert_eq!(s.state(), state);
  assert_eq!(execute(&mut s, "head 3"), data);
}
#[test]
fn actual_script_export_collision_and_invalid_keys_preserve_owner() {
  for collision in [false, true] {
    let d = TestDir::new();
    let source = d.path().join("vietnam.csv");
    let output = d.path().join(OUTPUT);
    fs::write(&source, INPUT).unwrap();
    if collision {
      fs::write(&output, "owner").unwrap();
    }
    let mut s = Session::new();
    let mut fit = None;
    for line in include_str!("../examples/mma24p1olscluster/household_ols_bootstrap.td").lines() {
      if line.starts_with('#') || line.is_empty() {
        continue;
      }
      let text = if line.starts_with("use ") {
        format!("use \"{}\"", source.display())
      } else if line.starts_with("export ") {
        format!("export \"{}\"", output.display())
      } else {
        line.to_owned()
      };
      let result = s.execute(parse_command(&text).unwrap());
      if collision && line.starts_with("export ") {
        assert!(result.is_err());
      } else {
        result.unwrap();
      }
      if line.starts_with("regress ") {
        fit = s.state().estimation;
        assert!(fit.is_some());
      } else if fit.is_some() {
        assert_eq!(s.state().estimation, fit);
      }
    }
    execute(&mut s, "predict after_export, xb");
    assert_eq!(s.state().estimation, fit);
    assert_eq!(fs::read_to_string(&source).unwrap(), INPUT);
    if collision {
      assert_eq!(fs::read_to_string(&output).unwrap(), "owner");
    }
  }
  for invalid in ["duplicate source", "missing group"] {
    let d = TestDir::new();
    let mut lines = INPUT.lines().map(str::to_owned).collect::<Vec<_>>();
    let mut row = lines[1].split(',').map(str::to_owned).collect::<Vec<_>>();
    if invalid == "duplicate source" {
      row[0] = "2".into();
    } else {
      let last = row.len() - 1;
      row[last] = "".into();
    }
    lines[1] = row.join(",");
    let source = lines.join("\n") + "\n";
    fs::write(d.path().join("vietnam.csv"), &source).unwrap();
    fs::write(d.path().join(OUTPUT), "owner").unwrap();
    let out = run(&d, "--json");
    assert!(!out.status.success(), "accepted {invalid}");
    assert!(
      !String::from_utf8(out.stdout)
        .unwrap()
        .contains("ClusterBootstrapOlsResult")
    );
    assert_eq!(
      fs::read_to_string(d.path().join("vietnam.csv")).unwrap(),
      source
    );
    assert_eq!(fs::read_to_string(d.path().join(OUTPUT)).unwrap(), "owner");
  }
}
