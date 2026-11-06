mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
};
use tabdat_domain::EstimationState;
use tabdat_language::{parse_command, script::parse_script};
use tabdat_runtime::Session;

const INPUT: &str = include_str!("fixtures/mma24_individual_pooled_bootstrap/input.csv");
const FITS: &str = include_str!("fixtures/mma24_individual_pooled_bootstrap/fits.csv");
const SUMMARY: &str = include_str!("fixtures/mma24_individual_pooled_bootstrap/summary.csv");
const DRAWS: &str = include_str!("fixtures/mma24_individual_pooled_bootstrap/draw_indices.csv");
const PREDICTIONS: &str =
  include_str!("fixtures/mma24_individual_pooled_bootstrap/predictions.csv");
const OUTPUT: &str = "individual_pooled_bootstrap_predictions.csv";
const SCRIPT: &str = include_str!("../examples/mma24p2poiscluster/individual_pooled_bootstrap.td");
const NAMES: [&str; 11] = [
  "intercept",
  "lnhhexp",
  "insurance",
  "sex",
  "age",
  "married",
  "illdays",
  "actdays",
  "injury",
  "illness",
  "educ",
];
fn script() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma24p2poiscluster/individual_pooled_bootstrap.td")
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(script())
    .output()
    .unwrap()
}
fn events(output: std::process::Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let e = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str::<Value>(line).unwrap())
    .collect::<Vec<_>>();
  assert!(
    e.iter()
      .all(|v| v["schema_version"] == 1 && v.get("error").is_none())
  );
  e
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let names = lines
    .next()
    .unwrap()
    .split(',')
    .map(|x| x.trim_matches('"'))
    .collect::<Vec<_>>();
  lines
    .map(|line| {
      let fields = line
        .split(',')
        .map(|x| x.trim_matches('"'))
        .collect::<Vec<_>>();
      assert_eq!(fields.len(), names.len());
      names.iter().copied().zip(fields).collect()
    })
    .collect()
}
fn near(a: f64, b: &str, key: &str) {
  let b = b.parse::<f64>().unwrap();
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9,
    "{key}: {a} != {b}"
  );
}

