mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};
use tabdat_domain::EstimationState;
use tabdat_language::parse_command;
use tabdat_runtime::Session;

const INPUT: &str = include_str!("fixtures/mma24_household_gee/negative/input.csv");
const REFERENCE: &str = include_str!("fixtures/mma24_household_gee/negative/reference.csv");
const PREDICTIONS: &str = include_str!("fixtures/mma24_household_gee/negative/predictions.csv");
const SCRIPT: &str = "examples/mma24p1olscluster/household_gee.td";
const OUTPUT: &str = "household_gee_predictions.csv";
const FIT: &str = "xtgee lnexp12m lnhhexp age sex hhsize farm educ, family(gaussian) link(identity) corr(exchangeable) id(commune)";
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let names = lines
    .next()
    .unwrap()
    .split(',')
    .map(|v| v.trim_matches('"'))
    .collect::<Vec<_>>();
  lines
    .map(|l| {
      let values = l
        .split(',')
        .map(|v| v.trim_matches('"'))
        .collect::<Vec<_>>();
      assert_eq!(names.len(), values.len());
      names.iter().copied().zip(values).collect()
    })
    .collect()
}
fn near(a: f64, b: f64, path: &str) {
  let tol = if path.ends_with("p_value") {
    b.abs() * 1e-8
  } else {
    1e-9
  };
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= tol,
    "{path}: {a} != {b}, tolerance {tol}"
  );
}
fn execute(s: &mut Session, text: &str) -> tabdat_domain::ExecutionResult {
  s.execute(parse_command(text).unwrap())
    .unwrap_or_else(|e| panic!("{text}: {e}"))
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SCRIPT))
    .output()
    .unwrap()
}
fn audit(fit: &Value, reference: &str) {
  let mut actual = BTreeMap::<String, f64>::new();
  for key in [
    "observation_count",
    "group_count",
    "minimum_group_size",
    "maximum_group_size",
    "singleton_group_count",
    "ordered_pair_count",
    "phi",
    "rho",
  ] {
    actual.insert(key.into(), fit[key].as_f64().unwrap());
  }
  actual.insert(
    "source_count".into(),
    fit["sample"]["total_observations"].as_f64().unwrap(),
  );
  actual.insert(
    "parameter_count".into(),
    fit["working"]["coefficients"].as_array().unwrap().len() as f64,
  );
  for kind in ["working", "sandwich"] {
    let inf = &fit[kind];
    let coefficients = inf["coefficients"].as_array().unwrap();
    for (i, c) in coefficients.iter().enumerate() {
      let name = c["name"].as_str().unwrap();
      for (field, json) in [
        ("estimate", "value"),
        ("standard_error", "standard_error"),
        ("statistic", "statistic"),
        ("p_value", "p_value"),
        ("ci_lower", "confidence_lower"),
        ("ci_upper", "confidence_upper"),
      ] {
        actual.insert(format!("{kind}.{name}.{field}"), c[json].as_f64().unwrap());
      }
      for (j, other) in coefficients.iter().enumerate() {
        actual.insert(
          format!(
            "{kind}.covariance.{name}.{}",
            other["name"].as_str().unwrap()
          ),
          inf["covariance"][i][j].as_f64().unwrap(),
        );
      }
    }
    for key in ["wald_statistic", "wald_df", "wald_p_value"] {
      actual.insert(format!("{kind}.{key}"), inf[key].as_f64().unwrap());
    }
  }
  let expected = rows(reference);
  assert_eq!(expected.len(), 198);
  assert_eq!(actual.len(), 198);
  for row in expected {
    let path = row["path"];
    near(
      *actual.get(path).unwrap_or_else(|| panic!("missing {path}")),
      row["value"].parse().unwrap(),
      path,
    );
  }
}
#[test]
fn actual_script_matches_reference_and_preserves_file_on_collision() {
  let d = TestDir::new();
  fs::write(d.path().join("vietnam.csv"), INPUT).unwrap();
  let output = run(&d, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str::<Value>(l).unwrap())
    .collect::<Vec<_>>();
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let fit = &events
    .iter()
    .find(|e| e["result_type"] == "GaussianGeeResult")
    .unwrap()["data"];
  assert_eq!(fit["estimator"], "gaussian_identity_exchangeable_gee");
  assert_eq!(fit["working_covariance"], "inverse_gee_bread");
  assert_eq!(fit["sandwich_covariance"], "uncorrected_unit_sandwich");
  assert_eq!(
    fit["sample"],
    serde_json::json!({"total_observations":170,"dropped_observations":2})
  );
  audit(fit, REFERENCE);
  let csv = fs::read_to_string(d.path().join(OUTPUT)).unwrap();
  let actual = rows(&csv);
  let expected = rows(PREDICTIONS)
    .into_iter()
    .map(|r| (r["source_row"], r))
    .collect::<BTreeMap<_, _>>();
  assert_eq!(actual.len(), 170);
  for a in actual {
    let b = &expected[a["source_row"]];
    assert_eq!(
      a["included_in_gee"],
      if b["selected"] == "1" {
        "true"
      } else {
        "false"
      }
    );
    for (name, key) in [
      ("gee_mean", "mean"),
      ("gee_xb", "xb"),
      ("gee_residual", "residual"),
    ] {
      if b[key].is_empty() {
        assert!(a[name].is_empty(), "row {} {name}", a["source_row"]);
      } else {
        near(a[name].parse().unwrap(), b[key].parse().unwrap(), name);
      }
    }
  }
  assert_eq!(
    fs::read_to_string(d.path().join("vietnam.csv")).unwrap(),
    INPUT
  );
  let t = TestDir::new();
  fs::write(t.path().join("vietnam.csv"), INPUT).unwrap();
  let out = run(&t, "--text");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  assert!(
    String::from_utf8(out.stdout)
      .unwrap()
      .contains("Gaussian identity-link exchangeable GEE")
  );
  assert_eq!(fs::read(t.path().join(OUTPUT)).unwrap(), csv.as_bytes());
  let collision = run(&d, "--json");
  assert!(!collision.status.success());
  assert_eq!(fs::read_to_string(d.path().join(OUTPUT)).unwrap(), csv);
}
#[test]
fn owned_state_and_atomic_failure_contracts() {
  let d = TestDir::new();
  let path = d.path().join("vietnam.csv");
  fs::write(&path, INPUT).unwrap();
  let mut s = Session::new();
  execute(&mut s, &format!("use \"{}\"", path.display()));
  execute(&mut s, FIT);
  let state = s.state().estimation.clone();
  let command = parse_command(FIT).unwrap();
  assert_eq!(
    serde_json::from_value::<tabdat_domain::Command>(serde_json::to_value(&command).unwrap())
      .unwrap(),
    command
  );
  assert_eq!(
    serde_json::from_value::<Option<EstimationState>>(serde_json::to_value(&state).unwrap())
      .unwrap(),
    state
  );
  let Some(EstimationState::GaussianGee(fit)) = &state else {
    panic!()
  };
  let beta = fit.working.coefficients[1].value;
  execute(&mut s, "predict before_mean");
  execute(
    &mut s,
    "sql \"\"\"SELECT * EXCLUDE(commune,lnhhexp),lnhhexp+1 AS lnhhexp FROM active WHERE source_row%2=0 ORDER BY source_row DESC\"\"\" into subset",
  );
  execute(&mut s, "predict after_mean, xb");
  execute(&mut s, "predict residual, residuals");
  execute(
    &mut s,
    &format!(
      "sql \"\"\"SELECT CASE WHEN MAX(ABS(after_mean-before_mean-({beta})))>1e-9 OR MAX(ABS(lnexp12m-after_mean-residual))>1e-9 THEN error('lost GEE state') ELSE 1 END AS valid FROM active\"\"\""
    ),
  );
  assert_eq!(s.state().estimation, state);
  for text in [
    "predict bad, pr",
    "predict bad, n",
    "predict after_mean, xb",
  ] {
    let before = s.state();
    let data = execute(&mut s, "head 3");
    assert!(s.execute(parse_command(text).unwrap()).is_err());
    assert_eq!(s.state(), before);
    assert_eq!(execute(&mut s, "head 3"), data);
  }
  // Missing group or an unknown predictor during a new fit must not discard the old fit.
  let before = s.state();
  let data = execute(&mut s, "head 3");
  assert!(s.execute(parse_command(FIT).unwrap()).is_err());
  assert_eq!(s.state(), before);
  assert_eq!(execute(&mut s, "head 3"), data);
  execute(
    &mut s,
    "sql \"\"\"SELECT * EXCLUDE(lnexp12m),CAST(NULL AS DOUBLE) AS lnexp12m FROM active\"\"\" into missing_y",
  );
  execute(&mut s, "predict missing_mean");
  execute(&mut s, "predict missing_residual, residuals");
  execute(
    &mut s,
    "sql \"\"\"SELECT CASE WHEN COUNT(missing_residual)!=0 OR COUNT(missing_mean)=0 THEN error('missing y handling') ELSE 1 END AS valid FROM active\"\"\"",
  );
  assert_eq!(s.state().estimation, state);

  let signed_columns = fit.working.coefficients[1..]
    .iter()
    .map(|c| {
      format!(
        "{}1.7e308 AS {}",
        if c.value < 0.0 { "-" } else { "" },
        c.name
      )
    })
    .collect::<Vec<_>>()
    .join(", ");
  execute(
    &mut s,
    &format!(
      "sql \"\"\"SELECT * EXCLUDE(lnhhexp,age,sex,hhsize,farm,educ),{signed_columns} FROM active\"\"\" into overflow"
    ),
  );
  let before = s.state();
  let data = execute(&mut s, "head 3");
  assert!(
    s.execute(parse_command("predict overflow_mean").unwrap())
      .is_err()
  );
  assert_eq!(s.state(), before);
  assert_eq!(execute(&mut s, "head 3"), data);
}

