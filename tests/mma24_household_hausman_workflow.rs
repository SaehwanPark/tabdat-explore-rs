mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
};
use tabdat_domain::ExecutionResult;
use tabdat_language::{parse_command, script::parse_script};
use tabdat_runtime::Session;

const SCRIPT: &str = include_str!("../examples/mma24p1olscluster/household_hausman.td");
const OUTPUT: &str = "household_hausman_predictions.csv";
fn fixture(case: &str, kind: &str) -> String {
  fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
    "crates/tabdat-stats/tests/fixtures/panel_joint_equal_group_hausman/{case}_{kind}.csv"
  )))
  .unwrap()
}
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma24p1olscluster/household_hausman.td"),
    )
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let keys: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let values: Vec<_> = line.split(',').collect();
      assert_eq!(keys.len(), values.len());
      keys.iter().copied().zip(values).collect()
    })
    .collect()
}
fn close(a: f64, b: f64, p: bool) {
  let tolerance = if p { b.abs() * 1e-8 } else { 1e-9 };
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= tolerance,
    "{a} != {b}, tolerance {tolerance}"
  );
}
fn check(case: &str, balanced: bool) {
  let dir = TestDir::new();
  let input = fixture(case, "input");
  fs::write(dir.path().join("vietnam.csv"), &input).unwrap();
  let out = run(&dir, "--json");
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
  assert_eq!(events.len(), 11);
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  assert_eq!(
    events[4]["result_type"],
    if balanced {
      "RandomEffectsResult"
    } else {
      "UnbalancedRandomEffectsGlsResult"
    }
  );
  assert_eq!(events[5]["result_type"], "JointPanelHausmanResult");
  let h = &events[5]["data"];
  let fit = &events[4]["data"];
  assert_eq!(
    h["covariance_scale_convention"],
    "separate_native_fe_and_re_residual_scales"
  );
  assert_eq!(h["re_component_method"], "equal_group_between");
  assert_eq!(h["reference_distribution"], "chi_square");
  assert_eq!(h["covariance"], "model_based_nonrobust");
  assert_eq!(h["test"], "joint_panel_fe_re_covariance_difference");
  assert!(
    h["assumptions"]
      .as_str()
      .unwrap()
      .contains("not cluster-robust")
  );
  let reference = fixture(case, "reference");
  let refs = rows(&reference);
  assert_eq!(refs.len(), 139);
  for r in refs {
    close(
      h.pointer(r["path"]).unwrap().as_f64().unwrap(),
      r["value"].parse().unwrap(),
      r["path"] == "/p_value",
    );
  }
  assert_eq!(h["candidate_observation_count"], h["observation_count"]);
  assert_eq!(h["dropped_observation_count"], 0);
  for i in 0..6 {
    assert_eq!(h["re_coefficients"][i], fit["coefficients"][i + 1]["value"]);
    for j in 0..6 {
      assert_eq!(
        h["re_covariance"][i][j],
        fit["covariance_matrix"][i + 1][j + 1]
      );
    }
  }
  let actual = fs::read_to_string(dir.path().join(OUTPUT)).unwrap();
  let expected = fixture(case, "predictions");
  let (a, b) = (rows(&actual), rows(&expected));
  assert_eq!(a.len(), b.len());
  assert_eq!(a.len(), if balanced { 192 } else { 199 });
  for (a, b) in a.iter().zip(b) {
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
  }
  fs::remove_file(dir.path().join(OUTPUT)).unwrap();
  let out = run(&dir, "--text");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let text = String::from_utf8(out.stdout).unwrap();
  assert!(text.contains(&format!(
    "Original-design candidate N = {}; excluded = 0",
    a.len()
  )));
  for label in [
    "Classical joint panel FE–RE Hausman",
    "FE minus RE covariance",
    "not cluster-robust",
    "separate_native_fe_and_re_residual_scales",
  ] {
    assert!(text.contains(label), "missing {label}");
  }
  let line = text
    .lines()
    .find(|l| l.starts_with("Chi-square(6) = "))
    .unwrap();
  let (stat, p) = line
    .strip_prefix("Chi-square(6) = ")
    .unwrap()
    .split_once("; asymptotic p = ")
    .unwrap();
  close(
    stat.parse().unwrap(),
    h["statistic"].as_f64().unwrap(),
    false,
  );
  close(p.parse().unwrap(), h["p_value"].as_f64().unwrap(), true);
  assert_eq!(fs::read_to_string(dir.path().join(OUTPUT)).unwrap(), actual);
  assert_eq!(
    fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
    input
  );
}
#[test]
fn balanced_script_full_joint_and_text_match_independent_r() {
  check("balanced", true);
}
#[test]
fn unbalanced_singleton_script_cohort_and_full_joint_match_independent_r() {
  check("unbalanced", false);
}

