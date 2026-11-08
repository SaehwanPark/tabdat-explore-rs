mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
};
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
fn reference(d: &TestDir) -> PathBuf {
  let path = d.path().join("reference");
  fs::create_dir(&path).unwrap();
  for (name, text) in [
    (
      "full.csv",
      include_str!("fixtures/mma23_decade_preparation/full.csv"),
    ),
    (
      "selected.csv",
      include_str!("fixtures/mma23_decade_preparation/selected.csv"),
    ),
    (
      "summaries.csv",
      include_str!("fixtures/mma23_decade_preparation/summaries.csv"),
    ),
  ] {
    fs::write(path.join(name), text).unwrap();
  }
  path
}
fn checker(d: &TestDir, raw: &Path, reference: &Path) -> std::process::Output {
  Command::new("python3")
    .arg(root().join("examples/mma23p1pannonlin/check_decade_preparation.py"))
    .arg(d.path())
    .arg(raw)
    .arg(reference)
    .arg("--control")
    .output()
    .unwrap()
}
fn audit(d: &TestDir, raw: &Path, reference: &Path) {
  let out = checker(d, raw, reference);
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
}
fn reject(d: &TestDir, raw: &Path, reference: &Path) {
  let out = checker(d, raw, reference);
  assert!(
    !out.status.success(),
    "checker accepted forged artifacts without an error"
  );
}
fn set_field(path: &Path, row: usize, field: usize, value: &str) {
  let text = fs::read_to_string(path).unwrap();
  let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
  let mut cells: Vec<String> = lines[row].split(',').map(str::to_owned).collect();
  cells[field] = value.to_owned();
  lines[row] = cells.join(",");
  fs::write(path, format!("{}\n", lines.join("\n"))).unwrap();
}
// Change a numeric field substantially enough to exceed the declared tolerance.
fn forge_field(path: &Path, row: usize, field: usize) -> String {
  let text = fs::read_to_string(path).unwrap();
  let old = text
    .lines()
    .nth(row)
    .unwrap()
    .split(',')
    .nth(field)
    .unwrap();
  let wrong = (old.parse::<f64>().unwrap() * 1.1 + 0.125).to_string();
  set_field(path, row, field, &wrong);
  wrong
}
fn forge_events(d: &TestDir, forge: impl FnOnce(&mut Vec<Value>)) {
  let path = d.path().join("result.jsonl");
  let original = fs::read_to_string(&path).unwrap();
  let mut events: Vec<Value> = original
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  forge(&mut events);
  let lines: Vec<_> = events.iter().map(Value::to_string).collect();
  fs::write(path, format!("{}\n", lines.join("\n"))).unwrap();
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
#[test]
fn independent_checker_rejects_forged_source_exports_results_references_and_raw_tokens() {
  let d = TestDir::new();
  assert!(prepare(&d, RAW, true).status.success());
  let out = run(&d, "--json");
  assert_eq!(events(&out).len(), 21);
  fs::write(d.path().join("result.jsonl"), &out.stdout).unwrap();
  let raw = d.path().join("raw.asc");
  let reference = reference(&d);
  audit(&d, &raw, &reference);
  for (name, row, field) in [
    ("patents_decade.csv", 1, 7),
    ("patents_decade_prepared.csv", 1, 11),
    ("patents_decade_analysis.csv", 1, 8),
  ] {
    let path = d.path().join(name);
    let original = fs::read_to_string(&path).unwrap();
    forge_field(&path, row, field);
    reject(&d, &raw, &reference);
    fs::write(path, original).unwrap();
  }
  for (name, row, field) in [
    ("full.csv", 1, 11),
    ("selected.csv", 1, 8),
    ("summaries.csv", 1, 3),
  ] {
    let path = reference.join(name);
    let original = fs::read_to_string(&path).unwrap();
    forge_field(&path, row, field);
    reject(&d, &raw, &reference);
    fs::write(path, original).unwrap();
  }
  let result = fs::read_to_string(d.path().join("result.jsonl")).unwrap();
  forge_events(&d, |events| {
    events[16]["data"]["rows"][0]["mean"] = serde_json::json!(999.0)
  });
  reject(&d, &raw, &reference);
  fs::write(d.path().join("result.jsonl"), &result).unwrap();
  forge_events(&d, |events| {
    events[17]["data"]["dataset"]["row_count"] = serde_json::json!(399)
  });
  reject(&d, &raw, &reference);
  fs::write(d.path().join("result.jsonl"), &result).unwrap();
  forge_events(&d, |events| {
    events[15]["data"]["metadata"]["variable_labels"][0][1] = serde_json::json!("Wrong")
  });
  reject(&d, &raw, &reference);
  fs::write(d.path().join("result.jsonl"), &result).unwrap();
  forge_events(&d, |events| {
    events.remove(5);
  });
  reject(&d, &raw, &reference);
  fs::write(d.path().join("result.jsonl"), &result).unwrap();
  // Shared native/reference corruption must still fail raw empirical reconstruction.
  let summaries = reference.join("summaries.csv");
  let original_summaries = fs::read_to_string(&summaries).unwrap();
  let wrong: f64 = forge_field(&summaries, 1, 3).parse().unwrap();
  forge_events(&d, |events| {
    events[16]["data"]["rows"][0]["mean"] = serde_json::json!(wrong)
  });
  reject(&d, &raw, &reference);
  fs::write(&summaries, original_summaries).unwrap();
  fs::write(d.path().join("result.jsonl"), &result).unwrap();
  // Shared export/reference corruption must still fail source/derived reconstruction.
  for (native, reference_file, row, field) in [
    ("patents_decade_prepared.csv", "full.csv", 1, 11),
    ("patents_decade_analysis.csv", "selected.csv", 1, 8),
  ] {
    let native_path = d.path().join(native);
    let reference_path = reference.join(reference_file);
    let native_original = fs::read_to_string(&native_path).unwrap();
    let reference_original = fs::read_to_string(&reference_path).unwrap();
    let wrong = forge_field(&native_path, row, field);
    set_field(&reference_path, row, field, &wrong);
    reject(&d, &raw, &reference);
    fs::write(native_path, native_original).unwrap();
    fs::write(reference_path, reference_original).unwrap();
  }
  // Raw-token corruption changes independently derived expectations.
  let mut changed: Vec<_> = RAW.split_whitespace().collect();
  changed[5] = "1.5";
  fs::write(d.path().join("raw-forged.asc"), changed.join(" ")).unwrap();
  reject(&d, &d.path().join("raw-forged.asc"), &reference);
  audit(&d, &raw, &reference);
}
