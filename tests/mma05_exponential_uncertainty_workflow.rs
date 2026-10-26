mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

const INPUT: &str = "source_row,y,x\n1,0.3,-1\n2,0.8,0\n3,0.5,1\n4,1.8,2\n5,2.2,3\n6,4,4\n";

fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma05p4margeffects/uncertainty.td"),
    )
    .output()
    .unwrap()
}

#[test]
fn exponential_effect_intervals_match_independent_profile_and_preserve_export() {
  let dir = TestDir::new();
  fs::write(dir.path().join("nls.csv"), INPUT).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let results: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let table = results
    .iter()
    .find(|v| v["data"]["headers"][1] == "effect")
    .unwrap();
  let rows = table["data"]["rows"].as_array().unwrap();
  assert_eq!(rows.len(), 3);
  // Base R profile-likelihood fit and central numerical delta gradients.
  let references = [
    (0.7701924350951258, 0.6851982307344401),
    (0.9899217234710385, 0.9958050033095290),
    (0.5603547567466023, 0.3594079324494844),
  ];
  for (row, (estimate, se)) in rows.iter().zip(references) {
    assert_eq!(row[0], 6);
    assert!((row[2].as_f64().unwrap() - estimate).abs() < 1e-10);
    assert!((row[3].as_f64().unwrap() - se).abs() < 1e-9);
    assert!((row[4].as_f64().unwrap() - (estimate - 1.959963984540054 * se)).abs() < 2e-9);
    assert!((row[5].as_f64().unwrap() - (estimate + 1.959963984540054 * se)).abs() < 2e-9);
  }
  let path = dir.path().join("effects_uncertainty.csv");
  let bytes = fs::read(&path).unwrap();
  assert!(!run(&dir, "--text").status.success());
  assert_eq!(fs::read(&path).unwrap(), bytes);
  fs::remove_file(&path).unwrap();
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert_eq!(fs::read(&path).unwrap(), bytes);
}

#[test]
fn bad_sample_stops_before_export() {
  for input in [
    "source_row,y,x\n1,0,-1\n2,1,0\n3,2,1\n",
    "source_row,y,x\n1,1,0\n2,2,0\n3,3,0\n",
    "source_row,y,x\n1,1,0\n1,2,1\n3,3,2\n",
  ] {
    let dir = TestDir::new();
    fs::write(dir.path().join("nls.csv"), input).unwrap();
    assert!(!run(&dir, "--json").status.success());
    assert!(!dir.path().join("effects_uncertainty.csv").exists());
  }
}