#[test]
fn current_data_reporting_and_export_collision_do_not_replace_owned_re() {
  for collision in [false, true] {
    let dir = TestDir::new();
    let input = fixture("unbalanced", "input");
    let source = dir.path().join("vietnam.csv");
    fs::write(&source, &input).unwrap();
    let output = dir.path().join(OUTPUT);
    if collision {
      fs::write(&output, "owner").unwrap();
    }
    let mut session = Session::new();
    let mut retained = None;
    let mut failures = 0;
    for command in parse_script(SCRIPT, Path::new("household_hausman.td")).unwrap() {
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
      } else {
        result.unwrap();
      }
      if command.text.starts_with("xtreg ") {
        retained = session.state().estimation;
        assert!(retained.is_some());
        let ExecutionResult::JointPanelHausman(expected) = session
          .execute(parse_command("estat hausman").unwrap())
          .unwrap()
        else {
          panic!()
        };
        session
          .execute(
            parse_command("sql \"\"\"SELECT 999 AS unrelated\"\"\" into hausman_current_probe")
              .unwrap(),
          )
          .unwrap();
        let before = session.state();
        let ExecutionResult::JointPanelHausman(actual) = session
          .execute(parse_command("estat hausman").unwrap())
          .unwrap()
        else {
          panic!()
        };
        assert_eq!(actual, expected);
        assert_eq!(session.state(), before);
        session
          .execute(parse_command("use household_hausman_cohort").unwrap())
          .unwrap();
      } else if retained.is_some() {
        assert_eq!(session.state().estimation, retained);
      }
    }
    assert_eq!(failures, usize::from(collision));
    session
      .execute(parse_command("predict hausman_after, xb").unwrap())
      .unwrap();
    assert_eq!(session.state().estimation, retained);
    session.execute(parse_command("sql \"\"\"SELECT CASE WHEN COUNT(*)!=199 OR MAX(ABS(hausman_after-re_xb))>1e-12 OR MAX(ABS(lnexp12m-re_xb-re_residual))>1e-12 THEN error('changed owned RE') ELSE 1 END AS valid FROM active\"\"\"").unwrap()).unwrap();
    if collision {
      assert_eq!(fs::read_to_string(output).unwrap(), "owner");
    }
    assert_eq!(fs::read_to_string(source).unwrap(), input);
  }
}
#[test]
fn actual_indefinite_covariance_fit_fails_without_mutating_re_or_data() {
  let dir = TestDir::new();
  let source = dir.path().join("vietnam.csv");
  fs::write(
    &source,
    include_str!("fixtures/mma24_household_re/interior_input.csv"),
  )
  .unwrap();
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use \"{}\"", source.display())).unwrap())
    .unwrap();
  session.execute(parse_command("xtreg lnexp12m lnhhexp age sex hhsize farm educ, re id(commune) time(source_row) vcmethod(equal_group_between)").unwrap()).unwrap();
  let signature = session
    .execute(parse_command("datasignature").unwrap())
    .unwrap();
  let before = session.state();
  let error = session
    .execute(parse_command("estat hausman").unwrap())
    .unwrap_err();
  assert!(
    error.to_string().contains("nonpositive definite"),
    "{error}"
  );
  assert_eq!(session.state(), before);
  assert_eq!(
    session
      .execute(parse_command("datasignature").unwrap())
      .unwrap(),
    signature
  );
}
#[test]
fn invalid_source_and_fit_failures_preserve_source_and_output_owner() {
  let base = fixture("unbalanced", "input");
  let data: Vec<Vec<String>> = base
    .lines()
    .map(|l| l.split(',').map(str::to_owned).collect())
    .collect();
  let mut cases = vec![format!("{base}{}\n", base.lines().nth(1).unwrap())];
  for (row, col, value) in [
    (1, 0, "-1"),
    (1, 0, "1.00000000000000001"),
    (1, 8, ""),
    (1, 8, "1.00000000000000001"),
    (1, 1, "NaN"),
    (4, 6, ""),
  ] {
    let mut bad = data.clone();
    bad[row][col] = value.into();
    cases.push(
      bad
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  for col in [6, 8] {
    let mut bad = data.clone();
    for row in bad.iter_mut().skip(1) {
      row[col] = "1".into();
    }
    cases.push(
      bad
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  cases.push(include_str!("fixtures/mma24_household_re/interior_input.csv").into());
  for (i, input) in cases.iter().enumerate() {
    let dir = TestDir::new();
    fs::write(dir.path().join("vietnam.csv"), input).unwrap();
    fs::write(dir.path().join(OUTPUT), "owner").unwrap();
    let out = run(&dir, "--json");
    assert_eq!(out.status.code(), Some(1), "invalid case {i}");
    assert!(!String::from_utf8_lossy(&out.stdout).contains("JointPanelHausmanResult"));
    assert_eq!(
      fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
      "owner"
    );
    assert_eq!(
      fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
      *input
    );
  }
}
