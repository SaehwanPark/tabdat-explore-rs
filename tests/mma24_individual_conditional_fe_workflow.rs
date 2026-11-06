mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

const INPUT: &str = include_str!("fixtures/mma24_individual_conditional_fe/input.csv");
const OUTPUT: &str = "individual_conditional_fe_predictions.csv";
const SCRIPT: &str = "examples/mma24p2poiscluster/individual_conditional_fe.td";
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SCRIPT))
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let names = lines.next().unwrap().split(',').collect::<Vec<_>>();
  lines
    .map(|l| {
      let values = l.split(',').collect::<Vec<_>>();
      assert_eq!(names.len(), values.len());
      names.iter().copied().zip(values).collect()
    })
    .collect()
}
fn near(a: f64, b: f64, path: &str) {
  assert!(a.is_finite() && b.is_finite());
  let tol = if path.ends_with("p_value") {
    b.abs() * 1e-8
  } else if path.contains("log_likelihood") {
    1e-8
  } else {
    1e-9
  };
  assert!((a - b).abs() <= tol, "{path}: {a} != {b}, tol={tol}");
}
fn execute(s: &mut Session, text: &str) -> tabdat_domain::ExecutionResult {
  s.execute(parse_command(text).unwrap())
    .unwrap_or_else(|e| panic!("{text}: {e}"))
}
#[test]
fn ten_slope_unbalanced_singleton_exclusions_and_normal_inference_match_r() {
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
    .find(|e| e["result_type"] == "ConditionalPoissonResult")
    .unwrap()["data"];
  assert_eq!(fit["observation_count"], 140);
  assert_eq!(fit["group_count"], 18);
  assert_eq!(fit["minimum_group_size"], 1);
  assert_eq!(fit["zero_total_observation_count"], 8);
  assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
  assert_eq!(fit["covariance"], "inverse_conditional_information");
  assert_eq!(fit["include_intercept"], false);
  let reference = rows(include_str!(
    "fixtures/mma24_individual_conditional_fe/reference.csv"
  ));
  assert_eq!(reference.len(), 179);
  for r in reference {
    near(
      fit.pointer(r["path"]).unwrap().as_f64().unwrap(),
      r["value"].parse().unwrap(),
      r["path"],
    );
  }
  let csv = fs::read_to_string(d.path().join(OUTPUT)).unwrap();
  let (actual, expected) = (
    rows(&csv),
    rows(include_str!(
      "fixtures/mma24_individual_conditional_fe/predictions.csv"
    )),
  );
  assert_eq!(actual.len(), 148);
  assert_eq!(actual.len(), expected.len());
  let mut kept = 0;
  for (a, b) in actual.iter().zip(expected) {
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for key in a.keys() {
      if *key == "included_in_fe" {
        assert_eq!(a[key], b[key].to_lowercase());
      } else if a[key].is_empty() || b[key].is_empty() {
        assert_eq!(a[key], b[key]);
      } else if ["fe_mean", "fe_residual", "fe_xb"].contains(key) {
        near(a[key].parse().unwrap(), b[key].parse().unwrap(), key);
      } else {
        assert_eq!(
          a[key].parse::<f64>().unwrap(),
          b[key].parse::<f64>().unwrap(),
          "{key}"
        );
      }
    }
    if a["included_in_fe"] == "true" {
      kept += 1;
      near(
        a["pharvis"].parse().unwrap(),
        a["fe_mean"].parse::<f64>().unwrap() + a["fe_residual"].parse::<f64>().unwrap(),
        "decomposition",
      );
    } else {
      assert_eq!(a["commune"], "1");
      assert_eq!(a["fe_mean"], "");
      assert_eq!(a["fe_residual"], "");
    }
  }
  assert_eq!(kept, 140);
  assert!(!actual.iter().any(|r| r["source_row"] == "149"));
  fs::remove_file(d.path().join(OUTPUT)).unwrap();
  let out = run(&d, "--text");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let text = String::from_utf8(out.stdout).unwrap();
  assert!(
    text.contains("Conditional fixed-effects Poisson (model-based)")
      && text.contains("not population/new-group forecasts")
  );
  assert_eq!(fs::read_to_string(d.path().join(OUTPUT)).unwrap(), csv);
  assert_eq!(
    fs::read_to_string(d.path().join("vietnam_individual.csv")).unwrap(),
    INPUT
  );
}
#[test]
fn invalid_source_rank_and_output_collision_preserve_files() {
  let lines = INPUT.lines().collect::<Vec<_>>();
  let mut variants = vec![format!("{INPUT}{}\n", lines[1])];
  for (column, value) in [
    (0, "-1"),
    (0, "1.00000000000000001"),
    (1, "-1"),
    (1, "0.5"),
    (1, "100001"),
    (12, ""),
    (12, "1.00000000000000001"),
    (2, "NaN"),
    (3, ""),
  ] {
    let mut rs = lines
      .iter()
      .map(|l| l.split(',').collect::<Vec<_>>())
      .collect::<Vec<_>>();
    rs[1][column] = value;
    variants.push(
      rs.iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  for (column, value) in [(1, "0"), (6, "1"), (12, "1")] {
    let mut rs = lines
      .iter()
      .map(|l| l.split(',').collect::<Vec<_>>())
      .collect::<Vec<_>>();
    for row in rs.iter_mut().skip(1) {
      if !row[1].is_empty() {
        row[column] = value;
      }
    }
    variants.push(
      rs.iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  for (i, input) in variants.iter().enumerate() {
    let d = TestDir::new();
    let source = d.path().join("vietnam_individual.csv");
    fs::write(&source, input).unwrap();
    let out = run(&d, "--json");
    assert!(!out.status.success(), "case {i} unexpectedly succeeded");
    assert!(!d.path().join(OUTPUT).exists());
    assert_eq!(fs::read_to_string(&source).unwrap(), *input);
  }
  let d = TestDir::new();
  fs::write(d.path().join("vietnam_individual.csv"), INPUT).unwrap();
  fs::write(d.path().join(OUTPUT), "owner").unwrap();
  assert!(!run(&d, "--json").status.success());
  assert_eq!(fs::read_to_string(d.path().join(OUTPUT)).unwrap(), "owner");
  assert_eq!(
    fs::read_to_string(d.path().join("vietnam_individual.csv")).unwrap(),
    INPUT
  );
}
fn small_session(d: &TestDir) -> Session {
  fs::write(
    d.path().join("small.csv"),
    "id,y,x\na,1,0\na,2,1\nb,2,0\nb,4,1\nsingle,3,7\nzero,0,0\nzero,0,1\na,,1\n",
  )
  .unwrap();
  let mut s = Session::new();
  execute(
    &mut s,
    &format!("use \"{}\"", d.path().join("small.csv").display()),
  );
  s
}
#[test]
fn retained_normalizations_do_not_renormalize_and_prediction_failures_are_atomic() {
  let d = TestDir::new();
  let mut s = small_session(&d);
  execute(&mut s, "xtpoisson y x, fe id(id)");
  let fit = s.state().estimation;
  execute(&mut s, "predict m, n");
  execute(&mut s, "predict r, residuals");
  execute(&mut s, "predict xb, xb");
  assert_eq!(s.state().estimation, fit);
  execute(
    &mut s,
    "sql \"\"\"SELECT CASE WHEN COUNT(*) FILTER (WHERE id='zero' AND m IS NULL AND r IS NULL)!=2 OR COUNT(*) FILTER (WHERE y IS NULL AND m IS NOT NULL AND r IS NULL)!=1 OR MAX(ABS(m-CASE id WHEN 'a' THEN pow(2,x) WHEN 'b' THEN 2*pow(2,x) WHEN 'single' THEN 3 END))>1e-9 THEN error('wrong conditional prediction') ELSE 1 END AS valid FROM active\"\"\"",
  );
  for command in [
    "predict m, n",
    "predict bad, pr",
    "predict bad, posterior_predictive interval",
    "lincom x",
    "test x = 0",
    "estat margins",
  ] {
    let before = s.state();
    let data = execute(&mut s, "head 100");
    assert!(
      s.execute(parse_command(command).unwrap()).is_err(),
      "accepted {command}"
    );
    assert_eq!(s.state(), before);
    assert_eq!(execute(&mut s, "head 100"), data);
  }
  execute(
    &mut s,
    "sql \"\"\"SELECT * FROM active WHERE id='a' AND x=0\"\"\" into subset",
  );
  execute(&mut s, "predict subset_mean, n");
  execute(
    &mut s,
    "sql \"\"\"SELECT CASE WHEN ABS(subset_mean-1)>1e-9 THEN error('renormalized subset') ELSE 1 END AS valid FROM active\"\"\"",
  );
  execute(&mut s, "replace x = x + 1");
  execute(&mut s, "predict changed_mean, n");
  execute(
    &mut s,
    "sql \"\"\"SELECT CASE WHEN ABS(changed_mean-2)>1e-9 THEN error('ignored current X') ELSE 1 END AS valid FROM active\"\"\"",
  );
  execute(
    &mut s,
    "sql \"\"\"SELECT 'unseen' AS id,y,x FROM active\"\"\" into unseen",
  );
  execute(&mut s, "predict unknown_mean, n");
  execute(
    &mut s,
    "sql \"\"\"SELECT CASE WHEN unknown_mean IS NOT NULL THEN error('new-group forecast') ELSE 1 END AS valid FROM active\"\"\"",
  );
  execute(
    &mut s,
    "sql \"\"\"SELECT 1::BIGINT AS id,y,x FROM active\"\"\" into wrong_type",
  );
  let before = s.state();
  assert!(s.execute(parse_command("predict bad, n").unwrap()).is_err());
  assert_eq!(s.state(), before);
  execute(&mut s, "drop id");
  execute(&mut s, "predict no_id_xb, xb");
  let before = s.state();
  assert!(s.execute(parse_command("predict bad, n").unwrap()).is_err());
  assert_eq!(s.state(), before);
  assert_eq!(s.state().estimation, fit);
}
#[test]
fn actual_script_retains_fit_after_reporting_and_successful_or_failed_export() {
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
    for line in include_str!("../examples/mma24p2poiscluster/individual_conditional_fe.td").lines()
    {
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
    execute(
      &mut s,
      "sql \"\"\"SELECT CASE WHEN COUNT(*)!=148 OR MAX(ABS(after_export-fe_mean))>1e-9 OR COUNT(*) FILTER (WHERE after_export IS NULL)!=8 THEN error('lost conditional fit after export') ELSE 1 END AS valid FROM active\"\"\"",
    );
    if collision {
      assert_eq!(fs::read_to_string(&output).unwrap(), "owner");
    }
    assert_eq!(fs::read_to_string(&source).unwrap(), INPUT);
  }
}

#[test]
fn prediction_work_bound_is_atomic() {
  let d = TestDir::new();
  let mut s = small_session(&d);
  execute(&mut s, "xtpoisson y x, fe id(id)");
  execute(
    &mut s,
    "sql \"\"\"SELECT 'a' AS id,1.0 AS y,0.0 AS x FROM range(40001)\"\"\" into large_prediction",
  );
  let before = s.state();
  let data = execute(&mut s, "head 3");
  let error = s
    .execute(parse_command("predict bad, n").unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("prediction rows<=40000"));
  assert_eq!(s.state(), before);
  assert_eq!(execute(&mut s, "head 3"), data);
}

#[test]
fn invalid_keys_count_and_rank_clear_failed_fit_without_data_mutation() {
  let d = TestDir::new();
  for transform in [
    "replace x = 1",
    "replace y = 0.5",
    "sql \"\"\"SELECT NULL::VARCHAR AS id,y,x FROM active\"\"\" into invalid_keys",
  ] {
    let mut s = small_session(&d);
    execute(&mut s, "xtpoisson y x, fe id(id)");
    let before = s.state();
    assert!(parse_command("xtpoisson y x, re id(id) robust").is_err());
    assert_eq!(s.state(), before);
    execute(&mut s, transform);
    let data = execute(&mut s, "head 100");
    assert!(
      s.execute(parse_command("xtpoisson y x, fe id(id)").unwrap())
        .is_err()
    );
    assert!(s.state().estimation.is_none());
    assert_eq!(execute(&mut s, "head 100"), data);
  }
}
