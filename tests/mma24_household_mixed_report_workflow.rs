mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
};
use tabdat_domain::{EstimateTableResult, ExecutionResult};
use tabdat_language::{parse_command, script::parse_script};
use tabdat_runtime::Session;

const SCRIPT: &str = include_str!("../examples/mma24p1olscluster/household_mixed_report.td");
const OUTPUT: &str = "household_mixed_report_predictions.csv";
fn fixture(case: &str, file: &str) -> String {
  fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
    "tests/fixtures/mma24_household_mixed_report/{case}/{file}"
  )))
  .unwrap()
}
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma24p1olscluster/household_mixed_report.td"),
    )
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let keys: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|l| {
      let values: Vec<_> = l.split(',').collect();
      assert_eq!(values.len(), keys.len());
      keys.iter().copied().zip(values).collect()
    })
    .collect()
}
fn expected_table(source: &Path) -> EstimateTableResult {
  let mut session = Session::new();
  for command in parse_script(SCRIPT, Path::new("household_mixed_report.td")).unwrap() {
    let text = if command.text == "use vietnam.csv" {
      format!("use \"{}\"", source.display())
    } else {
      command.text
    };
    let before = session.state();
    let result = session.execute(parse_command(&text).unwrap()).unwrap();
    if let ExecutionResult::EstimateTable(table) = result {
      assert_eq!(session.state(), before);
      return table;
    }
  }
  panic!("comparison missing")
}
fn check(case: &str, n: usize, g: usize) {
  let dir = TestDir::new();
  let input = fixture(case, "input.csv");
  let source = dir.path().join("vietnam.csv");
  fs::write(&source, &input).unwrap();
  let out = run(&dir, "--json");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  assert!(out.stderr.is_empty());
  let events: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  assert_eq!(events.len(), 19);
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  assert_eq!(events[13]["result_type"], "EstimateTableResult");
  let table: EstimateTableResult = serde_json::from_value(events[13]["data"].clone()).unwrap();
  // The runtime contract independently checks every selected R coefficient and
  // covariance; this checks the actual script/CLI wire against that typed path.
  assert_eq!(table, expected_table(&source));
  assert_eq!(
    table
      .models
      .iter()
      .map(|m| m.name.as_str())
      .collect::<Vec<_>>(),
    ["pa", "fe", "refgls", "remle"]
  );
  assert_eq!(
    table.parameter_names,
    [
      "intercept",
      "lnhhexp",
      "age",
      "sex",
      "hhsize",
      "farm",
      "educ"
    ]
  );
  for m in &table.models {
    assert_eq!(m.observation_count, n);
    assert_eq!(m.sample.total_observations, n);
    assert_eq!(m.sample.dropped_observations, 0);
    assert_eq!(m.panel.as_ref().unwrap().group_count, g);
    assert_eq!(m.r_squared, None);
    assert_eq!(m.covariance_matrix.len(), 7);
    assert!(m.covariance_matrix.iter().all(|r| r.len() == 7));
  }
  assert_eq!(
    table.models[1].inference_degrees_of_freedom,
    Some(n - g - 6)
  );
  assert_eq!(table.models[2].log_likelihood, None);
  assert!(table.models[3].log_likelihood.is_some());
  if case == "interior" {
    assert!(events[5]["data"]["rho"].as_f64().unwrap() > 0.0);
    assert!(events[11]["data"]["sigma_u_squared"].as_f64().unwrap() > 0.0);
  } else {
    assert!(events[5]["data"]["rho"].as_f64().unwrap() < 0.0);
    assert_eq!(events[11]["data"]["sigma_u_squared"], 0.0);
  }
  let output = dir.path().join(OUTPUT);
  let json_bytes = fs::read(&output).unwrap();
  let actual = String::from_utf8(json_bytes.clone()).unwrap();
  let expected = fixture(case, "ml_predictions.csv");
  let a = rows(&actual);
  let b = rows(&expected);
  let selected = rows(&input)
    .into_iter()
    .filter(|r| !r["lnexp12m"].is_empty())
    .collect::<Vec<_>>();
  assert_eq!(a.len(), n);
  assert_eq!(a.len(), b.len());
  assert_eq!(selected.len(), n);
  assert_eq!(actual.lines().next(), expected.lines().next());
  for ((a, b), s) in a.iter().zip(&b).zip(&selected) {
    for (key, value) in s {
      assert_eq!(
        a[key].parse::<f64>().unwrap(),
        value.parse::<f64>().unwrap()
      );
      assert_eq!(
        a[key].parse::<f64>().unwrap(),
        b[key].parse::<f64>().unwrap()
      );
    }
    for key in ["ml_xb", "ml_residual"] {
      let (a, b) = (
        a[key].parse::<f64>().unwrap(),
        b[key].parse::<f64>().unwrap(),
      );
      assert!(
        a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9,
        "{a} != {b}"
      );
    }
  }
  fs::remove_file(&output).unwrap();
  let out = run(&dir, "--text");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let text = String::from_utf8(out.stdout).unwrap();
  for label in [
    "Stored fit comparison (reporting snapshots, no refits)",
    "Estimate\tpa\tfe\trefgls\tremle",
    "Panel entity\tcommune\tcommune\tcommune\tcommune",
    "OLS R-squared\tNA\tNA\tNA\tNA",
    "unit-sandwich is uncorrected",
    "Reports cannot restore fits.",
    "Printed historical ML SE discrepancies remain unresolved.",
  ] {
    assert!(text.contains(label), "missing {label}");
  }
  assert!(text.contains(&format!("Coefficient reference\tasymptotic_standard_normal\tstudent_t({})\tasymptotic_standard_normal\tasymptotic_standard_normal",n-g-6)));
  assert_eq!(fs::read(&output).unwrap(), json_bytes);
  assert_eq!(fs::read_to_string(&source).unwrap(), input);
  let collision = run(&dir, "--json");
  assert!(!collision.status.success());
  assert!(!collision.stderr.is_empty());
  assert_eq!(fs::read(output).unwrap(), json_bytes);
  assert_eq!(fs::read_to_string(source).unwrap(), input);
}
#[test]
fn balanced_negative_gee_boundary_ml_script_and_wire_preserve_r_validated_reports() {
  check("balanced", 192, 32);
}
#[test]
fn unbalanced_singleton_script_and_exports_preserve_the_exact_selected_cohort() {
  check("unbalanced", 199, 32);
}
#[test]
fn positive_interior_ml_script_full_reports_and_population_predictions_match() {
  check("interior", 179, 16);
}

