mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

struct Fixture(TestDir);

impl Fixture {
  fn new() -> Self {
    let dir = TestDir::new();
    fs::write(
      dir.path().join("binary_gof_data.csv"),
      include_bytes!("../examples/binary-hosmer-lemeshow/binary_gof_data.csv"),
    )
    .unwrap();
    Self(dir)
  }

  fn run(&self, mode: &str, script: &str) -> Output {
    tabdat_bin()
      .current_dir(self.0.path())
      .args([mode, "-f"])
      .arg(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
          .join("examples/binary-hosmer-lemeshow")
          .join(script),
      )
      .output()
      .unwrap()
  }
}

fn envelopes(output: &Output) -> Vec<Value> {
  assert!(output.status.success(), "{output:?}");
  assert!(output.stderr.is_empty(), "{output:?}");
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

#[test]
fn grouped_binary_gof_runs_as_json_and_text_script_workflow() {
  let fixture = Fixture::new();
  for (script, estimator, statistic, p_value) in [
    ("logit.td", "logit", 2.8183849481943e-7, 0.999999859080762),
    ("probit.td", "probit", 0.00611597629901926, 0.99694668273387),
  ] {
    let output = fixture.run("--json", script);
    let rows = envelopes(&output);
    assert_eq!(rows.len(), 3);
    let event = rows.last().unwrap();
    assert_eq!(event["schema_version"], 1);
    assert_eq!(event["result_type"], "BinaryGoodnessOfFitResult");
    assert_eq!(event["error"], Value::Null);
    assert_eq!(event["data"]["estimator"], estimator);
    assert_eq!(event["data"]["outcome"], "y");
    assert_eq!(event["data"]["observation_count"], 159);
    assert_eq!(event["data"]["requested_group_count"], 10);
    assert_eq!(event["data"]["group_count"], 4);
    assert_eq!(event["data"]["degrees_of_freedom"], 2);
    assert_eq!(event["data"]["groups"].as_array().unwrap().len(), 4);
    assert!((event["data"]["statistic"].as_f64().unwrap() - statistic).abs() < 2e-10);
    assert!((event["data"]["p_value"].as_f64().unwrap() - p_value).abs() < 2e-10);

    let output = fixture.run("--text", script);
    assert!(output.status.success(), "{output:?}");
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Hosmer-Lemeshow grouped goodness-of-fit test"));
    assert!(text.contains("Requested groups = 10; actual groups = 4"));
    assert!(text.contains("sparse expected cells"));
  }
}
