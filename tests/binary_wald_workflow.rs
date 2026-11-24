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
      "tabdat-binary-wald-cli-{}-{}",
      std::process::id(),
      NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("binary_wald_data.csv"),
      include_str!("../crates/tabdat-stats/tests/fixtures/binary_wald_data.csv"),
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
  include_str!("../crates/tabdat-stats/tests/fixtures/binary_wald_reference.csv")
    .lines()
    .skip(1)
    .find_map(|line| {
      let fields = line.split(',').collect::<Vec<_>>();
      (fields[0] == family && fields[1] == covariance && fields[2] == restriction)
        .then(|| fields[field].parse().unwrap())
    })
    .unwrap()
}

fn script_source(family: &str, covariance: &str) -> String {
  let fit = match covariance {
    "model" => format!("{family} y x1 x2"),
    "robust" => format!("{family} y x1 x2, robust"),
    "cluster" => format!("{family} y x1 x2, cluster(group)"),
    _ => unreachable!(),
  };
  format!(
    "use binary_wald_data.csv\n{fit}\ntest x1 + x2 = 0.2\ntest (x1 + x2 = 0.2) (x1 - x2 = 0.4)\nreplace x1 = x1 + 100\ntest x1 + x2 = 0.2\n"
  )
}

#[test]
fn binary_test_json_and_terminal_use_the_asymptotic_chi_square_reference() {
  for family in ["logit", "probit"] {
    for covariance in ["model", "robust", "cluster"] {
      let fixture = Fixture::new();
      let script_path = fixture.script("wald.td", &script_source(family, covariance));
      let rows = json_lines(&fixture.run("--json", &script_path));
      let tests = rows
        .iter()
        .filter(|row| row["result_type"] == "TestResult")
        .collect::<Vec<_>>();
      assert_eq!(tests.len(), 3);
      for (test, restriction, df) in [
        (tests[0], "one", 1),
        (tests[1], "joint", 2),
        (tests[2], "one", 1),
      ] {
        assert_eq!(test["data"]["df"], df);
        assert_eq!(test["data"]["df_residual"], Value::Null);
        assert_eq!(test["data"]["is_chi2"], true);
        let expected_statistic = expected(family, covariance, restriction, 4);
        let actual_statistic = test["data"]["statistic"].as_f64().unwrap();
        assert!((actual_statistic - expected_statistic).abs() <= 1e-8);
        let expected_p = expected(family, covariance, restriction, 5);
        let actual_p = test["data"]["p_value"].as_f64().unwrap();
        assert!((actual_p - expected_p).abs() <= 1e-8);
      }

      let output = fixture.run("--text", &script_path);
      assert!(output.status.success());
      let text = String::from_utf8(output.stdout).unwrap();
      assert!(text.contains("chi-square(1)"), "{text}");
      assert!(text.contains("chi-square(2)"), "{text}");
    }
  }
}

#[test]
fn singular_binary_test_stops_before_later_script_effects() {
  let fixture = Fixture::new();
  let script_path = fixture.script(
    "singular.td",
    "use binary_wald_data.csv\nlogit y x1 x2, robust\ntest (x1 = 0) (x1 = 1)\nexport should-not-exist.csv\n",
  );
  let output = fixture.run("--json", &script_path);
  assert_eq!(output.status.code(), Some(1));
  assert!(
    String::from_utf8_lossy(&output.stderr).contains("constraints are collinear or singular")
  );
  assert!(!fixture.0.join("should-not-exist.csv").exists());
}
