mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
};
use tabdat_language::{parse_command, script::parse_script};
use tabdat_runtime::Session;

const SCRIPT: &str = include_str!("../examples/mma24p1olscluster/household_bp_lm.td");
const DIAGNOSTIC: &str = "household_bp_diagnostic.csv";
const PREDICTIONS: &str = "household_bp_predictions.csv";
fn fixture(case: &str, kind: &str) -> String {
  fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
    "tests/fixtures/mma24_household_bp_lm/{case}_{kind}.csv"
  )))
  .unwrap()
}
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma24p1olscluster/household_bp_lm.td"),
    )
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let names: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let values: Vec<_> = line.split(',').collect();
      assert_eq!(values.len(), names.len());
      names.iter().copied().zip(values).collect()
    })
    .collect()
}
fn close(a: f64, b: f64, p: bool) {
  let tolerance = if p { b.abs() * 1e-8 } else { 1e-9 };
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= tolerance,
    "{a} != {b}, tolerance {tolerance}"
  );
}
fn check(case: &str, negative: bool, balanced: bool) {
  let dir = TestDir::new();
  let input = fixture(case, "input");
  fs::write(dir.path().join("vietnam.csv"), &input).unwrap();
  let out = run(&dir, "--json");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let events: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  assert_eq!(events.len(), 14);
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let fit = &events[5]["data"];
  assert_eq!(events[5]["result_type"], "RegressionResult");
  assert_eq!(fit["estimator"], "ols");
  assert_eq!(fit["covariance"], "nonrobust");
  let fitref = fixture(case, "fit");
  let fitrows = rows(&fitref);
  assert_eq!(fitrows.len(), 56);
  for r in fitrows {
    close(
      fit.pointer(r["path"]).unwrap().as_f64().unwrap(),
      r["value"].parse().unwrap(),
      false,
    );
  }
  let diag = &events[9]["data"];
  assert_eq!(events[9]["result_type"], "TableResult");
  let headers = diag["headers"].as_array().unwrap();
  let values = diag["rows"][0].as_array().unwrap();
  assert_eq!(diag["rows"].as_array().unwrap().len(), 1);
  let numeric = |name: &str| {
    values[headers.iter().position(|h| h == name).unwrap()]
      .as_f64()
      .unwrap()
  };
  let diagref = fixture(case, "diagnostic");
  let expected = rows(&diagref);
  assert_eq!(expected.len(), 1);
  assert_eq!(expected[0].len(), 13);
  for (key, value) in &expected[0] {
    close(numeric(key), value.parse().unwrap(), *key == "p_value");
  }
  assert_eq!(numeric("signed_score") < 0.0, negative);
  assert_eq!(
    numeric("minimum_group_size") == numeric("maximum_group_size"),
    balanced
  );
  if balanced {
    let t = numeric("minimum_group_size");
    let a = numeric("group_residual_sum_squares") / numeric("residual_sum_of_squares") - 1.0;
    close(
      numeric("lm_statistic"),
      numeric("observations") / (2.0 * (t - 1.0)) * a * a,
      false,
    );
  } else {
    assert_eq!(numeric("singleton_groups"), 1.0);
  }
  assert_eq!(values[14], "legacy_squared_score_chi_square_1");
  let predictions = fs::read_to_string(dir.path().join(PREDICTIONS)).unwrap();
  let refpred = fixture(case, "predictions");
  let (actual, expected) = (rows(&predictions), rows(&refpred));
  assert_eq!(actual.len(), expected.len());
  assert_eq!(
    fit["observation_count"].as_u64().unwrap() as usize,
    actual.len()
  );
  for (a, b) in actual.iter().zip(expected) {
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for key in b.keys() {
      let av: f64 = a[key].parse().unwrap();
      let bv: f64 = b[key].parse().unwrap();
      if ["bp_xb", "bp_residual"].contains(key) {
        close(av, bv, false)
      } else {
        assert_eq!(av, bv, "{key}")
      }
    }
  }
  let diagbytes = fs::read(dir.path().join(DIAGNOSTIC)).unwrap();
  fs::remove_file(dir.path().join(DIAGNOSTIC)).unwrap();
  fs::remove_file(dir.path().join(PREDICTIONS)).unwrap();
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(String::from_utf8_lossy(&text.stdout).contains("legacy_squared_score_chi_square_1"));
  assert_eq!(fs::read(dir.path().join(DIAGNOSTIC)).unwrap(), diagbytes);
  assert_eq!(
    fs::read_to_string(dir.path().join(PREDICTIONS)).unwrap(),
    predictions
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
    input
  );
}
#[test]
fn balanced_lm_reduces_to_balanced_formula_and_matches_r_plm() {
  check("balanced", false, true);
}
#[test]
fn unbalanced_singleton_and_missing_sample_matches_r_plm() {
  check("unbalanced", false, false);
}
#[test]
fn negative_score_is_preserved_not_clipped_to_boundary() {
  check("negative", true, true);
}

