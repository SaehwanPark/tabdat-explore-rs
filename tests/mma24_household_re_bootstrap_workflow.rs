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

const INPUT: &str = include_str!("fixtures/mma24_household_re_bootstrap/mixed_input.csv");
const FITS: &str = include_str!("fixtures/mma24_household_re_bootstrap/mixed_fits.csv");
const SUMMARY: &str = include_str!("fixtures/mma24_household_re_bootstrap/mixed_summary.csv");
const PREDICTIONS: &str =
  include_str!("fixtures/mma24_household_re_bootstrap/mixed_predictions.csv");
const GROUPS: &str = include_str!("fixtures/mma24_household_re_bootstrap/mixed_groups.csv");
const DRAWS_HEAD: &str = include_str!("fixtures/mma24_household_re_bootstrap/draws_head.csv");
const SCRIPT: &str = include_str!("../examples/mma24p1olscluster/household_re_bootstrap.td");
const OUTPUTS: [&str; 4] = [
  "household_re_bootstrap_summary.csv",
  "household_re_bootstrap_fits.csv",
  "household_re_bootstrap_draws.csv",
  "household_re_bootstrap_predictions.csv",
];

fn script() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma24p1olscluster/household_re_bootstrap.td")
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(script())
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let header = lines
    .next()
    .unwrap()
    .split(',')
    .map(|s| s.trim_matches('"'))
    .collect::<Vec<_>>();
  lines
    .map(|line| {
      let fields = line
        .split(',')
        .map(|s| s.trim_matches('"'))
        .collect::<Vec<_>>();
      assert_eq!(header.len(), fields.len());
      header.iter().copied().zip(fields).collect()
    })
    .collect()
}
fn near(a: &str, b: &str, key: &str) {
  let a = a.parse::<f64>().unwrap();
  let b = b.parse::<f64>().unwrap();
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9,
    "{key}: {a} != {b}"
  );
}
fn checked(out: std::process::Output) -> Vec<Value> {
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let events = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|x| serde_json::from_str::<Value>(x).unwrap())
    .collect::<Vec<_>>();
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  events
}
#[test]
fn actual_unbalanced_script_matches_all_r_refits_and_preserves_original_predictions() {
  let d = TestDir::new();
  fs::write(d.path().join("vietnam.csv"), INPUT).unwrap();
  let events = checked(run(&d, "--json"));
  let summary_text = fs::read_to_string(d.path().join(OUTPUTS[0])).unwrap();
  let summary = rows(&summary_text);
  let reference = rows(SUMMARY);
  assert_eq!(summary.len(), 49);
  assert_eq!(reference.len(), 49);
  let numeric = [
    "parameter_index",
    "covariance_index",
    "estimate",
    "bootstrap_mean",
    "bias",
    "standard_error",
    "ci_lower",
    "ci_upper",
    "covariance",
    "replications",
    "communes_per_draw",
    "base_seed",
  ];
  for (a, b) in summary.iter().zip(reference.iter()) {
    assert_eq!(a.len(), b.len());
    for (k, v) in b {
      if numeric.contains(k) {
        near(a[k], v, k)
      } else {
        assert_eq!(a[k], *v, "{k}")
      }
    }
  }
  assert_eq!(summary[0]["parameter"], "_cons");
  assert_eq!(
    summary[0]["method"],
    "whole_commune_pairs_refit_equal_group_re_gls"
  );
  let table = &events
    .iter()
    .find(|e| e["result_type"] == "TableResult" && e["data"]["headers"][0] == "parameter")
    .unwrap()["data"];
  assert_eq!(table["rows"].as_array().unwrap().len(), 7);
  let native = &events
    .iter()
    .find(|e| e["result_type"] == "UnbalancedRandomEffectsGlsResult")
    .unwrap()["data"];
  assert_eq!(native["observation_count"], 169);
  assert_eq!(native["group_count"], 48);
  assert_eq!(native["variance_component_method"], "equal_group_between");
  for (i, c) in native["coefficients"]
    .as_array()
    .unwrap()
    .iter()
    .enumerate()
  {
    near(
      &c["value"].as_f64().unwrap().to_string(),
      summary[i * 7]["estimate"],
      "native coefficient",
    );
  }
  let fits_text = fs::read_to_string(d.path().join(OUTPUTS[1])).unwrap();
  let fits = rows(&fits_text);
  let expected_fits = rows(FITS);
  assert_eq!(fits.len(), 500);
  assert_eq!(expected_fits.len(), 500);
  let truncated = fits
    .iter()
    .filter(|r| r["u"].parse::<f64>().unwrap() == 0.0)
    .count();
  assert_eq!(truncated, 28);
  assert!(
    fits
      .iter()
      .any(|r| r["raw_u"].parse::<f64>().unwrap() > 0.0)
  );
  let baseline = rows(include_str!(
    "fixtures/mma24_household_re_bootstrap/mixed_baseline.csv"
  ));
  assert_eq!(baseline.len(), 1);
  for (native_key, reference_key) in [
    ("sigma_e_squared", "e"),
    ("raw_sigma_u_squared", "raw_u"),
    ("sigma_u_squared", "u"),
    ("theta_min", "theta_min"),
    ("theta_max", "theta_max"),
  ] {
    near(
      &native[native_key].as_f64().unwrap().to_string(),
      baseline[0][reference_key],
      native_key,
    );
  }
  let group_rows = rows(GROUPS);
  let group_sizes = group_rows
    .iter()
    .map(|r| {
      (
        r["commune"].parse::<usize>().unwrap(),
        r["observations"].parse::<usize>().unwrap(),
      )
    })
    .collect::<BTreeMap<_, _>>();
  assert_eq!(group_sizes.len(), 48);
  assert_eq!(*group_sizes.values().min().unwrap(), 1);
  assert!(group_sizes.values().max() > group_sizes.values().min());
  let draws_text = fs::read_to_string(d.path().join(OUTPUTS[2])).unwrap();
  let draws = rows(&draws_text);
  let head = rows(DRAWS_HEAD);
  assert_eq!(draws.len(), 24000);
  assert_eq!(&draws[..144], head);
  let mut counts = Vec::new();
  let (blocks, remainder) = draws.as_chunks::<48>();
  assert!(remainder.is_empty());
  for (i, block) in blocks.iter().enumerate() {
    let mut n = 0;
    let mut previous_step = 0;
    for (j, r) in block.iter().enumerate() {
      assert_eq!(r["replicate"].parse::<usize>().unwrap(), i + 1);
      assert_eq!(r["copy_id"].parse::<usize>().unwrap(), j + 1);
      let step = r["step"].parse::<usize>().unwrap();
      assert!(step > previous_step && step <= 1072);
      previous_step = step;
      n += group_sizes[&r["commune"].parse::<usize>().unwrap()];
    }
    counts.push(n);
  }
  assert_ne!(counts.iter().min(), counts.iter().max());
  for (i, (a, b)) in fits.iter().zip(expected_fits.iter()).enumerate() {
    assert_eq!(a["replicate"].parse::<usize>().unwrap(), i + 1);
    assert_eq!(a["sample_n"].parse::<usize>().unwrap(), counts[i]);
    for (k, v) in b {
      near(a[k], v, k)
    }
    for (j, k) in [
      "intercept",
      "lnhhexp",
      "age",
      "sex",
      "hhsize",
      "farm",
      "educ",
    ]
    .iter()
    .enumerate()
    {
      let field = format!("estimate_{k}");
      near(a[field.as_str()], summary[j * 7]["estimate"], k);
    }
  }
  let predictions_text = fs::read_to_string(d.path().join(OUTPUTS[3])).unwrap();
  let prediction = rows(&predictions_text);
  let expected = rows(PREDICTIONS);
  assert_eq!(prediction.len(), 169);
  for (a, b) in prediction.iter().zip(expected.iter()) {
    assert_eq!(a.len(), b.len());
    for (k, v) in b {
      near(a[k], v, k)
    }
  }
  assert_eq!(
    fs::read_to_string(d.path().join("vietnam.csv")).unwrap(),
    INPUT
  );
  let original = OUTPUTS.map(|f| fs::read(d.path().join(f)).unwrap());
  let text = TestDir::new();
  fs::write(text.path().join("vietnam.csv"), INPUT).unwrap();
  let out = run(&text, "--text");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  for (f, b) in OUTPUTS.iter().zip(original.iter()) {
    assert_eq!(&fs::read(text.path().join(f)).unwrap(), b);
  }
  let mut lines = INPUT.lines();
  let header = lines.next().unwrap();
  let reordered = format!(
    "{}\n{}\n",
    header,
    lines.rev().collect::<Vec<_>>().join("\n")
  );
  let reversed = TestDir::new();
  fs::write(reversed.path().join("vietnam.csv"), &reordered).unwrap();
  checked(run(&reversed, "--json"));
  for (f, b) in OUTPUTS.iter().zip(original.iter()) {
    assert_eq!(&fs::read(reversed.path().join(f)).unwrap(), b);
  }
  assert_eq!(
    fs::read_to_string(reversed.path().join("vietnam.csv")).unwrap(),
    reordered
  );
}
#[test]
fn every_zero_truncated_refit_matches_independent_r() {
  let d = TestDir::new();
  fs::write(
    d.path().join("vietnam.csv"),
    include_str!("fixtures/mma24_household_re_bootstrap/boundary_input.csv"),
  )
  .unwrap();
  checked(run(&d, "--json"));
  for (filename, reference, count) in [
    (
      OUTPUTS[0],
      include_str!("fixtures/mma24_household_re_bootstrap/boundary_summary.csv"),
      49,
    ),
    (
      OUTPUTS[1],
      include_str!("fixtures/mma24_household_re_bootstrap/boundary_fits.csv"),
      500,
    ),
    (
      OUTPUTS[3],
      include_str!("fixtures/mma24_household_re_bootstrap/boundary_predictions.csv"),
      169,
    ),
  ] {
    let text = fs::read_to_string(d.path().join(filename)).unwrap();
    let actual = rows(&text);
    let expected = rows(reference);
    assert_eq!(actual.len(), count);
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(expected.iter()) {
      if filename == OUTPUTS[1] {
        assert_eq!(a["u"].parse::<f64>().unwrap(), 0.0);
        assert!(a["raw_u"].parse::<f64>().unwrap() < 0.0);
      }
      for (key, value) in b {
        if value.parse::<f64>().is_ok() {
          near(a[key], value, key);
        } else {
          assert_eq!(a[key], *value, "{key}");
        }
      }
    }
  }
}

