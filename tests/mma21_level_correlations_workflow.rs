mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

const INPUT: &str = include_str!("fixtures/mma21_manual_gls_covariance/interior_inputs.csv");
const EXPECTED_ROWS: &str = include_str!("fixtures/mma21_level_correlations/interior_rows.csv");
const EXPECTED_MATRIX: &str =
  include_str!("fixtures/mma21_level_correlations/interior_correlations.csv");
const ROWS: &str = "panel_level_source_rows.csv";
const MATRIX: &str = "panel_level_correlations.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma21p1panel/level_correlations.td"),
    )
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let header: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let values: Vec<_> = line.split(',').collect();
      assert_eq!(header.len(), values.len());
      header.iter().copied().zip(values).collect()
    })
    .collect()
}
#[test]
fn observed_level_correlations_match_r_and_are_descriptive() {
  let dir = TestDir::new();
  fs::write(dir.path().join("MOM.csv"), INPUT).unwrap();
  let out = run(&dir, "--json");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let es: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  assert!(
    es.iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  assert!(
    serde_json::to_string(&es)
      .unwrap()
      .contains("descriptive, not a test or covariance correction")
  );
  let exports: Vec<_> = es
    .iter()
    .filter(|e| e["result_type"] == "ExportResult")
    .map(|e| &e["data"])
    .collect();
  assert_eq!(exports.len(), 2);
  for (e, name, count) in [(exports[0], ROWS, 160), (exports[1], MATRIX, 200)] {
    assert_eq!(e["path"], name);
    assert_eq!(e["dataset"]["path"], name);
    assert_eq!(e["dataset"]["row_count"], count);
  }
  let source = fs::read_to_string(dir.path().join(ROWS)).unwrap();
  let actual = rows(&source);
  let expected = rows(EXPECTED_ROWS);
  assert_eq!((actual.len(), expected.len()), (160, 160));
  for (a, b) in actual.iter().zip(expected) {
    assert_eq!(a.len(), 9);
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for (k, v) in b {
      assert_eq!(
        a[k].parse::<f64>().unwrap(),
        v.parse::<f64>().unwrap(),
        "{k}"
      );
    }
  }
  let matrix = fs::read_to_string(dir.path().join(MATRIX)).unwrap();
  let actual = rows(&matrix);
  let expected = rows(EXPECTED_MATRIX);
  assert_eq!((actual.len(), expected.len()), (200, 200));
  let table: Vec<_> = es
    .iter()
    .filter(|e| {
      e["result_type"] == "TableResult"
        && e["data"]["headers"] == json!(["variable", "year_a", "year_b", "people", "correlation"])
    })
    .collect();
  assert_eq!(table.len(), 1);
  let table_rows = table[0]["data"]["rows"].as_array().unwrap();
  assert_eq!(table_rows.len(), 200);
  for (i, ((a, b), r)) in actual.iter().zip(expected).zip(table_rows).enumerate() {
    let variable = if i < 100 { "lnhr" } else { "lnwg" };
    let x = (1979 + (i % 100) / 10) as f64;
    let y = (1979 + i % 10) as f64;
    assert_eq!(a["variable"], variable);
    assert_eq!(b["variable"], variable);
    assert_eq!(r[0], variable);
    assert_eq!(a["year_a"].parse::<f64>().unwrap(), x);
    assert_eq!(a["year_b"].parse::<f64>().unwrap(), y);
    assert_eq!(a["people"].parse::<f64>().unwrap(), 16.0);
    for key in ["year_a", "year_b", "people", "correlation"] {
      assert!(
        (a[key].parse::<f64>().unwrap() - b[key].parse::<f64>().unwrap()).abs() <= 1e-9,
        "{key}"
      );
    }
    assert_eq!(r[1].as_f64().unwrap(), x);
    assert_eq!(r[2].as_f64().unwrap(), y);
    assert_eq!(r[3], 16);
    let value = a["correlation"].parse::<f64>().unwrap();
    assert!((r[4].as_f64().unwrap() - value).abs() <= 1e-9);
    let transposed = (i / 100) * 100 + (i % 10) * 10 + (i % 100) / 10;
    assert!((value - actual[transposed]["correlation"].parse::<f64>().unwrap()).abs() <= 1e-9);
    if x == y {
      assert!((value - 1.0).abs() <= 1e-9);
    }
  }
  fs::remove_file(dir.path().join(ROWS)).unwrap();
  fs::remove_file(dir.path().join(MATRIX)).unwrap();
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(
    String::from_utf8(text.stdout)
      .unwrap()
      .contains("descriptive, not a test or covariance correction")
  );
  assert_eq!(fs::read_to_string(dir.path().join(ROWS)).unwrap(), source);
  assert_eq!(fs::read_to_string(dir.path().join(MATRIX)).unwrap(), matrix);
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    INPUT
  );
}
#[test]
fn invalid_input_and_constant_year_fail_before_any_export() {
  let lines: Vec<_> = INPUT.lines().collect();
  let mut variants = vec![
    format!("{INPUT}{}\n", lines[1]),
    format!("{}\n", lines[..lines.len() - 1].join("\n")),
  ];
  for (index, value) in [(0, "1.5"), (1, ""), (2, "NaN"), (7, "1.5"), (8, "1978")] {
    let mut changed = lines.clone();
    let mut cells: Vec<_> = changed[1].split(',').collect();
    cells[index] = value;
    let line = cells.join(",");
    changed[1] = &line;
    variants.push(format!("{}\n", changed.join("\n")));
  }
  for index in [1, 2] {
    let mut changed = vec![lines[0].to_owned()];
    for line in &lines[1..] {
      let mut cells: Vec<_> = line.split(',').collect();
      if cells[8] == "1979" {
        cells[index] = "2";
      }
      changed.push(cells.join(","));
    }
    variants.push(format!("{}\n", changed.join("\n")));
  }
  for (i, input) in variants.iter().enumerate() {
    let dir = TestDir::new();
    fs::write(dir.path().join("MOM.csv"), input).unwrap();
    let result = run(&dir, "--json");
    assert!(!result.status.success(), "variant {i}");
    assert!(!result.stderr.is_empty());
    if i >= 7 {
      assert!(String::from_utf8_lossy(&result.stderr).contains(":8: sql failed"));
      assert!(
        String::from_utf8_lossy(&result.stdout)
          .contains("Every year requires positive finite cross-person variance")
      );
    }
    assert!(!dir.path().join(ROWS).exists() && !dir.path().join(MATRIX).exists());
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      *input
    );
  }
}
#[test]
fn export_collisions_preserve_owner_and_document_partial_first_file() {
  for existing in [ROWS, MATRIX] {
    let dir = TestDir::new();
    fs::write(dir.path().join("MOM.csv"), INPUT).unwrap();
    fs::write(dir.path().join(existing), "existing owner\n").unwrap();
    let result = run(&dir, "--json");
    assert!(!result.status.success());
    assert_eq!(
      fs::read_to_string(dir.path().join(existing)).unwrap(),
      "existing owner\n"
    );
    if existing == ROWS {
      assert!(!dir.path().join(MATRIX).exists());
    } else {
      assert_eq!(
        rows(&fs::read_to_string(dir.path().join(ROWS)).unwrap()).len(),
        160
      );
    }
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      INPUT
    );
  }
}