#[test]
fn reporting_and_both_export_collisions_preserve_pooled_fit_and_source() {
  for collision in [None, Some(DIAGNOSTIC), Some(PREDICTIONS)] {
    let dir = TestDir::new();
    let input = fixture("unbalanced", "input");
    let source = dir.path().join("vietnam.csv");
    fs::write(&source, &input).unwrap();
    if let Some(name) = collision {
      fs::write(dir.path().join(name), "owner").unwrap();
    }
    let mut session = Session::new();
    let mut retained = None;
    let mut failures = 0;
    for command in parse_script(SCRIPT, Path::new("household_bp_lm.td")).unwrap() {
      let text = if command.text == "use vietnam.csv" {
        format!("use \"{}\"", source.display())
      } else if let Some(name) = command.text.strip_prefix("export ") {
        format!("export \"{}\"", dir.path().join(name).display())
      } else {
        command.text.clone()
      };
      let before = session.state();
      let result = session.execute(parse_command(&text).unwrap());
      if collision.is_some_and(|name| command.text == format!("export {name}")) {
        assert!(result.is_err());
        assert_eq!(session.state(), before);
        failures += 1;
      } else {
        result.unwrap();
      }
      if command.text.starts_with("regress ") {
        retained = session.state().estimation;
        assert!(retained.is_some());
      } else if retained.is_some() {
        assert_eq!(session.state().estimation, retained);
      }
    }
    assert_eq!(failures, usize::from(collision.is_some()));
    session
      .execute(parse_command("predict bp_after, xb").unwrap())
      .unwrap();
    assert_eq!(session.state().estimation, retained);
    session.execute(parse_command("sql \"\"\"SELECT CASE WHEN COUNT(*)!=150 OR MAX(ABS(bp_after-bp_xb))>1e-12 OR MAX(ABS(lnexp12m-bp_xb-bp_residual))>1e-12 THEN error('wrong retained pooled fit') ELSE 1 END AS valid FROM active\"\"\"").unwrap()).unwrap();
    if let Some(name) = collision {
      assert_eq!(fs::read_to_string(dir.path().join(name)).unwrap(), "owner");
    }
    assert_eq!(fs::read_to_string(source).unwrap(), input);
  }
}

#[test]
fn invalid_inputs_and_degenerate_designs_preserve_files() {
  let input = fixture("unbalanced", "input");
  let base: Vec<Vec<String>> = input
    .lines()
    .map(|l| l.split(',').map(str::to_owned).collect())
    .collect();
  let mut cases = vec![format!("{input}{}\n", input.lines().nth(1).unwrap())];
  for (row, col, value) in [
    (1, 0, "-1"),
    (1, 0, "1.00000000000000001"),
    (1, 8, ""),
    (1, 8, "1.00000000000000001"),
    (1, 1, "NaN"),
    (1, 2, "NaN"),
    (4, 6, ""),
  ] {
    let mut data = base.clone();
    data[row][col] = value.into();
    cases.push(
      data
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  for kind in [
    "one_group",
    "singletons",
    "singular",
    "constant_y",
    "nearly_perfect",
  ] {
    let mut data = base.clone();
    for (i, row) in data.iter_mut().enumerate().skip(1) {
      match kind {
        "one_group" => row[8] = "1".into(),
        "singletons" => row[8] = i.to_string(),
        "singular" => row[6] = "1".into(),
        "constant_y" => row[1] = "1".into(),
        _ => row[1] = (2.0 + row[7].parse::<f64>().unwrap() + 1e-14 * (i as f64).sin()).to_string(),
      }
    }
    cases.push(
      data
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  for (i, input) in cases.iter().enumerate() {
    let dir = TestDir::new();
    fs::write(dir.path().join("vietnam.csv"), input).unwrap();
    for name in [DIAGNOSTIC, PREDICTIONS] {
      fs::write(dir.path().join(name), "owner").unwrap();
    }
    let out = run(&dir, "--json");
    assert!(!out.status.success(), "invalid case {i} succeeded");
    assert_eq!(out.status.code(), Some(1));
    // Owners must not hide an accepted bad fit behind a later export collision.
    assert!(!String::from_utf8_lossy(&out.stdout).contains("lm_statistic"));
    for name in [DIAGNOSTIC, PREDICTIONS] {
      assert_eq!(fs::read_to_string(dir.path().join(name)).unwrap(), "owner");
    }
    assert_eq!(
      fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
      *input
    );
  }
}
