mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  collections::{BTreeMap, BTreeSet},
  fs,
  path::PathBuf,
};

const OUTPUTS: [&str; 3] = [
  "panel_fe_level_bootstrap_summary.csv",
  "panel_fe_level_bootstrap_draws.csv",
  "panel_fe_level_bootstrap_predictions.csv",
];
fn script() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma21p1panel/fe_level_bootstrap.td")
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(script())
    .output()
    .unwrap()
}
fn fixture(kind: &str, name: &str) -> String {
  fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
    "tests/fixtures/mma21_fe_level_bootstrap/{kind}_{name}.csv"
  )))
  .unwrap()
}
type Row = BTreeMap<String, String>;
fn records(text: &str) -> Vec<Row> {
  let mut lines = text.lines();
  let headers: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|l| {
      let values: Vec<_> = l.split(',').collect();
      assert_eq!(values.len(), headers.len());
      headers
        .iter()
        .zip(values)
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    })
    .collect()
}
fn number(r: &Row, k: &str) -> f64 {
  r[k].parse().unwrap()
}
fn close(a: f64, b: f64, context: &str) {
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9,
    "{context}: {a} != {b}"
  );
}
fn validate(kind: &str, zero: bool) {
  let d = TestDir::new();
  let input = fixture(kind, "inputs");
  fs::write(d.path().join("MOM.csv"), &input).unwrap();
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
  let original: Vec<_> = OUTPUTS
    .iter()
    .map(|f| fs::read(d.path().join(f)).unwrap())
    .collect();
  let summary = records(std::str::from_utf8(&original[0]).unwrap());
  let reference = records(&fixture(kind, "summary"));
  assert_eq!(summary.len(), 1);
  assert_eq!(reference.len(), 1);
  assert_eq!(summary[0].len(), reference[0].len() + 4);
  for (k, v) in &reference[0] {
    close(number(&summary[0], k), v.parse().unwrap(), k);
  }
  assert_eq!(summary[0]["base_seed"], "10001");
  assert_eq!(
    summary[0]["rng"],
    "xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1"
  );
  assert_eq!(
    summary[0]["method"],
    "whole_person_pairs_refit_normalized_level_fe"
  );
  assert_eq!(summary[0]["interval_method"], "type7_percentile_95");
  assert!(
    number(&summary[0], "estimate_intercept") > 6.0,
    "level intercept, not the centered auxiliary OLS intercept"
  );
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
        number(&reference[0], &format!("{prefix}_{suffix}")),
        "typed inference row",
      );
    }
  }
  let fits = records(&fixture(kind, "refits"));
  assert_eq!(fits.len(), 501);
  let native = &events
    .iter()
    .find(|e| e["result_type"] == "FixedEffectsResult")
    .unwrap()["data"];
  assert_eq!(native["estimator"], "within_fixed_effects");
  assert_eq!(native["common_intercept_inference"], "classical_student_t");
  assert_eq!(
    native["unit_effect_normalization"],
    "estimation_observation_weighted_zero_mean"
  );
  assert_eq!(native["df_residual"], 143);
  close(
    native["common_intercept"].as_f64().unwrap(),
    number(&fits[0], "intercept"),
    "original normalized common intercept",
  );
  close(
    native["residual_sum_of_squares"].as_f64().unwrap(),
    number(&fits[0], "within_rss"),
    "original FE RSS",
  );
  let draws = records(std::str::from_utf8(&original[1]).unwrap());
  assert_eq!(draws.len(), 8000);
  let mut repeats = 0;
  for rep in 1..=500 {
    let block = &draws[(rep - 1) * 16..rep * 16];
    let mut people = BTreeSet::new();
    for (position, r) in block.iter().enumerate() {
      assert_eq!(r.len(), 15);
      assert_eq!(number(r, "replicate"), rep as f64);
      assert_eq!(number(r, "copy_id"), (position + 1) as f64);
      assert_eq!(number(r, "step"), (position + 1) as f64);
      assert_eq!(number(r, "people_per_draw"), 16.0);
      assert_eq!(number(r, "base_seed"), 10001.0);
      let id = number(r, "person_id");
      assert!(id.fract() == 0.0 && (1.0..=16.0).contains(&id));
      if !people.insert(id as usize) {
        repeats += 1;
      }
      for k in [
        "intercept",
        "slope",
        "sample_mean_lnwg",
        "sample_mean_lnhr",
        "residual_df",
        "within_rss",
        "normalized_effect_mean",
      ] {
        close(number(r, k), number(&fits[rep], k), k);
      }
      close(
        number(r, "estimate_intercept"),
        number(&fits[0], "intercept"),
        "original intercept retained",
      );
      close(
        number(r, "estimate_slope"),
        number(&fits[0], "slope"),
        "original slope retained",
      );
      close(
        number(r, "intercept"),
        number(r, "sample_mean_lnhr") - number(r, "slope") * number(r, "sample_mean_lnwg"),
        "per-draw level normalization",
      );
      assert_eq!(number(r, "residual_df"), 143.0);
      assert!(number(r, "within_rss") >= 0.0);
    }
  }
  assert!(repeats > 0);
  assert!(number(&summary[0], "se_intercept") > 0.0);
  if zero {
    for k in ["se_slope", "covariance_ab", "covariance_bb"] {
      assert_eq!(number(&summary[0], k), 0.0);
    }
    assert_eq!(number(&summary[0], "lower_slope"), 0.25);
    assert_eq!(number(&summary[0], "upper_slope"), 0.25);
  } else {
    assert!(number(&summary[0], "se_slope") > 0.01);
  }
  let source = records(&input);
  let predictions = records(std::str::from_utf8(&original[2]).unwrap());
  assert_eq!(predictions.len(), source.len());
  let mut unit_means = BTreeMap::<usize, (f64, f64, usize)>::new();
  for r in &source {
    let u = unit_means.entry(number(r, "id") as usize).or_default();
    u.0 += number(r, "lnhr");
    u.1 += number(r, "lnwg");
    u.2 += 1;
  }
  let a = number(&fits[0], "intercept");
  let b = number(&fits[0], "slope");
  for (r, s) in predictions.iter().zip(&source) {
    assert_eq!(r.len(), s.len() + 4);
    for k in s.keys() {
      assert_eq!(number(r, k), number(s, k), "source {k}");
    }
    let (my, mx, n) = unit_means[&(number(s, "id") as usize)];
    let alpha = (my - b * mx) / n as f64;
    close(
      number(r, "fe_xb"),
      a + b * number(s, "lnwg"),
      "retained original FE xb",
    );
    close(
      number(r, "fe_xbu"),
      alpha + b * number(s, "lnwg"),
      "retained original unit conditional mean",
    );
    close(
      number(r, "fe_u"),
      alpha - a,
      "retained normalized unit effect",
    );
    close(
      number(r, "fe_residual"),
      number(s, "lnhr") - number(r, "fe_xbu"),
      "retained within residual",
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
  let stdout = String::from_utf8(text.stdout).unwrap();
  assert!(stdout.contains("bootstrap_se") && stdout.contains("percentile95_lower"));
  for (f, bytes) in OUTPUTS.iter().zip(original) {
    assert_eq!(fs::read(d.path().join(f)).unwrap(), bytes);
  }
}
#[test]
fn heterogeneous_fe_refits_renormalize_level_intercept_and_preserve_native_state() {
  validate("heterogeneous", false);
}
#[test]
fn zero_slope_bootstrap_variance_is_retained_with_singular_covariance_and_point_interval() {
  validate("zero_variance", true);
}
#[test]
fn fe_level_bootstrap_rejects_bad_panels_and_later_unidentified_draw_without_exports() {
  let input = fixture("heterogeneous", "inputs");
  let lines: Vec<_> = input.lines().collect();
  let mut missing: Vec<_> = lines[1].split(',').collect();
  missing[1] = "";
  let mut bounded: Vec<_> = lines[1].split(',').collect();
  bounded[2] = "21";
  let cases = [
    (format!("{}\n", lines[0]), false),
    (format!("{input}{}\n", lines[1]), false),
    (format!("{}\n", lines[..lines.len() - 1].join("\n")), false),
    (
      format!(
        "{}\n{}\n{}\n",
        lines[0],
        missing.join(","),
        lines[2..].join("\n")
      ),
      false,
    ),
    (
      format!(
        "{}\n{}\n{}\n",
        lines[0],
        bounded.join(","),
        lines[2..].join("\n")
      ),
      false,
    ),
    (fixture("rank_failure", "inputs"), true),
  ];
  for (input, started) in cases {
    let d = TestDir::new();
    fs::write(d.path().join("MOM.csv"), &input).unwrap();
    let output = run(&d, "--json");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
      String::from_utf8_lossy(&output.stdout).contains("FixedEffectsResult"),
      started
    );
    for f in OUTPUTS {
      assert!(!d.path().join(f).exists());
    }
    assert_eq!(fs::read_to_string(d.path().join("MOM.csv")).unwrap(), input);
  }
}
#[test]
fn fe_level_bootstrap_each_export_collision_preserves_owner_and_stops_later_commands() {
  for (stage, target) in OUTPUTS.iter().enumerate() {
    let d = TestDir::new();
    let input = fixture("heterogeneous", "inputs");
    fs::write(d.path().join("MOM.csv"), &input).unwrap();
    fs::write(d.path().join(target), "existing owner\n").unwrap();
    let output = run(&d, "--json");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
      fs::read_to_string(d.path().join(target)).unwrap(),
      "existing owner\n"
    );
    for (i, f) in OUTPUTS.iter().enumerate() {
      assert_eq!(d.path().join(f).exists(), i <= stage);
    }
    assert_eq!(fs::read_to_string(d.path().join("MOM.csv")).unwrap(), input);
  }
}
