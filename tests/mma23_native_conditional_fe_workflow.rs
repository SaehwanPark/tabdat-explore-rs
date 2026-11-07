mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
  process::{Command, Output},
};
use tabdat_domain::EstimationState;
use tabdat_language::{parse_command, script::parse_script};
use tabdat_runtime::Session;
const SCRIPT: &str = include_str!("../examples/mma23p1pannonlin/native_conditional_fe.td");
const EXPORT: &str = "native_conditional_fe_predictions.csv";
const RAW: &str = include_str!("fixtures/mma23_conditional_fe/source.asc");
fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn prepare(dir: &TestDir, raw: &str) {
  fs::write(dir.path().join("source.asc"), raw).unwrap();
  let out = Command::new("sh")
    .arg(root().join("examples/mma23p1pannonlin/prepare.sh"))
    .arg(dir.path().join("source.asc"))
    .arg(dir.path().join("patents.csv"))
    .output()
    .unwrap();
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
}
fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(root().join("examples/mma23p1pannonlin/native_conditional_fe.td"))
    .output()
    .unwrap()
}
fn events(out: &Output) -> Vec<Value> {
  String::from_utf8_lossy(&out.stdout)
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let keys: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let cells: Vec<_> = line.split(',').collect();
      assert_eq!(keys.len(), cells.len());
      keys.iter().copied().zip(cells).collect()
    })
    .collect()
}
fn close(a: f64, b: f64, label: &str) {
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9 + 1e-10 * b.abs(),
    "{label}: {a} vs {b}"
  );
}
fn check(dir: &TestDir, out: &Output) {
  assert!(
    out.status.success(),
    "{}\n{}",
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  );
  let es = events(out);
  assert_eq!(
    es.iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "IsidResult",
      "TableResult",
      "TableResult",
      "SqlCreateResult",
      "ConditionalPoissonResult",
      "PredictionResult",
      "PredictionResult",
      "PredictionResult",
      "SummarizeResult",
      "TableResult",
      "ExportResult"
    ]
  );
  assert!(
    es.iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let d = &es[5]["data"];
  let c = &d["coefficients"][0];
  for (key, value) in [
    ("observation_count", 50),
    ("joint_finite_observation_count", 60),
    ("candidate_group_count", 12),
    ("group_count", 10),
    ("zero_total_group_count", 2),
    ("zero_total_observation_count", 10),
    ("minimum_group_size", 5),
    ("maximum_group_size", 5),
  ] {
    assert_eq!(d[key], value);
  }
  for (key, value) in [
    ("estimator", "conditional_poisson_fe"),
    (
      "likelihood",
      "conditional_multinomial_given_selected_group_totals",
    ),
    ("covariance", "inverse_conditional_information"),
    ("reference_distribution", "asymptotic_standard_normal"),
    (
      "prediction_scope",
      "retained_positive_group_normalization_not_population_forecast",
    ),
  ] {
    assert_eq!(d[key], value);
  }
  assert_eq!(d["include_intercept"], false);
  assert_eq!(d["sample"]["total_observations"], 60);
  assert_eq!(d["sample"]["dropped_observations"], 10);
  assert_eq!(d["covariance_correction"], 1.0);
  assert_eq!(d["confidence_level"], 95);
  assert_eq!(d["converged"], true);
  assert_eq!(d["coefficients"].as_array().unwrap().len(), 1);
  assert_eq!(d["covariance_matrix"].as_array().unwrap().len(), 1);
  assert_eq!(d["covariance_matrix"][0].as_array().unwrap().len(), 1);
  assert_eq!(d["predictors"], serde_json::json!(["logr"]));
  assert_eq!(d["covariance_parameter_names"], serde_json::json!(["logr"]));
  assert_eq!(c["name"], "logr");
  for key in ["max_abs_scaled_score", "max_abs_scaled_newton_step"] {
    let v = d[key].as_f64().unwrap();
    assert!((0.0..=1e-10).contains(&v));
  }
  let actual = BTreeMap::from([
    ("candidate_rows", 60.0),
    ("candidate_firms", 12.0),
    ("retained_rows", 50.0),
    ("retained_firms", 10.0),
    ("excluded_rows", 10.0),
    ("excluded_firms", 2.0),
    ("beta", c["value"].as_f64().unwrap()),
    ("se", c["standard_error"].as_f64().unwrap()),
    ("z", c["statistic"].as_f64().unwrap()),
    ("p_value", c["p_value"].as_f64().unwrap()),
    ("ci_lower", c["confidence_lower"].as_f64().unwrap()),
    ("ci_upper", c["confidence_upper"].as_f64().unwrap()),
    (
      "conditional_ll",
      d["conditional_log_likelihood"].as_f64().unwrap(),
    ),
    (
      "information",
      1.0 / d["covariance_matrix"][0][0].as_f64().unwrap(),
    ),
  ]);
  let reference = include_str!("fixtures/mma23_conditional_fe/reference.csv");
  assert_eq!(rows(reference).len(), 14);
  for row in rows(reference) {
    let a = actual[row["field"]];
    let b: f64 = row["value"].parse().unwrap();
    if row["field"] == "p_value" {
      assert!((a - b).abs() <= 1e-8 * b);
    } else {
      close(a, b, row["field"]);
    }
  }
  let source = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let source_rows = rows(&source);
  let expected: BTreeMap<_, _> = source_rows
    .iter()
    .map(|r| {
      (
        (
          r["id"].parse::<u64>().unwrap(),
          r["year"].parse::<u64>().unwrap(),
        ),
        r,
      )
    })
    .collect();
  assert_eq!(expected.len(), 60);
  let export = fs::read_to_string(dir.path().join(EXPORT)).unwrap();
  let exported = rows(&export);
  let oracle = rows(include_str!(
    "fixtures/mma23_conditional_fe/predictions.csv"
  ));
  assert_eq!(exported.len(), 60);
  assert_eq!(oracle.len(), 60);
  assert_eq!(
    export.lines().next().unwrap(),
    format!(
      "{},selected_firm_total,included_in_native_fe,native_fe_mean,native_fe_xb,native_fe_residual",
      source.lines().next().unwrap()
    )
  );
  let mut fitted_totals = BTreeMap::<u64, f64>::new();
  let mut null = 0;
  for ((row, reference), (key, input)) in exported.iter().zip(&oracle).zip(&expected) {
    assert_eq!(
      (
        row["id"].parse::<u64>().unwrap(),
        row["year"].parse::<u64>().unwrap()
      ),
      *key
    );
    for field in [
      "id", "year", "cusip", "ardssic", "scisect", "logk", "sumpat", "logr", "pat",
    ] {
      if input[field].is_empty() {
        assert!(row[field].is_empty());
      } else {
        close(
          row[field].parse().unwrap(),
          input[field].parse().unwrap(),
          field,
        );
      }
    }
    let included = reference["included_in_fe"] == "TRUE";
    assert_eq!(
      row["included_in_native_fe"],
      if included { "true" } else { "false" }
    );
    close(
      row["native_fe_xb"].parse().unwrap(),
      actual["beta"] * row["logr"].parse::<f64>().unwrap(),
      "slope-only index",
    );
    for (field, ref_field) in [
      ("native_fe_mean", "fe_mean"),
      ("native_fe_residual", "fe_residual"),
    ] {
      if included {
        close(
          row[field].parse().unwrap(),
          reference[ref_field].parse().unwrap(),
          field,
        );
      } else {
        assert_eq!(row[field], "");
        assert_eq!(reference[ref_field], "");
      }
    }
    if included {
      let mu: f64 = row["native_fe_mean"].parse().unwrap();
      *fitted_totals.entry(key.0).or_default() += mu;
      close(
        row["native_fe_residual"].parse().unwrap(),
        row["pat"].parse::<f64>().unwrap() - mu,
        "residual",
      );
    } else {
      null += 1;
    }
  }
  assert_eq!(null, 10);
  assert_eq!(fitted_totals.len(), 10);
  for (id, total) in fitted_totals {
    let expected: f64 = source_rows
      .iter()
      .filter(|r| r["id"].parse::<u64>().unwrap() == id)
      .map(|r| r["pat"].parse::<f64>().unwrap())
      .sum();
    close(total, expected, "firm total");
  }
  assert_eq!(es[9]["data"]["rows"][0]["count"], 50);
  assert_eq!(es[9]["data"]["rows"][1]["count"], 50);
  assert_eq!(es[10]["data"]["rows"].as_array().unwrap().len(), 1);
}
#[test]
fn r_reference_every_source_prediction_and_text_json_exports() {
  let json = TestDir::new();
  prepare(&json, RAW);
  check(&json, &run(&json, "--json"));
  let text = TestDir::new();
  prepare(&text, RAW);
  let out = run(&text, "--text");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let stdout = String::from_utf8_lossy(&out.stdout);
  assert!(stdout.contains("Conditional") && stdout.contains("logr"));
  assert_eq!(
    fs::read(json.path().join(EXPORT)).unwrap(),
    fs::read(text.path().join(EXPORT)).unwrap()
  );
}
#[test]
fn permutation_and_missing_nonmodel_metadata_preserve_selection() {
  let base = TestDir::new();
  prepare(&base, RAW);
  check(&base, &run(&base, "--json"));
  let dir = TestDir::new();
  prepare(&dir, RAW);
  let source = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let mut lines = source.lines();
  let header = lines.next().unwrap();
  let mut rest: Vec<_> = lines.collect();
  rest.reverse();
  fs::write(
    dir.path().join("patents.csv"),
    format!("{header}\n{}\n", rest.join("\n")),
  )
  .unwrap();
  check(&dir, &run(&dir, "--json"));
  assert_eq!(
    fs::read(dir.path().join(EXPORT)).unwrap(),
    fs::read(base.path().join(EXPORT)).unwrap()
  );
  let mut tokens: Vec<_> = RAW.split_whitespace().map(str::to_owned).collect();
  for i in (1..tokens.len()).step_by(25) {
    tokens[i] = ".".into();
  }
  let dir = TestDir::new();
  prepare(&dir, &tokens.join(" "));
  check(&dir, &run(&dir, "--json"));
}
#[test]
fn invalid_and_unidentified_sources_fail_before_result_prediction_export() {
  let base = TestDir::new();
  prepare(&base, RAW);
  let source = fs::read_to_string(base.path().join("patents.csv")).unwrap();
  let lines: Vec<_> = source.lines().collect();
  let mutations = [
    (0, "0"),
    (0, "347"),
    (0, "1.5"),
    (0, ""),
    (1, "1974"),
    (1, "1980"),
    (1, "1975.5"),
    (1, ""),
    (8, "-1"),
    (8, "1001"),
    (8, ".5"),
    (8, "NaN"),
    (8, ""),
    (7, "21"),
    (7, "inf"),
    (7, ""),
    (2, ""),
    (3, "garbage"),
    (3, "inf"),
    (4, ""),
    (5, ""),
    (6, ""),
  ];
  let mut cases = Vec::new();
  for (column, value) in mutations {
    let mut record: Vec<_> = lines[1].split(',').map(str::to_owned).collect();
    record[column] = value.into();
    cases.push(format!(
      "{}\n{}\n{}\n",
      lines[0],
      record.join(","),
      lines[2..].join("\n")
    ));
  }
  for case in 0..5 {
    let mut records = lines[1..]
      .iter()
      .map(|r| r.split(',').map(str::to_owned).collect::<Vec<_>>())
      .collect::<Vec<_>>();
    match case {
      0 => {
        records.pop();
      }
      1 => {
        for row in &mut records {
          row[8] = "0".into();
        }
      }
      2 => {
        for row in &mut records {
          row[7] = "1".into();
        }
      }
      3 => {
        for row in &mut records {
          if row[0] != "2" {
            row[8] = "0".into();
          }
        }
      }
      _ => {
        records.push(records[0].clone());
      }
    }
    cases.push(format!(
      "{}\n{}\n",
      lines[0],
      records
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
    ));
  }
  assert_eq!(cases.len(), 27);
  for (i, input) in cases.iter().enumerate() {
    let dir = TestDir::new();
    fs::write(dir.path().join("patents.csv"), input).unwrap();
    let out = run(&dir, "--json");
    assert!(!out.status.success(), "case{i}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
      !stdout.contains("\"result_type\":\"ConditionalPoissonResult\""),
      "case{i}"
    );
    assert!(!stdout.contains("\"result_type\":\"PredictionResult\""));
    assert!(!dir.path().join(EXPORT).exists());
    assert_eq!(
      fs::read_to_string(dir.path().join("patents.csv")).unwrap(),
      *input
    );
  }
}
fn execute(session: &mut Session, text: &str) {
  session.execute(parse_command(text).unwrap()).unwrap();
}
#[test]
fn actual_script_ownership_retained_subset_normalization_and_export_collision() {
  for collision in [false, true] {
    let dir = TestDir::new();
    prepare(&dir, RAW);
    let input = fs::read(dir.path().join("patents.csv")).unwrap();
    if collision {
      fs::write(dir.path().join(EXPORT), "owner").unwrap();
    }
    let mut session = Session::new();
    let mut failed = 0;
    for command in parse_script(SCRIPT, Path::new("native_conditional_fe.td")).unwrap() {
      let text = if command.text == "use patents.csv" {
        format!("use \"{}\"", dir.path().join("patents.csv").display())
      } else if command.text == format!("export {EXPORT}") {
        format!("export \"{}\"", dir.path().join(EXPORT).display())
      } else {
        command.text
      };
      let before = session.state();
      let result = session.execute(parse_command(&text).unwrap());
      if collision && text.starts_with("export ") {
        assert!(result.is_err());
        assert_eq!(session.state(), before);
        failed += 1;
      } else {
        result.unwrap();
      }
    }
    assert_eq!(failed, usize::from(collision));
    assert!(matches!(
      session.state().estimation,
      Some(EstimationState::ConditionalPoisson(_))
    ));
    assert_eq!(
      session.active_table_name(),
      Some("native_conditional_fe_cohort")
    );
    execute(&mut session, "label variable logr \"Owned numeric LOGR\"");
    let before = session.state();
    execute(&mut session, "summarize native_fe_mean native_fe_residual");
    assert_eq!(session.state(), before);
    assert!(
      session
        .execute(parse_command("predict native_fe_mean, n").unwrap())
        .is_err()
    );
    assert_eq!(session.state(), before);
    execute(&mut session, "predict repeated, n");
    assert_eq!(session.state().estimation, before.estimation);
    assert_eq!(session.state().labels, before.labels);
    execute(
      &mut session,
      "sql \"\"\"SELECT CASE WHEN COUNT(*)=60 AND COUNT(repeated)=50 AND MAX(abs(repeated-native_fe_mean))<1e-12 THEN 1 ELSE error('lost FE ownership') END AS valid FROM active\"\"\"",
    );
    execute(
      &mut session,
      "sql \"\"\"SELECT id,logr,native_fe_mean AS previous_mean,native_fe_xb AS previous_xb FROM active WHERE year=1975\"\"\" into fe_subset",
    );
    execute(&mut session, "predict retained_again, n");
    execute(
      &mut session,
      "sql \"\"\"SELECT CASE WHEN COUNT(*)=12 AND COUNT(retained_again)=10 AND MAX(abs(retained_again-previous_mean))<1e-12 THEN 1 ELSE error('current subset was renormalized') END AS valid FROM active\"\"\"",
    );
    execute(
      &mut session,
      "sql \"\"\"SELECT CAST(999 AS BIGINT) AS id,logr,previous_xb FROM active\"\"\" into unknown_firms",
    );
    execute(&mut session, "predict unknown, n");
    execute(
      &mut session,
      "sql \"\"\"SELECT CASE WHEN COUNT(*)=12 AND COUNT(unknown)=0 THEN 1 ELSE error('unknown firm prediction') END AS valid FROM active\"\"\"",
    );
    execute(
      &mut session,
      "sql \"\"\"SELECT logr,previous_xb FROM active\"\"\" into no_id",
    );
    execute(&mut session, "predict no_id_index, xb");
    execute(
      &mut session,
      "sql \"\"\"SELECT CASE WHEN COUNT(*)=12 AND COUNT(no_id_index)=12 AND MAX(abs(no_id_index-previous_xb))<1e-12 THEN 1 ELSE error('xb requires no current ID') END AS valid FROM active\"\"\"",
    );
    assert_eq!(session.state().estimation, before.estimation);
    let prior = session.state();
    assert!(
      session
        .execute(parse_command("predict no_id_mean, n").unwrap())
        .is_err()
    );
    assert_eq!(session.state(), prior);
    assert!(
      session
        .execute(parse_command("xtpoisson pat logr, fe id(id)").unwrap())
        .is_err()
    );
    let mut cleared = prior;
    cleared.estimation = None;
    assert_eq!(session.state(), cleared);
    if collision {
      assert_eq!(
        fs::read_to_string(dir.path().join(EXPORT)).unwrap(),
        "owner"
      );
    }
    assert_eq!(fs::read(dir.path().join("patents.csv")).unwrap(), input);
  }
}
#[test]
fn prediction_column_owner_is_not_overwritten() {
  let dir = TestDir::new();
  prepare(&dir, RAW);
  let source = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let mut lines = source.lines();
  let input = format!(
    "{},native_fe_mean\n{}\n",
    lines.next().unwrap(),
    lines
      .map(|l| format!("{l},42"))
      .collect::<Vec<_>>()
      .join("\n")
  );
  fs::write(dir.path().join("patents.csv"), &input).unwrap();
  let out = run(&dir, "--json");
  assert!(!out.status.success());
  assert!(!dir.path().join(EXPORT).exists());
  assert_eq!(
    fs::read_to_string(dir.path().join("patents.csv")).unwrap(),
    input
  );
  let es = events(&out);
  assert!(
    es.iter()
      .any(|e| e["result_type"] == "ConditionalPoissonResult")
  );
  assert!(!es.iter().any(|e| e["result_type"] == "PredictionResult"));
}
