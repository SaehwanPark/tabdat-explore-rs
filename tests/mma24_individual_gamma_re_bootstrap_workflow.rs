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

const INPUT: &str = include_str!("fixtures/mma24_individual_gamma_re/input.csv");
const FITS: &str = include_str!("fixtures/mma24_individual_gamma_re_bootstrap/fits.csv");
const SUMMARY: &str = include_str!("fixtures/mma24_individual_gamma_re_bootstrap/summary.csv");
const DRAWS: &str = include_str!("fixtures/mma24_individual_gamma_re_bootstrap/draw_indices.csv");
const PREDICTIONS: &str =
  include_str!("fixtures/mma24_individual_gamma_re_bootstrap/predictions.csv");
const GROUPS: &str = include_str!("fixtures/mma24_individual_gamma_re_bootstrap/groups.csv");
const SCRIPT: &str =
  include_str!("../examples/mma24p2poiscluster/individual_gamma_re_bootstrap.td");
const OUTPUT: &str = "individual_gamma_re_bootstrap_predictions.csv";
const NAMES: [&str; 12] = [
  "_cons",
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
  "lnalpha",
];
fn script() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma24p2poiscluster/individual_gamma_re_bootstrap.td")
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(script())
    .output()
    .unwrap()
}
fn events(out: std::process::Output) -> Vec<Value> {
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let out = String::from_utf8(out.stdout).unwrap();
  let e: Vec<Value> = out
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  assert!(
    e.iter()
      .all(|v| v["schema_version"] == 1 && v.get("error").is_none())
  );
  e
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let names: Vec<_> = lines
    .next()
    .unwrap()
    .split(',')
    .map(|s| s.trim_matches('"'))
    .collect();
  lines
    .map(|line| {
      let fields: Vec<_> = line.split(',').map(|s| s.trim_matches('"')).collect();
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
fn actual_script_audits_all_distinct_copy_refits_and_predictions() {
  let d = TestDir::new();
  fs::write(d.path().join("vietnam_individual.csv"), INPUT).unwrap();
  let e = events(run(&d, "--json"));
  let r = &e
    .iter()
    .find(|v| v["result_type"] == "GammaPoissonBootstrapResult")
    .unwrap()["data"];
  assert_eq!(
    r["estimator"],
    "cluster_pairs_bootstrap_shared_gamma_poisson_re"
  );
  assert_eq!(r["observation_count"], 278);
  assert_eq!(r["zero_total_group_count"], 4);
  assert_eq!(r["singleton_group_count"], 2);
  assert_eq!(r["cluster_count"], 48);
  assert_eq!(r["entity_variable"], "commune");
  assert_eq!(r["seed"], "10001");
  assert_eq!(r["cluster_order"], "first_seen_complete_case");
  assert_eq!(r["cluster_index_base"], 0);
  assert_eq!(r["interval_method"], "type7_percentile_95");
  assert_eq!(r["covariance_convention"], "whole_cluster_pairs_b_minus_1");
  assert_eq!(r["covariance_parameter_names"], serde_json::json!(NAMES));
  let groups = rows(GROUPS);
  assert_eq!(groups.len(), 48);
  for (i, g) in groups.iter().enumerate() {
    assert_eq!(r["cluster_labels"][i], g["commune"]);
    assert_eq!(
      r["cluster_sizes"][i].as_u64().unwrap(),
      g["observations"].parse::<u64>().unwrap()
    );
  }
  let fits = rows(FITS);
  let draws = rows(DRAWS);
  assert_eq!(fits.len(), 500);
  assert_eq!(draws.len(), 500);
  for i in 0..500 {
    assert_eq!(fits[i]["replicate"].parse::<usize>().unwrap(), i + 1);
    assert_eq!(draws[i]["replicate"].parse::<usize>().unwrap(), i + 1);
    assert_eq!(
      r["replicate_observation_counts"][i].as_u64().unwrap(),
      fits[i]["sample_n"].parse::<u64>().unwrap()
    );
    let indices = r["replicate_cluster_indices"][i].as_array().unwrap();
    assert_eq!(indices.len(), 48);
    for (j, index) in indices.iter().enumerate() {
      assert_eq!(
        index.as_u64().unwrap(),
        draws[i][format!("copy_{}", j + 1).as_str()]
          .parse::<u64>()
          .unwrap()
      );
    }
    for (j, name) in NAMES.iter().enumerate() {
      near(
        r["replicate_coefficients"][i][j].as_f64().unwrap(),
        fits[i][name],
        name,
      );
    }
  }
  let summary = rows(SUMMARY);
  assert_eq!(summary.len(), 144);
  for (index, row) in summary.iter().enumerate() {
    let (i, j) = (index / 12, index % 12);
    assert_eq!(
      (
        row["parameter_index"].parse::<usize>().unwrap(),
        row["covariance_index"].parse::<usize>().unwrap()
      ),
      (i + 1, j + 1)
    );
    assert_eq!(
      (row["parameter"], row["covariance_parameter"]),
      (NAMES[i], NAMES[j])
    );
    let c = &r["coefficients"][i];
    for key in [
      "estimate",
      "bootstrap_mean",
      "bias",
      "standard_error",
      "ci_lower",
      "ci_upper",
    ] {
      near(c[key].as_f64().unwrap(), row[key], key);
    }
    near(
      r["covariance_matrix"][i][j].as_f64().unwrap(),
      row["covariance"],
      "covariance",
    );
  }
  let original = &e
    .iter()
    .find(|v| v["result_type"] == "GammaPoissonResult")
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
  near(
    original["ln_alpha"]["value"].as_f64().unwrap(),
    &r["coefficients"][11]["estimate"]
      .as_f64()
      .unwrap()
      .to_string(),
    "retained dispersion",
  );
  let text = fs::read_to_string(d.path().join(OUTPUT)).unwrap();
  let actual = rows(&text);
  let expected = rows(PREDICTIONS);
  assert_eq!(actual.len(), 278);
  assert_eq!(actual.len(), expected.len());
  for (a, b) in actual.iter().zip(&expected) {
    assert_eq!(a.len(), b.len());
    for (key, value) in b {
      if *key == "included_in_re" {
        assert_eq!(a[key].to_lowercase(), value.to_lowercase());
      } else if value.is_empty() {
        assert!(a[key].is_empty(), "{key}");
      } else {
        near(a[key].parse().unwrap(), value, key);
      }
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
  assert!(display.contains("Whole-group pairs-bootstrap shared-gamma Poisson RE"));
  for c in r["coefficients"].as_array().unwrap() {
    let n = |k: &str| c[k].as_f64().unwrap();
    let line = format!(
      "{}: estimate = {}; bootstrap mean = {}; bias = {}; SE = {}; 95% percentile CI = [{}, {}]",
      c["name"].as_str().unwrap(),
      n("estimate"),
      n("bootstrap_mean"),
      n("bias"),
      n("standard_error"),
      n("ci_lower"),
      n("ci_upper")
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
  let re = events(run(&reversed, "--json"));
  let rr = &re
    .iter()
    .find(|v| v["result_type"] == "GammaPoissonBootstrapResult")
    .unwrap()["data"];
  assert_eq!(rr, r);
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
fn full_width_seed_and_invalid_sources_preserve_files() {
  let d = TestDir::new();
  fs::write(d.path().join("vietnam_individual.csv"), INPUT).unwrap();
  let script = "use vietnam_individual.csv\nsql SELECT * FROM active WHERE pharvis IS NOT NULL ORDER BY source_row into cohort\nbootstrap pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, model(gamma_poisson_re) cluster(commune) seed(18446744073709551615) reps(2)\n";
  fs::write(d.path().join("seed.td"), script).unwrap();
  let out = tabdat_bin()
    .current_dir(d.path())
    .args(["--json", "-f", "seed.td"])
    .output()
    .unwrap();
  let e = events(out);
  let result = &e
    .iter()
    .find(|v| v["result_type"] == "GammaPoissonBootstrapResult")
    .unwrap()["data"];
  assert_eq!(result["seed"], "18446744073709551615");
  assert_eq!(result["sample"]["total_observations"], 278);
  assert_eq!(result["sample"]["dropped_observations"], 0);
  let lines = INPUT.lines().collect::<Vec<_>>();
  let mut bad_group = lines[1].split(',').collect::<Vec<_>>();
  bad_group[12] = "";
  let mut bad_count = lines[1].split(',').collect::<Vec<_>>();
  bad_count[1] = "100001";
  let mut large_group = lines[1].split(',').collect::<Vec<_>>();
  large_group[12] = "195";
  let mut large_source_row = lines[1].split(',').collect::<Vec<_>>();
  large_source_row[0] = "27767";
  let cases = [
    format!("{}\n{}\n{}\n", lines[0], lines[1], lines[1..].join("\n")),
    format!(
      "{}\n{}\n{}\n",
      lines[0],
      bad_group.join(","),
      lines[2..].join("\n")
    ),
    format!(
      "{}\n{}\n{}\n",
      lines[0],
      bad_count.join(","),
      lines[2..].join("\n")
    ),
    format!(
      "{}\n{}\n{}\n",
      lines[0],
      large_group.join(","),
      lines[2..].join("\n")
    ),
    format!(
      "{}\n{}\n{}\n",
      lines[0],
      large_source_row.join(","),
      lines[2..].join("\n")
    ),
  ];
  for input in cases {
    let work = TestDir::new();
    fs::write(work.path().join("vietnam_individual.csv"), &input).unwrap();
    let output = run(&work, "--json");
    assert_eq!(output.status.code(), Some(1));
    assert!(
      !String::from_utf8(output.stdout)
        .unwrap()
        .contains("GammaPoissonResult")
    );
    assert!(!work.path().join(OUTPUT).exists());
    assert_eq!(
      fs::read_to_string(work.path().join("vietnam_individual.csv")).unwrap(),
      input
    );
  }
}
#[test]
fn output_collision_preserves_source_owner_data_and_gamma_fit() {
  let d = TestDir::new();
  let source = d.path().join("vietnam_individual.csv");
  let owner = d.path().join(OUTPUT);
  fs::write(&source, INPUT).unwrap();
  fs::write(&owner, "owner\n").unwrap();
  let mut s = Session::new();
  let mut reached = false;
  for command in parse_script(SCRIPT, Path::new("individual_gamma_re_bootstrap.td")).unwrap() {
    let text = if command.text == "use vietnam_individual.csv" {
      format!("use \"{}\"", source.display())
    } else if command.text == format!("export {OUTPUT}") {
      format!("export \"{}\"", owner.display())
    } else {
      command.text.clone()
    };
    if command.text == format!("export {OUTPUT}") {
      let before = s.state();
      assert!(matches!(
        before.estimation,
        Some(EstimationState::GammaPoisson(_))
      ));
      let head = s.execute(parse_command("head 3").unwrap()).unwrap();
      assert!(s.execute(parse_command(&text).unwrap()).is_err());
      assert_eq!(s.state(), before);
      assert_eq!(s.execute(parse_command("head 3").unwrap()).unwrap(), head);
      s.execute(parse_command("predict retained_after_collision, n").unwrap())
        .unwrap();
      assert_eq!(s.state().estimation, before.estimation);
      reached = true;
      break;
    }
    s.execute(parse_command(&text).unwrap()).unwrap();
  }
  assert!(reached);
  assert_eq!(fs::read_to_string(&source).unwrap(), INPUT);
  assert_eq!(fs::read_to_string(&owner).unwrap(), "owner\n");
  let out = run(&d, "--json");
  assert_eq!(out.status.code(), Some(1));
  let output = String::from_utf8(out.stdout).unwrap();
  assert!(output.contains("GammaPoissonBootstrapResult") && output.contains("error"));
  assert_eq!(fs::read_to_string(owner).unwrap(), "owner\n");
  assert_eq!(fs::read_to_string(source).unwrap(), INPUT);
}
