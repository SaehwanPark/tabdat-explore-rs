mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf, process::Command};

const INPUT: &str = include_str!("fixtures/mma24_household/input.csv");
const FITS: &str = include_str!("fixtures/mma24_household/fits.csv");
const POST: &str = include_str!("fixtures/mma24_household/post.csv");
const PREDICTIONS: &str = include_str!("fixtures/mma24_household/predictions.csv");
const OUTPUT: &str = "household_ols_predictions.csv";

fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma24p1olscluster/household_ols.td"),
    )
    .output()
    .unwrap()
}

fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let header: Vec<_> = lines
    .next()
    .unwrap()
    .split(',')
    .map(|s| s.trim_matches('"'))
    .collect();
  lines
    .map(|line| {
      let values: Vec<_> = line.split(',').map(|s| s.trim_matches('"')).collect();
      assert_eq!(header.len(), values.len());
      header.iter().copied().zip(values).collect()
    })
    .collect()
}

fn close(a: f64, b: f64, p: bool) {
  assert!(a.is_finite() && b.is_finite());
  let tolerance = if p { b.abs() * 1e-8 } else { 1e-9 };
  assert!(
    (a - b).abs() <= tolerance,
    "{a} != {b}, tolerance {tolerance}"
  );
}

#[test]
fn synthetic_fit_predictions_state_and_text_match_independent_r() {
  let dir = TestDir::new();
  fs::write(dir.path().join("vietnam.csv"), INPUT).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let select = |kind: &str| {
    events
      .iter()
      .filter(|e| e["result_type"] == kind)
      .map(|e| &e["data"])
      .collect::<Vec<_>>()
  };
  let fits = select("RegressionResult");
  assert_eq!(fits.len(), 3);
  for (fit, covariance) in fits.iter().zip(["nonrobust", "robust", "cluster(commune)"]) {
    assert_eq!(fit["covariance"], covariance);
    assert_eq!(fit["observation_count"], 35);
    assert_eq!(fit["df_residual"], 28);
    assert_eq!(
      fit["predictors"],
      json!(["lnhhexp", "age", "sex", "hhsize", "farm", "educ"])
    );
  }
  let refs = rows(FITS);
  assert_eq!(refs.len(), 240);
  for row in refs {
    let fit = match row["model"] {
      "nonrobust" => fits[0],
      "robust" => fits[1],
      "cluster" => fits[2],
      _ => panic!(),
    };
    close(
      fit.pointer(row["path"]).unwrap().as_f64().unwrap(),
      row["value"].parse().unwrap(),
      row["path"].ends_with("p_value"),
    );
  }
  let table = &select("EstimateTableResult")[0]["models"];
  assert_eq!(table[0]["inference_degrees_of_freedom"], 28);
  assert_eq!(table[1]["inference_degrees_of_freedom"], 28);
  assert_eq!(table[2]["inference_degrees_of_freedom"], 5);
  assert_eq!(table[2]["cluster_count"], 6);
  let test = select("TestResult")[0];
  let lincom = select("LincomResult")[0];
  assert_eq!(test["df_residual"], 5);
  assert_eq!(lincom["df_residual"], 5);
  let post = rows(POST);
  assert_eq!(post.len(), 8);
  for row in post {
    let object = match row["result"] {
      "test" => test,
      "lincom" => lincom,
      _ => panic!(),
    };
    close(
      object.pointer(row["path"]).unwrap().as_f64().unwrap(),
      row["value"].parse().unwrap(),
      row["path"].ends_with("p_value"),
    );
  }
  close(
    test["statistic"].as_f64().unwrap(),
    lincom["statistic"].as_f64().unwrap().powi(2),
    false,
  );
  let prediction = fs::read_to_string(dir.path().join(OUTPUT)).unwrap();
  let (actual, expected) = (rows(&prediction), rows(PREDICTIONS));
  assert_eq!((actual.len(), expected.len()), (35, 35));
  for (a, b) in actual.iter().zip(expected.iter()) {
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for key in b.keys() {
      let av: f64 = a[key].parse().unwrap();
      let bv: f64 = b[key].parse().unwrap();
      if *key == "household_fitted" || *key == "household_residual" {
        close(av, bv, false)
      } else {
        assert_eq!(av, bv, "{key}");
      }
    }
  }
  assert!(
    actual
      .iter()
      .any(|r| r["source_row"] == "5" && r["lnexp12m"].parse::<f64>().unwrap() == 0.0)
  );
  assert!(!actual.iter().any(|r| r["source_row"] == "4"));
  fs::remove_file(dir.path().join(OUTPUT)).unwrap();
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(
    String::from_utf8(text.stdout)
      .unwrap()
      .contains("not weighted survey design or causal estimates")
  );
  assert_eq!(
    fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
    prediction
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
    INPUT
  );
}

#[test]
fn invalid_keys_rank_and_single_cluster_do_not_export_or_replace_source() {
  let lines: Vec<_> = INPUT.lines().collect();
  let mut variants = Vec::new();
  variants.push(format!("{INPUT}{}\n", lines[1]));
  for (column, value) in [
    (0, "-1"),
    (0, "1.00000000000000001"),
    (8, ""),
    (8, "1.00000000000000001"),
    (1, "NaN"),
    (2, "NaN"),
  ] {
    let mut rows: Vec<Vec<&str>> = lines.iter().map(|s| s.split(',').collect()).collect();
    rows[1][column] = value;
    variants.push(
      rows
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  for column in [6, 8] {
    let mut rows: Vec<Vec<&str>> = lines.iter().map(|s| s.split(',').collect()).collect();
    for row in rows.iter_mut().skip(1) {
      row[column] = "1";
    }
    variants.push(
      rows
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  for input in variants {
    let dir = TestDir::new();
    fs::write(dir.path().join("vietnam.csv"), &input).unwrap();
    let out = run(&dir, "--json");
    assert!(
      !out.status.success(),
      "invalid input unexpectedly succeeded: {input}"
    );
    assert!(!dir.path().join(OUTPUT).exists());
    assert_eq!(
      fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
      input
    );
  }
}

#[test]
fn preparation_rejects_unpinned_input_without_changing_files() {
  let dir = TestDir::new();
  let source = dir.path().join("forged.asc");
  let output = dir.path().join("vietnam.csv");
  fs::write(&source, "fake source").unwrap();
  fs::write(&output, "existing owner").unwrap();
  let run = Command::new("python3")
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma24p1olscluster/prepare.py"))
    .arg(&source)
    .arg(&output)
    .output()
    .unwrap();
  assert!(!run.status.success());
  assert_eq!(fs::read_to_string(&source).unwrap(), "fake source");
  assert_eq!(fs::read_to_string(&output).unwrap(), "existing owner");
}

#[test]
fn export_collision_preserves_owner_and_source() {
  let dir = TestDir::new();
  fs::write(dir.path().join("vietnam.csv"), INPUT).unwrap();
  fs::write(dir.path().join(OUTPUT), "previous owner").unwrap();
  let out = run(&dir, "--json");
  assert!(!out.status.success());
  assert_eq!(
    fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
    "previous owner"
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
    INPUT
  );
}