#[test]
fn actual_script_audits_all_refits_covariance_draws_predictions_and_text_mode() {
  let d = TestDir::new();
  fs::write(d.path().join("vietnam_individual.csv"), INPUT).unwrap();
  let e = events(run(&d, "--json"));
  let r = &e
    .iter()
    .find(|v| v["result_type"] == "ClusterBootstrapPoissonResult")
    .unwrap()["data"];
  assert_eq!(r["estimator"], "cluster_pairs_bootstrap_poisson");
  assert_eq!(r["observation_count"], 240);
  assert_eq!(r["cluster_count"], 21);
  assert_eq!(r["seed"], "10001");
  assert_eq!(r["cluster_variable"], "commune");
  assert_eq!(r["cluster_order"], "first_seen_complete_case");
  assert_eq!(r["cluster_index_base"], 0);
  assert_eq!(r["interval_method"], "type7_percentile_95");
  assert_eq!(r["covariance_convention"], "whole_cluster_pairs_b_minus_1");
  assert_eq!(r["covariance_parameter_names"], serde_json::json!(NAMES));
  let fits = rows(FITS);
  let draws = rows(DRAWS);
  assert_eq!(fits.len(), 500);
  assert_eq!(draws.len(), 500);
  for i in 0..500 {
    assert_eq!(fits[i]["rep"].parse::<usize>().unwrap(), i + 1);
    assert_eq!(draws[i]["replicate"].parse::<usize>().unwrap(), i + 1);
    assert_eq!(
      r["replicate_observation_counts"][i].as_u64().unwrap(),
      fits[i]["N"].parse::<u64>().unwrap()
    );
    let indices = r["replicate_cluster_indices"][i].as_array().unwrap();
    assert_eq!(indices.len(), 21);
    for (j, index) in indices.iter().enumerate() {
      assert_eq!(
        index.as_u64().unwrap(),
        draws[i][format!("copy_{}", j + 1).as_str()]
          .parse::<u64>()
          .unwrap()
      );
    }
    for (j, field) in NAMES.iter().enumerate() {
      near(
        r["replicate_coefficients"][i][j].as_f64().unwrap(),
        fits[i][format!("X{}", j + 1).as_str()],
        field,
      );
    }
  }
  let summary = rows(SUMMARY);
  assert_eq!(summary.len(), 121);
  for (k, row) in summary.iter().enumerate() {
    let (i, j) = (k / 11, k % 11);
    assert_eq!(
      (
        row["i"].parse::<usize>().unwrap(),
        row["j"].parse::<usize>().unwrap()
      ),
      (i + 1, j + 1)
    );
    let c = &r["coefficients"][i];
    assert_eq!(c["name"], NAMES[i]);
    for (field, key) in [
      ("estimate", "estimate"),
      ("bootstrap_mean", "mean"),
      ("bias", "bias"),
      ("standard_error", "se"),
      ("ci_lower", "lower"),
      ("ci_upper", "upper"),
    ] {
      near(c[field].as_f64().unwrap(), row[key], field);
    }
    near(
      r["covariance_matrix"][i][j].as_f64().unwrap(),
      row["covariance"],
      "covariance",
    );
  }
  let original = &e
    .iter()
    .find(|v| v["result_type"] == "PoissonResult")
    .unwrap()["data"];
  for i in 0..11 {
    assert_eq!(original["coefficients"][i]["name"], NAMES[i]);
    near(
      original["coefficients"][i]["value"].as_f64().unwrap(),
      &r["coefficients"][i]["estimate"]
        .as_f64()
        .unwrap()
        .to_string(),
      "retained point",
    );
  }
  let text = fs::read_to_string(d.path().join(OUTPUT)).unwrap();
  let actual = rows(&text);
  let expected = rows(PREDICTIONS);
  assert_eq!(actual.len(), 240);
  assert_eq!(actual.len(), expected.len());
  for (a, b) in actual.iter().zip(expected.iter()) {
    assert_eq!(a.len(), b.len());
    for (key, value) in b {
      near(a[key].parse::<f64>().unwrap(), value, key);
    }
  }
  assert_eq!(
    fs::read_to_string(d.path().join("vietnam_individual.csv")).unwrap(),
    INPUT
  );
  let t = TestDir::new();
  fs::write(t.path().join("vietnam_individual.csv"), INPUT).unwrap();
  let out = run(&t, "--text");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let display = String::from_utf8(out.stdout).unwrap();
  assert!(display.contains("Whole-cluster pairs-bootstrap Poisson"));
  for c in r["coefficients"].as_array().unwrap() {
    let number = |key: &str| c[key].as_f64().unwrap();
    let line = format!(
      "{}: estimate = {}; bootstrap mean = {}; bias = {}; SE = {}; 95% percentile CI = [{}, {}]",
      c["name"].as_str().unwrap(),
      number("estimate"),
      number("bootstrap_mean"),
      number("bias"),
      number("standard_error"),
      number("ci_lower"),
      number("ci_upper")
    );
    assert!(display.lines().any(|s| s == line), "missing {line}");
  }
  assert_eq!(fs::read(t.path().join(OUTPUT)).unwrap(), text.as_bytes());
  let mut lines = INPUT.lines();
  let header = lines.next().unwrap();
  let reversed_input = format!(
    "{}\n{}\n",
    header,
    lines.rev().collect::<Vec<_>>().join("\n")
  );
  let reversed = TestDir::new();
  fs::write(
    reversed.path().join("vietnam_individual.csv"),
    &reversed_input,
  )
  .unwrap();
  let reversed_events = events(run(&reversed, "--json"));
  let reversed_report = &reversed_events
    .iter()
    .find(|v| v["result_type"] == "ClusterBootstrapPoissonResult")
    .unwrap()["data"];
  assert_eq!(reversed_report, r);
  assert_eq!(
    fs::read(reversed.path().join(OUTPUT)).unwrap(),
    text.as_bytes()
  );
  assert_eq!(
    fs::read_to_string(reversed.path().join("vietnam_individual.csv")).unwrap(),
    reversed_input
  );
}

#[test]
fn cli_json_preserves_full_width_seed_and_complete_case_metadata() {
  let d = TestDir::new();
  fs::write(d.path().join("vietnam_individual.csv"), INPUT).unwrap();
  let script = "use vietnam_individual.csv\nbootstrap pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, model(poisson) cluster(commune) seed(18446744073709551615) reps(2)\n";
  fs::write(d.path().join("seed.td"), script).unwrap();
  let out = tabdat_bin()
    .current_dir(d.path())
    .args(["--json", "-f", "seed.td"])
    .output()
    .unwrap();
  let events = events(out);
  let result = &events
    .iter()
    .find(|v| v["result_type"] == "ClusterBootstrapPoissonResult")
    .unwrap()["data"];
  assert_eq!(result["seed"], "18446744073709551615");
  assert_eq!(result["sample"]["total_observations"], 241);
  assert_eq!(result["sample"]["dropped_observations"], 1);
}

