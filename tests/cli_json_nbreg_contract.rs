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
      "tabdat-nb2-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("mma20p1count.csv"), csv).unwrap();
    Self(root)
  }
  fn script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma20p1count/nb2.td")
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
    for command in commands {
      c.args(["-c", command]);
    }
    c.output().unwrap()
  }
  fn csv(&self) -> Vec<u8> {
    fs::read(self.0.join("nb2_predictions.csv")).unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn envelopes(out: &Output) -> Vec<Value> {
  String::from_utf8_lossy(&out.stdout)
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}
fn success(out: &Output) {
  assert!(out.status.success(), "{out:?}");
  assert!(out.stderr.is_empty(), "{out:?}");
}
#[test]
fn actual_nb2_script_reports_joint_dispersion_and_inference_and_preserves_all_source_fields() {
  let source = sample();
  let f = Fixture::new(&source);
  let out = f.run("--json");
  success(&out);
  let r = envelopes(&out);
  assert_eq!(r.len(), 12);
  assert!(
    r.iter()
      .all(|e| e["schema_version"] == 1 && e["error"].is_null())
  );
  assert_eq!(
    r.iter()
      .filter_map(|e| e["result_type"].as_str())
      .collect::<Vec<_>>(),
    vec![
      "LoadResult",
      "IsidResult",
      "SummarizeResult",
      "NbregResult",
      "NbregMarginalEffectsResult",
      "PredictionResult",
      "PredictionResult",
      "NbregResult",
      "NbregMarginalEffectsResult",
      "PredictionResult",
      "PredictionResult",
      "ExportResult"
    ]
  );
  for (mode, (fit, effects)) in [
    (&r[3]["data"], &r[4]["data"]),
    (&r[7]["data"], &r[8]["data"]),
  ]
  .iter()
  .enumerate()
  {
    assert_eq!(fit["estimator"], "nb2");
    assert_eq!(effects["estimator"], "nb2");
    assert_eq!(fit["observation_count"], 80);
    assert_eq!(fit["mean_parameter_count"], 18);
    assert_eq!(fit["parameter_count"], 19);
    assert_eq!(fit["df_residual"], 61);
    assert_eq!(
      fit["sample"]["retained_indices"],
      serde_json::json!((0..80).collect::<Vec<_>>())
    );
    assert_eq!(fit["covariance_parameter_names"][0], "intercept");
    assert_eq!(fit["covariance_parameter_names"][18], "lnalpha");
    assert_eq!(fit["covariance_matrix"].as_array().unwrap().len(), 19);
    assert_eq!(
      fit["covariance"],
      if mode == 0 {
        "model_based_joint_observed_information"
      } else {
        "robust_sandwich_n_over_n_minus_1"
      }
    );
    assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
    assert_eq!(fit["confidence_level"], 95.0);
    assert_eq!(fit["variance_function"], "mu_plus_alpha_mu_squared");
    assert_eq!(
      fit["null_model"],
      "intercept_only_mean_and_dispersion_reestimated"
    );
    assert_eq!(effects["covariance"], fit["covariance"]);
    assert_eq!(
      effects["covariance_role"],
      "beta_block_of_full_joint_beta_lnalpha_covariance"
    );
    assert_eq!(effects["effects"].as_array().unwrap().len(), 17);
    let d = &fit["dispersion"];
    assert!(d.get("p_value").is_none() && d.get("statistic").is_none());
    for bound in ["lower", "upper"] {
      assert!(
        (d[format!("alpha_confidence_{bound}")].as_f64().unwrap()
          - d[format!("log_alpha_confidence_{bound}")]
            .as_f64()
            .unwrap()
            .exp())
        .abs()
          < 1e-10
      );
    }
    if mode == 0 {
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
    let before: Vec<f64> = before.split(',').map(|v| v.parse().unwrap()).collect();
    let after: Vec<f64> = after.split(',').map(|v| v.parse().unwrap()).collect();
    assert_eq!(after.len(), 23);
    for (a, b) in before.iter().zip(&after) {
      assert!((a - b).abs() < 1e-12);
    }
    assert_eq!(after[19], after[21]);
    assert_eq!(after[20], after[22]);
    assert!((after[19] - after[20].exp()).abs() < 1e-10);
  }
  assert_eq!(
    fs::read_to_string(f.0.join("mma20p1count.csv")).unwrap(),
    source
  );
}
#[test]
fn actual_script_json_text_and_piped_outputs_publish_identical_csvs() {
  let source = sample();
  let json = Fixture::new(&source);
  success(&json.run("--json"));
  let text = Fixture::new(&source);
  let out = text.run("--text");
  success(&out);
  let output = String::from_utf8_lossy(&out.stdout);
  assert!(!output.contains("(not F).McFadden"));
  for phrase in [
    "Negative binomial NB2 log-link",
    "JOINT observed information",
    "N/(N-1)",
    "lnalpha",
    "NOT repeated-person/panel dependence",
    "No normal alpha=0 test",
    "NB2 continuous average partial effects",
  ] {
    assert!(output.contains(phrase), "{phrase}");
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
  success(&child.wait_with_output().unwrap());
  assert_eq!(json.csv(), text.csv());
  assert_eq!(json.csv(), piped.csv());
}
#[test]
fn existing_export_files_and_directories_are_preserved_at_the_originating_script_line() {
  for directory in [false, true] {
    let source = sample();
    let f = Fixture::new(&source);
    let target = f.0.join("nb2_predictions.csv");
    if directory {
      fs::create_dir(&target).unwrap();
      fs::write(target.join("sentinel"), "keep").unwrap();
    } else {
      fs::write(&target, "keep").unwrap();
    }
    let out = f.run("--json");
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("nb2.td:15:"));
    assert_eq!(
      fs::read_to_string(if directory {
        target.join("sentinel")
      } else {
        target
      })
      .unwrap(),
      "keep"
    );
    let results = envelopes(&out);
    assert_eq!(results.last().unwrap()["error"]["type"], "ScriptError");
    assert_eq!(
      results
        .iter()
        .filter(|r| r["result_type"] == "NbregResult")
        .count(),
      2
    );
    assert_eq!(
      fs::read_to_string(f.0.join("mma20p1count.csv")).unwrap(),
      source
    );
  }
}
#[test]
fn schema_bad_counts_and_duplicate_positions_fail_before_any_fit_or_export() {
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
    mutate(1, "100001"),
    mutate(0, "15"),
  ] {
    let f = Fixture::new(&csv);
    let out = f.run("--json");
    assert_eq!(out.status.code(), Some(1));
    assert!(!f.0.join("nb2_predictions.csv").exists());
    assert!(
      !envelopes(&out)
        .iter()
        .any(|r| r["result_type"] == "NbregResult")
    );
    assert_eq!(
      fs::read_to_string(f.0.join("mma20p1count.csv")).unwrap(),
      csv
    );
  }
}
#[test]
fn packaged_nbreg_examples_execute_and_discovery_exposes_bounds_and_joint_covariance() {
  let f = Fixture::new(&sample());
  let mut csv = String::from("visits,age,income\n");
  for line in sample().lines().skip(1) {
    let v: Vec<f64> = line.split(',').map(|s| s.parse().unwrap()).collect();
    csv.push_str(&format!("{},{},{}\n", v[1], v[2] + 40.0, v[3] + 5.0));
  }
  fs::write(f.0.join("visits.csv"), csv).unwrap();
  let topic = include_str!("../crates/tabdat-reporting/src/help/topics/nbreg.md");
  let examples: Vec<_> = topic
    .lines()
    .filter_map(|line| line.strip_prefix("- `").and_then(|s| s.strip_suffix('`')))
    .collect();
  assert_eq!(examples.len(), 8);
  success(&f.batch(&examples));
  assert!(f.0.join("nb_visits_predictions.csv").is_file());
  let out = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .args(["--json", "--describe-command", "nbreg"])
    .output()
    .unwrap();
  success(&out);
  let r: Value = serde_json::from_slice(&out.stdout).unwrap();
  assert_eq!(r["data"]["syntax"], "nbreg y [x1 x2] [, robust noconstant]");
  for phrase in [
    "NB2",
    "0..100000",
    "(-12,12)",
    "JOINT observed information",
    "N/(N-1)",
    "NOT panel-robust",
  ] {
    assert!(topic.contains(phrase), "{phrase}");
  }
}
#[test]
fn intercept_only_and_no_constant_json_do_not_invent_boundary_or_slope_tests() {
  let f = Fixture::new(&sample());
  let out = f.batch(&["use mma20p1count.csv", "nbreg MDU LC IDP, noconstant"]);
  success(&out);
  let r = envelopes(&out);
  let fit = &r[1]["data"];
  assert_eq!(fit["null_model"], "unit_mean_dispersion_reestimated");
  assert_eq!(
    fit["covariance_parameter_names"],
    serde_json::json!(["LC", "IDP", "lnalpha"])
  );
  assert_eq!(fit["likelihood_ratio_df"], 2);
  let out = f.batch(&["use mma20p1count.csv", "nbreg MDU, robust", "estat margins"]);
  success(&out);
  let r = envelopes(&out);
  let fit = &r[1]["data"];
  assert!(fit["likelihood_ratio_p_value"].is_null());
  assert_eq!(
    fit["omnibus_wald"]["unavailable_reason"],
    "no_non_intercept_coefficients"
  );
  assert_eq!(r[2]["data"]["effects"], serde_json::json!([]));
  assert!(fit["dispersion"].get("p_value").is_none());
}