#[test]
fn current_data_labels_and_export_collision_do_not_change_saved_models_or_owned_ml() {
  for collision in [false, true] {
    let dir = TestDir::new();
    let input = fixture("interior", "input.csv");
    let source = dir.path().join("vietnam.csv");
    fs::write(&source, &input).unwrap();
    let output = dir.path().join(OUTPUT);
    if collision {
      fs::write(&output, "owner").unwrap();
    }
    let mut session = Session::new();
    let mut report = None;
    let mut retained = None;
    let mut failures = 0;
    for command in parse_script(SCRIPT, Path::new("household_mixed_report.td")).unwrap() {
      let text = if command.text == "use vietnam.csv" {
        format!("use \"{}\"", source.display())
      } else if command.text.starts_with("export ") {
        format!("export \"{}\"", output.display())
      } else {
        command.text.clone()
      };
      let before = session.state();
      let result = session.execute(parse_command(&text).unwrap());
      if collision && command.text.starts_with("export ") {
        assert!(result.is_err());
        assert_eq!(session.state(), before);
        failures += 1;
      } else if let ExecutionResult::EstimateTable(table) = result.unwrap() {
        assert_eq!(session.state(), before);
        report = Some(table.clone());
        session
          .execute(parse_command("label variable lnhhexp \"Household expenditure\"").unwrap())
          .unwrap();
        let labelled = session.state();
        let ExecutionResult::EstimateTable(labelled_report) = session
          .execute(parse_command("estimates table pa fe refgls remle").unwrap())
          .unwrap()
        else {
          panic!()
        };
        assert_eq!(labelled_report, table);
        assert_eq!(session.state(), labelled);
        session
          .execute(parse_command("sql SELECT 999 AS unrelated into replaced").unwrap())
          .unwrap();
        let before = session.state();
        let ExecutionResult::EstimateTable(other) = session
          .execute(parse_command("estimates table pa fe refgls remle").unwrap())
          .unwrap()
        else {
          panic!()
        };
        assert_eq!(other, table);
        assert_eq!(session.state(), before);
        session
          .execute(parse_command("use household_mixed_cohort").unwrap())
          .unwrap();
      }
      if command.text.starts_with("estimates store ") {
        assert_eq!(session.state().estimation, before.estimation);
        assert_eq!(session.state().active_dataset, before.active_dataset);
        assert_eq!(session.state().labels, before.labels);
      }
      if command.text.contains(", mle ") {
        retained = session.state().estimation;
      } else if retained.is_some() {
        assert_eq!(session.state().estimation, retained);
      }
    }
    assert_eq!(failures, usize::from(collision));
    assert!(report.is_some());
    let before = session.state();
    let ExecutionResult::EstimateTable(other) = session
      .execute(parse_command("estimates table pa fe refgls remle").unwrap())
      .unwrap()
    else {
      panic!()
    };
    assert_eq!(Some(other), report);
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("predict after_report, xb").unwrap())
      .unwrap();
    session.execute(parse_command("sql SELECT CASE WHEN COUNT(*)!=179 OR MAX(ABS(after_report-ml_xb))>1e-12 OR MAX(ABS(lnexp12m-ml_xb-ml_residual))>1e-12 THEN error('changed ML ownership') ELSE 1 END AS valid FROM active").unwrap()).unwrap();
    assert_eq!(session.state().estimation, retained);
    if collision {
      assert_eq!(fs::read_to_string(output).unwrap(), "owner");
    }
    assert_eq!(fs::read_to_string(source).unwrap(), input);
  }
}

