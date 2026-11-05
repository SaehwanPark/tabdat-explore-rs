mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

const INTERIOR: &str = include_str!("fixtures/mma21_cross_section_1979/interior_inputs.csv");
const ORTHOGONAL: &str = include_str!("fixtures/mma21_cross_section_1979/orthogonal_inputs.csv");
const EXPORT: &str = "panel_1979_cross_section_predictions.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma21p1panel/cross_section_1979.td"),
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
fn check(input: &str, reference: &str, predictions: &str, orthogonal: bool) {
  let dir = TestDir::new();
  fs::write(dir.path().join("MOM.csv"), input).unwrap();
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
  let tests = select("TestResult");
  let lincoms = select("LincomResult");
  assert_eq!((fits.len(), tests.len(), lincoms.len()), (1, 1, 1));
  let fit = fits[0];
  let test = tests[0];
  let lincom = lincoms[0];
  assert_eq!(fit["outcome"], "lnhr");
  assert_eq!(fit["predictors"], json!(["lnwg"]));
  assert_eq!(fit["covariance"], "nonrobust");
  assert_eq!(fit["include_intercept"], true);
  assert_eq!(
    fit["sample"],
    json!({"total_observations":16,"dropped_observations":0})
  );
  assert_eq!(test["constraints"], json!(["lnwg = 0"]));
  assert_eq!(test["is_chi2"], false);
  assert_eq!(test["df"], 1);
  assert_eq!(test["df_residual"], 14);
  let reference = rows(reference);
  assert_eq!(reference.len(), 29);
  for row in reference {
    let object = match row["result_type"] {
      "RegressionResult" => fit,
      "LincomResult" => lincom,
      "TestResult" => test,
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
  assert!(
    (test["statistic"].as_f64().unwrap() - lincom["statistic"].as_f64().unwrap().powi(2)).abs()
      <= 1e-9
  );
  if orthogonal {
    assert!(lincom["estimate"].as_f64().unwrap().abs() <= 1e-12);
    assert!(test["statistic"].as_f64().unwrap() <= 1e-24);
    assert_eq!(test["p_value"], 1.0);
  } else {
    assert!(lincom["estimate"].as_f64().unwrap().abs() > 0.1);
  }
  assert_eq!(
    es[es.len() - 4..]
      .iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "SqlCreateResult",
      "TestResult",
      "LincomResult",
      "ExportResult"
    ]
  );
  assert!(
    serde_json::to_string(&es)
      .unwrap()
      .contains("not panel FE or a causal claim")
  );
  let exports = select("ExportResult");
  assert_eq!(exports.len(), 1);
  assert_eq!(exports[0]["path"], EXPORT);
  assert_eq!(exports[0]["dataset"]["path"], EXPORT);
  assert_eq!(exports[0]["dataset"]["row_count"], 16);
  let prediction = fs::read_to_string(dir.path().join(EXPORT)).unwrap();
  let actual = rows(&prediction);
  let expected = rows(predictions);
  assert_eq!((actual.len(), expected.len()), (16, 16));
  for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
    assert_eq!(a.len(), 11);
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    let val = |key: &str| a[key].parse::<f64>().unwrap();
    assert_eq!(val("source_row"), (i + 1) as f64);
    assert_eq!(val("id"), (i + 1) as f64);
    assert_eq!(val("year"), 1979.0);
    assert!((val("lnhr") - val("cross_fitted") - val("cross_residual")).abs() <= 1e-9);
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
      ]
      .contains(&key)
      {
        assert_eq!(x, y, "{key}");
      } else {
        assert!((x - y).abs() <= 1e-9, "{key}: {x} != {y}");
      }
    }
  }
  fs::remove_file(dir.path().join(EXPORT)).unwrap();
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let stdout = String::from_utf8(text.stdout).unwrap();
  assert!(stdout.contains("not panel FE or a causal claim") && stdout.contains("CI lower (95%)"));
  assert_eq!(
    fs::read_to_string(dir.path().join(EXPORT)).unwrap(),
    prediction
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    input
  );
}
#[test]
fn interior_1979_cross_section_matches_independent_qr() {
  check(
    INTERIOR,
    include_str!("fixtures/mma21_cross_section_1979/interior_reference.csv"),
    include_str!("fixtures/mma21_cross_section_1979/interior_rows.csv"),
    false,
  );
}
#[test]
fn orthogonal_1979_cross_section_retains_zero_slope() {
  check(
    ORTHOGONAL,
    include_str!("fixtures/mma21_cross_section_1979/orthogonal_reference.csv"),
    include_str!("fixtures/mma21_cross_section_1979/orthogonal_rows.csv"),
    true,
  );
}
#[test]
fn invalid_sample_keys_and_rank_fail_without_export() {
  let lines: Vec<_> = INTERIOR.lines().collect();
  let mut variants = vec![
    format!("{INTERIOR}{}\n", lines[1]),
    format!("{}\n", lines[..lines.len() - 1].join("\n")),
  ];
  for (line_index, index, value) in [
    (1, 0, "1.5"),
    (1, 1, ""),
    (17, 2, "NaN"),
    (1, 7, "1.5"),
    (1, 8, "1978"),
  ] {
    let mut changed = lines.clone();
    let mut c: Vec<_> = changed[line_index].split(',').collect();
    c[index] = value;
    let line = c.join(",");
    changed[line_index] = &line;
    variants.push(format!("{}\n", changed.join("\n")));
  }
  let mut changed = vec![lines[0].to_owned()];
  for line in &lines[1..] {
    let mut c: Vec<_> = line.split(',').collect();
    if c[8] == "1979" {
      c[2] = "2";
    }
    changed.push(c.join(","));
  }
  variants.push(format!("{}\n", changed.join("\n")));
  for (i, input) in variants.iter().enumerate() {
    let dir = TestDir::new();
    fs::write(dir.path().join("MOM.csv"), input).unwrap();
    let result = run(&dir, "--json");
    assert!(!result.status.success(), "variant {i}");
    assert!(!result.stderr.is_empty());
    assert!(!dir.path().join(EXPORT).exists());
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      *input
    );
  }
}
#[test]
fn export_collision_preserves_owner_and_input() {
  let dir = TestDir::new();
  fs::write(dir.path().join("MOM.csv"), INTERIOR).unwrap();
  fs::write(dir.path().join(EXPORT), "existing owner\n").unwrap();
  let result = run(&dir, "--json");
  assert!(!result.status.success());
  assert_eq!(
    fs::read_to_string(dir.path().join(EXPORT)).unwrap(),
    "existing owner\n"
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    INTERIOR
  );
}