#[test]
fn balanced_result_matches_independent_r_and_single_theta() {
  let d = TestDir::new();
  fs::write(
    d.path().join("vietnam.csv"),
    include_str!("fixtures/mma24_household_re_bootstrap/balanced_input.csv"),
  )
  .unwrap();
  let events = checked(run(&d, "--json"));
  let result = &events
    .iter()
    .find(|e| e["result_type"] == "RandomEffectsResult")
    .unwrap()["data"];
  assert_eq!(result["observation_count"], 192);
  assert_eq!(result["group_count"], 48);
  assert_eq!(result["variance_component_method"], "equal_group_between");
  let baseline = rows(include_str!(
    "fixtures/mma24_household_re_bootstrap/balanced_baseline.csv"
  ));
  for key in ["theta_min", "theta_max"] {
    near(
      &result["theta"].as_f64().unwrap().to_string(),
      baseline[0][key],
      key,
    );
  }
  for (filename, reference, count) in [
    (
      OUTPUTS[0],
      include_str!("fixtures/mma24_household_re_bootstrap/balanced_summary.csv"),
      49,
    ),
    (
      OUTPUTS[1],
      include_str!("fixtures/mma24_household_re_bootstrap/balanced_fits.csv"),
      500,
    ),
  ] {
    let text = fs::read_to_string(d.path().join(filename)).unwrap();
    let actual = rows(&text);
    let expected = rows(reference);
    assert_eq!(actual.len(), count);
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(expected.iter()) {
      for (key, value) in b {
        if value.parse::<f64>().is_ok() {
          near(a[key], value, key)
        } else {
          assert_eq!(a[key], *value, "{key}")
        }
      }
    }
  }
}

