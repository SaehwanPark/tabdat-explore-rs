mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

const INPUT: &str = include_str!("fixtures/mma21_manual_gls_covariance/interior_inputs.csv");
const ROWS: &str = "panel_pooled_residual_rows.csv";
const MATRIX: &str = "panel_pooled_residual_correlations.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma21p1panel/pooled_residual_correlations.td"),
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
fn interleaved_pooled_correlations_match_independent_qr_and_matrix() {
  let dir = TestDir::new();
  fs::write(dir.path().join("MOM.csv"), INPUT).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let es: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  let select = |kind: &str| {
    es.iter()
      .filter(|e| e["result_type"] == kind)
      .map(|e| &e["data"])
      .collect::<Vec<_>>()
  };
  let fit = select("RegressionResult");
  let lincom = select("LincomResult");
  assert_eq!((fit.len(), lincom.len()), (1, 1));
  assert_eq!(fit[0]["covariance"], "nonrobust");
  assert_eq!(
    fit[0]["sample"],
    json!({"total_observations":160,"dropped_observations":0})
  );
  let reference = rows(include_str!(
    "fixtures/mma21_pooled_residual_correlations/reference.csv"
  ));
  assert_eq!(reference.len(), 25);
  for row in reference {
    let object = match row["result_type"] {
      "RegressionResult" => fit[0],
      "LincomResult" => lincom[0],
      _ => panic!(),
    };
    let actual = object.pointer(row["path"]).unwrap().as_f64().unwrap();
    let expected: f64 = row["value"].parse().unwrap();
    let tolerance = if row["path"].ends_with("p_value") {
      expected.abs() * 1e-8
    } else {
      1e-9
    };
    assert!(
      actual.is_finite() && (actual - expected).abs() <= tolerance,
      "{row:?}: {actual} != {expected}"
    );
  }
  assert_eq!(
    es[es.len() - 4..]
      .iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "SqlCreateResult",
      "TableResult",
      "LincomResult",
      "ExportResult"
    ]
  );
  assert!(
    serde_json::to_string(&es)
      .unwrap()
      .contains("descriptive, not a test or covariance correction")
  );
  let prediction = fs::read_to_string(dir.path().join(ROWS)).unwrap();
  let matrix = fs::read_to_string(dir.path().join(MATRIX)).unwrap();
  let actual = rows(&prediction);
  let expected = rows(include_str!(
    "fixtures/mma21_pooled_residual_correlations/rows.csv"
  ));
  assert_eq!((actual.len(), expected.len()), (160, 160));
  for (a, b) in actual.iter().zip(expected) {
    assert_eq!(a.len(), 11);
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for (key, value) in b {
      let x: f64 = a[key].parse().unwrap();
      let y: f64 = value.parse().unwrap();
      if [
        "source_row",
        "lnhr",
        "lnwg",
        "kids",
        "ageh",
        "agesq",
        "disab",
        "id",
        "year",
      ]
      .contains(&key)
      {
        assert_eq!(x, y, "{key}");
      } else {
        assert!((x - y).abs() <= 1e-9, "{key}: {x} != {y}");
      }
    }
  }
  let actual = rows(&matrix);
  let expected = rows(include_str!(
    "fixtures/mma21_pooled_residual_correlations/correlations.csv"
  ));
  assert_eq!((actual.len(), expected.len()), (100, 100));
  let tables = select("TableResult");
  let table = tables.last().unwrap();
  assert_eq!(
    table["headers"],
    json!(["year_a", "year_b", "people", "correlation"])
  );
  let table_rows = table["rows"].as_array().unwrap();
  assert_eq!(table_rows.len(), 100);
  for (i, ((a, b), table_row)) in actual.iter().zip(expected).zip(table_rows).enumerate() {
    let x = (1979 + i / 10) as f64;
    let y = (1979 + i % 10) as f64;
    assert_eq!(a["year_a"].parse::<f64>().unwrap(), x);
    assert_eq!(a["year_b"].parse::<f64>().unwrap(), y);
    assert_eq!(a["people"].parse::<f64>().unwrap(), 16.0);
    for key in ["year_a", "year_b", "people", "correlation"] {
      assert!(
        (a[key].parse::<f64>().unwrap() - b[key].parse::<f64>().unwrap()).abs() <= 1e-9,
        "{key}"
      );
    }
    assert_eq!(table_row[0].as_f64().unwrap(), x);
    assert_eq!(table_row[1].as_f64().unwrap(), y);
    assert_eq!(table_row[2], 16);
    assert!(
      (table_row[3].as_f64().unwrap() - a["correlation"].parse::<f64>().unwrap()).abs() <= 1e-9
    );
  }
  fs::remove_file(dir.path().join(ROWS)).unwrap();
  fs::remove_file(dir.path().join(MATRIX)).unwrap();
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let stdout = String::from_utf8(text.stdout).unwrap();
  assert!(stdout.contains("descriptive, not a test") && stdout.contains("CI lower (95%)"));
  assert_eq!(
    fs::read_to_string(dir.path().join(ROWS)).unwrap(),
    prediction
  );
  assert_eq!(fs::read_to_string(dir.path().join(MATRIX)).unwrap(), matrix);
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    INPUT
  );
}
#[test]
fn invalid_keys_samples_rank_and_constant_year_fail_before_export() {
  let lines: Vec<_> = INPUT.lines().collect();
  let mut variants = vec![
    format!("{INPUT}{}\n", lines[1]),
    format!("{}\n", lines[..lines.len() - 1].join("\n")),
  ];
  for (index, value) in [(2, ""), (7, "1.5")] {
    let mut changed = lines.clone();
    let mut cells: Vec<_> = changed[1].split(',').collect();
    cells[index] = value;
    let line = cells.join(",");
    changed[1] = &line;
    variants.push(format!("{}\n", changed.join("\n")));
  }
  for kind in 0..2 {
    let mut changed = vec![lines[0].to_owned()];
    for line in &lines[1..] {
      let mut cells: Vec<_> = line.split(',').map(str::to_owned).collect();
      if kind == 0 {
        cells[2] = "2".to_owned();
      } else if cells[8] == "1979" {
        cells[1] = "7".to_owned();
        cells[2] = "2".to_owned();
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
    if i == 5 {
      // The SQL boundary masks DuckDB error text; the preceding scope note provides recovery guidance.
      assert!(String::from_utf8_lossy(&result.stderr).contains(":11: sql failed"));
      assert!(
        String::from_utf8_lossy(&result.stdout)
          .contains("Every year requires positive finite cross-person residual variance")
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
fn both_export_collisions_preserve_owners_and_explicit_partial_output() {
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
