mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

const INPUT: &str = include_str!("fixtures/mma21_fd_residual_correlations/interior_inputs.csv");
const PAIRS: &str = "panel_individual_lnhr_pairs.csv";
const CORRELATIONS: &str = "panel_individual_lnhr_correlations.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma21p1panel/individual_autocorrelation.td"),
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
fn within_person_correlations_match_independent_r_and_not_cross_person() {
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
      .contains("nine adjacent years within each person, not people across years")
  );
  assert!(
    serde_json::to_string(&es)
      .unwrap()
      .contains("descriptive, not a test or covariance correction")
  );
  let select = |kind: &str| {
    es.iter()
      .filter(|e| e["result_type"] == kind)
      .map(|e| &e["data"])
      .collect::<Vec<_>>()
  };
  assert_eq!(
    es[es.len() - 4..]
      .iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "ExportResult",
      "SqlCreateResult",
      "TableResult",
      "ExportResult"
    ]
  );
  let exports = select("ExportResult");
  assert_eq!(exports.len(), 2);
  for (e, name, count) in [(exports[0], PAIRS, 144), (exports[1], CORRELATIONS, 16)] {
    assert_eq!(e["path"], name);
    assert_eq!(e["dataset"]["path"], name);
    assert_eq!(e["dataset"]["row_count"], count);
  }
  let pairs = fs::read_to_string(dir.path().join(PAIRS)).unwrap();
  let correlations = fs::read_to_string(dir.path().join(CORRELATIONS)).unwrap();
  let actual = rows(&pairs);
  let expected = rows(include_str!(
    "fixtures/mma21_individual_autocorrelation/interior_pairs.csv"
  ));
  assert_eq!((actual.len(), expected.len()), (144, 144));
  for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
    assert_eq!(a.len(), 12);
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    let val = |key: &str| a[key].parse::<f64>().unwrap();
    assert_eq!(val("id"), (1 + i / 9) as f64);
    assert_eq!(val("year"), (1980 + i % 9) as f64);
    assert_eq!(val("previous_year") + 1.0, val("year"));
    assert_eq!(val("source_row") - 16.0, val("previous_source_row"));
    for (key, value) in b {
      assert_eq!(val(key), value.parse::<f64>().unwrap(), "{key}");
    }
  }
  let actual = rows(&correlations);
  let expected = rows(include_str!(
    "fixtures/mma21_individual_autocorrelation/interior_correlations.csv"
  ));
  assert_eq!((actual.len(), expected.len()), (16, 16));
  let tables = select("TableResult");
  let table = tables.last().unwrap();
  assert_eq!(table["headers"], json!(["id", "pairs", "correlation"]));
  let table_rows = table["rows"].as_array().unwrap();
  assert_eq!(table_rows.len(), 16);
  for (i, ((a, b), r)) in actual.iter().zip(expected).zip(table_rows).enumerate() {
    assert_eq!(a["id"].parse::<f64>().unwrap(), (i + 1) as f64);
    assert_eq!(a["pairs"].parse::<f64>().unwrap(), 9.0);
    assert_eq!(r[0].as_f64().unwrap(), (i + 1) as f64);
    assert_eq!(r[1], 9);
    for key in ["id", "pairs", "correlation"] {
      assert!(
        (a[key].parse::<f64>().unwrap() - b[key].parse::<f64>().unwrap()).abs() <= 1e-9,
        "{key}"
      );
    }
    assert!((r[2].as_f64().unwrap() - a["correlation"].parse::<f64>().unwrap()).abs() <= 1e-9);
  }
  // Independent R cross-person correlation for the control's 1979/1980 values.
  assert!((actual[0]["correlation"].parse::<f64>().unwrap() - 0.485248998051049).abs() > 0.1);
  fs::remove_file(dir.path().join(PAIRS)).unwrap();
  fs::remove_file(dir.path().join(CORRELATIONS)).unwrap();
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(
    String::from_utf8(text.stdout)
      .unwrap()
      .contains("nine adjacent years within each person, not people across years")
  );
  assert_eq!(fs::read_to_string(dir.path().join(PAIRS)).unwrap(), pairs);
  assert_eq!(
    fs::read_to_string(dir.path().join(CORRELATIONS)).unwrap(),
    correlations
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    INPUT
  );
}
#[test]
fn invalid_keys_missing_sample_and_current_or_lag_constant_fail_before_exports() {
  let lines: Vec<_> = INPUT.lines().collect();
  let mut variants = vec![
    format!("{INPUT}{}\n", lines[1]),
    format!("{}\n", lines[..lines.len() - 1].join("\n")),
  ];
  for (index, value) in [(0, "1.5"), (1, ""), (2, "NaN"), (7, "1.5"), (8, "1978")] {
    let mut changed = lines.clone();
    let mut c: Vec<_> = changed[1].split(',').collect();
    c[index] = value;
    let line = c.join(",");
    changed[1] = &line;
    variants.push(format!("{}\n", changed.join("\n")));
  }
  for current in [true, false] {
    let mut changed = vec![lines[0].to_owned()];
    for line in &lines[1..] {
      let mut c: Vec<_> = line.split(',').collect();
      let year: usize = c[8].parse().unwrap();
      if c[7] == "1" && ((current && year >= 1980) || (!current && year <= 1987)) {
        c[1] = "7";
      }
      changed.push(c.join(","));
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
      assert!(String::from_utf8_lossy(&result.stderr).contains(":11: sql failed"));
      assert!(
        String::from_utf8_lossy(&result.stdout)
          .contains("population standard deviation greater than 1e-12")
      );
    }
    assert!(!dir.path().join(PAIRS).exists() && !dir.path().join(CORRELATIONS).exists());
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      *input
    );
  }
}
#[test]
fn threshold_rejects_unresolved_but_accepts_resolved_control() {
  let lines: Vec<_> = INPUT.lines().collect();
  for (step, should_succeed) in [(1e-13, false), (1e-11, true), (1e-9, true), (1e-8, true)] {
    let mut changed = vec![lines[0].to_owned()];
    for line in &lines[1..] {
      let mut c: Vec<_> = line.split(',').map(str::to_owned).collect();
      if c[7] == "1" {
        let t = c[8].parse::<usize>().unwrap() - 1979;
        c[1] = (7.0 + step * t as f64).to_string();
      }
      changed.push(c.join(","));
    }
    let input = format!("{}\n", changed.join("\n"));
    let dir = TestDir::new();
    fs::write(dir.path().join("MOM.csv"), &input).unwrap();
    let result = run(&dir, "--json");
    assert_eq!(
      result.status.success(),
      should_succeed,
      "{}",
      String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(dir.path().join(PAIRS).exists(), should_succeed);
    assert_eq!(dir.path().join(CORRELATIONS).exists(), should_succeed);
    if should_succeed {
      let output = fs::read_to_string(dir.path().join(CORRELATIONS)).unwrap();
      let correlation: f64 = rows(&output)[0]["correlation"].parse().unwrap();
      assert!(
        (correlation - 1.0).abs() <= 1e-9,
        "step {step}: {correlation}"
      );
    }
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      input
    );
  }
}
#[test]
fn export_collisions_preserve_owners_and_partial_first_file() {
  for existing in [PAIRS, CORRELATIONS] {
    let dir = TestDir::new();
    fs::write(dir.path().join("MOM.csv"), INPUT).unwrap();
    fs::write(dir.path().join(existing), "existing owner\n").unwrap();
    let result = run(&dir, "--json");
    assert!(!result.status.success());
    assert_eq!(
      fs::read_to_string(dir.path().join(existing)).unwrap(),
      "existing owner\n"
    );
    if existing == PAIRS {
      assert!(!dir.path().join(CORRELATIONS).exists());
    } else {
      assert_eq!(
        rows(&fs::read_to_string(dir.path().join(PAIRS)).unwrap()).len(),
        144
      );
    }
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      INPUT
    );
  }
}
