mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf};

fn input() -> String {
  let mut csv = String::from("source_row,id,year,lnhr,lnwg\n");
  for (i, line) in
    include_str!("../crates/tabdat-stats/tests/fixtures/panel_classical_hausman/data.csv")
      .lines()
      .skip(1)
      .enumerate()
  {
    let v: Vec<_> = line.split(',').collect();
    let year: usize = v[1].parse().unwrap();
    csv.push_str(&format!(
      "{},{},{},{},{}\n",
      i + 1,
      v[0],
      1978 + year,
      v[2],
      v[3]
    ));
  }
  csv
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma21p1panel/classical_hausman.td"),
    )
    .output()
    .unwrap()
}

#[test]
fn classical_panel_hausman_text_json_and_no_clobber() {
  let d = TestDir::new();
  fs::write(d.path().join("MOM.csv"), input()).unwrap();
  let o = run(&d, "--json");
  assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
  let events: Vec<Value> = String::from_utf8(o.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let result = &events
    .iter()
    .find(|v| v["result_type"] == "ClassicalPanelHausmanResult")
    .unwrap()["data"];
  assert_eq!(
    result["test"],
    "classical_panel_fe_re_covariance_difference"
  );
  assert_eq!(result["reference_distribution"], "asymptotic_chi_square");
  assert_eq!(result["covariance"], "model_based_nonrobust");
  for line in
    include_str!("../crates/tabdat-stats/tests/fixtures/panel_classical_hausman/reference.csv")
      .lines()
      .skip(1)
  {
    let (field, expected) = line.split_once(',').unwrap();
    let actual = result[field].as_f64().unwrap();
    let expected: f64 = expected.parse().unwrap();
    assert!(
      (actual - expected).abs() <= 1e-10 + 1e-10 * expected.abs(),
      "{field}: {actual} != {expected}"
    );
  }
  let path = d.path().join("panel_classical_hausman_predictions.csv");
  let bytes = fs::read(&path).unwrap();
  assert_eq!(String::from_utf8_lossy(&bytes).lines().count(), 201);
  assert!(!run(&d, "--json").status.success());
  assert_eq!(fs::read(&path).unwrap(), bytes);
  fs::remove_file(&path).unwrap();
  let text = run(&d, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(String::from_utf8_lossy(&text.stdout).contains("not cluster robust"));
  assert_eq!(fs::read(&path).unwrap(), bytes);
}