#[test]
fn raw_source_and_fit_failures_preserve_input_and_existing_export_owner() {
  let input = fixture("unbalanced", "input.csv");
  let mut cases = Vec::new();
  for (column, value) in [(0, "1.25"), (8, "0"), (7, ""), (1, "Inf")] {
    let mut lines: Vec<_> = input.lines().map(str::to_owned).collect();
    let mut cells: Vec<_> = lines[1].split(',').map(str::to_owned).collect();
    cells[column] = value.into();
    lines[1] = cells.join(",");
    cases.push(lines.join("\n") + "\n");
  }
  let mut lines: Vec<_> = input.lines().map(str::to_owned).collect();
  let missing = (1..lines.len())
    .find(|i| lines[*i].split(',').nth(1) == Some(""))
    .unwrap();
  let mut cells: Vec<_> = lines[missing].split(',').map(str::to_owned).collect();
  cells[0] = "1".into();
  lines[missing] = cells.join(",");
  cases.push(lines.join("\n") + "\n");
  let mut lines: Vec<_> = input.lines().map(str::to_owned).collect();
  let mut cells: Vec<_> = lines[missing].split(',').map(str::to_owned).collect();
  cells[7] = "".into();
  lines[missing] = cells.join(",");
  cases.push(lines.join("\n") + "\n");
  let mut few = String::from(input.lines().next().unwrap());
  few.push('\n');
  let mut rank = few.clone();
  let mut oversized = few.clone();
  let base: Vec<_> = input.lines().nth(1).unwrap().split(',').collect();
  for line in input.lines().skip(1) {
    let mut c: Vec<_> = line.split(',').map(str::to_owned).collect();
    c[8] = "1".into();
    few.push_str(&(c.join(",") + "\n"));
    c = line.split(',').map(str::to_owned).collect();
    c[2] = c[7].clone();
    rank.push_str(&(c.join(",") + "\n"));
  }
  for i in 1..=40001 {
    let mut c: Vec<_> = base.iter().map(|s| s.to_string()).collect();
    c[0] = i.to_string();
    oversized.push_str(&(c.join(",") + "\n"));
  }
  cases.extend([few, rank, oversized]);
  for input in cases {
    let dir = TestDir::new();
    let source = dir.path().join("vietnam.csv");
    let output = dir.path().join(OUTPUT);
    fs::write(&source, &input).unwrap();
    fs::write(&output, "owner").unwrap();
    let out = run(&dir, "--json");
    assert!(!out.status.success());
    assert!(!out.stderr.is_empty());
    assert_eq!(fs::read_to_string(source).unwrap(), input);
    assert_eq!(fs::read_to_string(output).unwrap(), "owner");
  }
}
