mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};

const INTERIOR: &str = include_str!("fixtures/mma21_reml_bootstrap/interior_inputs.csv");
const BOUNDARY: &str = include_str!("fixtures/mma21_reml_bootstrap/boundary_inputs.csv");
const OUTPUTS: [&str; 3] = [
  "panel_re_gls_refit_bootstrap_summary.csv",
  "panel_re_gls_refit_bootstrap_draws.csv",
  "panel_re_gls_refit_bootstrap_predictions.csv",
];
fn script() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma21p1panel/random_effects_gls_refit_bootstrap.td")
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(script())
    .output()
    .unwrap()
}
fn records(text: &str) -> Vec<BTreeMap<String, String>> {
  let mut lines = text.lines();
  let header: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|l| {
      let values: Vec<_> = l.split(',').collect();
      assert_eq!(values.len(), header.len());
      header
        .iter()
        .zip(values)
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    })
    .collect()
}
fn number(r: &BTreeMap<String, String>, k: &str) -> f64 {
  r[k].parse().unwrap()
}
fn close(a: f64, e: f64, tolerance: f64, context: &str) {
  assert!(
    a.is_finite() && e.is_finite() && (a - e).abs() <= tolerance,
    "{context}: {a} != {e}"
  );
}
fn validate(input: &str, refits: &str, summary: &str, expect_boundary: bool) {
  let d = TestDir::new();
  fs::write(d.path().join("MOM.csv"), input).unwrap();
  let output = run(&d, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  assert!(
    events
      .iter()
      .any(|e| e["result_type"] == "RandomEffectsResult"
        && e["data"]["estimator"] == "balanced_swamy_arora_random_effects_gls")
  );
  let bytes: Vec<_> = OUTPUTS
    .iter()
    .map(|f| fs::read(d.path().join(f)).unwrap())
    .collect();
  let actual_summary = records(std::str::from_utf8(&bytes[0]).unwrap());
  let expected_summary = records(summary);
  assert_eq!(actual_summary.len(), 1);
  assert_eq!(expected_summary.len(), 1);
  let table = &events
    .iter()
    .find(|e| e["result_type"] == "TableResult" && e["data"]["headers"][0] == "parameter")
    .unwrap()["data"];
  assert_eq!(
    table["headers"],
    serde_json::json!([
      "parameter",
      "estimate",
      "bootstrap_se",
      "percentile95_lower",
      "percentile95_upper"
    ])
  );
  assert_eq!(table["rows"].as_array().unwrap().len(), 2);
  for (i, name, suffix) in [(0, "_cons", "intercept"), (1, "lnwg", "slope")] {
    assert_eq!(table["rows"][i][0], name);
    for (j, prefix) in [(1, "estimate"), (2, "se"), (3, "lower"), (4, "upper")] {
      close(
        table["rows"][i][j].as_f64().unwrap(),
        number(&expected_summary[0], &format!("{prefix}_{suffix}")),
        1e-9,
        "readable/structured bootstrap summary",
      );
    }
  }
  for (k, e) in &expected_summary[0] {
    close(number(&actual_summary[0], k), e.parse().unwrap(), 1e-9, k);
  }
  assert_eq!(
    actual_summary[0]["rng"],
    "xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1"
  );
  let fits = records(refits);
  assert_eq!(fits.len(), 501);
  let native = &events
    .iter()
    .find(|e| e["result_type"] == "RandomEffectsResult")
    .unwrap()["data"];
  for key in [
    "sigma_e_squared",
    "sigma_u_squared",
    "raw_sigma_u_squared",
    "theta",
    "transformed_rss",
  ] {
    close(
      native[key].as_f64().unwrap(),
      number(&fits[0], key),
      1e-9,
      key,
    );
  }
  let draws = records(std::str::from_utf8(&bytes[1]).unwrap());
  assert_eq!(draws.len(), 8000);
  let mut boundary_count = 0;
  let mut repeated_copies = 0;
  for rep in 1..=500 {
    let block = &draws[(rep - 1) * 16..rep * 16];
    let mut people = std::collections::BTreeSet::new();
    for (position, r) in block.iter().enumerate() {
      assert_eq!(number(r, "replicate"), rep as f64);
      assert_eq!(number(r, "copy_id"), (position + 1) as f64);
      assert_eq!(number(r, "step"), (position + 1) as f64);
      let id = number(r, "person_id");
      assert!(id.fract() == 0.0 && (1.0..=16.0).contains(&id));
      if !people.insert(id as usize) {
        repeated_copies += 1;
      }
      for key in [
        "intercept",
        "slope",
        "sigma_e_squared",
        "sigma_u_squared",
        "raw_sigma_u_squared",
        "theta",
        "within_slope",
        "between_slope",
        "transformed_rss",
      ] {
        close(number(r, key), number(&fits[rep], key), 1e-9, key);
      }
    }
    let r = &block[0];
    if number(r, "raw_sigma_u_squared") <= 0.0 {
      boundary_count += 1;
      assert_eq!(number(r, "sigma_u_squared"), 0.0);
      assert_eq!(number(r, "theta"), 0.0);
    } else {
      assert_eq!(
        number(r, "sigma_u_squared"),
        number(r, "raw_sigma_u_squared")
      );
      assert!(number(r, "theta") > 0.0 && number(r, "theta") < 1.0);
    }
    assert!(number(r, "sigma_e_squared") > 1e-12);
    assert!(number(r, "transformed_rss") > 1e-12);
  }
  assert!(repeated_copies > 0);
  if expect_boundary {
    assert!(boundary_count > 0);
  } else {
    assert_eq!(boundary_count, 0);
    let minimum = fits
      .iter()
      .skip(1)
      .map(|r| number(r, "theta"))
      .fold(f64::INFINITY, f64::min);
    let maximum = fits
      .iter()
      .skip(1)
      .map(|r| number(r, "theta"))
      .fold(f64::NEG_INFINITY, f64::max);
    assert!(
      maximum - minimum > 1e-4,
      "theta must be refitted, not fixed across draws"
    );
  }
  assert!(
    number(&actual_summary[0], "estimate_intercept") > 6.0,
    "population-scale intercept, not quasi-intercept"
  );
  let source = records(input);
  let predictions = records(std::str::from_utf8(&bytes[2]).unwrap());
  assert_eq!(predictions.len(), source.len());
  for (r, s) in predictions.iter().zip(&source) {
    for k in s.keys() {
      assert_eq!(number(r, k), number(s, k), "source {k}");
    }
    let xb = number(&fits[0], "intercept") + number(&fits[0], "slope") * number(s, "lnwg");
    close(number(r, "regls_xb"), xb, 1e-9, "retained original xb");
    close(
      number(r, "regls_residual"),
      number(s, "lnhr") - xb,
      1e-9,
      "retained residual",
    );
  }
  for f in OUTPUTS {
    fs::remove_file(d.path().join(f)).unwrap();
  }
  let text = run(&d, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(String::from_utf8_lossy(&text.stdout).contains("bootstrap_se"));
  assert!(String::from_utf8_lossy(&text.stdout).contains("percentile95_lower"));
  for (i, f) in OUTPUTS.iter().enumerate() {
    assert_eq!(fs::read(d.path().join(f)).unwrap(), bytes[i]);
  }
}
#[test]
fn interior_whole_person_refits_match_all_independent_r_fits_and_text_exports() {
  validate(
    INTERIOR,
    include_str!("fixtures/mma21_regls_refit_bootstrap/interior_refits.csv"),
    include_str!("fixtures/mma21_regls_refit_bootstrap/interior_summary.csv"),
    false,
  );
}
#[test]
fn zero_unit_variance_refits_are_retained_not_dropped_and_match_r() {
  validate(
    BOUNDARY,
    include_str!("fixtures/mma21_regls_refit_bootstrap/boundary_refits.csv"),
    include_str!("fixtures/mma21_regls_refit_bootstrap/boundary_summary.csv"),
    true,
  );
}
#[test]
fn malformed_incomplete_and_failed_replicate_publish_no_exports() {
  let lines: Vec<_> = INTERIOR.lines().collect();
  let mut missing_row: Vec<_> = lines[1].split(',').collect();
  missing_row[1] = "";
  let missing = INTERIOR.replacen(lines[1], &missing_row.join(","), 1);
  let inputs = [
    format!("{INTERIOR}{}\n", lines[1]),
    lines[..lines.len() - 1].join("\n"),
    format!("{}\n", lines[0]),
    missing,
    include_str!("fixtures/mma21_reml_bootstrap/rank_failure_inputs.csv").to_string(),
  ];
  for (index, input) in inputs.iter().enumerate() {
    let d = TestDir::new();
    fs::write(d.path().join("MOM.csv"), input).unwrap();
    let before = fs::read(d.path().join("MOM.csv")).unwrap();
    let output = run(&d, "--json");
    assert!(
      !output.status.success(),
      "failure case {index} unexpectedly succeeded"
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fs::read(d.path().join("MOM.csv")).unwrap(), before);
    for f in OUTPUTS {
      assert!(!d.path().join(f).exists());
    }
    if index == 4 {
      assert!(
        String::from_utf8_lossy(&output.stdout).contains("RandomEffectsResult"),
        "original fit should succeed before failed resampling"
      );
      assert!(String::from_utf8_lossy(&output.stderr).contains(":10:"));
    }
  }
}
#[test]
fn each_output_collision_preserves_existing_bytes_and_stops_later_exports() {
  for (stage, target) in OUTPUTS.iter().enumerate() {
    let d = TestDir::new();
    fs::write(d.path().join("MOM.csv"), INTERIOR).unwrap();
    fs::write(d.path().join(target), b"sentinel\n").unwrap();
    let output = run(&d, "--json");
    assert!(!output.status.success());
    assert_eq!(fs::read(d.path().join(target)).unwrap(), b"sentinel\n");
    for (i, f) in OUTPUTS.iter().enumerate() {
      if i < stage {
        assert!(d.path().join(f).exists());
      }
      if i > stage {
        assert!(!d.path().join(f).exists());
      }
    }
  }
}
