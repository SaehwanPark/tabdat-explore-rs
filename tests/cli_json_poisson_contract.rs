use serde_json::Value;
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
const PREDICTORS: &str = "LC,IDP,LPI,FMDE,PHYSLIM,NDISEASE,HLTHG,HLTHF,HLTHP,LINC,LFAM,EDUCDEC,AGE,FEMALE,CHILD,FEMCHILD,BLACK";
fn sample() -> String {
  let mut csv = format!("prepared_row,MDU,{PREDICTORS}\n");
  for i in 1..=80 {
    csv.push_str(&format!("{},{}", i * 7 % 83 + 1, i * 13 % 17));
    for j in 0..17 {
      csv.push_str(&format!(
        ",{}",
        ((i * (j + 3) * 7 + j * j) % 97) as f64 / 23.0 - 2.0
      ));
    }
    csv.push('\n');
  }
  csv
}
struct Fixture(PathBuf);
impl Fixture {
  fn new(csv: &str) -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-poisson-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("mma20p1count.csv"), csv).unwrap();
    Self(root)
  }
  fn script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma20p1count/poisson.td")
  }
  fn run(&self, flag: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args([flag, "-f"])
      .arg(Self::script())
      .output()
      .unwrap()
  }
  fn batch(&self, commands: &[&str]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"));
    c.current_dir(&self.0).arg("--json");
    for text in commands {
      c.args(["-c", text]);
    }
    c.output().unwrap()
  }
  fn csv(&self) -> Vec<u8> {
    fs::read(self.0.join("poisson_predictions.csv")).unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn envelopes(output: &Output) -> Vec<Value> {
  std::str::from_utf8(&output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn success(output: &Output) {
  assert!(output.status.success(), "{output:?}");
}

#[test]
fn actual_count_script_has_stable_typed_inference_and_preserves_every_source_row() {
  let source = sample();
  let f = Fixture::new(&source);
  let out = f.run("--json");
  success(&out);
  let results = envelopes(&out);
  assert_eq!(results.len(), 12);
  assert!(
    results
      .iter()
      .all(|r| r["schema_version"] == 1 && r["error"].is_null())
  );
  let fits: Vec<_> = results
    .iter()
    .filter(|r| r["result_type"] == "PoissonResult")
    .map(|r| &r["data"])
    .collect();
  assert_eq!(fits.len(), 2);
  let effects: Vec<_> = results
    .iter()
    .filter(|r| r["result_type"] == "PoissonMarginalEffectsResult")
    .map(|r| &r["data"])
    .collect();
  assert_eq!(effects.len(), 2);
  for (i, (fit, effect)) in fits.iter().zip(effects).enumerate() {
    assert_eq!(fit["estimator"], "poisson");
    assert_eq!(fit["observation_count"], 80);
    assert_eq!(fit["df_residual"], 62);
    assert_eq!(
      fit["sample"]["retained_indices"],
      serde_json::json!((0..80).collect::<Vec<_>>())
    );
    assert_eq!(fit["sample"]["total_observations"], 80);
    assert_eq!(fit["sample"]["dropped_observations"], 0);
    assert_eq!(
      fit["covariance"],
      if i == 0 {
        "model_based_poisson_information"
      } else {
        "robust_sandwich_n_over_n_minus_1"
      }
    );
    assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
    assert_eq!(fit["confidence_level"], 95.0);
    assert_eq!(
      effect["reference_distribution"],
      fit["reference_distribution"]
    );
    assert_eq!(effect["covariance"], fit["covariance"]);
    assert_eq!(effect["effects"].as_array().unwrap().len(), 17);
    assert_eq!(effect["sample"], "retained_estimation_design");
    assert_eq!(fit["covariance_parameter_names"][0], "intercept");
    for (j, name) in PREDICTORS.split(',').enumerate() {
      assert_eq!(fit["covariance_parameter_names"][j + 1], name);
    }
    assert_eq!(fit["null_model"], "intercept_only_estimated_mean");
    assert_eq!(
      fit["dispersion_inference"],
      "descriptive_not_a_panel_or_specification_test"
    );
    if i == 0 {
      assert_eq!(
        fit["likelihood_ratio_reference_distribution"],
        "asymptotic_chi_square"
      );
      assert!(fit["omnibus_wald"].is_null());
    } else {
      assert!(fit["likelihood_ratio_p_value"].is_null());
      assert!(fit["likelihood_ratio_reference_distribution"].is_null());
      assert_eq!(fit["omnibus_wald"]["degrees_of_freedom"], 17);
    }
  }
  let csv = String::from_utf8(f.csv()).unwrap();
  assert_eq!(csv.lines().count(), 81);
  for (before, after) in source.lines().skip(1).zip(csv.lines().skip(1)) {
    let before: Vec<f64> = before.split(',').map(|s| s.parse().unwrap()).collect();
    let after: Vec<f64> = after.split(',').map(|s| s.parse().unwrap()).collect();
    assert_eq!(after.len(), 23);
    for (a, b) in before.iter().zip(&after) {
      assert!((a - b).abs() < 1e-12);
    }
    assert_eq!(after[19], after[21]);
    assert_eq!(after[20], after[22]);
    assert!((after[19] - after[20].exp()).abs() < 1e-10);
    assert!(after[19] > 0.0);
  }
  assert_eq!(
    fs::read_to_string(f.0.join("mma20p1count.csv")).unwrap(),
    source
  );
}

#[test]
fn json_text_and_piped_actual_script_publish_identical_ordered_csvs() {
  let source = sample();
  let json = Fixture::new(&source);
  success(&json.run("--json"));
  let text = Fixture::new(&source);
  let out = text.run("--text");
  success(&out);
  let rendered = String::from_utf8(out.stdout).unwrap();
  for phrase in [
    "Poisson log-link regression",
    "Pearson dispersion",
    "NOT repeated-person/panel dependence",
    "N/(N-1)",
    "Poisson continuous average partial effects",
    "P(two-sided)",
  ] {
    assert!(rendered.contains(phrase), "{phrase}");
  }
  let piped = Fixture::new(&source);
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&piped.0)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child
    .stdin
    .take()
    .unwrap()
    .write_all(format!("run \"{}\"\nexit\n", Fixture::script().display()).as_bytes())
    .unwrap();
  let out = child.wait_with_output().unwrap();
  success(&out);
  assert!(String::from_utf8_lossy(&out.stdout).contains("Poisson log-link regression"));
  assert_eq!(json.csv(), text.csv());
  assert_eq!(json.csv(), piped.csv());
}

#[test]
fn actual_script_export_collisions_preserve_existing_targets_and_source() {
  let source = sample();
  for directory in [false, true] {
    let f = Fixture::new(&source);
    let target = f.0.join("poisson_predictions.csv");
    if directory {
      fs::create_dir(&target).unwrap();
      fs::write(target.join("sentinel"), "keep").unwrap();
    } else {
      fs::write(&target, "keep").unwrap();
    }
    let out = f.run("--json");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
      if directory {
        fs::read_to_string(target.join("sentinel"))
      } else {
        fs::read_to_string(target)
      }
      .unwrap(),
      "keep"
    );
    assert_eq!(
      fs::read_to_string(f.0.join("mma20p1count.csv")).unwrap(),
      source
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("poisson.td:15:"));
    let results = envelopes(&out);
    assert_eq!(results.last().unwrap()["error"]["type"], "ScriptError");
    assert!(!results.iter().any(|r| r["result_type"] == "ExportResult"));
    assert_eq!(
      results
        .iter()
        .filter(|r| r["result_type"] == "PoissonResult")
        .count(),
      2
    );
  }
}

#[test]
fn schema_invalid_counts_and_duplicate_positions_fail_before_any_fit_or_export() {
  let source = sample();
  let mutate = |column: usize, value: &str| {
    let mut lines: Vec<String> = source.lines().map(str::to_owned).collect();
    let mut row: Vec<_> = lines[1].split(',').map(str::to_owned).collect();
    row[column] = value.into();
    lines[1] = row.join(",");
    lines.join("\n") + "\n"
  };
  for csv in [
    source.replacen("MDU,", "missing,", 1),
    mutate(1, "-1"),
    mutate(1, "0.5"),
    mutate(0, "15"),
  ] {
    let f = Fixture::new(&csv);
    let out = f.run("--json");
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(
      fs::read_to_string(f.0.join("mma20p1count.csv")).unwrap(),
      csv
    );
    assert!(!f.0.join("poisson_predictions.csv").exists());
    assert!(
      !envelopes(&out)
        .iter()
        .any(|r| r["result_type"] == "PoissonResult")
    );
  }
}

#[test]
fn packaged_count_help_examples_execute_on_their_advertised_columns() {
  let f = Fixture::new(&sample());
  let mut csv = String::from("visits,age,income\n");
  for line in sample().lines().skip(1) {
    let values: Vec<f64> = line.split(',').map(|s| s.parse().unwrap()).collect();
    csv.push_str(&format!(
      "{},{},{}\n",
      values[1],
      values[2] + 40.0,
      values[3] + 5.0
    ));
  }
  fs::write(f.0.join("visits.csv"), csv).unwrap();
  let topic = include_str!("../crates/tabdat-reporting/src/help/topics/poisson.md");
  let examples: Vec<_> = topic
    .lines()
    .filter_map(|line| line.strip_prefix("- `").and_then(|s| s.strip_suffix('`')))
    .collect();
  assert_eq!(examples.len(), 8);
  let out = f.batch(&examples);
  success(&out);
  assert!(f.0.join("visits_predictions.csv").is_file());
  let topic = include_str!("../crates/tabdat-reporting/src/help/topics/predict.md");
  let block = topic
    .split("Count example:")
    .nth(1)
    .unwrap()
    .split("Data and model notes:")
    .next()
    .unwrap();
  let mut examples = vec!["use visits.csv"];
  examples.extend(
    block
      .lines()
      .filter_map(|line| line.strip_prefix("- `").and_then(|s| s.strip_suffix('`'))),
  );
  assert_eq!(examples.len(), 4);
  success(&f.batch(&examples));
}

#[test]
fn no_constant_and_zero_variance_json_never_claim_unavailable_inference() {
  let f = Fixture::new(&sample());
  let out = f.batch(&["use mma20p1count.csv", "poisson MDU LC IDP, noconstant"]);
  success(&out);
  let r = envelopes(&out);
  let fit = &r[1]["data"];
  assert_eq!(fit["null_model"], "unit_mean_all_coefficients_zero");
  assert_eq!(fit["include_intercept"], false);
  assert_eq!(fit["likelihood_ratio_df"], 2);
  assert_eq!(
    fit["covariance_parameter_names"],
    serde_json::json!(["LC", "IDP"])
  );
  let out = f.batch(&[
    "use mma20p1count.csv",
    "replace MDU = 1",
    "poisson MDU, robust",
    "estat margins",
  ]);
  success(&out);
  let r = envelopes(&out);
  let fit = &r[2]["data"];
  let coefficient = &fit["coefficients"][0];
  assert_eq!(coefficient["standard_error"], 0.0);
  assert!(coefficient["statistic"].is_null());
  assert!(coefficient["p_value"].is_null());
  assert!(coefficient["confidence_lower"].is_null());
  assert!(coefficient["confidence_upper"].is_null());
  assert!(fit["omnibus_wald"]["statistic"].is_null());
  assert_eq!(
    fit["omnibus_wald"]["unavailable_reason"],
    "no_non_intercept_coefficients"
  );
  assert_eq!(r[3]["data"]["effects"], serde_json::json!([]));
}
