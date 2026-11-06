mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

const OUTPUT: &str = "household_re_gls_predictions.csv";
const EQUAL_OUTPUT: &str = "household_equal_group_re_predictions.csv";
fn run(dir: &TestDir, mode: &str, equal_group: bool) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(if equal_group {
        "examples/mma24p1olscluster/household_equal_group_re.td"
      } else {
        "examples/mma24p1olscluster/household_re_gls.td"
      }),
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
fn check(input: &str, reference: &str, predictions: &str, boundary: bool, equal_group: bool) {
  let output = if equal_group { EQUAL_OUTPUT } else { OUTPUT };
  let dir = TestDir::new();
  fs::write(dir.path().join("vietnam.csv"), input).unwrap();
  let out = run(&dir, "--json", equal_group);
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let events: Vec<Value> = String::from_utf8(out.stdout)
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
    .find(|e| e["result_type"] == "UnbalancedRandomEffectsGlsResult")
    .unwrap()["data"];
  assert_eq!(fit["observation_count"], 179);
  assert_eq!(fit["group_count"], 16);
  assert_eq!(fit["minimum_group_size"], 1);
  assert_eq!(fit["time_variable"], "source_row");
  assert_eq!(
    fit["variance_component_method"],
    if equal_group {
      "equal_group_between"
    } else {
      "unbalanced_swamy_arora"
    }
  );
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
  if boundary {
    assert_eq!(fit["sigma_u_squared"], 0.0);
    assert!(fit["raw_sigma_u_squared"].as_f64().unwrap() < 0.0);
    assert_eq!(fit["theta_max"], 0.0)
  } else {
    assert!(fit["sigma_u_squared"].as_f64().unwrap() > 0.0);
    assert!(fit["theta_max"].as_f64().unwrap() > fit["theta_min"].as_f64().unwrap())
  }
  let refs = rows(reference);
  assert_eq!(refs.len(), 108);
  for r in refs {
    close(
      fit.pointer(r["path"]).unwrap().as_f64().unwrap(),
      r["value"].parse().unwrap(),
      r["path"].ends_with("p_value"),
    );
  }
  let text = fs::read_to_string(dir.path().join(output)).unwrap();
  let (a, b) = (rows(&text), rows(predictions));
  assert_eq!((a.len(), b.len()), (179, 179));
  for (a, b) in a.iter().zip(b.iter()) {
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for key in b.keys() {
      let av: f64 = a[key].parse().unwrap();
      let bv: f64 = b[key].parse().unwrap();
      if ["re_xb", "re_residual"].contains(key) {
        close(av, bv, false)
      } else {
        assert_eq!(av, bv, "{key}")
      }
    }
    close(
      a["lnexp12m"].parse::<f64>().unwrap(),
      a["re_xb"].parse::<f64>().unwrap() + a["re_residual"].parse::<f64>().unwrap(),
      false,
    );
  }
  assert!(!a.iter().any(|r| r["source_row"] == "4"));
  fs::remove_file(dir.path().join(output)).unwrap();
  let out = run(&dir, "--text", equal_group);
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let readable = String::from_utf8(out.stdout).unwrap();
  assert!(readable.contains("source_row is a unique key, not time"));
  assert!(readable.contains(if equal_group {
    "Equal-group between MSE with mean inverse-size correction"
  } else {
    "Swamy-Arora"
  }));
  if equal_group {
    assert!(!readable.contains("Swamy-Arora"));
  }
  assert_eq!(fs::read_to_string(dir.path().join(output)).unwrap(), text);
  assert_eq!(
    fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
    input
  );
}
#[test]
fn unbalanced_interior_matches_r_plm_and_qr() {
  check(
    include_str!("fixtures/mma24_household_re/interior_input.csv"),
    include_str!("fixtures/mma24_household_re/interior_reference.csv"),
    include_str!("fixtures/mma24_household_re/interior_predictions.csv"),
    false,
    false,
  )
}
#[test]
fn negative_raw_component_is_truncated_and_matches_r_plm() {
  check(
    include_str!("fixtures/mma24_household_re/boundary_input.csv"),
    include_str!("fixtures/mma24_household_re/boundary_reference.csv"),
    include_str!("fixtures/mma24_household_re/boundary_predictions.csv"),
    true,
    false,
  )
}
#[test]
fn retained_re_state_survives_reporting_and_export_collision() {
  for (script, output_name) in [
    (
      include_str!("../examples/mma24p1olscluster/household_re_gls.td"),
      OUTPUT,
    ),
    (
      include_str!("../examples/mma24p1olscluster/household_equal_group_re.td"),
      EQUAL_OUTPUT,
    ),
  ] {
    for collision in [false, true] {
      let dir = TestDir::new();
      let source = dir.path().join("vietnam.csv");
      let output = dir.path().join(output_name);
      let input = include_str!("fixtures/mma24_household_re/interior_input.csv");
      fs::write(&source, input).unwrap();
      if collision {
        fs::write(&output, "owner").unwrap();
      }
      let mut session = Session::new();
      let mut retained = None;
      for line in script.lines() {
        if line.starts_with('#') || line.is_empty() {
          continue;
        }
        let command = if line == "use vietnam.csv" {
          format!("use \"{}\"", source.display())
        } else if line.starts_with("export ") {
          format!("export \"{}\"", output.display())
        } else {
          line.to_owned()
        };
        let result = session.execute(parse_command(&command).unwrap());
        if collision && line.starts_with("export ") {
          assert!(result.is_err());
        } else {
          result.unwrap();
        }
        if line.starts_with("xtreg ") {
          retained = session.state().estimation;
          assert!(retained.is_some());
        } else if retained.is_some() {
          assert_eq!(session.state().estimation, retained);
        }
      }
      session
        .execute(parse_command("predict re_after, xb").unwrap())
        .unwrap();
      assert_eq!(session.state().estimation, retained);
      session.execute(parse_command("sql \"\"\"SELECT CASE WHEN COUNT(*)!=179 OR MAX(ABS(re_after-re_xb))>1e-12 OR MAX(ABS(lnexp12m-re_xb-re_residual))>1e-12 THEN error('wrong retained RE prediction') ELSE 1 END AS valid FROM active\"\"\"").unwrap()).unwrap();
      if collision {
        assert_eq!(fs::read_to_string(&output).unwrap(), "owner");
      } else {
        assert_eq!(rows(&fs::read_to_string(&output).unwrap()).len(), 179);
      }
      assert_eq!(fs::read_to_string(&source).unwrap(), input);
    }
  }
}

