mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
  process::Command,
};
use tabdat_domain::EstimationState;
use tabdat_language::{parse_command, script::parse_script};
use tabdat_runtime::Session;
const SCRIPT: &str = include_str!("../examples/mma23p1pannonlin/native_poisson_gee.td");
const EXPORT: &str = "native_poisson_gee_predictions.csv";
const POSITIVE: &str = include_str!("fixtures/mma23_conditional_fe/source.asc");
const NEGATIVE: &str = include_str!("fixtures/mma23_exchangeable_gee/negative/source.asc");
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
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(root().join("examples/mma23p1pannonlin/native_poisson_gee.td"))
    .output()
    .unwrap()
}
fn events(out: &std::process::Output) -> Vec<Value> {
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
  let tol = 1e-9 + 1e-10 * b.abs();
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= tol,
    "{label}: {a} vs {b}"
  );
}
fn check(dir: &TestDir, out: &std::process::Output, case: &str) {
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
      "PoissonGeeResult",
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
  assert_eq!(es[10]["data"]["rows"].as_array().unwrap().len(), 1);
  let d = &es[5]["data"];
  assert_eq!(d["observation_count"], 60);
  assert_eq!(d["group_count"], 12);
  assert_eq!(d["group_size"], 5);
  assert_eq!(d["working_scale"], 1.0);
  assert_eq!(d["estimator"], "poisson_log_exchangeable_gee");
  assert_eq!(d["family"], "poisson");
  assert_eq!(d["link"], "log");
  assert_eq!(d["sample"]["dropped_observations"], 0);
  assert!(d.get("log_likelihood").is_none());
  assert_eq!(d["rho"].as_f64().unwrap() < 0.0, case == "negative");
  close(
    d["sandwich_correction"].as_f64().unwrap(),
    12.0 / 11.0,
    "correction",
  );
  let mut actual = BTreeMap::from([
    ("observations".to_owned(), 60.0),
    ("firms".into(), 12.0),
    (
      "intercept".into(),
      d["working"]["coefficients"][0]["value"].as_f64().unwrap(),
    ),
    (
      "slope".into(),
      d["working"]["coefficients"][1]["value"].as_f64().unwrap(),
    ),
    ("rho".into(), d["rho"].as_f64().unwrap()),
  ]);
  for (name, method) in [("working", "working"), ("cluster", "sandwich")] {
    let inf = &d[method];
    for (suffix, i, j) in [("aa", 0, 0), ("ab", 0, 1), ("bb", 1, 1)] {
      actual.insert(
        format!("{name}_{suffix}"),
        inf["covariance"][i][j].as_f64().unwrap(),
      );
    }
    for (i, label) in ["intercept", "slope"].iter().enumerate() {
      for (prefix, field) in [
        ("se", "standard_error"),
        ("z", "statistic"),
        ("lo", "confidence_lower"),
        ("hi", "confidence_upper"),
      ] {
        actual.insert(
          format!("{name}_{prefix}_{label}"),
          inf["coefficients"][i][field].as_f64().unwrap(),
        );
      }
    }
  }
  let reference = root()
    .join("tests/fixtures/mma23_exchangeable_gee")
    .join(case);
  let oracle = fs::read_to_string(reference.join("reference.csv")).unwrap();
  assert_eq!(actual.len(), 27);
  assert_eq!(rows(&oracle).len(), 27);
  for row in rows(&oracle) {
    close(
      actual[row["field"]],
      row["value"].parse().unwrap(),
      row["field"],
    );
  }
  let source = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let export = fs::read_to_string(dir.path().join(EXPORT)).unwrap();
  let prediction = fs::read_to_string(reference.join("predictions.csv")).unwrap();
  let source = rows(&source);
  let exported = rows(&export);
  let oracle = rows(&prediction);
  assert_eq!(exported.len(), 60);
  assert_eq!(oracle.len(), 60);
  let expected: BTreeMap<_, _> = source
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
  for ((row, reference), (key, input)) in exported.iter().zip(&oracle).zip(&expected) {
    assert_eq!(
      (
        row["id"].parse::<u64>().unwrap(),
        row["year"].parse::<u64>().unwrap()
      ),
      *key
    );
    assert_eq!(row["included_in_native_gee"], "true");
    assert_eq!(row.len(), 14);
    for (field, value) in *input {
      if value.is_empty() {
        assert!(row[field].is_empty());
      } else {
        close(row[field].parse().unwrap(), value.parse().unwrap(), field);
      }
    }
    for (field, reference_field) in [
      ("native_gee_mean", "gee_mu"),
      ("native_gee_residual", "gee_residual"),
    ] {
      close(
        row[field].parse().unwrap(),
        reference[reference_field].parse().unwrap(),
        field,
      );
    }
    let mu: f64 = row["native_gee_mean"].parse().unwrap();
    let xb: f64 = row["native_gee_xb"].parse().unwrap();
    close(mu, xb.exp(), "mean/index");
    close(
      row["native_gee_residual"].parse().unwrap(),
      row["pat"].parse::<f64>().unwrap() - mu,
      "residual",
    );
    let total: f64 = source
      .iter()
      .filter(|r| r["id"] == row["id"])
      .map(|r| r["pat"].parse::<f64>().unwrap())
      .sum();
    close(
      row["selected_firm_total"].parse().unwrap(),
      total,
      "group total",
    );
  }
}
#[test]
fn positive_and_negative_native_script_r_fields_all_rows_and_json_text_exports() {
  for (case, raw) in [("positive", POSITIVE), ("negative", NEGATIVE)] {
    let mut baseline = None;
    for mode in ["--json", "--text"] {
      let dir = TestDir::new();
      prepare(&dir, raw);
      let input = fs::read(dir.path().join("patents.csv")).unwrap();
      let out = run(&dir, mode);
      assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
      );
      if mode == "--json" {
        check(&dir, &out, case);
      } else {
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains("Poisson log-link exchangeable GEE"));
        assert!(text.contains("Fixed working variance scale1"));
        assert!(text.contains("Group sandwich covariance"));
      }
      let output = fs::read(dir.path().join(EXPORT)).unwrap();
      if let Some(previous) = &baseline {
        assert_eq!(&output, previous);
      } else {
        baseline = Some(output);
      }
      assert_eq!(fs::read(dir.path().join("patents.csv")).unwrap(), input);
    }
  }
}
#[test]
fn permutation_and_all_missing_nonmodel_metadata_do_not_select_sample() {
  let baseline = TestDir::new();
  prepare(&baseline, POSITIVE);
  check(&baseline, &run(&baseline, "--json"), "positive");
  let dir = TestDir::new();
  prepare(&dir, POSITIVE);
  let input = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let mut lines = input.lines();
  let header = lines.next().unwrap();
  let mut rest: Vec<_> = lines.collect();
  rest.reverse();
  fs::write(
    dir.path().join("patents.csv"),
    format!("{header}\n{}\n", rest.join("\n")),
  )
  .unwrap();
  check(&dir, &run(&dir, "--json"), "positive");
  assert_eq!(
    fs::read(dir.path().join(EXPORT)).unwrap(),
    fs::read(baseline.path().join(EXPORT)).unwrap()
  );
  let mut tokens: Vec<_> = POSITIVE.split_whitespace().map(str::to_owned).collect();
  for i in (1..tokens.len()).step_by(25) {
    tokens[i] = ".".into();
  }
  let dir = TestDir::new();
  prepare(&dir, &tokens.join(" "));
  check(&dir, &run(&dir, "--json"), "positive");
}
#[test]
fn invalid_or_singular_sources_fail_without_native_result_prediction_or_export() {
  let base = TestDir::new();
  prepare(&base, POSITIVE);
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
  for (index, (column, value)) in mutations.iter().enumerate() {
    let dir = TestDir::new();
    let mut record: Vec<_> = lines[1].split(',').map(str::to_owned).collect();
    record[*column] = value.to_string();
    let input = format!(
      "{}\n{}\n{}\n",
      lines[0],
      record.join(","),
      lines[2..].join("\n")
    );
    fs::write(dir.path().join("patents.csv"), &input).unwrap();
    let out = run(&dir, "--json");
    assert!(!out.status.success(), "mutation{index}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
      !stdout.contains("\"result_type\":\"PoissonGeeResult\""),
      "mutation{index}"
    );
    assert!(!stdout.contains("\"result_type\":\"PredictionResult\""));
    assert!(!dir.path().join(EXPORT).exists());
    assert_eq!(
      fs::read_to_string(dir.path().join("patents.csv")).unwrap(),
      input
    );
  }
  for case in 0..4 {
    let dir = TestDir::new();
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
      _ => {
        records.truncate(10);
      }
    }
    let input = format!(
      "{}\n{}\n",
      lines[0],
      records
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
    );
    fs::write(dir.path().join("patents.csv"), input).unwrap();
    let out = run(&dir, "--json");
    assert!(!out.status.success(), "singular{case}");
    assert!(!String::from_utf8_lossy(&out.stdout).contains("\"result_type\":\"PoissonGeeResult\""));
    assert!(!dir.path().join(EXPORT).exists());
  }
}
#[test]
fn whole_script_keeps_owned_model_labels_and_existing_export_on_collision() {
  for collision in [false, true] {
    let dir = TestDir::new();
    prepare(&dir, POSITIVE);
    if collision {
      fs::write(dir.path().join(EXPORT), "owner").unwrap();
    }
    let input = fs::read(dir.path().join("patents.csv")).unwrap();
    let mut session = Session::new();
    let mut failures = 0;
    for command in parse_script(SCRIPT, Path::new("native_poisson_gee.td")).unwrap() {
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
        failures += 1;
      } else {
        result.unwrap();
      }
    }
    assert_eq!(failures, usize::from(collision));
    assert!(matches!(
      session.state().estimation,
      Some(EstimationState::PoissonGee(_))
    ));
    assert_eq!(
      session.active_table_name(),
      Some("native_poisson_gee_cohort")
    );
    session
      .execute(parse_command("label variable logr \"Owned numeric LOGR\"").unwrap())
      .unwrap();
    let before = session.state();
    session
      .execute(parse_command("summarize native_gee_mean native_gee_residual").unwrap())
      .unwrap();
    assert_eq!(session.state(), before);
    assert!(
      session
        .execute(parse_command("predict native_gee_mean, n").unwrap())
        .is_err()
    );
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("predict repeated, n").unwrap())
      .unwrap();
    assert_eq!(session.state().estimation, before.estimation);
    assert_eq!(session.state().labels, before.labels);
    session.execute(parse_command("sql \"\"\"SELECT CASE WHEN COUNT(*)=60 AND MAX(abs(repeated-native_gee_mean))<1e-12 THEN 1 ELSE error('lost actual GEE ownership') END AS valid FROM active\"\"\"").unwrap()).unwrap();
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
fn existing_prediction_column_is_not_overwritten_by_script() {
  let dir = TestDir::new();
  prepare(&dir, POSITIVE);
  let source = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let mut lines = source.lines();
  let input = format!(
    "{},native_gee_mean\n{}\n",
    lines.next().unwrap(),
    lines
      .map(|line| format!("{line},42"))
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
  assert!(es.iter().any(|e| e["result_type"] == "PoissonGeeResult"));
  assert!(!es.iter().any(|e| e["result_type"] == "PredictionResult"));
}
