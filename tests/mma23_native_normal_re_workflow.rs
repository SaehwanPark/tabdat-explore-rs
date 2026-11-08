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
const SCRIPT: &str = include_str!("../examples/mma23p1pannonlin/native_normal_re.td");
const EXPORT: &str = "native_normal_re_predictions.csv";
fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn prepare(dir: &TestDir) {
  fs::write(
    dir.path().join("source.asc"),
    include_str!("fixtures/mma23_gamma_re_bootstrap/source.asc"),
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
    .arg(root().join("examples/mma23p1pannonlin/native_normal_re.td"))
    .output()
    .unwrap()
}
fn close(a: f64, b: f64, label: &str) {
  let tol = if label.starts_with("p_") && b > 0. {
    1e-9 * b
  } else {
    1e-9 + 1e-10 * b.abs()
  };
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= tol,
    "{label}: {a} vs {b}"
  );
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let header: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let fields: Vec<_> = line.split(',').collect();
      assert_eq!(fields.len(), header.len());
      header.iter().copied().zip(fields).collect()
    })
    .collect()
}
fn check(dir: &TestDir, out: &std::process::Output) {
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let events: Vec<Value> = String::from_utf8_lossy(&out.stdout)
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  assert_eq!(
    events
      .iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "IsidResult",
      "TableResult",
      "TableResult",
      "SqlCreateResult",
      "NormalPoissonResult",
      "PredictionResult",
      "PredictionResult",
      "PredictionResult",
      "SummarizeResult",
      "TableResult",
      "ExportResult"
    ]
  );
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  assert_eq!(events[3]["data"]["rows"], json!([[40]]));
  let d = &events[5]["data"];
  assert_eq!(
    d["covariance_parameter_names"],
    json!(["_cons", "logr", "lnsigma2"])
  );
  for (key, expected) in [
    ("estimator", json!("normal_random_intercept_poisson_re")),
    (
      "random_effects_distribution",
      json!("mean_zero_gaussian_log_intercept"),
    ),
    ("observation_count", json!(200)),
    ("group_count", json!(40)),
    ("zero_total_group_count", json!(8)),
    ("singleton_group_count", json!(0)),
    (
      "sample",
      json!({"total_observations":200,"dropped_observations":0}),
    ),
    (
      "covariance",
      json!("inverse_full_joint_observed_information"),
    ),
    ("covariance_correction", json!(1.0)),
    (
      "ln_variance_test_null",
      json!("lnsigma2=0_equivalently_variance=1_not_zero_variance"),
    ),
    (
      "variance_likelihood_ratio_reference",
      json!("equal_mixture_chi_square_0_1"),
    ),
    ("quadrature_discrepancy_is_error_bound", json!(false)),
    ("converged", json!(true)),
  ] {
    assert_eq!(d[key], expected, "{key}");
  }
  for (key, limit) in [
    ("max_abs_scaled_score", 1e-10),
    ("max_abs_scaled_newton_step", 1e-12),
    ("max_relative_quadrature_difference", 1e-9),
  ] {
    let v = d[key].as_f64().unwrap();
    assert!(v >= 0. && v <= limit);
  }
  let co = [
    &d["coefficients"][0],
    &d["coefficients"][1],
    &d["ln_variance"],
  ];
  let mut actual: BTreeMap<String, f64> = BTreeMap::from([
    ("candidate_rows".into(), 200.),
    ("firms".into(), 40.),
    ("zero_total_firms".into(), 8.),
  ]);
  for (name, field) in [
    ("sigma", "sigma"),
    ("se_sigma", "sigma_standard_error"),
    ("lower_sigma", "sigma_confidence_lower"),
    ("upper_sigma", "sigma_confidence_upper"),
    ("log_likelihood", "log_likelihood"),
    ("poisson_log_likelihood", "poisson_log_likelihood"),
    ("lr_variance_zero", "variance_likelihood_ratio"),
    ("p_lr_variance_zero", "variance_likelihood_ratio_p_value"),
  ] {
    actual.insert(name.into(), d[field].as_f64().unwrap());
  }
  for (name, c) in ["intercept", "slope", "lnvariance"].into_iter().zip(co) {
    for (prefix, field) in [
      ("estimate", "value"),
      ("se", "standard_error"),
      ("z", "statistic"),
      ("p", "p_value"),
      ("lower", "confidence_lower"),
      ("upper", "confidence_upper"),
    ] {
      actual.insert(format!("{prefix}_{name}"), c[field].as_f64().unwrap());
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
    actual.insert(
      format!("covariance_{name}"),
      d["covariance_matrix"][i][j].as_f64().unwrap(),
    );
  }
  for i in 0..3 {
    for j in 0..3 {
      close(
        d["covariance_matrix"][i][j].as_f64().unwrap(),
        d["covariance_matrix"][j][i].as_f64().unwrap(),
        "covariance symmetry after unit transformation",
      );
    }
  }
  let summary = &events[9]["data"]["rows"];
  for (name, i, field) in [
    ("mean_mu", 0, "mean"),
    ("min_mu", 0, "minimum"),
    ("max_mu", 0, "maximum"),
    ("mean_residual", 1, "mean"),
  ] {
    actual.insert(name.into(), summary[i][field].as_f64().unwrap());
  }
  let oracle: BTreeMap<_, _> = include_str!("fixtures/mma23_normal_re/reference.csv")
    .lines()
    .skip(1)
    .map(|line| {
      let (k, v) = line.split_once(',').unwrap();
      (k, v.parse::<f64>().unwrap())
    })
    .collect();
  assert_eq!((actual.len(), oracle.len()), (39, 42));
  for (name, value) in &actual {
    close(*value, oracle[name.as_str()], name);
  }
  let exported = fs::read_to_string(dir.path().join(EXPORT)).unwrap();
  let exported = rows(&exported);
  let predictions = rows(include_str!("fixtures/mma23_normal_re/predictions.csv"));
  assert_eq!((exported.len(), predictions.len()), (200, 200));
  let source = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let source = rows(&source);
  let source: BTreeMap<_, _> = source
    .iter()
    .map(|r| {
      (
        (
          r["id"].parse::<usize>().unwrap(),
          r["year"].parse::<usize>().unwrap(),
        ),
        r,
      )
    })
    .collect();
  let shift = d["ln_variance"]["value"].as_f64().unwrap().exp() / 2.;
  for (r, p) in exported.iter().zip(predictions) {
    assert_eq!(r.len(), 14);
    assert_eq!(r["included_in_native_re"], "true");
    let key = (
      r["id"].parse::<usize>().unwrap(),
      r["year"].parse::<usize>().unwrap(),
    );
    for (field, value) in source[&key].iter() {
      if value.is_empty() {
        assert_eq!(r[field], "");
      } else {
        assert_eq!(
          r[field].parse::<f64>().unwrap(),
          value.parse::<f64>().unwrap()
        );
      }
    }
    for field in ["id", "year", "logr", "pat"] {
      assert_eq!(
        r[field].parse::<f64>().unwrap(),
        p[field].parse::<f64>().unwrap()
      );
    }
    let mu = r["native_re_mean"].parse::<f64>().unwrap();
    let xb = r["native_re_xb"].parse::<f64>().unwrap();
    let resid = r["native_re_residual"].parse::<f64>().unwrap();
    close(mu, p["marginal_mu"].parse().unwrap(), "R marginal mean");
    close(resid, p["marginal_residual"].parse().unwrap(), "R residual");
    close(mu, (xb + shift).exp(), "population shift");
    close(
      resid,
      r["pat"].parse::<f64>().unwrap() - mu,
      "residual identity",
    );
  }
}
#[test]
fn reference_full_covariance_population_exports_json_text_and_metadata_permutation() {
  let mut baseline = None;
  for variant in 0..4 {
    let dir = TestDir::new();
    prepare(&dir);
    if variant >= 2 {
      let input = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
      let mut lines = input.lines();
      let header = lines.next().unwrap();
      let mut data: Vec<String> = lines.map(str::to_owned).collect();
      if variant == 2 {
        data.reverse();
      } else {
        for line in &mut data {
          let mut fields: Vec<_> = line.split(',').collect();
          fields[3] = "";
          *line = fields.join(",");
        }
      }
      fs::write(
        dir.path().join("patents.csv"),
        format!("{header}\n{}\n", data.join("\n")),
      )
      .unwrap();
    }
    let input = fs::read(dir.path().join("patents.csv")).unwrap();
    let out = run(&dir, if variant == 1 { "--text" } else { "--json" });
    assert!(
      out.status.success(),
      "{}",
      String::from_utf8_lossy(&out.stderr)
    );
    if variant == 1 {
      let text = String::from_utf8(out.stdout).unwrap();
      for phrase in [
        "Normal random-intercept Poisson",
        "lnsigma2",
        "not an error bound",
        "population n=exp(Xbeta+variance/2)",
      ] {
        assert!(text.contains(phrase), "{phrase}");
      }
    } else {
      check(&dir, &out);
    }
    let current = fs::read(dir.path().join(EXPORT)).unwrap();
    if variant < 3 {
      if let Some(ref previous) = baseline {
        assert_eq!(&current, previous);
      }
      baseline = Some(current);
    }
    assert_eq!(fs::read(dir.path().join("patents.csv")).unwrap(), input);
  }
}
#[test]
fn owned_normal_predictions_labels_reports_and_export_no_clobber() {
  for collision in [false, true] {
    let dir = TestDir::new();
    prepare(&dir);
    if collision {
      fs::write(dir.path().join(EXPORT), "owner").unwrap();
    }
    let input = fs::read(dir.path().join("patents.csv")).unwrap();
    let mut session = Session::new();
    for command in parse_script(SCRIPT, Path::new("native_normal_re.td")).unwrap() {
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
      } else {
        result.unwrap();
      }
    }
    assert!(matches!(
      session.state().estimation,
      Some(EstimationState::NormalPoisson(_))
    ));
    session
      .execute(parse_command("label variable logr \"Original LOGR\"").unwrap())
      .unwrap();
    let before = session.state();
    for text in [
      "summarize native_re_mean native_re_residual",
      "estimates store unsupported",
      "predict native_re_mean, n",
    ] {
      let result = session.execute(parse_command(text).unwrap());
      assert_eq!(result.is_ok(), text.starts_with("summarize"));
      assert_eq!(session.state(), before);
    }
    session
      .execute(parse_command("predict repeated_population, n").unwrap())
      .unwrap();
    assert_eq!(session.state().estimation, before.estimation);
    assert_eq!(session.state().labels, before.labels);
    session.execute(parse_command("sql \"\"\"SELECT CASE WHEN COUNT(*)=200 AND COUNT(DISTINCT id)=40 AND COUNT(*) FILTER (WHERE selected_firm_total=0)=40 AND MAX(abs(repeated_population-native_re_mean))<1e-12 THEN 1 ELSE error('lost owned normal population prediction') END AS valid FROM active\"\"\"").unwrap()).unwrap();
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
fn invalid_source_duplicate_incomplete_and_singular_design_never_export() {
  let base = TestDir::new();
  prepare(&base);
  let input = fs::read_to_string(base.path().join("patents.csv")).unwrap();
  let lines: Vec<_> = input.lines().collect();
  let mut cases = vec![
    input.clone() + lines[1] + "\n",
    lines[..200].join("\n") + "\n",
    lines[0].to_owned() + "\n",
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
    let mut fields: Vec<_> = lines[1].split(',').collect();
    fields[col] = value;
    let mut changed = lines.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    changed[1] = fields.join(",");
    cases.push(changed.join("\n") + "\n");
  }
  let singular = lines
    .iter()
    .enumerate()
    .map(|(i, line)| {
      if i == 0 {
        return line.to_string();
      }
      let mut f: Vec<_> = line.split(',').collect();
      f[7] = "1";
      f.join(",")
    })
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  cases.push(singular);
  assert_eq!(cases.len(), 24);
  for (i, case) in cases.into_iter().enumerate() {
    let dir = TestDir::new();
    fs::write(dir.path().join("patents.csv"), case).unwrap();
    let out = run(&dir, "--json");
    assert!(!out.status.success(), "case{i}");
    assert!(!dir.path().join(EXPORT).exists(), "case{i}");
  }
}
