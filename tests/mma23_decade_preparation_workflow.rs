mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};
const RAW: &str = include_str!("fixtures/mma23_gamma_re_bootstrap/source.asc");
fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn prepare(d: &TestDir, raw: &str, decade: bool) -> std::process::Output {
  fs::write(d.path().join("raw.asc"), raw).unwrap();
  let mut command = Command::new("sh");
  command
    .arg(root().join("examples/mma23p1pannonlin/prepare.sh"))
    .arg(d.path().join("raw.asc"))
    .arg(d.path().join("patents_decade.csv"));
  if decade {
    command.arg("--full-decade");
  }
  command.output().unwrap()
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(root().join("examples/mma23p1pannonlin/decade_preparation.td"))
    .output()
    .unwrap()
}
fn near(a: f64, b: f64) {
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-10 + 1e-12 * b.abs(),
    "{a} vs {b}"
  );
}
fn csv(native: &str, reference: &str, missing: bool) {
  let actual: Vec<_> = native.lines().collect();
  let expected: Vec<_> = reference.lines().collect();
  assert_eq!(actual.len(), expected.len());
  assert_eq!(actual[0], expected[0]);
  for (a, b) in actual.iter().skip(1).zip(expected.iter().skip(1)) {
    let a: Vec<_> = a.split(',').collect();
    let b: Vec<_> = b.split(',').collect();
    assert_eq!(a.len(), 21);
    assert_eq!(b.len(), 21);
    for j in 0..21 {
      let expected = if missing && j == 3 { "" } else { b[j] };
      if expected.is_empty() {
        assert_eq!(a[j], "");
      } else if j < 9 {
        assert_eq!(
          a[j].parse::<f64>().unwrap(),
          expected.parse::<f64>().unwrap()
        );
      } else {
        near(a[j].parse().unwrap(), expected.parse().unwrap());
      }
    }
  }
}
fn check(d: &TestDir, events: &[Value], missing: bool) {
  assert!(
    events
      .iter()
      .all(|v| v["schema_version"] == 1 && v.get("error").is_none())
  );
  csv(
    &fs::read_to_string(d.path().join("patents_decade_prepared.csv")).unwrap(),
    include_str!("fixtures/mma23_decade_preparation/full.csv"),
    missing,
  );
  csv(
    &fs::read_to_string(d.path().join("patents_decade_analysis.csv")).unwrap(),
    include_str!("fixtures/mma23_decade_preparation/selected.csv"),
    missing,
  );
  let summaries: Vec<_> = events
    .iter()
    .filter(|v| v["result_type"] == "SummarizeResult")
    .collect();
  assert_eq!(summaries.len(), 2);
  for (i, cohort) in ["full", "selected"].iter().enumerate() {
    let facts: Vec<Vec<_>> = include_str!("fixtures/mma23_decade_preparation/summaries.csv")
      .lines()
      .skip(1)
      .map(|l| l.split(',').collect())
      .filter(|r: &Vec<&str>| r[0] == *cohort)
      .collect();
    assert_eq!(facts.len(), 21);
    let actual = summaries[i]["data"]["rows"].as_array().unwrap();
    assert_eq!(actual.len(), 21);
    for (a, b) in actual.iter().zip(facts) {
      assert_eq!(a["variable"], b[1]);
      for (j, key) in ["count", "mean", "std_dev", "minimum", "maximum"]
        .iter()
        .enumerate()
      {
        if missing && b[1] == "ardssic" {
          if j == 0 {
            assert_eq!(a[key], 0);
          } else {
            assert!(a[key].is_null());
          }
        } else if b[j + 2].is_empty() {
          assert!(a[key].is_null());
        } else {
          near(a[key].as_f64().unwrap(), b[j + 2].parse().unwrap());
        }
      }
    }
  }
}
fn events(out: &std::process::Output) -> Vec<Value> {
  assert!(
    out.status.success(),
    "{}\n{}",
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  );
  String::from_utf8_lossy(&out.stdout)
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect()
}
#[test]
fn full_history_selected_lags_and_summaries_match_r_text_and_no_clobber() {
  let d = TestDir::new();
  assert!(prepare(&d, RAW, true).status.success());
  let e = events(&run(&d, "--json"));
  check(&d, &e, false);
  let full = fs::read(d.path().join("patents_decade_prepared.csv")).unwrap();
  let selected = fs::read(d.path().join("patents_decade_analysis.csv")).unwrap();
  let text = TestDir::new();
  assert!(prepare(&text, RAW, true).status.success());
  assert!(run(&text, "--text").status.success());
  assert_eq!(
    full,
    fs::read(text.path().join("patents_decade_prepared.csv")).unwrap()
  );
  assert_eq!(
    selected,
    fs::read(text.path().join("patents_decade_analysis.csv")).unwrap()
  );
  assert!(!run(&d, "--json").status.success());
  assert_eq!(
    full,
    fs::read(d.path().join("patents_decade_prepared.csv")).unwrap()
  );
  assert_eq!(
    selected,
    fs::read(d.path().join("patents_decade_analysis.csv")).unwrap()
  );
  let old = TestDir::new();
  assert!(prepare(&old, RAW, false).status.success());
  let raw = fs::read_to_string(d.path().join("patents_decade.csv")).unwrap();
  let mut lines = raw.lines();
  let mut projected = vec![lines.next().unwrap()];
  projected.extend(lines.filter(|l| l.split(',').nth(1).unwrap().parse::<i32>().unwrap() >= 1975));
  assert_eq!(
    fs::read_to_string(old.path().join("patents_decade.csv")).unwrap(),
    format!("{}\n", projected.join("\n"))
  );
}
#[test]
fn shuffled_input_and_wholly_missing_nonmodel_metadata_preserve_history() {
  for missing in [false, true] {
    let d = TestDir::new();
    assert!(prepare(&d, RAW, true).status.success());
    let p = d.path().join("patents_decade.csv");
    let raw = fs::read_to_string(&p).unwrap();
    let mut lines = raw.lines();
    let header = lines.next().unwrap();
    let mut records: Vec<_> = lines
      .map(|l| {
        let mut cells: Vec<_> = l.split(',').collect();
        if missing {
          cells[3] = "";
        }
        cells.join(",")
      })
      .collect();
    records.reverse();
    fs::write(&p, format!("{header}\n{}\n", records.join("\n"))).unwrap();
    check(&d, &events(&run(&d, "--json")), missing);
  }
}
#[test]
fn raw_and_key_failures_never_publish_invalid_exports_or_overwrite_inputs() {
  let invalid = TestDir::new();
  assert!(!prepare(&invalid, "1 2 3", true).status.success());
  assert!(!invalid.path().join("patents_decade.csv").exists());
  let owned = TestDir::new();
  fs::write(owned.path().join("patents_decade.csv"), "owned").unwrap();
  assert!(!prepare(&owned, RAW, true).status.success());
  assert_eq!(
    fs::read_to_string(owned.path().join("patents_decade.csv")).unwrap(),
    "owned"
  );
  for case in 0..3 {
    let d = TestDir::new();
    assert!(prepare(&d, RAW, true).status.success());
    let p = d.path().join("patents_decade.csv");
    let raw = fs::read_to_string(&p).unwrap();
    let mut lines: Vec<_> = raw.lines().map(str::to_owned).collect();
    match case {
      0 => {
        lines.remove(1);
      }
      1 => {
        lines[1] = lines[2].clone();
      }
      _ => {
        let mut cells: Vec<_> = lines[1].split(',').collect();
        cells[1] = "1980";
        lines[1] = cells.join(",");
      }
    }
    let changed = format!("{}\n", lines.join("\n"));
    fs::write(&p, &changed).unwrap();
    assert!(!run(&d, "--json").status.success());
    assert_eq!(fs::read_to_string(p).unwrap(), changed);
    assert!(!d.path().join("patents_decade_prepared.csv").exists());
    assert!(!d.path().join("patents_decade_analysis.csv").exists());
  }
}
