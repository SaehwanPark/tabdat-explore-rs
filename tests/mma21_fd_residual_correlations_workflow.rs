mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

const INPUT: &str = include_str!("fixtures/mma21_fd_residual_correlations/interior_inputs.csv");
const ROWS: &str = "panel_fd_residual_rows.csv";
const MATRIX: &str = "panel_fd_residual_correlations.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma21p1panel/fd_residual_correlations.td"),
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
fn interleaved_fd_fit_and_correlations_match_independent_qr() {
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
  let select = |kind: &str| {
    es.iter()
      .filter(|e| e["result_type"] == kind)
      .map(|e| &e["data"])
      .collect::<Vec<_>>()
  };
  let fits = select("RegressionResult");
  let lincoms = select("LincomResult");
  assert_eq!((fits.len(), lincoms.len()), (1, 1));
  let fit = fits[0];
  let lincom = lincoms[0];
  assert_eq!(fit["outcome"], "dlnhr");
  assert_eq!(fit["predictors"], json!(["dlnwg"]));
  assert_eq!(fit["covariance"], "nonrobust");
  assert_eq!(fit["include_intercept"], true);
  assert_eq!(
    fit["sample"],
    json!({"total_observations":144,"dropped_observations":0})
  );
  let reference = rows(include_str!(
    "fixtures/mma21_fd_residual_correlations/interior_reference.csv"
  ));
  assert_eq!(reference.len(), 25);
  for row in reference {
    let object = match row["result_type"] {
      "RegressionResult" => fit,
      "LincomResult" => lincom,
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
    es[es.len() - 5..]
      .iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "ExportResult",
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
  let exports = select("ExportResult");
  assert_eq!(exports.len(), 2);
  for (e, name, count) in [(exports[0], ROWS, 144), (exports[1], MATRIX, 243)] {
    assert_eq!(e["path"], name);
    assert_eq!(e["dataset"]["path"], name);
    assert_eq!(e["dataset"]["row_count"], count);
  }
  let prediction = fs::read_to_string(dir.path().join(ROWS)).unwrap();
  let matrix = fs::read_to_string(dir.path().join(MATRIX)).unwrap();
  let actual = rows(&prediction);
  let expected = rows(include_str!(
    "fixtures/mma21_fd_residual_correlations/interior_rows.csv"
  ));
  assert_eq!((actual.len(), expected.len()), (144, 144));
  for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
    assert_eq!(a.len(), 17);
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    let val = |key: &str| a[key].parse::<f64>().unwrap();
    assert_eq!(val("id"), (1 + i / 9) as f64);
    assert_eq!(val("year"), (1980 + i % 9) as f64);
    assert_eq!(val("previous_year") + 1.0, val("year"));
    assert_eq!(val("source_row") - 16.0, val("previous_source_row"));
    assert!((val("lnhr") - val("previous_lnhr") - val("dlnhr")).abs() <= 1e-9);
    assert!((val("lnwg") - val("previous_lnwg") - val("dlnwg")).abs() <= 1e-9);
    assert!((val("dlnhr") - val("fd_fitted") - val("fd_residual")).abs() <= 1e-9);
    for (key, value) in b {
      let x = val(key);
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
        "previous_source_row",
        "previous_year",
        "previous_lnhr",
        "previous_lnwg",
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
    "fixtures/mma21_fd_residual_correlations/interior_correlations.csv"
  ));
  assert_eq!((actual.len(), expected.len()), (243, 243));
  let tables = select("TableResult");
  let table = tables.last().unwrap();
  assert_eq!(
    table["headers"],
    json!(["variable", "year_a", "year_b", "people", "correlation"])
  );
  let table_rows = table["rows"].as_array().unwrap();
  assert_eq!(table_rows.len(), 243);
  for (i, ((a, b), r)) in actual.iter().zip(expected).zip(table_rows).enumerate() {
    let variable = ["dlnhr", "dlnwg", "fd_residual"][i / 81];
    let x = (1980 + (i % 81) / 9) as f64;
    let y = (1980 + i % 9) as f64;
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
    let transposed = (i / 81) * 81 + (i % 9) * 9 + (i % 81) / 9;
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
fn invalid_rank_and_all_three_unresolved_series_fail_before_export() {
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
  let first: BTreeMap<_, _> = lines[1..]
    .iter()
    .filter_map(|line| {
      let c: Vec<_> = line.split(',').collect();
      (c[8] == "1979").then_some((
        c[7].to_owned(),
        (c[1].parse::<f64>().unwrap(), c[2].parse::<f64>().unwrap()),
      ))
    })
    .collect();
  for kind in 0..4 {
    let mut changed = vec![lines[0].to_owned()];
    for line in &lines[1..] {
      let mut c: Vec<_> = line.split(',').map(str::to_owned).collect();
      let g: usize = c[7].parse().unwrap();
      let t: usize = c[8].parse::<usize>().unwrap() - 1979;
      match kind {
        0 => c[2] = "2".to_owned(),
        1 | 2 if t == 1 => {
          let previous = first[&c[7]];
          if kind == 1 {
            c[1] = (previous.0 + 0.25).to_string();
          } else {
            c[2] = (previous.1 + 0.25).to_string();
          }
        }
        3 => {
          let x = 2.0 + t as f64 / 8.0 + (g % 4) as f64 * t as f64 / 64.0;
          let e = ((g - 1) / 4) as f64 - 1.5;
          let h = t.saturating_sub(1) as f64;
          c[2] = x.to_string();
          c[1] = (7.0 + 2.0 * x + e * h * h / 64.0).to_string();
        }
        _ => {}
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
    if i >= 8 {
      assert!(String::from_utf8_lossy(&result.stderr).contains(":13: sql failed"));
      assert!(
        String::from_utf8_lossy(&result.stdout).contains("standard deviation greater than 1e-12")
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
fn resolution_guard_distinguishes_below_and_above_threshold() {
  let lines: Vec<_> = INPUT.lines().collect();
  let first: BTreeMap<_, _> = lines[1..]
    .iter()
    .filter_map(|line| {
      let c: Vec<_> = line.split(',').collect();
      (c[8] == "1979").then_some((c[7].to_owned(), c[2].parse::<f64>().unwrap()))
    })
    .collect();
  for (step, should_succeed) in [(1e-13, false), (1e-11, true)] {
    let mut changed = vec![lines[0].to_owned()];
    for line in &lines[1..] {
      let mut c: Vec<_> = line.split(',').map(str::to_owned).collect();
      if c[8] == "1980" {
        let g: f64 = c[7].parse().unwrap();
        c[2] = (first[&c[7]] + 0.25 + step * g).to_string();
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
      "step {step}: {}",
      String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(dir.path().join(ROWS).exists(), should_succeed);
    assert_eq!(dir.path().join(MATRIX).exists(), should_succeed);
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      input
    );
  }
}
#[test]
fn export_collisions_preserve_owners_and_partial_first_file() {
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
        144
      );
    }
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      INPUT
    );
  }
}
