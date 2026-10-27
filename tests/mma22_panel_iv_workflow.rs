mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

fn run(dir: &TestDir) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args(["--json", "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma22p1panel/base_2sls.td"))
    .output()
    .unwrap()
}

fn fixture() -> String {
  let mut state = 0x22f1_9a34_u64;
  let mut draw = || {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    (state % 10000) as f64 / 10000.0
  };
  let mut data = String::from("lnhr,lnwg,kids,ageh,agesq,disab,id,year\n");
  for id in 1..=40 {
    for year in 1979..=1988 {
      let lnhr = draw();
      let lnwg = draw();
      let kids = draw();
      let ageh = draw();
      let agesq = draw();
      let disab = draw();
      data.push_str(&format!(
        "{lnhr},{lnwg},{kids},{ageh},{agesq},{disab},{id},{year}\n"
      ));
    }
  }
  data
}

#[test]
fn complete_panels_fit_both_covariances_and_export_design() {
  let dir = TestDir::new();
  fs::write(dir.path().join("MOMprecise.csv"), fixture()).unwrap();
  let output = run(&dir);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let values: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fits: Vec<_> = values
    .iter()
    .filter(|v| v["result_type"] == "RegressionResult")
    .collect();
  assert_eq!(fits.len(), 2);
  for fit in fits {
    assert_eq!(fit["data"]["observation_count"], 320);
    assert_eq!(fit["data"]["sample"]["dropped_observations"], 0);
    assert_eq!(fit["data"]["include_intercept"], false);
    assert_eq!(fit["data"]["coefficients"].as_array().unwrap().len(), 5);
  }
  let path = dir.path().join("panel_iv_design.csv");
  let original = fs::read(&path).unwrap();
  assert_eq!(String::from_utf8_lossy(&original).lines().count(), 321);
  assert!(!run(&dir).status.success());
  assert_eq!(fs::read(&path).unwrap(), original);

  let shuffled_dir = TestDir::new();
  let fixture = fixture();
  let mut rows: Vec<_> = fixture.lines().collect();
  let header = rows.remove(0);
  rows.reverse();
  let shuffled = format!("{header}\n{}\n", rows.join("\n"));
  fs::write(shuffled_dir.path().join("MOMprecise.csv"), shuffled).unwrap();
  let shuffled_output = run(&shuffled_dir);
  assert!(
    shuffled_output.status.success(),
    "{}",
    String::from_utf8_lossy(&shuffled_output.stderr)
  );
  assert_eq!(
    fs::read(shuffled_dir.path().join("panel_iv_design.csv")).unwrap(),
    original
  );
}

#[test]
fn duplicate_or_missing_years_and_rank_failure_do_not_export() {
  let original = fixture();
  let lines: Vec<_> = original.lines().collect();
  let duplicate = format!("{}\n{}\n", original, lines[1]);
  let missing = format!(
    "{}\n",
    lines
      .iter()
      .enumerate()
      .filter(|(i, _)| *i != 3)
      .map(|(_, line)| *line)
      .collect::<Vec<_>>()
      .join("\n")
  );
  let constant = original
    .lines()
    .enumerate()
    .map(|(i, line)| {
      if i == 0 {
        format!("{line}\n")
      } else {
        let mut row: Vec<_> = line.split(',').collect();
        row[1] = "0";
        format!("{}\n", row.join(","))
      }
    })
    .collect::<String>();
  for data in [duplicate, missing, constant] {
    let dir = TestDir::new();
    fs::write(dir.path().join("MOMprecise.csv"), data).unwrap();
    let output = run(&dir);
    assert!(!output.status.success());
    assert!(!dir.path().join("panel_iv_design.csv").exists());
  }
}
