use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-count-lincom-cli-{}-{}",
      std::process::id(),
      NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("count_lincom_data.csv"),
      include_str!("../examples/poisson-offset/poisson_offset.csv"),
    )
    .unwrap();
    Self(root)
  }

  fn script(&self, name: &str, source: &str) -> PathBuf {
    let path = self.0.join(name);
    fs::write(&path, source).unwrap();
    path
  }

  fn run(&self, mode: &str, path: &std::path::Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args([mode, "-f", path.to_str().unwrap()])
      .output()
      .unwrap()
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn json_lines(output: &Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  String::from_utf8(output.stdout.clone())
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

fn expected(family: &str, covariance: &str, combination: &str, field: usize) -> f64 {
  include_str!("../crates/tabdat-stats/tests/fixtures/count_lincom_reference.csv")
    .lines()
    .skip(1)
    .find_map(|line| {
      let fields = line.split(',').collect::<Vec<_>>();
      (fields[0] == family && fields[1] == covariance && fields[2] == combination)
        .then(|| fields[field].parse().unwrap())
    })
    .unwrap()
}

fn fit_command(family: &str, covariance: &str) -> String {
  let prefix = if family == "poisson" {
    "poisson"
  } else {
    "nbreg"
  };
  match covariance {
    "model" => format!("{prefix} y x z, offset(log_time)"),
    "robust" => format!("{prefix} y x z, offset(log_time) robust"),
    "cluster" => format!("{prefix} y x z, offset(log_time) cluster(person)"),
    _ => unreachable!(),
  }
}

#[test]
fn count_lincom_reports_normal_reference_in_json_and_text() {
  for family in ["poisson", "nb2"] {
    for covariance in ["model", "robust", "cluster"] {
      let fixture = Fixture::new();
      let script = fixture.script(
        "lincom.td",
        &format!(
          "use count_lincom_data.csv\n{}\nlincom x\nlincom 2 + x - z\nlincom x + z - 0.25\n",
          fit_command(family, covariance)
        ),
      );
      let results = json_lines(&fixture.run("--json", &script));
      let lincoms = results
        .iter()
        .filter(|row| row["result_type"] == "LincomResult")
        .collect::<Vec<_>>();
      assert_eq!(lincoms.len(), 3);
      for (result, combination) in lincoms.into_iter().zip(["slope", "contrast", "sum"]) {
        assert_eq!(result["data"]["df_residual"], Value::Null);
        for (field, reference_field) in [
          ("estimate", 3),
          ("standard_error", 4),
          ("statistic", 5),
          ("p_value", 6),
          ("ci_lower", 7),
          ("ci_upper", 8),
          ("ci_level", 9),
        ] {
          let actual = result["data"][field].as_f64().unwrap();
          assert!(
            (actual - expected(family, covariance, combination, reference_field)).abs() <= 1e-8,
            "{field}: {actual}"
          );
        }
      }

      let terminal = fixture.run("--text", &script);
      assert!(terminal.status.success());
      let text = String::from_utf8(terminal.stdout).unwrap();
      assert!(text.contains("asymptotic standard-normal reference; residual df = NA"));
      assert!(text.contains("Std. error\tz\tP(two-sided)"));
    }
  }
}

#[test]
fn zero_se_count_lincom_uses_null_json_and_na_terminal_inference() {
  let fixture = Fixture::new();
  let script = fixture.script(
    "zero-se.td",
    "use count_lincom_data.csv\npoisson y x z, offset(log_time)\nlincom 2\n",
  );

  let results = json_lines(&fixture.run("--json", &script));
  let result = results
    .iter()
    .find(|row| row["result_type"] == "LincomResult")
    .unwrap();
  let data = &result["data"];
  assert_eq!(data["estimate"], 2.0);
  assert_eq!(data["standard_error"], 0.0);
  assert_eq!(data["statistic"], Value::Null);
  assert_eq!(data["p_value"], Value::Null);
  assert_eq!(data["ci_lower"], 2.0);
  assert_eq!(data["ci_upper"], 2.0);
  assert_eq!(data["df_residual"], Value::Null);

  let terminal = fixture.run("--text", &script);
  assert!(terminal.status.success());
  let text = String::from_utf8(terminal.stdout).unwrap();
  assert!(text.contains("asymptotic standard-normal reference; residual df = NA"));
  assert!(text.contains("2\t0\tNA\tNA\t2\t2"));
}

#[test]
fn unsupported_dispersion_lincom_stops_before_later_script_effects() {
  let fixture = Fixture::new();
  let script = fixture.script(
    "unknown.td",
    "use count_lincom_data.csv\nnbreg y x z\nlincom lnalpha\nexport should-not-exist.csv\n",
  );
  let output = fixture.run("--json", &script);
  assert_eq!(output.status.code(), Some(1));
  assert!(
    String::from_utf8_lossy(&output.stderr)
      .contains("variable 'lnalpha' not found in active model coefficients")
  );
  assert!(!fixture.0.join("should-not-exist.csv").exists());
}
