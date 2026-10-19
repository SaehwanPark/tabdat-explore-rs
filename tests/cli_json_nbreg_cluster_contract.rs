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
      "tabdat-nb2-person-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("mma20p1count_person.csv"), csv).unwrap();
    Self(root)
  }
  fn script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma20p1count/nb2_clustered.td")
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
    fs::read(self.0.join("nb2_person_predictions.csv")).unwrap()
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
fn actual_nb2_person_script_has_full_joint_covariance_transformed_dispersion_and_exact_groups() {
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
      "NbregResult",
      "NbregMarginalEffectsResult",
      "PredictionResult",
      "PredictionResult",
      "ExportResult"
    ]
  );
  assert!(
    r.iter()
      .all(|e| e["schema_version"] == 1 && e["error"].is_null())
  );
  let fit = &r[5]["data"];
  let effects = &r[6]["data"];
  assert_eq!(fit["estimator"], "nb2");
  assert_eq!(fit["observation_count"], 80);
  assert_eq!(fit["df_residual"], 61);
  assert_eq!(fit["parameter_count"], 19);
  assert_eq!(
    fit["covariance"],
    "cluster_joint_score_sandwich_g_over_g_minus_1"
  );
  assert_eq!(effects["covariance"], fit["covariance"]);
  assert_eq!(fit["covariance_parameter_names"][18], "lnalpha");
  let v = fit["covariance_matrix"].as_array().unwrap();
  assert_eq!(v.len(), 19);
  assert!(v.iter().all(|row| row.as_array().unwrap().len() == 19));
  assert!(v[0][18].as_f64().unwrap().abs() > 0.);
  assert_eq!(
    fit["cluster"],
    serde_json::json!({"variable":"person_id","group_count":20,"finite_sample_multiplier":20./19.,"independence_assumption":"across_clusters"})
  );
  assert_eq!(effects["cluster"], fit["cluster"]);
  assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
  assert_eq!(fit["confidence_level"], 95.);
  assert!(
    fit["likelihood_ratio_p_value"].is_null()
      && fit["likelihood_ratio_reference_distribution"].is_null()
  );
  assert_eq!(fit["omnibus_wald"]["degrees_of_freedom"], 17);
  assert_eq!(
    effects["covariance_role"],
    "beta_block_of_full_joint_beta_lnalpha_covariance"
  );
  assert_eq!(
    fit["dispersion_boundary_test"],
    "not_implemented_no_normal_alpha_zero_test"
  );
  let d = &fit["dispersion"];
  let alpha = d["alpha"].as_f64().unwrap();
  let se = d["log_alpha_standard_error"].as_f64().unwrap();
  assert!((se * se - v[18][18].as_f64().unwrap()).abs() < 1e-12);
  assert!((d["alpha_standard_error"].as_f64().unwrap() - alpha * se).abs() < 1e-12);
  for field in ["lower", "upper"] {
    let log = d[format!("log_alpha_confidence_{field}")].as_f64().unwrap();
    let a = d[format!("alpha_confidence_{field}")].as_f64().unwrap();
    assert!((a - log.exp()).abs() < 1e-12);
  }
  assert!(d.get("p_value").is_none() && d.get("statistic").is_none());
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
      "prepared_row,original_row,person_id,study_year,MDU,{PREDICTORS},nb2_person_mu,nb2_person_xb"
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
fn actual_nb2_person_script_text_and_piped_match_json_without_row_independence_mislabel() {
  let source = sample();
  let json = Fixture::new(&source);
  success(&json.run("--json"));
  let text = Fixture::new(&source);
  let o = text.run("--text");
  success(&o);
  let output = String::from_utf8_lossy(&o.stdout);
  for phrase in [
    "FULL joint score sandwich",
    "G/(G-1)",
    "retained independent clusters G = 20",
    "independence ACROSS clusters",
    "NOT G-1 Student t",
    "LR p-value unavailable under clustered",
    "FULL joint beta/lnalpha covariance",
    "No normal alpha=0 test",
  ] {
    assert!(output.contains(phrase), "{phrase}");
  }
  assert!(!output.contains("Robust assumes independent rows"));
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
fn nb2_person_script_key_domain_and_export_failures_preserve_sources_and_targets() {
  let source = sample();
  let mutate = |column: usize, value: &str| {
    let mut rows: Vec<String> = source.lines().map(str::to_owned).collect();
    let mut fields: Vec<String> = rows[1].split(',').map(str::to_owned).collect();
    fields[column] = value.into();
    rows[1] = fields.join(",");
    rows.join("\n") + "\n"
  };
  for source in [
    source.replacen("MDU,", "absent,", 1),
    mutate(0, "15"),
    mutate(1, "1002"),
    mutate(2, ""),
    mutate(3, "2"),
    mutate(4, "-1"),
    mutate(4, "0.5"),
    mutate(4, "100001"),
  ] {
    let f = Fixture::new(&source);
    let o = f.run("--json");
    assert_eq!(o.status.code(), Some(1));
    assert!(!f.0.join("nb2_person_predictions.csv").exists());
    assert!(
      !records(&o)
        .iter()
        .any(|r| r["result_type"] == "NbregResult")
    );
    assert_eq!(
      fs::read_to_string(f.0.join("mma20p1count_person.csv")).unwrap(),
      source
    );
  }
  for directory in [false, true] {
    let f = Fixture::new(&source);
    let target = f.0.join("nb2_person_predictions.csv");
    if directory {
      fs::create_dir(&target).unwrap();
      fs::write(target.join("sentinel"), "keep").unwrap();
    } else {
      fs::write(&target, "keep").unwrap();
    }
    let o = f.run("--json");
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("nb2_clustered.td:13:"));
    assert_eq!(
      fs::read_to_string(if directory {
        target.join("sentinel")
      } else {
        target
      })
      .unwrap(),
      "keep"
    );
    assert_eq!(records(&o).last().unwrap()["error"]["type"], "ScriptError");
    assert_eq!(
      fs::read_to_string(f.0.join("mma20p1count_person.csv")).unwrap(),
      source
    );
  }
}
#[test]
fn nb2_cluster_intercept_and_no_constant_keep_null_and_wald_roles_distinct() {
  let f = Fixture::new(&sample());
  let o = f.batch(&[
    "use mma20p1count_person.csv",
    "nbreg MDU, cluster(person_id)",
    "estat margins",
  ]);
  success(&o);
  let r = records(&o);
  assert_eq!(r[1]["data"]["likelihood_ratio_df"], 0);
  assert!(r[1]["data"]["likelihood_ratio_p_value"].is_null());
  assert_eq!(
    r[1]["data"]["omnibus_wald"]["unavailable_reason"],
    "no_non_intercept_coefficients"
  );
  assert_eq!(r[2]["data"]["effects"], serde_json::json!([]));
  assert_eq!(r[1]["data"]["cluster"], r[2]["data"]["cluster"]);
  let o = f.batch(&[
    "use mma20p1count_person.csv",
    "nbreg MDU LC IDP, cluster(person_id) noconstant",
  ]);
  success(&o);
  let r = records(&o);
  assert_eq!(
    r[1]["data"]["null_model"],
    "unit_mean_dispersion_reestimated"
  );
  assert_eq!(
    r[1]["data"]["covariance_parameter_names"],
    serde_json::json!(["LC", "IDP", "lnalpha"])
  );
  assert!(r[1]["data"]["likelihood_ratio_p_value"].is_null());
}
