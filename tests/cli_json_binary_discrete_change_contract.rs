use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-discrete-change-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("binary_discrete_change.csv"),
      include_str!("../examples/binary-discrete-change/binary_discrete_change.csv"),
    )
    .unwrap();
    Self(root)
  }

  fn run(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn close(value: &Value, expected: f64) {
  assert!((value.as_f64().unwrap() - expected).abs() < 2e-10);
}

#[test]
fn binary_discrete_change_scripts_run_on_json_and_text_cli_surfaces() {
  let fixture = Fixture::new();
  let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  for (script, model, covariance, expected, standard_error) in [
    (
      "logit_clustered.td",
      "logit",
      "cluster_sandwich_g_over_g_minus_1",
      0.183329669025602,
      0.0882884728319724,
    ),
    (
      "probit_robust.td",
      "probit",
      "robust_sandwich_n_over_n_minus_1",
      0.182161202616056,
      0.0874502399969682,
    ),
  ] {
    let script_path = root.join("examples/binary-discrete-change").join(script);
    let output = fixture.run(&["--json", "-f", script_path.to_str().unwrap()]);
    assert!(
      output.status.success(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
    );
    let rows = String::from_utf8(output.stdout)
      .unwrap()
      .lines()
      .map(|line| serde_json::from_str::<Value>(line).unwrap())
      .collect::<Vec<_>>();
    assert_eq!(
      rows.last().unwrap()["result_type"],
      "BinaryDiscreteChangeResult"
    );
    let data = &rows.last().unwrap()["data"];
    assert_eq!(data["estimator"], model);
    assert_eq!(data["covariance"], covariance);
    assert_eq!(data["observation_count"], 120);
    close(&data["effect_estimate"]["value"], expected);
    close(&data["effect_estimate"]["standard_error"], standard_error);

    let output = fixture.run(&["--text", "-f", script_path.to_str().unwrap()]);
    assert!(
      output.status.success(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("discrete average probability change"));
    assert!(text.contains("This is not a causal effect"));
  }
}
