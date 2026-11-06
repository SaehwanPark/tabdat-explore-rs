mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

const OUTPUT: &str = "household_re_ml_predictions.csv";
const SCRIPT: &str = include_str!("../examples/mma24p1olscluster/household_re_ml.td");
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma24p1olscluster/household_re_ml.td"),
    )
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let header: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|l| {
      let values: Vec<_> = l.split(',').collect();
      assert_eq!(header.len(), values.len());
      header.iter().copied().zip(values).collect()
    })
    .collect()
}
fn close(a: f64, b: f64, p: bool) {
  assert!(a.is_finite() && b.is_finite());
  let tol = if p { b.abs() * 1e-8 } else { 1e-9 };
  assert!((a - b).abs() <= tol, "{a} != {b}, tol {tol}");
}
fn check(input: &str, reference: &str, predictions: &str, boundary: bool, null_boundary: bool) {
  let dir = TestDir::new();
  fs::write(dir.path().join("vietnam.csv"), input).unwrap();
  let output = run(&dir, "--json");
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
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let fit = &events
    .iter()
    .find(|e| e["result_type"] == "UnbalancedRandomEffectsMlResult")
    .unwrap()["data"];
  assert_eq!(fit["observation_count"], 179);
  assert_eq!(fit["group_count"], 16);
  assert_eq!(fit["minimum_group_size"], 1);
  assert_eq!(fit["time_variable"], "source_row");
  assert_eq!(fit["reference_distribution"], "normal");
  assert_eq!(
    fit["covariance_parameter_names"],
    json!([
      "intercept",
      "lnhhexp",
      "age",
      "sex",
      "hhsize",
      "farm",
      "educ"
    ])
  );
  assert_eq!(fit["ml"]["unit_variance_boundary"], boundary);
  assert_eq!(
    fit["ml"]["likelihood_ratio_p_value"].is_null(),
    boundary || null_boundary
  );
  assert_eq!(
    fit["ml"]["likelihood_ratio_unavailable_reason"].is_string(),
    boundary || null_boundary
  );
  if boundary {
    assert_eq!(fit["sigma_u_squared"], 0.0);
    assert_eq!(fit["theta_max"], 0.0);
    assert!(fit["ml"]["variance_component_inference"]["sigma_u_squared"].is_null());
    assert_eq!(fit["ml"]["unit_variance_test"]["p_value"], 1.0);
  } else {
    assert!(fit["sigma_u_squared"].as_f64().unwrap() > 0.0);
  }
  let refs = rows(reference);
  assert_eq!(
    refs.len(),
    210 - if boundary { 22 } else { 0 } - if boundary || null_boundary { 1 } else { 0 }
  );
  for r in refs {
    let actual = fit.pointer(r["path"]).unwrap().as_f64().unwrap();
    let expected: f64 = r["value"].parse().unwrap();
    if r["path"] == "/ml/profile_score" && !boundary {
      assert!(actual.abs() < 1e-8 && expected.abs() < 1e-8);
    } else {
      close(actual, expected, r["path"].ends_with("p_value"));
    }
  }
  let csv = fs::read_to_string(dir.path().join(OUTPUT)).unwrap();
  let (a, b) = (rows(&csv), rows(predictions));
  assert_eq!((a.len(), b.len()), (179, 179));
  for (a, b) in a.iter().zip(b.iter()) {
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for key in b.keys() {
      let av: f64 = a[key].parse().unwrap();
      let bv: f64 = b[key].parse().unwrap();
      if ["ml_xb", "ml_residual"].contains(key) {
        close(av, bv, false)
      } else {
        assert_eq!(av, bv, "{key}")
      }
    }
    close(
      a["lnexp12m"].parse::<f64>().unwrap(),
      a["ml_xb"].parse::<f64>().unwrap() + a["ml_residual"].parse::<f64>().unwrap(),
      false,
    );
  }
  assert!(!a.iter().any(|r| r["source_row"] == "4"));
  fs::remove_file(dir.path().join(OUTPUT)).unwrap();
  let out = run(&dir, "--text");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let readable = String::from_utf8(out.stdout).unwrap();
  assert!(readable.contains("source_row is a unique key, not time"));
  let lines: Vec<_> = readable.lines().collect();
  let start = lines
    .iter()
    .position(|l| l.starts_with("Parameter\tCoefficient\t"))
    .unwrap()
    + 1;
  for (line, c) in lines[start..start + 7]
    .iter()
    .zip(fit["coefficients"].as_array().unwrap())
  {
    let cells: Vec<_> = line.split('\t').collect();
    assert_eq!(cells.len(), 7);
    assert_eq!(cells[0], c["name"].as_str().unwrap());
    for (index, key) in [
      "value",
      "standard_error",
      "statistic",
      "p_value",
      "ci_lower",
      "ci_upper",
    ]
    .iter()
    .enumerate()
    {
      assert_eq!(
        cells[index + 1].parse::<f64>().unwrap(),
        c[*key].as_f64().unwrap(),
        "{key}"
      )
    }
  }
  assert_eq!(fs::read_to_string(dir.path().join(OUTPUT)).unwrap(), csv);
  assert_eq!(
    fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
    input
  );
}
#[test]
fn unbalanced_interior_matches_full_r_information_and_predictions() {
  check(
    include_str!("fixtures/mma24_household_re/interior_input.csv"),
    include_str!("fixtures/mma24_household_re_ml/interior_reference.csv"),
    include_str!("fixtures/mma24_household_re_ml/interior_predictions.csv"),
    false,
    false,
  )
}
#[test]
fn restricted_null_boundary_withholds_slope_lr_reference() {
  check(
    include_str!("fixtures/mma24_household_re/boundary_input.csv"),
    include_str!("fixtures/mma24_household_re_ml/restricted_boundary_reference.csv"),
    include_str!("fixtures/mma24_household_re_ml/restricted_boundary_predictions.csv"),
    false,
    true,
  )
}
#[test]
fn zero_unit_variance_has_constrained_information_and_boundary_test() {
  check(
    include_str!("fixtures/mma24_household_re_ml/zero_boundary_input.csv"),
    include_str!("fixtures/mma24_household_re_ml/zero_boundary_reference.csv"),
    include_str!("fixtures/mma24_household_re_ml/zero_boundary_predictions.csv"),
    true,
    true,
  )
}