#[test]
fn all_zero_resampled_communes_abort_after_original_fit_without_export() {
  let mut lines = INPUT.lines();
  let mut input = format!("{}\n", lines.next().unwrap());
  let mut i = 0;
  for line in lines {
    let mut f = line.split(',').map(str::to_owned).collect::<Vec<_>>();
    if f[1].is_empty() {
      continue;
    }
    if i < 120 {
      f[12] = "1".into();
    } else {
      f[12] = "2".into();
      f[1] = "0".into();
    }
    i += 1;
    input.push_str(&f.join(","));
    input.push('\n');
  }
  assert_eq!(i, 240);
  let d = TestDir::new();
  fs::write(d.path().join("vietnam_individual.csv"), &input).unwrap();
  let out = run(&d, "--json");
  assert_eq!(out.status.code(), Some(1));
  let stdout = String::from_utf8(out.stdout).unwrap();
  assert!(stdout.contains("\"PoissonResult\""));
  assert!(stdout.contains("replicate 5 failed"));
  assert!(stdout.contains("all-zero outcome"));
  assert!(!stdout.contains("ClusterBootstrapPoissonResult"));
  assert!(!d.path().join(OUTPUT).exists());
  assert_eq!(
    fs::read_to_string(d.path().join("vietnam_individual.csv")).unwrap(),
    input
  );
}

#[test]
fn prediction_export_collision_preserves_owner_and_source() {
  let d = TestDir::new();
  fs::write(d.path().join("vietnam_individual.csv"), INPUT).unwrap();
  fs::write(d.path().join(OUTPUT), "existing owner\n").unwrap();
  let out = run(&d, "--json");
  assert_eq!(out.status.code(), Some(1));
  assert_eq!(
    fs::read_to_string(d.path().join(OUTPUT)).unwrap(),
    "existing owner\n"
  );
  assert_eq!(
    fs::read_to_string(d.path().join("vietnam_individual.csv")).unwrap(),
    INPUT
  );
  let e = String::from_utf8(out.stdout).unwrap();
  assert!(e.contains("ClusterBootstrapPoissonResult"));
  assert!(e.contains("error"));
}

#[test]
fn collision_keeps_original_native_poisson_fit_and_current_data() {
  let d = TestDir::new();
  let source = d.path().join("vietnam_individual.csv");
  let owner = d.path().join(OUTPUT);
  fs::write(&source, INPUT).unwrap();
  fs::write(&owner, "owner\n").unwrap();
  let mut session = Session::new();
  let mut saw_collision = false;
  for command in parse_script(SCRIPT, Path::new("individual_pooled_bootstrap.td")).unwrap() {
    let text = if command.text == "use vietnam_individual.csv" {
      format!("use \"{}\"", source.display())
    } else if command.text == format!("export {OUTPUT}") {
      format!("export \"{}\"", owner.display())
    } else {
      command.text.clone()
    };
    if command.text == format!("export {OUTPUT}") {
      let before = session.state();
      assert!(matches!(
        before.estimation,
        Some(EstimationState::PoissonFit(_))
      ));
      let head = session.execute(parse_command("head 3").unwrap()).unwrap();
      assert!(session.execute(parse_command(&text).unwrap()).is_err());
      assert_eq!(session.state(), before);
      assert_eq!(
        session.execute(parse_command("head 3").unwrap()).unwrap(),
        head
      );
      session
        .execute(parse_command("predict retained_after_collision, n").unwrap())
        .unwrap();
      assert_eq!(session.state().estimation, before.estimation);
      saw_collision = true;
      break;
    }
    session.execute(parse_command(&text).unwrap()).unwrap();
  }
  assert!(saw_collision);
  assert_eq!(fs::read_to_string(source).unwrap(), INPUT);
  assert_eq!(fs::read_to_string(owner).unwrap(), "owner\n");
}

#[test]
fn invalid_source_and_keys_fail_before_fit_without_exports() {
  let lines = INPUT.lines().collect::<Vec<_>>();
  let mut no_group = lines[1].split(',').collect::<Vec<_>>();
  no_group[12] = "";
  let mut too_large = lines[1].split(',').collect::<Vec<_>>();
  too_large[12] = "195";
  let mut bad_count = lines[1].split(',').collect::<Vec<_>>();
  bad_count[1] = "100001";
  let cases = [
    format!("{}\n{}\n{}\n", lines[0], lines[1], lines[1..].join("\n")),
    format!(
      "{}\n{}\n{}\n",
      lines[0],
      no_group.join(","),
      lines[2..].join("\n")
    ),
    format!(
      "{}\n{}\n{}\n",
      lines[0],
      too_large.join(","),
      lines[2..].join("\n")
    ),
    format!(
      "{}\n{}\n{}\n",
      lines[0],
      bad_count.join(","),
      lines[2..].join("\n")
    ),
  ];
  for input in cases {
    let d = TestDir::new();
    fs::write(d.path().join("vietnam_individual.csv"), &input).unwrap();
    let out = run(&d, "--json");
    assert_eq!(out.status.code(), Some(1));
    assert!(
      !String::from_utf8(out.stdout)
        .unwrap()
        .contains("PoissonResult")
    );
    assert!(!d.path().join(OUTPUT).exists());
    assert_eq!(
      fs::read_to_string(d.path().join("vietnam_individual.csv")).unwrap(),
      input
    );
  }
}