#[test]
fn actual_script_export_success_and_collision_keep_owned_gee() {
  for collision in [false, true] {
    let d = TestDir::new();
    let source = d.path().join("vietnam.csv");
    let output = d.path().join(OUTPUT);
    fs::write(&source, INPUT).unwrap();
    if collision {
      fs::write(&output, "owner").unwrap();
    }
    let mut s = Session::new();
    let mut fit = None;
    for line in include_str!("../examples/mma24p1olscluster/household_gee.td").lines() {
      if line.starts_with('#') || line.is_empty() {
        continue;
      }
      let text = if line.starts_with("use ") {
        format!("use \"{}\"", source.display())
      } else if line.starts_with("export ") {
        format!("export \"{}\"", output.display())
      } else {
        line.to_owned()
      };
      let result = s.execute(parse_command(&text).unwrap());
      if collision && line.starts_with("export ") {
        assert!(result.is_err());
      } else {
        result.unwrap();
      }
      if line.starts_with("xtgee ") {
        fit = s.state().estimation;
        assert!(fit.is_some());
      } else if fit.is_some() {
        assert_eq!(s.state().estimation, fit);
      }
    }
    execute(&mut s, "predict after_export, xb");
    assert_eq!(s.state().estimation, fit);
    assert_eq!(fs::read_to_string(&source).unwrap(), INPUT);
    if collision {
      assert_eq!(fs::read_to_string(&output).unwrap(), "owner");
    }
  }
}

