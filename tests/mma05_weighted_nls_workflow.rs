#![forbid(unsafe_code)]
use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const CONTROL: &str = "source_row,y,x\n1,2.2,-1\n2,1.6,0\n3,2.2,1\n";
struct Fixture(PathBuf);
impl Fixture {
  fn new(input: &str) -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-mma05-weighted-{}-{}",
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
      .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma05p2nls/weighted.td"))
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
fn values(output: Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
#[test]
fn control_fits_both_fixed_weights_and_preserves_text_json_csv() {
  let json = Fixture::new(CONTROL);
  let text = Fixture::new(CONTROL);
  let leaves = values(json.run("--json"));
  let fits: Vec<_> = leaves
    .iter()
    .filter(|v| v["result_type"] == "NlsResult")
    .map(|v| &v["data"])
    .collect();
  assert_eq!(fits.len(), 5);
  let critical = 1.0 / (std::f64::consts::PI * 0.025).tan();
  for (i, fit) in fits.iter().enumerate() {
    assert_eq!(
      fit["sample"]["retained_indices"],
      serde_json::json!([0, 1, 2])
    );
    assert_eq!(fit["sample"]["weights"], Value::Null);
    assert_eq!(fit["start_values"], serde_json::json!([1.0, 0.0]));
    assert_eq!(fit["residual_degrees_of_freedom"], 1);
    close(
      fit["coefficients"][0]["value"].as_f64().unwrap(),
      -2.0_f64.ln(),
    );
    close(fit["coefficients"][1]["value"].as_f64().unwrap(), 0.0);
    if i == 0 {
      continue;
    }
    let robust = i % 2 == 0;
    assert_eq!(fit["robust"], robust);
    close(
      fit["residual_sum_of_squares"].as_f64().unwrap(),
      if i <= 2 { 0.12 } else { 0.06 },
    );
    for (j, variance) in [0.02_f64, if robust { 0.015 } else { 0.03 }]
      .iter()
      .enumerate()
    {
      let c = &fit["coefficients"][j];
      let estimate = if j == 0 { -2.0_f64.ln() } else { 0.0 };
      let se = variance.sqrt();
      close(c["standard_error"].as_f64().unwrap(), se);
      close(c["statistic"].as_f64().unwrap(), estimate / se);
      close(
        c["p_value"].as_f64().unwrap(),
        1.0 - 2.0 / std::f64::consts::PI * (estimate / se).abs().atan(),
      );
      close(
        c["confidence_lower"].as_f64().unwrap(),
        estimate - critical * se,
      );
      close(
        c["confidence_upper"].as_f64().unwrap(),
        estimate + critical * se,
      );
      for k in 0..2 {
        close(
          fit["covariance_matrix"][j][k].as_f64().unwrap(),
          if j == k { *variance } else { 0.0 },
        );
      }
    }
  }
  let human = text.run("--text");
  assert!(
    human.status.success(),
    "{}",
    String::from_utf8_lossy(&human.stderr)
  );
  assert!(human.stderr.is_empty());
  assert!(
    String::from_utf8(human.stdout)
      .unwrap()
      .contains("fixed first-fit weights")
  );
  let csv = fs::read(json.0.join("weighted_predictions.csv")).unwrap();
  assert_eq!(
    csv,
    fs::read(text.0.join("weighted_predictions.csv")).unwrap()
  );
  let content = String::from_utf8(csv).unwrap();
  let mut lines = content.lines();
  let headers: Vec<_> = lines.next().unwrap().split(',').collect();
  let table: Vec<Vec<f64>> = lines
    .map(|line| line.split(',').map(|v| v.parse().unwrap()).collect())
    .collect();
  assert_eq!(table.len(), 3);
  for (i, row) in table.iter().enumerate() {
    let get = |name: &str| row[headers.iter().position(|v| *v == name).unwrap()];
    close(get("source_row"), (i + 1) as f64);
    close(get("preliminary_mean"), 2.0);
    close(get("wnls_weight"), 0.5);
    close(get("fgnls_weight"), 0.25);
    for method in ["wnls", "fgnls"] {
      close(get(&format!("{method}_level_mean")), 2.0);
      close(
        get(&format!("{method}_level_residual")),
        [0.2, -0.4, 0.2][i],
      );
      close(get(&format!("{method}_rate")), 0.5);
    }
  }
}
#[test]
fn invalid_sources_and_output_collision_leave_inputs_and_outputs_intact() {
  let mut too_many = String::from("source_row,y,x\n");
  for i in 1..=10001 {
    too_many.push_str(&format!("{i},1,{}\n", i % 3));
  }
  for input in [
    "source_row,y,x\n1,1,-1\n2,2,1\n",
    "source_row,y,x\n1,1,-1\n1,2,0\n3,1,1\n",
    "source_row,y,x\n2,1,-1\n1,2,0\n3,1,1\n",
    "source_row,y,x\n1,0,-1\n2,2,0\n3,1,1\n",
    "source_row,y,x\n1,1,-1\n2,,0\n3,1,1\n",
    "source_row,y,x\n1,1,1\n2,2,1\n3,3,1\n",
    "source_row,y,x\n1,1,-1\n2,1,0\n3,1,1\n",
    &too_many,
  ] {
    let f = Fixture::new(input);
    let result = f.run("--json");
    assert_eq!(
      result.status.code(),
      Some(1),
      "{}",
      String::from_utf8_lossy(&result.stderr)
    );
    assert!(!f.0.join("weighted_predictions.csv").exists());
    assert_eq!(fs::read_to_string(f.0.join("nls.csv")).unwrap(), input);
  }
  let f = Fixture::new(CONTROL);
  let target = f.0.join("weighted_predictions.csv");
  fs::write(&target, b"existing sentinel\n").unwrap();
  assert_eq!(f.run("--json").status.code(), Some(1));
  assert_eq!(fs::read(target).unwrap(), b"existing sentinel\n");
  assert_eq!(fs::read_dir(&f.0).unwrap().count(), 2);
}
#[test]
fn native_global_weight_rescaling_preserves_coefficients_and_both_covariances() {
  let f = Fixture::new(CONTROL);
  let result = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .args(["--json", "-c", "use nls.csv", "-c", "sql select *, y/2 as z1, 2 as s1, y/6 as z2, 6 as s2 from active order by source_row into scales",
      "-c", "nl z1 = exp(-a-b*x)/s1, params(a b) start(1 0)",
      "-c", "nl z1 = exp(-a-b*x)/s1, params(a b) start(1 0) robust",
      "-c", "nl z2 = exp(-a-b*x)/s2, params(a b) start(1 0)",
      "-c", "nl z2 = exp(-a-b*x)/s2, params(a b) start(1 0) robust"])
    .current_dir(&f.0).output().unwrap();
  let leaves = values(result);
  let fits: Vec<_> = leaves
    .iter()
    .filter(|v| v["result_type"] == "NlsResult")
    .map(|v| &v["data"])
    .collect();
  assert_eq!(fits.len(), 4);
  for mode in 0..2 {
    for i in 0..2 {
      close(
        fits[mode]["coefficients"][i]["value"].as_f64().unwrap(),
        fits[mode + 2]["coefficients"][i]["value"].as_f64().unwrap(),
      );
      for j in 0..2 {
        close(
          fits[mode]["covariance_matrix"][i][j].as_f64().unwrap(),
          fits[mode + 2]["covariance_matrix"][i][j].as_f64().unwrap(),
        );
      }
    }
    close(
      fits[mode]["residual_sum_of_squares"].as_f64().unwrap() / 9.0,
      fits[mode + 2]["residual_sum_of_squares"].as_f64().unwrap(),
    );
  }
}