#[test]
fn correlated_control_matches_all_independent_r_refits() {
  let d = TestDir::new();
  fs::write(
    d.path().join("vietnam.csv"),
    include_str!("fixtures/mma24_household_fe_bootstrap/conditioned_input.csv"),
  )
  .unwrap();
  checked(run(&d, "--json"));
  for (filename, reference, count) in [
    (
      OUTPUTS[0],
      include_str!("fixtures/mma24_household_re_bootstrap/conditioned_summary.csv"),
      49,
    ),
    (
      OUTPUTS[1],
      include_str!("fixtures/mma24_household_re_bootstrap/conditioned_fits.csv"),
      500,
    ),
  ] {
    let text = fs::read_to_string(d.path().join(filename)).unwrap();
    let actual = rows(&text);
    let expected = rows(reference);
    assert_eq!(actual.len(), count);
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(expected.iter()) {
      for (key, value) in b {
        if value.parse::<f64>().is_ok() {
          near(a[key], value, key);
        } else {
          assert_eq!(a[key], *value, "{key}");
        }
      }
    }
  }
}

fn numerical_stress_input(collinear: bool) -> String {
  let mut lines = INPUT.lines();
  let mut output = format!("{}\n", lines.next().unwrap());
  for line in lines {
    let mut fields = line.split(',').map(str::to_owned).collect::<Vec<_>>();
    if collinear {
      let x: f64 = fields[2].parse().unwrap();
      let educ: f64 = fields[7].parse().unwrap();
      fields[7] = format!("{:.17}", x + 0.001 * educ);
    } else if !fields[1].is_empty() {
      let values = fields[2..8]
        .iter()
        .map(|x| x.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
      let group: f64 = fields[8].parse().unwrap();
      let y: f64 = fields[1].parse().unwrap();
      let linear = values
        .iter()
        .zip([0.4, -0.2, 0.3, 0.15, -0.25, 0.1])
        .map(|(x, b)| x * b)
        .sum::<f64>();
      fields[1] = format!("{:.17}", 1.0 + 0.1 * group + linear + 1e-5 * y);
    }
    output.push_str(&fields.join(","));
    output.push('\n');
  }
  output
}

fn variance_failure_input(original_failure: bool) -> String {
  let mut lines = INPUT.lines();
  let mut output = format!("{}\n", lines.next().unwrap());
  for line in lines {
    let mut fields = line.split(',').map(str::to_owned).collect::<Vec<_>>();
    if !fields[1].is_empty() {
      let y: f64 = fields[1].parse().unwrap();
      let group: f64 = fields[8].parse().unwrap();
      let value = if original_failure {
        group
      } else {
        let linear = fields[2..8]
          .iter()
          .map(|x| x.parse::<f64>().unwrap())
          .zip([0.4, -0.2, 0.3, 0.15, -0.25, 0.1])
          .map(|(x, b)| x * b)
          .sum::<f64>();
        1.0 + 0.1 * group + linear + if group == 6.0 { y } else { 0.0 }
      };
      fields[1] = format!("{value:.17}");
    }
    output.push_str(&fields.join(","));
    output.push('\n');
  }
  output
}

fn failing_rank_input(original_rank_failure: bool) -> String {
  let mut out = String::from("source_row,lnexp12m,lnhhexp,age,sex,hhsize,farm,educ,commune\n");
  for g in 1..=8 {
    for t in 0..2 {
      let mut x = [0; 6];
      if g <= 6 && t == 1 && !(original_rank_failure && g == 6) {
        x[g - 1] = 1;
      }
      let y =
        g as f64 + 0.5 * x.iter().sum::<i32>() as f64 + if g >= 7 && t == 1 { 0.1 } else { 0.0 };
      out.push_str(&format!(
        "{},{y},{},{},{},{},{},{},{}\n",
        (g - 1) * 2 + t + 1,
        x[0],
        x[1],
        x[2],
        x[3],
        x[4],
        x[5],
        g
      ));
    }
  }
  out
}
#[test]
fn invalid_keys_rank_variance_and_numerical_failures_stop_before_exports() {
  let input = INPUT.lines().collect::<Vec<_>>();
  let mut missing = input[1].split(',').collect::<Vec<_>>();
  let last = missing.len() - 1;
  missing[last] = "";
  let mut out_of_range = input[1].split(',').collect::<Vec<_>>();
  out_of_range[last] = "195";
  let mut extreme_predictor = input[1].split(',').collect::<Vec<_>>();
  extreme_predictor[2] = "1000001";
  let mut large_group = INPUT.to_owned();
  for id in 171..210 {
    let mut fields = input[1].split(',').map(str::to_owned).collect::<Vec<_>>();
    fields[0] = id.to_string();
    large_group.push_str(&fields.join(","));
    large_group.push('\n');
  }
  let cases = [
    (
      format!("{}\n{}\n{}\n", input[0], input[1], input[1..].join("\n")),
      false,
    ),
    (
      format!(
        "{}\n{}\n{}\n",
        input[0],
        missing.join(","),
        input[2..].join("\n")
      ),
      false,
    ),
    (
      format!(
        "{}\n{}\n{}\n",
        input[0],
        out_of_range.join(","),
        input[2..].join("\n")
      ),
      false,
    ),
    (large_group, false),
    (
      format!(
        "{}\n{}\n{}\n",
        input[0],
        extreme_predictor.join(","),
        input[2..].join("\n")
      ),
      false,
    ),
    (failing_rank_input(true), false),
    (failing_rank_input(false), true),
    (variance_failure_input(true), false),
    (variance_failure_input(false), true),
    (numerical_stress_input(true), true),
    (numerical_stress_input(false), true),
  ];
  for (input, started) in cases {
    let d = TestDir::new();
    fs::write(d.path().join("vietnam.csv"), &input).unwrap();
    let out = run(&d, "--json");
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
      stdout.contains("UnbalancedRandomEffectsGlsResult")
        || stdout.contains("\"RandomEffectsResult\""),
      started,
      "{stdout}"
    );
    for f in OUTPUTS {
      assert!(!d.path().join(f).exists());
    }
    assert_eq!(
      fs::read_to_string(d.path().join("vietnam.csv")).unwrap(),
      input
    );
  }
}
#[test]
fn failed_resample_preserves_native_fit_and_current_data() {
  for input in [failing_rank_input(false), variance_failure_input(false)] {
    let d = TestDir::new();
    let source = d.path().join("vietnam.csv");
    fs::write(&source, &input).unwrap();
    let mut session = Session::new();
    let mut saw_failure = false;
    for command in parse_script(SCRIPT, Path::new("household_re_bootstrap.td")).unwrap() {
      let text = if command.text == "use vietnam.csv" {
        format!("use \"{}\"", source.display())
      } else {
        command.text.clone()
      };
      if command.text.starts_with("sql \"\"\"\nWITH RECURSIVE") {
        let before = session.state();
        let head = session.execute(parse_command("head 3").unwrap()).unwrap();
        assert!(matches!(
          before.estimation,
          Some(EstimationState::RandomEffects(_))
        ));
        assert!(session.execute(parse_command(&text).unwrap()).is_err());
        assert_eq!(session.state(), before);
        assert_eq!(
          session.execute(parse_command("head 3").unwrap()).unwrap(),
          head
        );
        session
          .execute(parse_command("predict retained_after_failure, xb").unwrap())
          .unwrap();
        assert_eq!(session.state().estimation, before.estimation);
        saw_failure = true;
        break;
      }
      session.execute(parse_command(&text).unwrap()).unwrap();
    }
    assert!(saw_failure);
    assert_eq!(fs::read_to_string(source).unwrap(), input);
    for name in OUTPUTS {
      assert!(!d.path().join(name).exists());
    }
  }
}

