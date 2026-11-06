mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};
use tabdat_domain::{EstimationState, XtPoissonCommand, XtPoissonEstimator};
use tabdat_language::parse_command;
use tabdat_runtime::Session;
const INPUT: &str = include_str!("fixtures/mma24_individual_gamma_re/input.csv");
const SCRIPT: &str = "examples/mma24p2poiscluster/individual_gamma_re.td";
const OUTPUT: &str = "individual_gamma_re_predictions.csv";
const FIT: &str = "xtpoisson pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, re id(commune)";
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
  } else if path.contains("log_likelihood") {
    1e-8
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
fn session(d: &TestDir) -> Session {
  let file = d.path().join("vietnam_individual.csv");
  fs::write(&file, INPUT).unwrap();
  let mut s = Session::new();
  execute(&mut s, &format!("use \"{}\"", file.display()));
  execute(
    &mut s,
    "sql \"\"\"SELECT * FROM active WHERE pharvis IS NOT NULL ORDER BY source_row\"\"\" into cases",
  );
  execute(&mut s, FIT);
  s
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SCRIPT))
    .output()
    .unwrap()
}
#[test]
fn complete_script_matches_236_r_quantities_all_rows_and_text_json_exports() {
  let d = TestDir::new();
  fs::write(d.path().join("vietnam_individual.csv"), INPUT).unwrap();
  let out = run(&d, "--json");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let events = String::from_utf8(out.stdout)
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
    .find(|e| e["result_type"] == "GammaPoissonResult")
    .unwrap()["data"];
  assert_eq!(fit["estimator"], "shared_gamma_poisson_re");
  assert_eq!(fit["covariance"], "inverse_full_joint_observed_information");
  assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
  assert_eq!(fit["include_intercept"], true);
  assert_eq!(fit["observation_count"], 278);
  assert_eq!(fit["group_count"], 48);
  assert_eq!(fit["zero_total_group_count"], 4);
  assert_eq!(fit["singleton_group_count"], 2);
  assert_eq!(
    fit["sample"],
    serde_json::json!({"total_observations":278,"dropped_observations":0})
  );
  let reference = rows(include_str!(
    "fixtures/mma24_individual_gamma_re/reference.csv"
  ));
  assert_eq!(reference.len(), 236);
  for row in reference {
    let path = format!("/{}", row["path"]).replacen("/covariance/", "/covariance_matrix/", 1);
    near(
      fit.pointer(&path).unwrap().as_f64().unwrap(),
      row["value"].parse().unwrap(),
      &path,
    );
  }
  let csv = fs::read_to_string(d.path().join(OUTPUT)).unwrap();
  let actual = rows(&csv);
  let expected = rows(include_str!(
    "fixtures/mma24_individual_gamma_re/predictions.csv"
  ))
  .into_iter()
  .map(|r| (r["source_row"], r))
  .collect::<BTreeMap<_, _>>();
  assert_eq!(actual.len(), 278);
  let mut last = 0;
  let mut zero_rows = 0;
  for a in actual {
    let id = a["source_row"].parse::<usize>().unwrap();
    assert!(id > last);
    last = id;
    let b = &expected[a["source_row"]];
    assert_eq!(a.len(), b.len() + 2);
    for (key, value) in b {
      if key.starts_with("re_") {
        near(a[key].parse().unwrap(), value.parse().unwrap(), key);
      } else {
        assert_eq!(
          a[key].parse::<f64>().unwrap(),
          value.parse::<f64>().unwrap(),
          "{key}"
        );
      }
    }
    assert_eq!(a["included_in_re"], "true");
    if a["selected_commune_total"] == "0" {
      zero_rows += 1;
      assert!(a["re_mean"].parse::<f64>().unwrap() > 0.0);
    }
  }
  assert!(zero_rows > 0);
  assert_eq!(
    fs::read_to_string(d.path().join("vietnam_individual.csv")).unwrap(),
    INPUT
  );
  let text = TestDir::new();
  fs::write(text.path().join("vietnam_individual.csv"), INPUT).unwrap();
  let out = run(&text, "--text");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  assert!(
    String::from_utf8(out.stdout)
      .unwrap()
      .contains("Shared-gamma random-effects Poisson")
  );
  assert_eq!(fs::read(text.path().join(OUTPUT)).unwrap(), csv.as_bytes());
  let collision = run(&d, "--json");
  assert!(!collision.status.success());
  assert_eq!(fs::read_to_string(d.path().join(OUTPUT)).unwrap(), csv);
}
#[test]
fn owned_population_predictions_do_not_need_current_ids_and_fail_atomically() {
  let d = TestDir::new();
  let mut s = session(&d);
  let fit = s.state().estimation.clone();
  let Some(EstimationState::GammaPoisson(r)) = &fit else {
    panic!()
  };
  let beta = r
    .coefficients
    .iter()
    .find(|c| c.name == "lnhhexp")
    .unwrap()
    .value;
  execute(&mut s, "predict before_mean, n");
  execute(
    &mut s,
    "sql \"\"\"SELECT * EXCLUDE(commune,lnhhexp),lnhhexp+1 AS lnhhexp FROM active WHERE source_row%2=0 ORDER BY source_row DESC\"\"\" into subset",
  );
  execute(&mut s, "predict after_mean, n");
  execute(&mut s, "predict residual, residuals");
  execute(
    &mut s,
    &format!(
      "sql \"\"\"SELECT CASE WHEN MAX(ABS(after_mean-before_mean*EXP({beta}))/(1+after_mean))>1e-9 OR MAX(ABS(pharvis-after_mean-residual))>1e-9 THEN error('lost population prediction state') ELSE 1 END AS valid FROM active\"\"\""
    ),
  );
  assert_eq!(s.state().estimation, fit);
  for text in [
    "predict bad, pr",
    "predict bad, posterior_predictive interval",
    "predict after_mean, n",
  ] {
    let data = execute(&mut s, "head 3");
    let before = s.state();
    assert!(s.execute(parse_command(text).unwrap()).is_err());
    assert_eq!(s.state(), before);
    assert_eq!(execute(&mut s, "head 3"), data);
  }
  execute(
    &mut s,
    "sql \"\"\"SELECT * EXCLUDE(lnhhexp),1e308 AS lnhhexp FROM active\"\"\" into overflow",
  );
  let before = s.state();
  let data = execute(&mut s, "head 3");
  assert!(s.execute(parse_command("predict bad, n").unwrap()).is_err());
  assert_eq!(s.state(), before);
  assert_eq!(execute(&mut s, "head 3"), data);
  execute(&mut s, "predict finite_index, xb");
  execute(
    &mut s,
    "sql \"\"\"SELECT * EXCLUDE(pharvis),CAST(NULL AS DOUBLE) AS pharvis FROM active\"\"\" into missing_y",
  );
  execute(&mut s, "predict missing_residual, residuals");
  execute(
    &mut s,
    "sql \"\"\"SELECT CASE WHEN COUNT(missing_residual)!=0 THEN error('missing outcome not NULL') ELSE 1 END AS valid FROM active\"\"\"",
  );
  assert_eq!(s.state().estimation, fit);
}
#[test]
fn current_prediction_schema_failures_are_atomic_and_missing_inputs_give_null() {
  for query in [
    "SELECT * EXCLUDE(educ) FROM active",
    "SELECT * EXCLUDE(educ),CAST(educ AS VARCHAR) AS educ FROM active",
  ] {
    let d = TestDir::new();
    let mut s = session(&d);
    execute(
      &mut s,
      &format!("sql \"\"\"{query}\"\"\" into invalid_prediction"),
    );
    let data = execute(&mut s, "head 3");
    let before = s.state();
    assert!(s.execute(parse_command("predict bad, n").unwrap()).is_err());
    assert_eq!(s.state(), before);
    assert_eq!(execute(&mut s, "head 3"), data);
  }
  for value in ["NULL::DOUBLE", "'NaN'::DOUBLE"] {
    let d = TestDir::new();
    let mut s = session(&d);
    let fit = s.state().estimation;
    execute(
      &mut s,
      &format!(
        "sql \"\"\"SELECT * EXCLUDE(educ),{value} AS educ FROM active\"\"\" into missing_prediction"
      ),
    );
    execute(&mut s, "predict missing_mean, n");
    execute(&mut s, "predict missing_index, xb");
    execute(
      &mut s,
      "sql \"\"\"SELECT CASE WHEN COUNT(missing_mean)!=0 OR COUNT(missing_index)!=0 THEN error('nonfinite input not NULL') ELSE 1 END AS valid FROM active\"\"\"",
    );
    assert_eq!(s.state().estimation, fit);
  }
}

