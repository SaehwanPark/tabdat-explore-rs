mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};
const INPUT: &str = include_str!("fixtures/mma21_fe_level_bootstrap/heterogeneous_inputs.csv");
const OUTPUT: &str = "panel_between_bootstrap_b500_predictions.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma21p1panel/between_bootstrap_b500.td"),
    )
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let headers: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|l| {
      let values: Vec<_> = l.split(',').collect();
      assert_eq!(values.len(), headers.len());
      headers.iter().copied().zip(values).collect()
    })
    .collect()
}
#[test]
fn b500_between_script_matches_independent_qr_and_preserves_point_predictions() {
  let d = TestDir::new();
  fs::write(d.path().join("MOM.csv"), INPUT).unwrap();
  let output = run(&d, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let fit = &events
    .iter()
    .find(|e| e["result_type"] == "BootstrapOlsResult")
    .unwrap()["data"];
  assert_eq!(fit["replications"], 500);
  assert_eq!(fit["seed"], "10001");
  assert_eq!(
    fit["sample"],
    serde_json::json!({"total_observations":16,"dropped_observations":0})
  );
  let reference = rows(include_str!("fixtures/mma21_between_b500/reference.csv"));
  assert_eq!(reference.len(), 18);
  for row in reference {
    let a = fit.pointer(row["path"]).unwrap().as_f64().unwrap();
    let b: f64 = row["value"].parse().unwrap();
    assert!(
      a.is_finite() && (a - b).abs() <= 1e-9,
      "{row:?}: {a} != {b}"
    );
  }
  let csv = fs::read_to_string(d.path().join(OUTPUT)).unwrap();
  let a = rows(&csv);
  let b = rows(include_str!("fixtures/mma21_between_b500/predictions.csv"));
  assert_eq!(a.len(), 16);
  assert_eq!(a.len(), b.len());
  for (a, b) in a.iter().zip(b) {
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for (k, y) in b {
      let x: f64 = a[k].parse().unwrap();
      let y: f64 = y.parse().unwrap();
      assert!((x - y).abs() <= 1e-9, "{k}: {x} != {y}");
    }
  }
  let collided = run(&d, "--json");
  assert_eq!(collided.status.code(), Some(1));
  assert_eq!(fs::read_to_string(d.path().join(OUTPUT)).unwrap(), csv);
  fs::remove_file(d.path().join(OUTPUT)).unwrap();
  let text = run(&d, "--text");
  assert!(text.status.success());
  assert_eq!(fs::read_to_string(d.path().join(OUTPUT)).unwrap(), csv);
  assert_eq!(fs::read_to_string(d.path().join("MOM.csv")).unwrap(), INPUT);
}
#[test]
fn b500_between_preflight_and_first_unidentified_draw_leave_no_export() {
  let lines: Vec<_> = INPUT.lines().collect();
  let mut missing: Vec<_> = lines[1].split(',').collect();
  missing[1] = "";
  for input in [
    format!("{INPUT}{}\n", lines[1]),
    format!("{}\n", lines[..lines.len() - 1].join("\n")),
    format!(
      "{}\n{}\n{}\n",
      lines[0],
      missing.join(","),
      lines[2..].join("\n")
    ),
  ] {
    let d = TestDir::new();
    fs::write(d.path().join("MOM.csv"), &input).unwrap();
    let out = run(&d, "--json");
    assert_eq!(out.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("RegressionResult"));
    assert!(!d.path().join(OUTPUT).exists());
    assert_eq!(fs::read_to_string(d.path().join("MOM.csv")).unwrap(), input);
  }
  let d = TestDir::new();
  let mut input = String::from("source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year\n");
  for id in 1..=3 {
    for t in 0..10 {
      input.push_str(&format!(
        "{},{},{},0,30,900,0,{id},{}\n",
        (id - 1) * 10 + t + 1,
        7.0 + 0.1 * (id * id) as f64 + 0.01 * t as f64,
        id as f64 + 0.01 * t as f64,
        1979 + t
      ));
    }
  }
  fs::write(d.path().join("MOM.csv"), &input).unwrap();
  let out = run(&d, "--json");
  assert_eq!(out.status.code(), Some(1));
  let stdout = String::from_utf8(out.stdout).unwrap();
  assert!(stdout.contains("RegressionResult") && !stdout.contains("BootstrapOlsResult"));
  let error = String::from_utf8_lossy(&out.stderr);
  assert!(
    error.contains("bootstrap") && error.contains("replicate 1"),
    "{error}"
  );
  assert!(!d.path().join(OUTPUT).exists());
  assert_eq!(fs::read_to_string(d.path().join("MOM.csv")).unwrap(), input);
}
