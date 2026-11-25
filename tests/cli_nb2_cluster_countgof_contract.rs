mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, process::Output};

const DATA: &str =
  include_str!("../crates/tabdat-stats/tests/fixtures/nb2_cluster_count_gof_data.csv");

fn fixture() -> TestDir {
  let dir = TestDir::new();
  fs::write(dir.path().join("clustered.csv"), DATA).unwrap();
  dir
}

fn run(dir: &TestDir, output: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([
      output,
      "-c",
      "use clustered.csv",
      "-c",
      "nbreg y x, cluster(cluster)",
      "-c",
      "estat countgof, max(4)",
    ])
    .output()
    .unwrap()
}

fn records(output: &Output) -> Vec<Value> {
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

#[test]
fn json_and_terminal_report_cluster_adjusted_nb2_grouped_gof_contract() {
  let dir = fixture();
  let output = run(&dir, "--json");
  assert!(output.status.success(), "{output:?}");
  assert!(output.stderr.is_empty(), "{output:?}");
  let envelopes = records(&output);
  assert_eq!(envelopes.len(), 3);
  let report = &envelopes[2];
  assert_eq!(report["result_type"], "CountGoodnessOfFitResult");
  let data = &report["data"];
  assert_eq!(
    data["method"],
    "cluster_adjusted_score_influence_chi_square"
  );
  assert_eq!(
    data["null_hypothesis"],
    "conditional_NB2_marginal_distribution_is_correctly_specified"
  );
  assert_eq!(data["reference_distribution"]["family"], "chi_square");
  assert_eq!(data["degrees_of_freedom"], 4);
  assert_eq!(data["cluster"]["variable"], "cluster");
  assert_eq!(data["cluster"]["group_count"], 96);
  assert_eq!(data["cluster"]["covariance_correction"], 96.0 / 95.0);
  assert_eq!(data["moment_covariance"].as_array().unwrap().len(), 4);
  assert_eq!(data["table"]["bins"].as_array().unwrap().len(), 5);
  assert_eq!(
    data["assumptions"],
    serde_json::json!([
      "independent_clusters",
      "correctly_specified_conditional_NB2_marginal_distribution",
      "many_independent_clusters_no_few_cluster_guarantee"
    ])
  );

  let text = run(&dir, "--text");
  assert!(text.status.success(), "{text:?}");
  assert!(text.stderr.is_empty(), "{text:?}");
  let text = String::from_utf8_lossy(&text.stdout);
  for phrase in [
    "Cluster-adjusted score-influence CGF",
    "Cluster variable cluster; G = 96; covariance multiplier =",
    "independent clusters",
    "there is no few-cluster guarantee",
  ] {
    assert!(text.contains(phrase), "missing {phrase:?}: {text}");
  }
}
