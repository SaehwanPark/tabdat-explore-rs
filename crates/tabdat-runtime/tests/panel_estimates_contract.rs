use std::path::PathBuf;
use tabdat_domain::{EstimateSnapshot, EstimateTableResult, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

const X: &str = "lnexp12m lnhhexp age sex hhsize farm educ";
fn run(s: &mut Session, command: &str) -> ExecutionResult {
  s.execute(parse_command(command).unwrap()).unwrap()
}
fn controls(case: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../../tests/fixtures/mma24_household_mixed_report")
    .join(case)
}
fn fixture(case: &str) -> Session {
  let path = controls(case).join("input.csv");
  let mut s = Session::new();
  run(&mut s, &format!("use \"{}\"", path.display()));
  run(
    &mut s,
    "sql SELECT * FROM active WHERE lnexp12m IS NOT NULL ORDER BY source_row into mixed_cohort",
  );
  s
}
fn table(s: &mut Session, names: &str) -> EstimateTableResult {
  let ExecutionResult::EstimateTable(result) = run(s, &format!("estimates table {names}")) else {
    panic!("table")
  };
  result
}
fn rejected(s: &mut Session, command: &str, message: &str) {
  let before = s.state();
  let error = s
    .execute(parse_command(command).unwrap())
    .unwrap_err()
    .to_string();
  assert!(error.contains(message), "{error}");
  assert_eq!(s.state(), before);
}

fn reference_gates(case: &str, kind: &str, model: &EstimateSnapshot) -> usize {
  let json = serde_json::to_value(model).unwrap();
  let text = std::fs::read_to_string(controls(case).join(format!("{kind}_reference.csv"))).unwrap();
  let mut checked = 0;
  for line in text.lines().skip(1) {
    let cells: Vec<_> = line.split(',').map(|c| c.trim_matches('"')).collect();
    let (section, path, value) = if kind == "fe" {
      (cells[0], cells[1], cells[2])
    } else {
      ("", cells[0], cells[1])
    };
    let mut target = None;
    if kind == "gee" {
      let chosen = if model.cluster_count.is_some() {
        "sandwich"
      } else {
        "working"
      };
      let parts: Vec<_> = path.split('.').collect();
      let index = |name: &str| {
        model
          .covariance_parameter_names
          .iter()
          .position(|n| n == if name == "_cons" { "intercept" } else { name })
          .unwrap()
      };
      if parts[0] == chosen {
        if parts[1] == "covariance" {
          target = Some(format!(
            "/covariance_matrix/{}/{}",
            index(parts[2]),
            index(parts[3])
          ));
        } else if !parts[1].starts_with("wald_") && !parts[2].starts_with("ci_") {
          target = Some(format!(
            "/coefficients/{}/{}",
            index(parts[1]),
            if parts[2] == "estimate" {
              "value"
            } else {
              parts[2]
            }
          ));
        }
      } else if path == "observation_count" {
        target = Some("/observation_count".into());
      } else if path == "group_count" {
        target = Some("/panel/group_count".into());
      }
    } else if kind == "fe" {
      if section == "common"
        && ["/estimate", "/standard_error", "/statistic", "/p_value"].contains(&path)
      {
        target = Some(format!(
          "/coefficients/0/{}",
          if path == "/estimate" {
            "value"
          } else {
            &path[1..]
          }
        ));
      } else if section == "fit" {
        if let Some(rest) = path.strip_prefix("/normalized_level/covariance_matrix/") {
          target = Some(format!("/covariance_matrix/{rest}"));
        } else if let Some(rest) = path.strip_prefix("/normalized_level/parameters/") {
          target = Some(format!("/coefficients/{rest}/value"));
        } else if let Some(rest) = path.strip_prefix("/coefficients/") {
          let (index, field) = rest.split_once('/').unwrap();
          target = Some(format!(
            "/coefficients/{}/{}",
            index.parse::<usize>().unwrap() + 1,
            field
          ));
        } else if path == "/observation_count" || path == "/df_residual" {
          target = Some(path.into());
        } else if path == "/group_count" {
          target = Some("/panel/group_count".into());
        }
      }
    } else if path.starts_with("/covariance_matrix/")
      || (path.starts_with("/coefficients/") && !path.contains("/ci_"))
      || path == "/observation_count"
      || path == "/df_residual"
    {
      target = Some(path.into());
    } else if path == "/group_count" {
      target = Some("/panel/group_count".into());
    } else if path == "/ml/log_likelihood" {
      target = Some("/log_likelihood".into());
    }
    if let Some(target) = target {
      let actual = json.pointer(&target).unwrap().as_f64().unwrap();
      let expected: f64 = value.parse().unwrap();
      assert!(actual.is_finite() && expected.is_finite());
      assert!(
        (actual - expected).abs() <= 1e-9,
        "{case} {kind} {path}: {actual} != {expected}"
      );
      if path.ends_with("p_value") {
        if expected == 0.0 {
          assert_eq!(actual, 0.0);
        } else {
          assert!(
            (actual - expected).abs() <= expected.abs() * 1e-8,
            "{case} {kind} {path}"
          );
        }
      }
      checked += 1;
    }
  }
  checked
}

#[test]
fn panel_reports_copy_native_inference_align_common_intercepts_and_preserve_ownership() {
  for (case, n, g) in [
    ("balanced", 192, 32),
    ("unbalanced", 199, 32),
    ("interior", 179, 16),
  ] {
    let mut s = fixture(case);
    let ExecutionResult::GaussianGee(gee) = run(
      &mut s,
      &format!("xtgee {X}, family(gaussian) link(identity) corr(exchangeable) id(commune)"),
    ) else {
      panic!("GEE")
    };
    rejected(&mut s, "estimates store ambiguous", "covariance(working)");
    run(&mut s, "estimates store pa, covariance(working)");
    run(&mut s, "estimates store pa_robust, covariance(sandwich)");
    let stored = table(&mut s, "pa pa_robust");
    for (saved, original) in stored.models.iter().zip([&gee.working, &gee.sandwich]) {
      assert_eq!(saved.covariance_matrix, original.covariance);
      for (i, (a, b)) in saved
        .coefficients
        .iter()
        .zip(&original.coefficients)
        .enumerate()
      {
        assert_eq!(a.name, if i == 0 { "intercept" } else { &b.name });
        assert_eq!(a.value, b.value);
        assert_eq!(a.standard_error, Some(b.standard_error));
        assert_eq!(a.statistic, Some(b.statistic));
        assert_eq!(a.p_value, Some(b.p_value));
      }
      assert_eq!(saved.inference_degrees_of_freedom, None);
      assert_eq!(saved.panel.as_ref().unwrap().time_variable, None);
    }
    assert_eq!(stored.models[0].cluster_count, None);
    assert_eq!(stored.models[1].cluster_count, Some(g));
    assert_eq!(stored.models[1].covariance, "unit_sandwich_uncorrected");
    assert_eq!(reference_gates(case, "gee", &stored.models[0]), 79);
    assert_eq!(reference_gates(case, "gee", &stored.models[1]), 79);
    let ExecutionResult::FixedEffects(fe) = run(
      &mut s,
      &format!("xtreg {X}, fe id(commune) time(source_row)"),
    ) else {
      panic!("FE")
    };
    run(&mut s, "estimates store fe");
    let saved = table(&mut s, "fe").models.remove(0);
    assert_eq!(saved.coefficients[0].value, fe.result.common_intercept);
    assert_eq!(saved.coefficients[1..], fe.result.within.coefficients);
    assert_eq!(
      saved.covariance_matrix,
      fe.result
        .normalized_level
        .as_ref()
        .unwrap()
        .covariance
        .matrix
    );
    assert_eq!(saved.inference_degrees_of_freedom, Some(n - g - 6));
    assert_eq!(
      saved.panel.as_ref().unwrap().intercept_convention,
      "observation_weighted_zero_effects"
    );
    for (option, name) in [
      ("re vcmethod(equal_group_between)", "refgls"),
      ("mle", "remle"),
    ] {
      let ExecutionResult::RandomEffects(fit) = run(
        &mut s,
        &format!("xtreg {X}, {option} id(commune) time(source_row)"),
      ) else {
        panic!("RE")
      };
      let before = s.state();
      run(&mut s, &format!("estimates store {name}"));
      assert_eq!(s.state().estimation, before.estimation);
      let saved = table(&mut s, name).models.remove(0);
      assert_eq!(saved.coefficients, fit.result.gls.coefficients);
      assert_eq!(saved.covariance_matrix, fit.result.gls.covariance.matrix);
      assert_eq!(
        saved.log_likelihood,
        fit.result.ml.as_ref().map(|m| m.log_likelihood)
      );
      assert_eq!(saved.inference_degrees_of_freedom, None);
    }
    let before = s.state();
    let report = table(&mut s, "pa fe refgls remle");
    assert_eq!(s.state(), before);
    assert_eq!(
      report
        .models
        .iter()
        .map(|m| m.name.as_str())
        .collect::<Vec<_>>(),
      ["pa", "fe", "refgls", "remle"]
    );
    assert_eq!(
      report.parameter_names,
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
    for m in &report.models {
      assert_eq!(m.observation_count, n);
      assert_eq!(m.panel.as_ref().unwrap().group_count, g);
      assert_eq!(m.sample.total_observations, n);
      assert_eq!(m.sample.dropped_observations, 0);
      assert_eq!(m.r_squared, None);
      assert_eq!(m.root_mse, None);
      let json = serde_json::to_value(m).unwrap();
      assert_eq!(
        serde_json::from_value::<EstimateSnapshot>(json.clone()).unwrap(),
        *m
      );
      let mut legacy = json;
      legacy.as_object_mut().unwrap().remove("panel");
      assert_eq!(
        serde_json::from_value::<EstimateSnapshot>(legacy)
          .unwrap()
          .panel,
        None
      );
    }
    assert_eq!(reference_gates(case, "fe", &report.models[1]), 87);
    assert_eq!(reference_gates(case, "re_gls", &report.models[2]), 80);
    assert_eq!(reference_gates(case, "re_ml", &report.models[3]), 81);
    assert_eq!(report.models[0].log_likelihood, None);
    assert_eq!(report.models[1].log_likelihood, None);
    assert_eq!(report.models[2].log_likelihood, None);
    assert!(report.models[3].log_likelihood.is_some());
    run(&mut s, "sql SELECT 1 AS unrelated into replacement");
    let before = s.state();
    assert_eq!(table(&mut s, "pa fe refgls remle"), report);
    assert_eq!(s.state(), before);
  }
}

#[test]
fn covariance_choice_and_unsupported_panel_covariance_fail_atomically() {
  let mut s = fixture("interior");
  run(
    &mut s,
    &format!("xtreg {X}, fe id(commune) time(source_row)"),
  );
  rejected(
    &mut s,
    "estimates store wrong, covariance(working)",
    "only available for Gaussian GEE",
  );
  run(&mut s, "estimates store fe");
  rejected(&mut s, "estimates store fe", "already stored");
  run(
    &mut s,
    &format!("xtreg {X}, fe id(commune) time(source_row) cluster(commune)"),
  );
  rejected(&mut s, "estimates store clustered_fe", "model-based");
  run(
    &mut s,
    &format!("xtreg {X}, re id(commune) time(source_row) cluster(commune)"),
  );
  rejected(&mut s, "estimates store clustered_re", "model-based");
  let ExecutionResult::RandomEffects(swamy) = run(
    &mut s,
    &format!("xtreg {X}, re id(commune) time(source_row)"),
  ) else {
    panic!()
  };
  run(&mut s, "estimates store swamy");
  let saved = table(&mut s, "swamy").models.remove(0);
  assert_eq!(saved.estimator, "swamy_arora_random_effects_gls");
  assert_eq!(saved.coefficients, swamy.result.gls.coefficients);
  assert_eq!(saved.covariance_matrix, swamy.result.gls.covariance.matrix);
  assert_eq!(saved.log_likelihood, None);
  run(&mut s, "regress lnexp12m lnhhexp");
  rejected(
    &mut s,
    "estimates store linear, covariance(sandwich)",
    "only available for Gaussian GEE",
  );
  run(&mut s, "estimates store linear");
  let model = table(&mut s, "linear").models.remove(0);
  assert_eq!(model.panel, None);
  assert!(
    !serde_json::to_value(model)
      .unwrap()
      .as_object()
      .unwrap()
      .contains_key("panel")
  );
  run(
    &mut s,
    "sql SELECT source_row, lnexp12m, lnhhexp AS intercept, age, sex, hhsize, farm, educ, commune FROM active into aliased",
  );
  run(
    &mut s,
    "xtgee lnexp12m intercept age sex hhsize farm educ, family(gaussian) link(identity) corr(exchangeable) id(commune)",
  );
  rejected(
    &mut s,
    "estimates store ambiguous_gee, covariance(working)",
    "coefficient names are ambiguous",
  );
  run(
    &mut s,
    "xtreg lnexp12m intercept age sex hhsize farm educ, fe id(commune) time(source_row)",
  );
  rejected(
    &mut s,
    "estimates store ambiguous_fe",
    "coefficient names are ambiguous",
  );
}