#[test]
fn each_collision_preserves_owner_and_stops_later_exports() {
  for (stage, name) in OUTPUTS.iter().enumerate() {
    let d = TestDir::new();
    fs::write(d.path().join("vietnam.csv"), INPUT).unwrap();
    fs::write(d.path().join(name), "owner\n").unwrap();
    let out = run(&d, "--json");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(fs::read_to_string(d.path().join(name)).unwrap(), "owner\n");
    for (i, f) in OUTPUTS.iter().enumerate() {
      assert_eq!(d.path().join(f).exists(), i <= stage);
    }
    assert_eq!(
      fs::read_to_string(d.path().join("vietnam.csv")).unwrap(),
      INPUT
    );
  }
}
#[test]
fn actual_script_retains_native_re_through_exports_and_collision() {
  let d = TestDir::new();
  let source = d.path().join("vietnam.csv");
  fs::write(&source, INPUT).unwrap();
  fs::write(d.path().join(OUTPUTS[3]), "owner").unwrap();
  let commands = parse_script(SCRIPT, Path::new("household_re_bootstrap.td")).unwrap();
  let mut session = Session::new();
  let mut retained = None;
  for command in commands {
    let text = if command.text == "use vietnam.csv" {
      format!("use \"{}\"", source.display())
    } else if let Some(name) = command.text.strip_prefix("export ") {
      format!("export \"{}\"", d.path().join(name).display())
    } else {
      command.text.clone()
    };
    let result = session.execute(parse_command(&text).unwrap());
    if command.text == format!("export {}", OUTPUTS[3]) {
      assert!(result.is_err());
    } else {
      result.unwrap_or_else(|e| panic!("line {}: {e}", command.start_line));
    }
    if command.text.starts_with("xtreg ") {
      retained = session.state().estimation;
      assert!(matches!(retained, Some(EstimationState::RandomEffects(_))));
    } else if retained.is_some() {
      assert_eq!(session.state().estimation, retained);
    }
  }
  session
    .execute(parse_command("predict still_owned, xb").unwrap())
    .unwrap();
  assert_eq!(session.state().estimation, retained);
  assert_eq!(
    fs::read_to_string(d.path().join(OUTPUTS[3])).unwrap(),
    "owner"
  );
  assert_eq!(fs::read_to_string(&source).unwrap(), INPUT);
}