#[test]
fn bad_new_fits_clear_stale_models_without_changing_data_and_syntax_preserves_fit() {
  for query in [
    "SELECT * EXCLUDE(pharvis),0.5 AS pharvis FROM active",
    "SELECT * EXCLUDE(pharvis),CAST(1.00000000000000000001 AS DECIMAL(38,20)) AS pharvis FROM active",
    "SELECT * EXCLUDE(commune,pharvis),CASE WHEN source_row=1 THEN NULL ELSE commune END AS commune,CASE WHEN source_row=1 THEN NULL ELSE pharvis END AS pharvis FROM active",
    "SELECT * EXCLUDE(educ),lnhhexp AS educ FROM active",
    "SELECT * EXCLUDE(pharvis),0::BIGINT AS pharvis FROM active",
  ] {
    let d = TestDir::new();
    let mut s = session(&d);
    let fit = s.state().estimation;
    assert!(parse_command("xtpoisson pharvis educ, re id(commune) robust").is_err());
    assert_eq!(s.state().estimation, fit);
    execute(&mut s, &format!("sql \"\"\"{query}\"\"\" into invalid"));
    let data = execute(&mut s, "head 3");
    let mut before = s.state();
    assert!(s.execute(parse_command(FIT).unwrap()).is_err());
    before.estimation = None;
    assert_eq!(s.state(), before);
    assert_eq!(execute(&mut s, "head 3"), data);
  }
}
#[test]
fn prediction_work_limit_and_export_success_or_collision_preserve_owned_fit() {
  let d = TestDir::new();
  let mut s = session(&d);
  execute(
    &mut s,
    "sql \"\"\"SELECT * FROM active CROSS JOIN range(200)\"\"\" into large",
  );
  let data = execute(&mut s, "head 3");
  let before = s.state();
  let e = s
    .execute(parse_command("predict bad, n").unwrap())
    .unwrap_err();
  assert!(e.to_string().contains("prediction rows<=40000"));
  assert_eq!(s.state(), before);
  assert_eq!(execute(&mut s, "head 3"), data);
  for collision in [false, true] {
    let d = TestDir::new();
    let source = d.path().join("vietnam_individual.csv");
    let output = d.path().join(OUTPUT);
    fs::write(&source, INPUT).unwrap();
    if collision {
      fs::write(&output, "owner").unwrap();
    }
    let mut s = Session::new();
    let mut fit = None;
    for line in include_str!("../examples/mma24p2poiscluster/individual_gamma_re.td").lines() {
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
      let out = s.execute(parse_command(&text).unwrap());
      if collision && line.starts_with("export ") {
        assert!(out.is_err());
      } else {
        out.unwrap();
      }
      if line.starts_with("xtpoisson ") {
        fit = s.state().estimation;
        assert!(fit.is_some());
      } else if fit.is_some() {
        assert_eq!(s.state().estimation, fit);
      }
    }
    execute(&mut s, "predict after_export, n");
    assert_eq!(s.state().estimation, fit);
    assert_eq!(fs::read_to_string(&source).unwrap(), INPUT);
    if collision {
      assert_eq!(fs::read_to_string(&output).unwrap(), "owner");
    }
  }
}
#[test]
fn invalid_script_source_preserves_source_and_existing_export_owner() {
  for (column, value) in [(1, "0.5"), (12, "0"), (1, "")] {
    let d = TestDir::new();
    let mut lines = INPUT.lines().map(str::to_owned).collect::<Vec<_>>();
    let mut row = lines[1].split(',').map(str::to_owned).collect::<Vec<_>>();
    row[column] = value.into();
    lines[1] = row.join(",");
    let source = lines.join("\n") + "\n";
    fs::write(d.path().join("vietnam_individual.csv"), &source).unwrap();
    fs::write(d.path().join(OUTPUT), "owner").unwrap();
    let out = run(&d, "--json");
    assert!(!out.status.success());
    assert!(
      !String::from_utf8(out.stdout)
        .unwrap()
        .contains("GammaPoissonResult")
    );
    assert_eq!(
      fs::read_to_string(d.path().join("vietnam_individual.csv")).unwrap(),
      source
    );
    assert_eq!(fs::read_to_string(d.path().join(OUTPUT)).unwrap(), "owner");
  }
}

#[test]
fn legacy_internal_fe_command_defaults_remain_fe() {
  let c: XtPoissonCommand = serde_json::from_value(
    serde_json::json!({"outcome":"y","predictors":["x"],"entity_variable":"id"}),
  )
  .unwrap();
  assert_eq!(c.estimator, XtPoissonEstimator::ConditionalFe);
  let c = parse_command(FIT).unwrap();
  assert_eq!(
    serde_json::from_value::<tabdat_domain::Command>(serde_json::to_value(&c).unwrap()).unwrap(),
    c
  );
}
