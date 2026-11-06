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
fn invalid_keys_original_rank_and_singular_resamples_fail_before_exports() {
  let input = INPUT.lines().collect::<Vec<_>>();
  let mut missing = input[1].split(',').collect::<Vec<_>>();
  let last = missing.len() - 1;
  missing[last] = "";
  let mut out_of_range = input[1].split(',').collect::<Vec<_>>();
  out_of_range[last] = "195";
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
    (failing_rank_input(true), false),
    (failing_rank_input(false), true),
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
  let d = TestDir::new();
  let source = d.path().join("vietnam.csv");
  let input = failing_rank_input(false);
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
