mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Output, Stdio},
};
#[test]
fn countprob_discovery_explains_bounds_retention_and_distributional_limits() {
  let o = tabdat_bin()
    .args(["--json", "--describe-command", "estat"])
    .output()
    .unwrap();
  success(&o);
  let schema: Value = serde_json::from_slice(&o.stdout).unwrap();
  assert!(
    schema["data"]["syntax"]
      .as_str()
      .unwrap()
      .contains("estat countprob [, max(10)]")
  );
  assert!(
    schema["data"]["options"]
      .as_array()
      .unwrap()
      .iter()
      .any(|o| o["name"] == "max")
  );
  let o = tabdat_bin()
    .args(["--json", "--help-topic", "estat"])
    .output()
    .unwrap();
  success(&o);
  let help: Value = serde_json::from_slice(&o.stdout).unwrap();
  let text = help["data"]["text"].as_str().unwrap();
  for phrase in [
    "integer1..100",
    "**>=10**",
    "average of individual conditional probabilities",
    "ORIGINAL retained",
    "NO formal goodness-of-fit test",
    "printed Table20.6 NB2 row",
  ] {
    assert!(text.contains(phrase), "{phrase}");
  }
}

const PREDICTORS: &str = "LC,IDP,LPI,FMDE,PHYSLIM,NDISEASE,HLTHG,HLTHF,HLTHP,LINC,LFAM,EDUCDEC,AGE,FEMALE,CHILD,FEMCHILD,BLACK";
fn source() -> String {
  let mut csv = format!("prepared_row,MDU,{PREDICTORS}\n");
  for i in 1..=80 {
    csv.push_str(&format!("{},{}", i * 7 % 83 + 1, i * 13 % 17));
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
fn fixture() -> TestDir {
  let d = TestDir::new();
  fs::write(d.path().join("mma20p1count.csv"), source()).unwrap();
  d
}
fn script() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma20p1count/count_probabilities.td")
}
fn run(d: &TestDir, flag: &str) -> Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([flag, "-f"])
    .arg(script())
    .output()
    .unwrap()
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
fn predictions(d: &TestDir) -> Vec<u8> {
  fs::read(d.path().join("count_probability_predictions.csv")).unwrap()
}

#[test]
fn actual_script_reports_retained_distributional_bins_without_inventing_uncertainty_or_tests() {
  let d = fixture();
  let o = run(&d, "--json");
  success(&o);
  let r = records(&o);
  assert_eq!(
    r.iter()
      .map(|v| v["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "IsidResult",
      "SummarizeResult",
      "PoissonResult",
      "CountProbabilityResult",
      "PredictionResult",
      "PredictionResult",
      "NbregResult",
      "CountProbabilityResult",
      "PredictionResult",
      "PredictionResult",
      "ExportResult"
    ]
  );
  assert!(
    r.iter()
      .all(|e| e["schema_version"] == 1 && e["error"].is_null())
  );
  for (index, fit, estimator) in [(4, 3, "poisson"), (8, 7, "nb2")] {
    let t = &r[index]["data"];
    assert_eq!(t["estimator"], estimator);
    assert_eq!(t["distribution"]["family"], estimator);
    assert_eq!(t["outcome"], "MDU");
    assert_eq!(t["observation_count"], 80);
    assert_eq!(t["tail_from"], 10);
    assert_eq!(t["sample"], r[fit]["data"]["sample"]);
    assert_eq!(t["sample_role"], "retained_estimation_sample");
    assert_eq!(
      t["probability_role"],
      "mean_of_individual_plugin_conditional_probabilities"
    );
    assert_eq!(t["covariance_role"], "does_not_establish_distribution");
    assert_eq!(t["uncertainty"], "not_computed");
    assert_eq!(t["goodness_of_fit_test"], "not_performed");
    assert!(t.get("p_value").is_none() && t.get("standard_error").is_none());
    let bins = t["bins"].as_array().unwrap();
    assert_eq!(bins.len(), 11);
    let mut fitted = 0.;
    for (i, b) in bins.iter().enumerate() {
      assert_eq!(b["lower_bound"], i);
      assert_eq!(
        b["upper_bound"],
        if i == 10 { Value::Null } else { Value::from(i) }
      );
      let observed = (1..=80).filter(|j| ((j * 13) % 17).min(10) == i).count();
      assert_eq!(b["observed_count"], observed);
      assert_eq!(b["observed_fraction"], observed as f64 / 80.);
      let p = b["fitted_fraction"].as_f64().unwrap();
      assert!((0.0..=1.0).contains(&p));
      assert!((b["expected_count"].as_f64().unwrap() - 80. * p).abs() < 1e-12);
      fitted += p;
    }
    assert!((fitted - 1.).abs() < 1e-12);
  }
  assert_eq!(
    r[8]["data"]["distribution"]["alpha"],
    r[7]["data"]["dispersion"]["alpha"]
  );
  let output = String::from_utf8(predictions(&d)).unwrap();
  assert_eq!(
    output.lines().next().unwrap(),
    format!("prepared_row,MDU,{PREDICTORS},poisson_mu,poisson_xb,nb2_mu,nb2_xb")
  );
  assert_eq!(output.lines().count(), 81);
  for (a, b) in source().lines().skip(1).zip(output.lines().skip(1)) {
    let a: Vec<f64> = a.split(',').map(|x| x.parse().unwrap()).collect();
    let b: Vec<f64> = b.split(',').map(|x| x.parse().unwrap()).collect();
    assert_eq!(b.len(), 23);
    for (a, b) in a.iter().zip(&b) {
      assert!((a - b).abs() < 1e-12);
    }
    for (mean, xb) in [(19, 20), (21, 22)] {
      assert!((b[mean] - b[xb].exp()).abs() < 1e-10);
    }
  }
  assert_eq!(
    fs::read_to_string(d.path().join("mma20p1count.csv")).unwrap(),
    source()
  );
}

#[test]
fn actual_script_text_and_piped_have_same_predictions_and_explicit_assumptions() {
  let json = fixture();
  success(&run(&json, "--json"));
  let text = fixture();
  let o = run(&text, "--text");
  success(&o);
  let t = String::from_utf8_lossy(&o.stdout);
  for phrase in [
    "Observed/fitted count frequencies: Poisson",
    "Observed/fitted count frequencies: NB2",
    "ORIGINAL retained sample",
    "not the probability at an average mean",
    "NOT guaranteed by robust/cluster QMLE covariance",
    "No uncertainty or formal goodness-of-fit test",
    ">=10",
  ] {
    assert!(t.contains(phrase), "{phrase}");
  }
  let piped = fixture();
  let mut c = tabdat_bin()
    .current_dir(piped.path())
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  c.stdin
    .take()
    .unwrap()
    .write_all(format!("run \"{}\"\nexit\n", script().display()).as_bytes())
    .unwrap();
  success(&c.wait_with_output().unwrap());
  assert_eq!(predictions(&json), predictions(&text));
  assert_eq!(predictions(&json), predictions(&piped));
}

#[test]
fn actual_script_collisions_preserve_sources_and_targets_and_invalid_requests_are_typed() {
  for directory in [false, true] {
    let d = fixture();
    let target = d.path().join("count_probability_predictions.csv");
    if directory {
      fs::create_dir(&target).unwrap();
      fs::write(target.join("sentinel"), "keep").unwrap();
    } else {
      fs::write(&target, "keep").unwrap();
    }
    let o = run(&d, "--json");
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("count_probabilities.td:15:"));
    assert_eq!(records(&o).last().unwrap()["error"]["type"], "ScriptError");
    assert_eq!(
      fs::read_to_string(if directory {
        target.join("sentinel")
      } else {
        target
      })
      .unwrap(),
      "keep"
    );
    assert_eq!(
      fs::read_to_string(d.path().join("mma20p1count.csv")).unwrap(),
      source()
    );
  }
  let d = fixture();
  let o = tabdat_bin()
    .current_dir(d.path())
    .args(["--json", "-c", "estat countprob"])
    .output()
    .unwrap();
  assert_eq!(o.status.code(), Some(1));
  assert_eq!(records(&o)[0]["error"]["type"], "ExecutionError");
  assert_eq!(
    records(&o)[0]["error"]["message"],
    "estat countprob requires a prior Poisson or NB2 fit"
  );
  let o = tabdat_bin()
    .current_dir(d.path())
    .args(["--json", "-c", "estat countprob, max(101)"])
    .output()
    .unwrap();
  assert_eq!(o.status.code(), Some(2));
  assert!(String::from_utf8_lossy(&o.stderr).contains("integer from 1 to 100"));
}