#[test]
fn retained_model_and_source_survive_export_success_and_collision() {
  for collision in [false, true] {
    let d = TestDir::new();
    let source = d.path().join("vietnam.csv");
    let output = d.path().join(OUTPUT);
    let input = include_str!("fixtures/mma24_household_re/interior_input.csv");
    fs::write(&source, input).unwrap();
    if collision {
      fs::write(&output, "owner").unwrap();
    }
    let mut session = Session::new();
    let mut retained = None;
    for line in SCRIPT.lines() {
      if line.starts_with('#') || line.is_empty() {
        continue;
      }
      let cmd = if line == "use vietnam.csv" {
        format!("use \"{}\"", source.display())
      } else if line.starts_with("export ") {
        format!("export \"{}\"", output.display())
      } else {
        line.to_owned()
      };
      let result = session.execute(parse_command(&cmd).unwrap());
      if collision && line.starts_with("export ") {
        assert!(result.is_err());
      } else {
        result.unwrap();
      }
      if line.starts_with("xtreg ") {
        retained = session.state().estimation;
        assert!(retained.is_some())
      } else if retained.is_some() {
        assert_eq!(session.state().estimation, retained)
      }
    }
    session
      .execute(parse_command("predict ml_after, xb").unwrap())
      .unwrap();
    assert_eq!(session.state().estimation, retained);
    session.execute(parse_command("sql \"\"\"SELECT CASE WHEN COUNT(*)!=179 OR MAX(ABS(ml_after-ml_xb))>1e-12 OR MAX(ABS(lnexp12m-ml_xb-ml_residual))>1e-12 THEN error('wrong retained ML prediction') ELSE 1 END AS valid FROM active\"\"\"").unwrap()).unwrap();
    if collision {
      assert_eq!(fs::read_to_string(&output).unwrap(), "owner")
    } else {
      assert_eq!(rows(&fs::read_to_string(&output).unwrap()).len(), 179)
    }
    assert_eq!(fs::read_to_string(source).unwrap(), input);
  }
}
#[test]
fn invalid_sources_and_fit_failures_preserve_source_and_owner() {
  let input = include_str!("fixtures/mma24_household_re/interior_input.csv");
  let lines: Vec<_> = input.lines().collect();
  let mut variants = vec![format!("{input}{}\n", lines[1])];
  for (col, value) in [
    (0, "-1"),
    (0, "1.00000000000000001"),
    (8, ""),
    (8, "1.00000000000000001"),
    (1, "NaN"),
    (2, "NaN"),
  ] {
    let mut rs: Vec<Vec<&str>> = lines.iter().map(|l| l.split(',').collect()).collect();
    rs[1][col] = value;
    variants.push(
      rs.iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  for col in [6, 8] {
    let mut rs: Vec<Vec<&str>> = lines.iter().map(|l| l.split(',').collect()).collect();
    for r in rs.iter_mut().skip(1) {
      r[col] = "1";
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
    fs::write(d.path().join("vietnam.csv"), input).unwrap();
    fs::write(d.path().join(OUTPUT), "owner").unwrap();
    let out = run(&d, "--json");
    assert!(!out.status.success(), "invalid case {i} succeeded");
    assert_eq!(fs::read_to_string(d.path().join(OUTPUT)).unwrap(), "owner");
    assert_eq!(
      fs::read_to_string(d.path().join("vietnam.csv")).unwrap(),
      *input
    );
  }
}
