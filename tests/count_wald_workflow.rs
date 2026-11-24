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
      "tabdat-count-wald-cli-{}-{}",
      std::process::id(),
      NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("count_wald_data.csv"),
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

fn expected(family: &str, covariance: &str, restriction: &str, field: usize) -> f64 {
  include_str!("../crates/tabdat-stats/tests/fixtures/count_wald_reference.csv")
    .lines()
    .skip(1)
    .find_map(|line| {
      let fields = line.split(',').collect::<Vec<_>>();
      (fields[0] == family && fields[1] == covariance && fields[2] == restriction)
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
fn count_tests_report_chi_square_in_json_and_text() {
  for family in ["poisson", "nb2"] {
    for covariance in ["model", "robust", "cluster"] {
      let fixture = Fixture::new();
      let script = fixture.script(
        "wald.td",
        &format!(
          "use count_wald_data.csv\n{}\ntest x = 0\ntest x = z + 0.25\ntest x z\n",
          fit_command(family, covariance)
        ),
      );
      let rows = json_lines(&fixture.run("--json", &script));
      let tests = rows
        .iter()
        .filter(|row| row["result_type"] == "TestResult")
        .collect::<Vec<_>>();
      assert_eq!(tests.len(), 3);
      for (test, restriction, df) in [
        (tests[0], "one", 1),
        (tests[1], "contrast", 1),
        (tests[2], "joint", 2),
      ] {
        assert_eq!(test["data"]["df"], df);
        assert_eq!(test["data"]["df_residual"], Value::Null);
        assert_eq!(test["data"]["is_chi2"], true);
        let statistic = test["data"]["statistic"].as_f64().unwrap();
        assert!((statistic - expected(family, covariance, restriction, 4)).abs() <= 1e-8);
        let p_value = test["data"]["p_value"].as_f64().unwrap();
        assert!((p_value - expected(family, covariance, restriction, 5)).abs() <= 1e-8);
      }

      let terminal = fixture.run("--text", &script);
      assert!(terminal.status.success());
      let text = String::from_utf8(terminal.stdout).unwrap();
      assert!(text.contains("chi-square(1)"), "{text}");
      assert!(text.contains("chi-square(2)"), "{text}");
    }
  }
}

#[test]
fn singular_count_test_stops_before_later_script_effects() {
  let fixture = Fixture::new();
  let script = fixture.script(
    "singular.td",
    "use count_wald_data.csv\npoisson y x z, robust\ntest (x = 0) (x = 1)\nexport should-not-exist.csv\n",
  );
  let output = fixture.run("--json", &script);
  assert_eq!(output.status.code(), Some(1));
  assert!(
    String::from_utf8_lossy(&output.stderr).contains("constraints are collinear or singular")
  );
  assert!(!fixture.0.join("should-not-exist.csv").exists());
}
