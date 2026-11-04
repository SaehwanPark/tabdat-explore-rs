use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../examples/mma21p1panel/gaussian_pa.td");
const FILES: [&str; 4] = ["fit", "coefficients", "predictions", "unit_scores"];
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Dir(PathBuf);
impl Dir {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-gaussian-pa-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    Self(path)
  }
}
impl Drop for Dir {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn fixture(kind: &str, name: &str) -> String {
  fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
    "tests/fixtures/mma21_gaussian_pa/{kind}_{name}.csv"
  )))
  .unwrap()
}
fn output_path(dir: &Path, name: &str) -> PathBuf {
  dir.join(format!("panel_gaussian_pa_{name}.csv"))
}
fn run(dir: &Path, script: &str, json: bool) -> Output {
  let path = dir.join("workflow.td");
  fs::write(&path, script).unwrap();
  let mut cmd = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"));
  cmd.current_dir(dir);
  cmd.arg(if json { "--json" } else { "--text" });
  cmd.arg("-f").arg(path).output().unwrap()
}
type Row = BTreeMap<String, String>;
fn rows(text: &str) -> Vec<Row> {
  let mut lines = text.lines();
  let headers: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let values: Vec<_> = line.split(',').collect();
      assert_eq!(headers.len(), values.len());
      headers
        .iter()
        .zip(values)
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    })
    .collect()
}
fn close(a: &str, b: &str) {
  let a: f64 = a.parse().unwrap();
  let b: f64 = b.parse().unwrap();
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9,
    "{a} != {b}"
  );
}
fn compare(actual: &Row, expected: &Row) {
  assert_eq!(
    actual.keys().collect::<Vec<_>>(),
    expected.keys().collect::<Vec<_>>()
  );
  for (key, value) in expected {
    if matches!(key.as_str(), "covariance" | "parameter") {
      assert_eq!(&actual[key], value);
    } else {
      close(&actual[key], value);
    }
  }
}
fn success(kind: &str, positive: bool) {
  let dir = Dir::new();
  let input = fixture(kind, "inputs");
  fs::write(dir.0.join("MOM.csv"), &input).unwrap();
  let output = run(&dir.0, SCRIPT, true);
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
  let mut original = Vec::new();
  for name in FILES {
    let bytes = fs::read_to_string(output_path(&dir.0, name)).unwrap();
    let actual = rows(&bytes);
    let expected = rows(&fixture(kind, name));
    assert_eq!(actual.len(), expected.len());
    if name == "fit" {
      assert_eq!(actual.len(), 1);
      let mut numeric = actual[0].clone();
      assert_eq!(
        numeric.remove("convention").unwrap(),
        "legacy_pearson_exchangeable_rss_over_n"
      );
      assert_eq!(
        numeric.remove("unit_covariance").unwrap(),
        "uncorrected_unit_score_sandwich"
      );
      assert_eq!(numeric.remove("inference").unwrap(), "normal_95_no_pvalues");
      assert_eq!(numeric.remove("gee_join_key").unwrap(), "1");
      let iteration: usize = numeric.remove("iterations").unwrap().parse().unwrap();
      assert!((1..100).contains(&iteration));
      compare(&numeric, &expected[0]);
      let rho: f64 = numeric["working_correlation"].parse().unwrap();
      assert_eq!(rho > 0.0, positive);
      assert!(rho > -1.0 / 9.0 && rho < 1.0);
    } else {
      for (a, b) in actual.iter().zip(&expected) {
        compare(a, b);
      }
    }
    if name == "predictions" {
      for (i, (a, b)) in actual.iter().zip(rows(&input)).enumerate() {
        assert_eq!(a["source_row"].parse::<usize>().unwrap(), i + 1);
        for (key, value) in b {
          assert_eq!(
            a[&key].parse::<f64>().unwrap(),
            value.parse::<f64>().unwrap()
          );
        }
      }
    }
    if name == "coefficients" {
      let table = events
        .iter()
        .find(|e| e["result_type"] == "TableResult" && e["data"]["headers"][0] == "covariance")
        .unwrap();
      let headers = table["data"]["headers"].as_array().unwrap();
      let values = table["data"]["rows"].as_array().unwrap();
      assert_eq!(headers.len(), 7);
      assert_eq!(values.len(), 4);
      for (row, expected) in values.iter().zip(&expected) {
        for (header, value) in headers.iter().zip(row.as_array().unwrap()) {
          let key = header.as_str().unwrap();
          if matches!(key, "covariance" | "parameter") {
            assert_eq!(value.as_str().unwrap(), expected[key]);
          } else {
            assert!(value.is_number());
            close(&value.to_string(), &expected[key]);
          }
        }
      }
    }
    original.push(bytes);
  }
  let status = events
    .iter()
    .find(|e| e["result_type"] == "StatusResult")
    .unwrap();
  assert_eq!(status["data"]["estimation_family"], "linear");
  assert_eq!(
    status["data"]["active_table_name"],
    "gaussian_pa_predictions"
  );
  assert_eq!(status["data"]["linear_model"]["observation_count"], 320);
  for name in FILES {
    fs::remove_file(output_path(&dir.0, name)).unwrap();
  }
  let state_script = format!("{SCRIPT}\npredict retained_pols, xb\nexport retained_check.csv\n");
  let text = run(&dir.0, &state_script, false);
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let stdout = String::from_utf8(text.stdout).unwrap();
  assert!(stdout.contains("uncorrected_unit_sandwich") && stdout.contains("normal95_lower"));
  for (name, bytes) in FILES.iter().zip(original) {
    assert_eq!(
      fs::read_to_string(output_path(&dir.0, name)).unwrap(),
      bytes
    );
  }
  let retained = rows(&fs::read_to_string(dir.0.join("retained_check.csv")).unwrap());
  assert_eq!(retained.len(), 320);
  let mut distinct = false;
  for row in retained {
    close(&row["retained_pols"], &row["pols_mean"]);
    distinct |= (row["retained_pols"].parse::<f64>().unwrap()
      - row["gee_mean"].parse::<f64>().unwrap())
    .abs()
      > 1e-8;
  }
  assert!(
    distinct,
    "retained pooled OLS must not be silently relabelled as GEE"
  );
}
#[test]
fn positive_gaussian_pa_matches_r_gee_dense_covariance_and_retained_pols() {
  success("positive", true);
}
#[test]
fn negative_gaussian_pa_matches_r_gee_without_random_intercept_clipping() {
  success("negative", false);
}

