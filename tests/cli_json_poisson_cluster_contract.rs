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
  let mut csv = format!("prepared_row,original_row,person_id,study_year,MDU,{PREDICTORS}\n");
  for i in 1..=80 {
    csv.push_str(&format!(
      "{},{},{},{},{}",
      i * 7 % 83 + 1,
      i + 1000,
      100 + i % 20,
      (i - 1) / 20 + 1,
      i * 13 % 17
    ));
    for j in 0..17 {
      csv.push_str(&format!(
        ",{}",
        ((i * (j + 3) * 7 + j * j) % 97) as f64 / 23. - 2.
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
      "tabdat-poisson-person-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("mma20p1count_person.csv"), csv).unwrap();
    Self(root)
  }
  fn script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma20p1count/poisson_clustered.td")
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
    fs::read(self.0.join("poisson_person_predictions.csv")).unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn success(o: &Output) {
  assert!(o.status.success(), "{o:?}");
  assert!(o.stderr.is_empty(), "{o:?}");
}
fn records(o: &Output) -> Vec<Value> {
  String::from_utf8_lossy(&o.stdout)
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect()
}
#[test]
fn actual_person_script_reports_exact_groups_normal_inference_and_preserves_source_identity() {
  let source = sample();
  let f = Fixture::new(&source);
  let o = f.run("--json");
  success(&o);
  let r = records(&o);
  assert_eq!(
    r.iter()
      .filter_map(|e| e["result_type"].as_str())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "IsidResult",
      "IsidResult",
      "IsidResult",
      "SummarizeResult",
      "PoissonResult",
      "PoissonMarginalEffectsResult",
      "PredictionResult",
      "PredictionResult",
      "ExportResult"
    ]
  );
  assert!(
    r.iter()
      .all(|r| r["schema_version"] == 1 && r["error"].is_null())
  );
  let fit = &r[5]["data"];
  let effects = &r[6]["data"];
  assert_eq!(fit["observation_count"], 80);
  assert_eq!(fit["df_residual"], 62);
  assert_eq!(fit["covariance"], "cluster_sandwich_g_over_g_minus_1");
  assert_eq!(effects["covariance"], fit["covariance"]);
  assert_eq!(fit["cluster"]["variable"], "person_id");
  assert_eq!(fit["cluster"]["group_count"], 20);
  assert!((fit["cluster"]["finite_sample_multiplier"].as_f64().unwrap() - 20. / 19.).abs() < 1e-15);
  assert_eq!(fit["cluster"]["independence_assumption"], "across_clusters");
  assert_eq!(fit["cluster"], effects["cluster"]);
  assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
  assert_eq!(
    effects["reference_distribution"],
    fit["reference_distribution"]
  );
  assert_eq!(fit["confidence_level"], 95.);
  assert!(
    fit["likelihood_ratio_p_value"].is_null()
      && fit["likelihood_ratio_reference_distribution"].is_null()
  );
  assert_eq!(fit["omnibus_wald"]["degrees_of_freedom"], 17);
  assert_eq!(
    fit["omnibus_wald"]["reference_distribution"],
    "asymptotic_chi_square"
  );
  assert_eq!(
    fit["sample"]["retained_indices"],
    serde_json::json!((0..80).collect::<Vec<_>>())
  );
  assert_eq!(
    fit["sample"]["cluster_groups"],
    serde_json::json!(
      (1..=80)
        .map(|i| (100 + i % 20).to_string())
        .collect::<Vec<_>>()
    )
  );
  let output = String::from_utf8(f.csv()).unwrap();
  assert_eq!(output.lines().count(), 81);
  assert_eq!(
    output.lines().next().unwrap(),
    format!(
      "prepared_row,original_row,person_id,study_year,MDU,{PREDICTORS},poisson_person_mu,poisson_person_xb"
    )
  );
  for (a, b) in source.lines().skip(1).zip(output.lines().skip(1)) {
    let a: Vec<f64> = a.split(',').map(|v| v.parse().unwrap()).collect();
    let b: Vec<f64> = b.split(',').map(|v| v.parse().unwrap()).collect();
    assert_eq!(b.len(), 24);
    for (a, b) in a.iter().zip(&b) {
      assert!((a - b).abs() < 1e-12);
    }
    assert!((b[22] - b[23].exp()).abs() < 1e-10);
  }
  assert_eq!(
    fs::read_to_string(f.0.join("mma20p1count_person.csv")).unwrap(),
    source
  );
}
#[test]
fn actual_person_script_json_text_and_piped_publish_identical_csvs() {
  let source = sample();
  let json = Fixture::new(&source);
  success(&json.run("--json"));
  let text = Fixture::new(&source);
  let o = text.run("--text");
  success(&o);
  let output = String::from_utf8_lossy(&o.stdout);
  for phrase in [
    "G/(G-1)",
    "retained independent clusters G = 20",
    "independence ACROSS clusters",
    "NOT G-1 Student t",
    "LR p-value unavailable under clustered",
    "Poisson continuous average partial effects",
    "outer Newton iterations",
  ] {
    assert!(output.contains(phrase), "{phrase}");
  }
  assert!(!output.contains("independent rows, NOT repeated-person"));
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
fn person_script_export_collisions_do_not_replace_files_or_directories() {
  for directory in [false, true] {
    let source = sample();
    let f = Fixture::new(&source);
    let target = f.0.join("poisson_person_predictions.csv");
    if directory {
      fs::create_dir(&target).unwrap();
      fs::write(target.join("sentinel"), "keep").unwrap();
    } else {
      fs::write(&target, "keep").unwrap();
    }
    let o = f.run("--json");
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("poisson_clustered.td:13:"));
    assert_eq!(
      fs::read_to_string(if directory {
        target.join("sentinel")
      } else {
        target
      })
      .unwrap(),
      "keep"
    );
    let r = records(&o);
    assert_eq!(r.last().unwrap()["error"]["type"], "ScriptError");
    assert_eq!(
      r.iter()
        .filter(|r| r["result_type"] == "PoissonResult")
        .count(),
      1
    );
    assert_eq!(
      fs::read_to_string(f.0.join("mma20p1count_person.csv")).unwrap(),
      source
    );
  }
}
#[test]
fn key_schema_and_finite_count_failures_stop_before_fit_or_export() {
  let source = sample();
  let mutate = |column: usize, value: &str| {
    let mut lines: Vec<String> = source.lines().map(str::to_owned).collect();
    let mut fields: Vec<String> = lines[1].split(',').map(str::to_owned).collect();
    fields[column] = value.into();
    lines[1] = fields.join(",");
    lines.join("\n") + "\n"
  };
  for source in [
    source.replacen("MDU,", "missing,", 1),
    mutate(0, "15"),
    mutate(1, "1002"),
    mutate(2, ""),
    mutate(4, "-1"),
    mutate(4, "0.5"),
    mutate(4, "9007199254740992"),
  ] {
    let f = Fixture::new(&source);
    let o = f.run("--json");
    assert_eq!(o.status.code(), Some(1));
    assert!(!f.0.join("poisson_person_predictions.csv").exists());
    assert!(
      !records(&o)
        .iter()
        .any(|r| r["result_type"] == "PoissonResult")
    );
    assert_eq!(
      fs::read_to_string(f.0.join("mma20p1count_person.csv")).unwrap(),
      source
    );
  }
}
#[test]
fn noncluster_shapes_stay_unchanged_and_intercept_only_cluster_inference_is_not_invented() {
  let f = Fixture::new(&sample());
  for option in ["", ", robust"] {
    let command = format!("poisson MDU LC IDP{option}");
    let o = f.batch(&["use mma20p1count_person.csv", &command, "estat margins"]);
    success(&o);
    let r = records(&o);
    for index in [1, 2] {
      assert!(r[index]["data"].get("cluster").is_none());
      assert_eq!(
        r[index]["data"]["covariance"],
        if option.is_empty() {
          "model_based_poisson_information"
        } else {
          "robust_sandwich_n_over_n_minus_1"
        }
      );
    }
  }
  let o = f.batch(&[
    "use mma20p1count_person.csv",
    "poisson MDU, cluster(person_id)",
    "estat margins",
  ]);
  success(&o);
  let r = records(&o);
  assert_eq!(
    r[1]["data"]["omnibus_wald"]["unavailable_reason"],
    "no_non_intercept_coefficients"
  );
  assert_eq!(r[1]["data"]["likelihood_ratio_df"], 0);
  assert!(r[1]["data"]["likelihood_ratio_p_value"].is_null());
  assert_eq!(r[2]["data"]["effects"], serde_json::json!([]));
  assert_eq!(r[1]["data"]["cluster"], r[2]["data"]["cluster"]);
  let o = f.batch(&[
    "use mma20p1count_person.csv",
    "poisson MDU LC IDP, cluster(person_id) noconstant",
  ]);
  success(&o);
  let r = records(&o);
  assert_eq!(
    r[1]["data"]["null_model"],
    "unit_mean_all_coefficients_zero"
  );
  assert_eq!(
    r[1]["data"]["covariance_parameter_names"],
    serde_json::json!(["LC", "IDP"])
  );
}
