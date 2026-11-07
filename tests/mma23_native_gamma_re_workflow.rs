mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
  process::Command,
};
use tabdat_domain::EstimationState;
use tabdat_language::{parse_command, script::parse_script};
use tabdat_runtime::Session;
const SCRIPT: &str = include_str!("../examples/mma23p1pannonlin/native_gamma_re.td");
const EXPORT: &str = "native_gamma_re_predictions.csv";
fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn prepare(dir: &TestDir) {
  fs::write(
    dir.path().join("source.asc"),
    include_str!("fixtures/mma23_conditional_fe/source.asc"),
  )
  .unwrap();
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
    .arg(root().join("examples/mma23p1pannonlin/native_gamma_re.td"))
    .output()
    .unwrap()
}
fn events(output: &std::process::Output) -> Vec<Value> {
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}
fn close(a: f64, b: f64, label: &str) {
  let tolerance = if label.starts_with("p_") && b > 0.0 {
    1e-9 * b
  } else {
    1e-9 + 1e-10 * b.abs()
  };
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= tolerance,
    "{label}: {a} != {b}"
  );
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let header: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let values: Vec<_> = line.split(',').collect();
      assert_eq!(header.len(), values.len());
      header.iter().copied().zip(values).collect()
    })
    .collect()
}
fn check(dir: &TestDir, es: &[Value]) {
  assert_eq!(
    es.iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    vec![
      "LoadResult",
      "IsidResult",
      "TableResult",
      "TableResult",
      "SqlCreateResult",
      "GammaPoissonResult",
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
  assert_eq!(
    es[3]["data"],
    json!({"headers":["five_year_firms"],"rows":[[12]]})
  );
  assert_eq!(es[10]["data"]["rows"].as_array().unwrap().len(), 1);
  let d = &es[5]["data"];
  let co = d["coefficients"].as_array().unwrap();
  let ln = &d["ln_alpha"];
  let cov = &d["covariance_matrix"];
  assert_eq!(
    d["covariance_parameter_names"],
    json!(["_cons", "logr", "lnalpha"])
  );
  for (key, value) in [
    ("estimator", json!("shared_gamma_poisson_re")),
    (
      "covariance",
      json!("inverse_full_joint_observed_information"),
    ),
    ("covariance_correction", json!(1.0)),
    ("observation_count", json!(60)),
    ("group_count", json!(12)),
    ("zero_total_group_count", json!(2)),
    ("minimum_group_size", json!(5)),
    ("maximum_group_size", json!(5)),
    (
      "sample",
      json!({"total_observations":60,"dropped_observations":0}),
    ),
    (
      "reference_distribution",
      json!("asymptotic_standard_normal"),
    ),
    (
      "ln_alpha_test_null",
      json!("ln_alpha=0_equivalently_alpha=1_not_zero_variance"),
    ),
    (
      "dispersion_likelihood_ratio_reference",
      json!("equal_mixture_chi_square_0_1"),
    ),
    (
      "prediction_scope",
      json!("population_marginal_not_posterior_group_effects"),
    ),
    ("converged", json!(true)),
  ] {
    assert_eq!(d[key], value, "{key}");
  }
  assert!(d["max_abs_scaled_score"].as_f64().unwrap() <= 1e-10);
  assert!(d["max_abs_scaled_newton_step"].as_f64().unwrap() <= 1e-10);
  let mut values: BTreeMap<String, f64> = BTreeMap::new();
  for (name, value) in ["intercept", "slope", "lnalpha"]
    .into_iter()
    .zip([&co[0], &co[1], ln])
  {
    for (prefix, field) in [
      ("estimate", "value"),
      ("se", "standard_error"),
      ("z", "statistic"),
      ("p", "p_value"),
      ("lower", "confidence_lower"),
      ("upper", "confidence_upper"),
    ] {
      values.insert(format!("{prefix}_{name}"), value[field].as_f64().unwrap());
    }
  }
  for (name, i, j) in [
    ("aa", 0, 0),
    ("ab", 0, 1),
    ("at", 0, 2),
    ("bb", 1, 1),
    ("bt", 1, 2),
    ("tt", 2, 2),
  ] {
    values.insert(format!("covariance_{name}"), cov[i][j].as_f64().unwrap());
    close(
      cov[i][j].as_f64().unwrap(),
      cov[j][i].as_f64().unwrap(),
      name,
    );
  }
  for (name, field) in [
    ("candidate_rows", "observation_count"),
    ("firms", "group_count"),
    ("zero_total_firms", "zero_total_group_count"),
    ("alpha", "alpha"),
    ("se_alpha", "alpha_standard_error"),
    ("lower_alpha", "alpha_confidence_lower"),
    ("upper_alpha", "alpha_confidence_upper"),
    ("log_likelihood", "log_likelihood"),
    ("poisson_log_likelihood", "poisson_log_likelihood"),
    ("lr_alpha_zero", "dispersion_likelihood_ratio"),
    ("p_lr_alpha_zero", "dispersion_likelihood_ratio_p_value"),
  ] {
    values.insert(name.into(), d[field].as_f64().unwrap());
  }
  values.insert(
    "mean_prediction".into(),
    es[9]["data"]["rows"][0]["mean"].as_f64().unwrap(),
  );
  values.insert(
    "mean_residual".into(),
    es[9]["data"]["rows"][1]["mean"].as_f64().unwrap(),
  );
  let reference = rows(include_str!("fixtures/mma23_gamma_re/reference.csv"));
  assert_eq!(values.len(), 37);
  assert_eq!(reference.len(), 37);
  for r in reference {
    close(values[r["field"]], r["value"].parse().unwrap(), r["field"]);
  }
  let exported = fs::read_to_string(dir.path().join(EXPORT)).unwrap();
  let output = rows(&exported);
  let source = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let input = rows(&source);
  let reference = rows(include_str!("fixtures/mma23_gamma_re/predictions.csv"));
  assert_eq!(output.len(), 60);
  assert_eq!(reference.len(), 60);
  let map: BTreeMap<_, _> = input
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
  for (r, b) in output.iter().zip(reference) {
    let key = (r["id"].parse().unwrap(), r["year"].parse().unwrap());
    let a = map[&key];
    for (name, value) in a {
      if value.is_empty() {
        assert!(r[name].is_empty());
      } else {
        assert_eq!(
          r[name].parse::<f64>().unwrap(),
          value.parse::<f64>().unwrap()
        );
      }
    }
    for field in ["id", "year", "logr", "pat"] {
      assert_eq!(
        r[field].parse::<f64>().unwrap(),
        b[field].parse::<f64>().unwrap()
      );
    }
    close(
      r["native_re_mean"].parse().unwrap(),
      b["marginal_mu"].parse().unwrap(),
      "prediction",
    );
    close(
      r["native_re_residual"].parse().unwrap(),
      b["marginal_residual"].parse().unwrap(),
      "residual",
    );
    assert_eq!(r["included_in_native_re"], "true");
    let total: f64 = input
      .iter()
      .filter(|s| s["id"] == r["id"])
      .map(|s| s["pat"].parse::<f64>().unwrap())
      .sum();
    assert_eq!(r["selected_firm_total"].parse::<f64>().unwrap(), total);
    close(
      r["native_re_xb"].parse().unwrap(),
      co[0]["value"].as_f64().unwrap()
        + co[1]["value"].as_f64().unwrap() * r["logr"].parse::<f64>().unwrap(),
      "log mean",
    );
  }
  assert!(output.windows(2).all(|w| (
    w[0]["id"].parse::<u64>().unwrap(),
    w[0]["year"].parse::<u64>().unwrap()
  ) < (
    w[1]["id"].parse::<u64>().unwrap(),
    w[1]["year"].parse::<u64>().unwrap()
  )));
}
#[test]
fn native_reference_full_covariance_source_and_predictions_match_json_and_text() {
  let mut exported = None;
  for mode in ["--json", "--text"] {
    let dir = TestDir::new();
    prepare(&dir);
    let input = fs::read(dir.path().join("patents.csv")).unwrap();
    let out = run(&dir, mode);
    assert!(
      out.status.success(),
      "{}",
      String::from_utf8_lossy(&out.stderr)
    );
    if mode == "--json" {
      check(&dir, &events(&out));
    } else {
      let text = String::from_utf8(out.stdout).unwrap();
      assert!(text.contains("Native shared mean-one Gamma firm effects"));
      assert!(text.contains("lnalpha"));
    }
    let current = fs::read(dir.path().join(EXPORT)).unwrap();
    if let Some(previous) = exported {
      assert_eq!(current, previous);
    }
    exported = Some(current);
    assert_eq!(fs::read(dir.path().join("patents.csv")).unwrap(), input);
  }
}
#[test]
fn source_permutation_restores_canonical_fit_and_export_order() {
  let mut baseline = None;
  for reverse in [false, true] {
    let dir = TestDir::new();
    prepare(&dir);
    if reverse {
      let input = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
      let lines: Vec<_> = input.lines().collect();
      fs::write(
        dir.path().join("patents.csv"),
        std::iter::once(lines[0])
          .chain(lines[1..].iter().rev().copied())
          .collect::<Vec<_>>()
          .join("\n")
          + "\n",
      )
      .unwrap();
    }
    let out = run(&dir, "--json");
    assert!(out.status.success());
    check(&dir, &events(&out));
    let current = fs::read(dir.path().join(EXPORT)).unwrap();
    if let Some(previous) = baseline {
      assert_eq!(current, previous);
    }
    baseline = Some(current);
  }
}
#[test]
fn all_missing_nonmodel_ardssic_preserves_every_row_and_native_fit() {
  let dir = TestDir::new();
  prepare(&dir);
  let input = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let mut lines = input.lines();
  let mut altered = lines.next().unwrap().to_owned() + "\n";
  for line in lines {
    let mut values: Vec<_> = line.split(',').collect();
    values[3] = "";
    altered += &(values.join(",") + "\n");
  }
  fs::write(dir.path().join("patents.csv"), &altered).unwrap();
  let out = run(&dir, "--json");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let es = events(&out);
  check(&dir, &es);
  assert_eq!(es[2]["data"]["rows"][0][2], 60);
  assert_eq!(
    fs::read_to_string(dir.path().join("patents.csv")).unwrap(),
    altered
  );
}
#[test]
fn retained_native_gamma_predictions_labels_and_export_collision_state_are_owned() {
  for collision in [false, true] {
    let dir = TestDir::new();
    prepare(&dir);
    if collision {
      fs::write(dir.path().join(EXPORT), "owner").unwrap();
    }
    let input = fs::read(dir.path().join("patents.csv")).unwrap();
    let mut session = Session::new();
    let mut failures = 0;
    for command in parse_script(SCRIPT, Path::new("native_gamma_re.td")).unwrap() {
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
    let state = session.state();
    assert!(matches!(
      &state.estimation,
      Some(EstimationState::GammaPoisson(_))
    ));
    session
      .execute(parse_command("label variable logr \"Original numeric LOGR\"").unwrap())
      .unwrap();
    let before = session.state();
    session
      .execute(parse_command("summarize native_re_mean native_re_residual").unwrap())
      .unwrap();
    assert_eq!(session.state(), before);
    assert!(
      session
        .execute(parse_command("predict native_re_mean, n").unwrap())
        .is_err()
    );
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("predict repeated_population, n").unwrap())
      .unwrap();
    assert_eq!(session.state().estimation, before.estimation);
    let before = session.state();
    session.execute(parse_command("sql \"\"\"SELECT CASE WHEN COUNT(*)=60 AND COUNT(DISTINCT id)=12 AND COUNT(*) FILTER (WHERE selected_firm_total=0)=10 AND MAX(abs(repeated_population-native_re_mean))<1e-12 THEN 1 ELSE error('lost native RE source or owned prediction') END AS valid FROM active\"\"\"").unwrap()).unwrap();
    assert_eq!(session.state(), before);
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
fn invalid_source_and_singular_design_stop_before_re_prediction_or_export() {
  let dir = TestDir::new();
  prepare(&dir);
  let input = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let base: Vec<Vec<String>> = input
    .lines()
    .map(|s| s.split(',').map(str::to_owned).collect())
    .collect();
  let mut cases = vec![
    input.clone() + &format!("{}\n", input.lines().nth(1).unwrap()),
    input.lines().take(60).collect::<Vec<_>>().join("\n") + "\n",
    base[0].join(",") + "\n",
  ];
  for (col, value) in [
    (0, "0"),
    (0, "1.5"),
    (0, "347"),
    (0, ""),
    (1, "1974"),
    (1, "1975.5"),
    (8, "-1"),
    (8, "0.5"),
    (8, "1001"),
    (8, ""),
    (8, "NaN"),
    (7, ""),
    (7, "Inf"),
    (7, "21"),
    (3, "Inf"),
    (3, "oops"),
    (2, ""),
    (4, "NaN"),
    (5, "Inf"),
    (6, ""),
  ] {
    let mut data = base.clone();
    data[1][col] = value.into();
    cases.push(
      data
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  let mut singular = base.clone();
  for row in &mut singular[1..] {
    row[7] = "0".into();
  }
  cases.push(
    singular
      .iter()
      .map(|r| r.join(","))
      .collect::<Vec<_>>()
      .join("\n")
      + "\n",
  );
  assert_eq!(cases.len(), 24);
  for (index, input) in cases.iter().enumerate() {
    let dir = TestDir::new();
    fs::write(dir.path().join("patents.csv"), input).unwrap();
    fs::write(dir.path().join(EXPORT), "owner").unwrap();
    let out = run(&dir, "--json");
    assert!(!out.status.success(), "case {index}");
    assert!(
      !events(&out).iter().any(
        |e| ["GammaPoissonResult", "PredictionResult", "ExportResult"]
          .iter()
          .any(|v| e["result_type"] == *v)
      ),
      "case {index} reached fit/prediction/export"
    );
    assert_eq!(
      fs::read_to_string(dir.path().join(EXPORT)).unwrap(),
      "owner"
    );
    assert_eq!(
      fs::read_to_string(dir.path().join("patents.csv")).unwrap(),
      *input
    );
  }
}
#[test]
fn noninterior_fit_clears_stale_model_but_preserves_snapshot_data_and_labels() {
  let dir = TestDir::new();
  prepare(&dir);
  let input = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let mut lines = input.lines();
  let mut altered = lines.next().unwrap().to_owned() + "\n";
  for line in lines {
    let mut values: Vec<_> = line.split(',').collect();
    values[8] = "1";
    altered += &(values.join(",") + "\n");
  }
  fs::write(dir.path().join("patents.csv"), &altered).unwrap();
  let mut session = Session::new();
  session
    .execute(
      parse_command(&format!(
        "use \"{}\"",
        dir.path().join("patents.csv").display()
      ))
      .unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command("poisson pat logr").unwrap())
    .unwrap();
  session
    .execute(parse_command("estimates store prior_poisson").unwrap())
    .unwrap();
  session
    .execute(parse_command("label variable logr \"Prior owned predictor\"").unwrap())
    .unwrap();
  let mut expected = session.state();
  assert!(expected.estimation.is_some());
  assert_eq!(expected.stored_estimates.len(), 1);
  let error = session
    .execute(parse_command("xtpoisson pat logr, re id(id)").unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("xtpoisson re"), "{error}");
  expected.estimation = None;
  assert_eq!(session.state(), expected);
  assert!(
    session
      .execute(parse_command("predict prior_poisson_mean, n").unwrap())
      .is_err()
  );
  assert_eq!(session.state(), expected);
  session
    .execute(parse_command("estimates table prior_poisson").unwrap())
    .unwrap();
  assert_eq!(session.state(), expected);
  session.execute(parse_command("sql \"\"\"SELECT CASE WHEN COUNT(*)=60 AND MIN(pat)=1 AND MAX(pat)=1 THEN 1 ELSE error('source changed on failed fit') END AS valid FROM active\"\"\"").unwrap()).unwrap();
  assert_eq!(session.state(), expected);
  assert_eq!(
    fs::read_to_string(dir.path().join("patents.csv")).unwrap(),
    altered
  );
}