#[test]
fn gaussian_pa_rejects_bad_panels_design_correlation_and_exhausted_iteration_budget() {
  let input = fixture("positive", "inputs");
  let lines: Vec<_> = input.lines().collect();
  let mut missing: Vec<_> = lines[1].split(',').collect();
  missing[1] = "";
  let cases = [
    (format!("{input}{}\n", lines[1]), false, SCRIPT.to_owned()),
    (
      format!("{}\n", lines[..lines.len() - 1].join("\n")),
      false,
      SCRIPT.to_owned(),
    ),
    (
      format!(
        "{}\n{}\n{}\n",
        lines[0],
        missing.join(","),
        lines[2..].join("\n")
      ),
      false,
      SCRIPT.to_owned(),
    ),
    (format!("{}\n", lines[0]), false, SCRIPT.to_owned()),
    (fixture("rank", "inputs"), true, SCRIPT.to_owned()),
    (fixture("singular", "inputs"), true, SCRIPT.to_owned()),
    (
      input.clone(),
      true,
      SCRIPT.replacen("iteration<100", "iteration<0", 1),
    ),
  ];
  assert!(SCRIPT.contains("iteration<100"));
  for (input, started, script) in cases {
    let dir = Dir::new();
    fs::write(dir.0.join("MOM.csv"), &input).unwrap();
    let output = run(&dir.0, &script, true);
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.contains("RegressionResult"), started, "{stdout}");
    for file in FILES {
      assert!(!output_path(&dir.0, file).exists());
    }
    assert_eq!(fs::read_to_string(dir.0.join("MOM.csv")).unwrap(), input);
  }
}
#[test]
fn gaussian_pa_export_collisions_preserve_existing_files_and_ordered_partial_outputs() {
  for (stage, file) in FILES.iter().enumerate() {
    let dir = Dir::new();
    let input = fixture("positive", "inputs");
    fs::write(dir.0.join("MOM.csv"), &input).unwrap();
    fs::write(output_path(&dir.0, file), "existing owner\n").unwrap();
    let output = run(&dir.0, SCRIPT, true);
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
      fs::read_to_string(output_path(&dir.0, file)).unwrap(),
      "existing owner\n"
    );
    for (i, name) in FILES.iter().enumerate() {
      assert_eq!(output_path(&dir.0, name).exists(), i <= stage);
      if i < stage {
        assert_ne!(
          fs::read_to_string(output_path(&dir.0, name)).unwrap(),
          "existing owner\n"
        );
      }
    }
    assert_eq!(fs::read_to_string(dir.0.join("MOM.csv")).unwrap(), input);
  }
}