#[test]
fn numerical_fit_failure_preserves_previous_fit_and_current_table() {
  let d = TestDir::new();
  let path = d.path().join("vietnam.csv");
  fs::write(&path, INPUT).unwrap();
  let mut s = Session::new();
  execute(&mut s, &format!("use \"{}\"", path.display()));
  execute(&mut s, FIT);
  execute(
    &mut s,
    "sql \"\"\"SELECT * EXCLUDE(commune), 1 AS commune FROM active\"\"\" into one_group",
  );
  let before = s.state();
  let data = execute(&mut s, "head 3");
  let err = s.execute(parse_command(FIT).unwrap()).unwrap_err();
  assert!(err.to_string().contains("G>parameter count"), "{err}");
  assert_eq!(s.state(), before);
  assert_eq!(execute(&mut s, "head 3"), data);
  execute(&mut s, "predict still_usable");
  assert_eq!(s.state().estimation, before.estimation);
}

#[test]
fn joint_missing_group_selection_preserves_wide_decimal_key_identity() {
  let d = TestDir::new();
  let path = d.path().join("vietnam.csv");
  fs::write(&path, INPUT).unwrap();
  let mut s = Session::new();
  execute(&mut s, &format!("use \"{}\"", path.display()));
  execute(
    &mut s,
    "sql \"\"\"SELECT * EXCLUDE(commune), CASE WHEN source_row=1 THEN NULL ELSE CAST(commune AS DECIMAL(38,0))+CAST('9007199254740992' AS DECIMAL(38,0)) END AS commune FROM active\"\"\" into wide_ids",
  );
  execute(&mut s, FIT);
  let Some(EstimationState::GaussianGee(fit)) = s.state().estimation else {
    panic!()
  };
  assert_eq!(fit.group_count, 47);
  assert_eq!(fit.observation_count, 167);
  assert_eq!(fit.sample.total_observations, 170);
  assert_eq!(fit.sample.dropped_observations, 3);
  assert!(!fit.sample.retained_indices.contains(&0));
  execute(&mut s, "predict marginal");
  execute(
    &mut s,
    "sql \"\"\"SELECT CASE WHEN COUNT(*) FILTER (WHERE source_row=1 AND commune IS NULL AND marginal IS NOT NULL)!=1 THEN error('marginal prediction must not require group') ELSE 1 END AS valid FROM active\"\"\"",
  );
}

#[test]
fn bounded_candidate_groups_and_prediction_rows_fail_atomically() {
  let d = TestDir::new();
  let path = d.path().join("vietnam.csv");
  fs::write(&path, INPUT).unwrap();
  let mut s = Session::new();
  execute(&mut s, &format!("use \"{}\"", path.display()));
  execute(&mut s, FIT);
  execute(
    &mut s,
    "sql \"\"\"SELECT * EXCLUDE(commune),source_row+1000*r.i AS commune FROM active CROSS JOIN range(60) r(i)\"\"\" into too_many_groups",
  );
  let before = s.state();
  let data = execute(&mut s, "head 3");
  let err = s.execute(parse_command(FIT).unwrap()).unwrap_err();
  assert!(
    err.to_string().contains("at most10000 candidate groups"),
    "{err}"
  );
  assert_eq!(s.state(), before);
  assert_eq!(execute(&mut s, "head 3"), data);
  execute(
    &mut s,
    "sql \"\"\"SELECT * FROM active CROSS JOIN range(4)\"\"\" into too_many_rows",
  );
  let before = s.state();
  let data = execute(&mut s, "head 3");
  let err = s
    .execute(parse_command("predict bad").unwrap())
    .unwrap_err();
  assert!(err.to_string().contains("prediction rows<=40000"), "{err}");
  assert_eq!(s.state(), before);
  assert_eq!(execute(&mut s, "head 3"), data);
}
