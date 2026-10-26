#![forbid(unsafe_code)]
use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
const CONTROL: &str = "source_row,y,x\n1,1.1,-1\n2,0.8,0\n3,1.1,1\n";
struct Fixture(PathBuf);
impl Fixture {
  fn new(input: &str) -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-mma05-nls-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("nls.csv"), input).unwrap();
    Self(path)
  }
  fn run(&self, mode: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .args([mode, "-f"])
      .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma05p2nls/nls.td"))
      .current_dir(&self.0)
      .output()
      .unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= 1e-11 + 1e-10 * expected.abs(),
    "{actual:.17e} != {expected:.17e}"
  );
}
#[test]
fn closed_form_control_runs_real_nls_script_both_covariances_and_identical_text_json_csv() {
  let json = Fixture::new(CONTROL);
  let text = Fixture::new(CONTROL);
  let result = json.run("--json");
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
  assert!(result.stderr.is_empty());
  let leaves: Vec<Value> = String::from_utf8(result.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fits: Vec<_> = leaves
    .iter()
    .filter(|r| r["result_type"] == "NlsResult")
    .map(|r| &r["data"])
    .collect();
  assert_eq!(fits.len(), 2);
  // At a=b=0: mu1, residuals(.1,-.2,.1), J=(-1,-x), J'r0 and observed
  // half-loss Hessian diag(3,1.8)>0. Exact GN covariance follows by 2x2 algebra.
  let critical = 1.0 / (std::f64::consts::PI * 0.025).tan();
  for (fit, variance) in fits.iter().zip([[0.02_f64, 0.03], [0.02, 0.015]]) {
    assert_eq!(fit["observation_count"], 3);
    assert_eq!(fit["residual_degrees_of_freedom"], 1);
    assert_eq!(
      fit["inference"]["reference_distribution"],
      "approximate_student_t"
    );
    close(fit["residual_sum_of_squares"].as_f64().unwrap(), 0.06);
    assert_eq!(
      fit["covariance_correction"],
      if fit["robust"] == true { 3.0 } else { 1.0 }
    );
    for i in 0..2 {
      let c = &fit["coefficients"][i];
      close(c["value"].as_f64().unwrap(), 0.0);
      close(c["standard_error"].as_f64().unwrap(), variance[i].sqrt());
      close(c["statistic"].as_f64().unwrap(), 0.0);
      close(c["p_value"].as_f64().unwrap(), 1.0);
      close(
        c["confidence_lower"].as_f64().unwrap(),
        -critical * variance[i].sqrt(),
      );
      close(
        c["confidence_upper"].as_f64().unwrap(),
        critical * variance[i].sqrt(),
      );
      for j in 0..2 {
        close(
          fit["covariance_matrix"][i][j].as_f64().unwrap(),
          if i == j { variance[i] } else { 0.0 },
        );
      }
    }
  }
  let result = text.run("--text");
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
  assert!(result.stderr.is_empty());
  let human = String::from_utf8(result.stdout).unwrap();
  assert!(human.contains("Approximate Student t(1)"));
  assert!(human.contains("HC1 Gauss-Newton"));
  let csv = fs::read(json.0.join("nls_predictions.csv")).unwrap();
  assert_eq!(csv, fs::read(text.0.join("nls_predictions.csv")).unwrap());
  let csv = String::from_utf8(csv).unwrap();
  let mut lines = csv.lines();
  assert_eq!(
    lines.next().unwrap(),
    "source_row,y,x,mean_classical,residual_classical,mean_hc1,residual_hc1,fitted_rate,squared_residual"
  );
  let rows: Vec<_> = lines
    .map(|l| {
      l.split(',')
        .map(|v| v.parse::<f64>().unwrap())
        .collect::<Vec<_>>()
    })
    .collect();
  assert_eq!(rows.len(), 3);
  for (i, row) in rows.iter().enumerate() {
    close(row[0], (i + 1) as f64);
    close(row[1], [1.1, 0.8, 1.1][i]);
    close(row[2], [-1.0, 0.0, 1.0][i]);
    for j in [3, 5, 7] {
      close(row[j], 1.0);
    }
    for j in [4, 6] {
      close(row[j], [0.1, -0.2, 0.1][i]);
    }
    close(row[8], [0.01, 0.04, 0.01][i]);
  }
}
#[test]
fn nls_source_script_rejects_bad_or_nonregular_controls_before_file_publication() {
  let mut too_many = String::from("source_row,y,x\n");
  for i in 1..=10001 {
    too_many.push_str(&format!("{i},1,{}\n", i % 3));
  }
  for input in [
    "source_row,y,x\n1,1,-1\n2,2,1\n",
    "source_row,y,x\n1,1,-1\n1,2,0\n3,1,1\n",
    "source_row,y,x\n1,0,-1\n2,2,0\n3,1,1\n",
    "source_row,y,x\n1,1,-1\n2,,0\n3,1,1\n",
    "source_row,y,x\n1,1,-1\n2,1,0\n3,1,1\n",
    &too_many,
  ] {
    let f = Fixture::new(input);
    let result = f.run("--json");
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("nls.td:"));
    assert!(!f.0.join("nls_predictions.csv").exists());
    assert_eq!(fs::read_to_string(f.0.join("nls.csv")).unwrap(), input);
  }
}
#[test]
fn nls_export_collision_never_clobbers_existing_file_or_leaves_staging() {
  let f = Fixture::new(CONTROL);
  let target = f.0.join("nls_predictions.csv");
  fs::write(&target, b"existing output sentinel\n").unwrap();
  let result = f.run("--json");
  assert_eq!(result.status.code(), Some(1));
  assert!(String::from_utf8_lossy(&result.stderr).contains("nls.td:"));
  assert_eq!(fs::read(target).unwrap(), b"existing output sentinel\n");
  assert_eq!(fs::read_to_string(f.0.join("nls.csv")).unwrap(), CONTROL);
  assert_eq!(fs::read_dir(&f.0).unwrap().count(), 2);
}
#[test]
fn nls_development_prepare_rejects_unpinned_source_before_creating_any_output() {
  let f = Fixture::new(CONTROL);
  let output = f.0.join("prepared.csv");
  let result = Command::new("python3")
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma05p2nls/prepare.py"))
    .arg(f.0.join("nls.csv"))
    .arg(&output)
    .output()
    .unwrap();
  assert!(!result.status.success());
  assert!(String::from_utf8_lossy(&result.stderr).contains("SHA256"));
  assert!(!output.exists());
  assert_eq!(fs::read_dir(&f.0).unwrap().count(), 1);
}
