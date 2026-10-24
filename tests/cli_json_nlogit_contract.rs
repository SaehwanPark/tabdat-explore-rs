use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::Command,
  sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
const MODEL: &str = r#"use seed.csv
sql """WITH people AS (
 SELECT row_number() OVER () AS id,p,q,income,selected
 FROM (VALUES ([0,1,0,0],[0,0,0,1],[6,12,16,48]),([0,0,0,1],[0,1,0,0],[8,24,9,18]),([1,0,0,1],[0,0,1,0],[12,6,75,50])) AS cells(p,q,counts)
 CROSS JOIN range(2) AS incomes(income)
 CROSS JOIN range(1,5) AS choices(selected)
 CROSS JOIN LATERAL range(CAST(counts[selected] * CASE WHEN selected<=2 THEN pow(3,income) ELSE 1 END AS BIGINT)) AS copies(replica)
)
SELECT id,alt,CASE WHEN alt<=2 THEN 1 ELSE 2 END AS nest,CAST(alt=selected AS INTEGER) AS chosen,
 p[alt] AS p,q[alt] AS q,CAST(alt<=2 AS INTEGER) AS dshore,CASE WHEN alt<=2 THEN income ELSE 0 END AS dshorey
FROM people CROSS JOIN range(1,5) AS alternatives(alt) ORDER BY id,alt""" into exact_choices
isid id alt
nlogit chosen p q, group(id) alternative(alt) nest(nest) upper(dshore dshorey) nonnormalized
predict baseline, pr
replace dshorey = dshorey+dshore
predict income_shift, pr
"#;
impl Fixture {
  fn new() -> Self {
    let p = std::env::temp_dir().join(format!(
      "tabdat-nlogit-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&p).unwrap();
    fs::write(p.join("seed.csv"), "seed\n0\n").unwrap();
    fs::write(p.join("plain.td"), MODEL).unwrap();
    fs::write(
      p.join("model.td"),
      format!("{MODEL}export predictions.csv\n"),
    )
    .unwrap();
    Self(p)
  }
  fn run(&self, args: &[&str]) -> std::process::Output {
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
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

#[test]
fn cli_v1_and_text_disclose_non_normalization_joint_covariance_and_scale_one_diagnostics() {
  let f = Fixture::new();
  let output = f.run(&["--json", "-f", "model.td"]);
  assert!(output.status.success(), "{output:?}");
  let records = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str::<Value>(l).unwrap())
    .collect::<Vec<_>>();
  assert!(records.iter().all(|r| r["schema_version"] == 1));
  let fit = &records
    .iter()
    .find(|r| r["result_type"] == "NonNormalizedNestedLogitResult")
    .unwrap()["data"];
  assert_eq!(fit["estimator"], "nlogit");
  assert_eq!(
    fit["parameterization"],
    "non_normalized_conditional_utilities"
  );
  assert_eq!(fit["preferred_rum_normalization"], false);
  assert_eq!(
    fit["covariance"],
    "model_based_joint_observed_nested_choice_hessian"
  );
  assert_eq!(fit["independent_unit"], "choice_set");
  assert_eq!(fit["covariance_correction"], 1.);
  assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
  assert_eq!(fit["confidence_level"], 95.);
  assert_eq!(fit["coefficient_test_null"], 0.);
  assert_eq!(fit["alternative_levels"], serde_json::json!([1, 2, 3, 4]));
  assert_eq!(fit["nest_levels"], serde_json::json!([1, 2]));
  assert_eq!(fit["alternative_nests"], serde_json::json!([1, 1, 2, 2]));
  assert_eq!(
    fit["chosen_counts"],
    serde_json::json!([104, 168, 200, 232])
  );
  for (key, want) in [
    ("observation_count", 2816),
    ("choice_set_count", 704),
    ("choice_set_size", 4),
    ("parameter_count", 6),
    ("lower_predictor_count", 2),
    ("independence_df", 2),
    ("excluded_choice_set_count", 0),
  ] {
    assert_eq!(fit[key], want);
  }
  for (coefficient, expected) in fit["coefficients"].as_array().unwrap().iter().zip([
    2_f64.ln(),
    3_f64.ln(),
    2_f64.ln(),
    3_f64.ln(),
  ]) {
    close(coefficient["value"].as_f64().unwrap(), expected);
  }
  let scales = fit["dissimilarities"].as_array().unwrap();
  assert_eq!(scales.len(), 2);
  for (d, expected) in scales.iter().zip([2., 3.]) {
    close(d["estimate"]["value"].as_f64().unwrap(), expected);
    assert_eq!(d["test_null"], 1.);
  }
  let covariance = fit["covariance_matrix"].as_array().unwrap();
  assert_eq!(covariance.len(), 6);
  assert!(covariance.iter().all(|r| r.as_array().unwrap().len() == 6));
  assert_eq!(
    fit["covariance_parameter_names"].as_array().unwrap().len(),
    6
  );
  assert_eq!(fit["random_utility_iv_bounds_satisfied"], false);
  assert_eq!(
    fit["independence_null"],
    "both_positive_nest_scales_equal_one_slopes_reestimated"
  );
  assert!(fit["independence_likelihood_ratio"].as_f64().unwrap() > 0.);
  assert!(fit.get("null_log_likelihood").is_none());
  assert!(fit.get("likelihood_ratio").is_none());
  assert_eq!(
    records
      .iter()
      .filter(|r| r["result_type"] == "PredictionResult")
      .count(),
    2
  );
  let exported = fs::read(f.0.join("predictions.csv")).unwrap();
  let contents = String::from_utf8(exported.clone()).unwrap();
  let lines = contents.lines().skip(1).collect::<Vec<_>>();
  assert_eq!(lines.len(), 2816);
  for set in lines.as_chunks::<4>().0 {
    let rows = set
      .iter()
      .map(|line| {
        line
          .split(',')
          .map(|v| v.parse::<f64>().unwrap())
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();
    let utility = rows
      .iter()
      .map(|r| (2_f64.ln() * r[4] + 3_f64.ln() * r[5]).exp())
      .collect::<Vec<_>>();
    let shore = utility[0] + utility[1];
    let boat = utility[2] + utility[3];
    // Exported upper income has already shifted jointly on the two shore rows.
    for (column, income) in [(8, rows[0][7] - 1.), (9, rows[0][7])] {
      let wshore = 2. * 3_f64.powf(income) * shore.powi(2);
      let wboat = boat.powi(3);
      for i in 0..4 {
        let expected = if i < 2 {
          utility[i] / shore * wshore / (wshore + wboat)
        } else {
          utility[i] / boat * wboat / (wshore + wboat)
        };
        close(rows[i][column], expected);
      }
    }
  }
  let collision = f.run(&["--json", "-f", "model.td"]);
  assert!(!collision.status.success());
  assert_eq!(fs::read(f.0.join("predictions.csv")).unwrap(), exported);
  let text = f.run(&["--text", "-f", "plain.td"]);
  assert!(text.status.success(), "{text:?}");
  let text = String::from_utf8(text.stdout).unwrap();
  for phrase in [
    "Non-normalized two-level nested choice",
    "NOT the preferred common-utility RUM normalization",
    "observed-Hessian covariance",
    "H0=1",
    "independence LR chi2(2)",
    "WARNING: fitted inclusive values violate RUM bounds",
  ] {
    assert!(text.contains(phrase), "{phrase}: {text}");
  }
}

#[test]
fn native_schema_exposes_required_roles_and_exact_explicit_choice_modes() {
  let f = Fixture::new();
  let out = f.run(&["--json", "--describe-command", "nlogit"]);
  assert!(out.status.success());
  let record: Value = serde_json::from_slice(&out.stdout).unwrap();
  let schema = &record["data"];
  assert_eq!(schema["name"], "nlogit");
  assert_eq!(schema["help_topic"], "nlogit");
  for name in ["group", "alternative", "nest"] {
    let option = schema["options"]
      .as_array()
      .unwrap()
      .iter()
      .find(|o| o["name"] == name)
      .unwrap();
    assert_eq!(option["required"], true);
  }
  for name in ["upper", "nonnormalized", "normalized", "common"] {
    let option = schema["options"]
      .as_array()
      .unwrap()
      .iter()
      .find(|o| o["name"] == name)
      .unwrap();
    assert_eq!(option["required"], false);
  }
  assert!(
    schema["syntax"]
      .as_str()
      .unwrap()
      .contains("(nonnormalized | normalized common)")
  );
  let help = f.run(&["--json", "--help-topic", "nlogit"]);
  assert!(help.status.success(), "{help:?}");
  let help = String::from_utf8(help.stdout).unwrap();
  assert!(help.contains("normalized common"), "{help}");
  assert!(help.contains("mixing modes fails"), "{help}");
}