#[test]
fn equal_group_interior_matches_independent_qr() {
  check(
    include_str!("fixtures/mma24_household_re/interior_input.csv"),
    include_str!("fixtures/mma24_household_equal_group_re/interior_reference.csv"),
    include_str!("fixtures/mma24_household_equal_group_re/interior_predictions.csv"),
    false,
    true,
  )
}

#[test]
fn equal_group_negative_raw_component_is_truncated() {
  check(
    include_str!("fixtures/mma24_household_re/boundary_input.csv"),
    include_str!("fixtures/mma24_household_equal_group_re/boundary_reference.csv"),
    include_str!("fixtures/mma24_household_equal_group_re/boundary_predictions.csv"),
    true,
    true,
  )
}

#[test]
fn invalid_source_and_export_collision_preserve_owner() {
  for equal_group in [false, true] {
    check_invalid_source(equal_group);
  }
}
fn check_invalid_source(equal_group: bool) {
  let output = if equal_group { EQUAL_OUTPUT } else { OUTPUT };
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
    let out = run(&d, "--json", equal_group);
    assert!(!out.status.success(), "invalid case {i} succeeded");
    assert!(!d.path().join(output).exists());
    assert_eq!(
      fs::read_to_string(d.path().join("vietnam.csv")).unwrap(),
      *input
    );
  }
  let d = TestDir::new();
  fs::write(d.path().join("vietnam.csv"), input).unwrap();
  fs::write(d.path().join(output), "owner").unwrap();
  assert!(!run(&d, "--json", equal_group).status.success());
  assert_eq!(fs::read_to_string(d.path().join(output)).unwrap(), "owner");
  assert_eq!(
    fs::read_to_string(d.path().join("vietnam.csv")).unwrap(),
    input
  );
}
