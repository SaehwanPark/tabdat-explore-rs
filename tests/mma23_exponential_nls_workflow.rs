mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  collections::BTreeMap,
  fs,
  path::PathBuf,
  process::{Command, Output},
};

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn prepare(d: &TestDir) {
  fs::write(
    d.path().join("source.asc"),
    include_str!("fixtures/mma23_conditional_fe/source.asc"),
  )
  .unwrap();
  let out = Command::new("sh")
    .current_dir(d.path())
    .arg(root().join("examples/mma23p1pannonlin/prepare.sh"))
    .args(["source.asc", "patents.csv"])
    .output()
    .unwrap();
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
}
fn run(d: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(root().join("examples/mma23p1pannonlin/exponential_nls.td"))
    .output()
    .unwrap()
}
fn close(a: f64, b: f64) {
  assert!(
    a.is_finite() && (a - b).abs() <= 1e-9 + 1e-10 * b.abs(),
    "{a} != {b}"
  );
}
fn report(out: Output) -> (Value, Value) {
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
  let fit = events
    .iter()
    .find(|e| e["result_type"] == "NlsResult")
    .unwrap()["data"]
    .clone();
  let table = events
    .iter()
    .find(|e| {
      e["result_type"] == "TableResult"
        && e["data"]["headers"]
          .as_array()
          .unwrap()
          .contains(&Value::from("cluster_aa"))
    })
    .unwrap();
  let keys = table["data"]["headers"].as_array().unwrap();
  let values = table["data"]["rows"][0].as_array().unwrap();
  (
    fit,
    Value::Object(
      keys
        .iter()
        .zip(values)
        .map(|(k, v)| (k.as_str().unwrap().into(), v.clone()))
        .collect(),
    ),
  )
}
fn reference() -> BTreeMap<String, f64> {
  include_str!("fixtures/mma23_exponential_nls/reference.csv")
    .lines()
    .skip(1)
    .map(|l| {
      let (key, val) = l.split_once(',').unwrap();
      (key.to_owned(), val.parse().unwrap())
    })
    .collect()
}
fn check_report(fit: &Value, r: &Value) {
  let refs = reference();
  let mut fields = 0;
  for (key, value) in &refs {
    if key.starts_with("native_") || key.starts_with("score_scale_") {
      continue;
    }
    close(r[key].as_f64().unwrap(), *value);
    fields += 1;
  }
  assert_eq!(fields, 27);
  assert_eq!(r["working_covariance"], "rss_over_n_working_gauss_newton");
  assert_eq!(
    r["cluster_covariance"],
    "firm_score_gauss_newton_G_over_G_minus_1"
  );
  assert_eq!(r["inference"], "asymptotic_normal; p-values not computed");
  assert_eq!(fit["observation_count"], 60);
  assert_eq!(fit["sample"]["dropped_observations"], 0);
  assert_eq!(fit["covariance"], "working_homoskedastic_gauss_newton");
  for (i, key) in ["intercept", "slope"].iter().enumerate() {
    close(
      fit["coefficients"][i]["value"].as_f64().unwrap(),
      refs[*key],
    );
    close(
      fit["coefficients"][i]["standard_error"].as_f64().unwrap(),
      refs[&format!("native_se_{key}")],
    );
  }
  for key in ["a", "b"] {
    assert!(
      r[&format!("score_{key}")].as_f64().unwrap().abs()
        <= 1e-11 * (1. + refs[&format!("score_scale_{key}")])
    );
  }
}
fn predictions(csv: &str) -> BTreeMap<(usize, usize), Vec<f64>> {
  let mut lines = csv.lines();
  let heads: Vec<_> = lines.next().unwrap().split(',').collect();
  let col = |name| heads.iter().position(|h| *h == name).unwrap();
  let mut result = BTreeMap::new();
  for l in lines {
    let r: Vec<_> = l.split(',').collect();
    let key = (
      r[col("id")].parse().unwrap(),
      r[col("year")].parse().unwrap(),
    );
    let vals = ["logr", "pat", "nls_mu", "nls_residual"]
      .iter()
      .map(|k| r[col(k)].parse().unwrap())
      .collect();
    assert!(result.insert(key, vals).is_none());
  }
  result
}

#[test]
fn r_reference_source_predictions_firm_covariance_and_order() {
  let d = TestDir::new();
  prepare(&d);
  let input = fs::read(d.path().join("patents.csv")).unwrap();
  let (fit, r) = report(run(&d, "--json"));
  check_report(&fit, &r);
  let path = d.path().join("exponential_nls_predictions.csv");
  let bytes = fs::read(&path).unwrap();
  let actual = predictions(&String::from_utf8(bytes.clone()).unwrap());
  let expected = predictions(include_str!(
    "fixtures/mma23_exponential_nls/predictions.csv"
  ));
  assert_eq!(actual.len(), 60);
  assert_eq!(actual.len(), expected.len());
  let mut scores: BTreeMap<usize, (f64, f64)> = BTreeMap::new();
  for (key, values) in &actual {
    for (x, y) in values.iter().zip(&expected[key]) {
      close(*x, *y);
    }
    let (x, mu, residual) = (values[0], values[2], values[3]);
    let score = scores.entry(key.0).or_default();
    score.0 += mu * residual;
    score.1 += mu * x * residual;
  }
  assert_eq!(scores.len(), 12);
  for line in include_str!("fixtures/mma23_exponential_nls/scores.csv")
    .lines()
    .skip(1)
  {
    let cols: Vec<_> = line.split(',').collect();
    let id: usize = cols[0].parse().unwrap();
    close(scores[&id].0, cols[1].parse().unwrap());
    close(scores[&id].1, cols[2].parse().unwrap());
  }
  assert_eq!(fs::read(d.path().join("patents.csv")).unwrap(), input);
  assert!(!run(&d, "--json").status.success());
  assert_eq!(fs::read(&path).unwrap(), bytes);
  fs::remove_file(&path).unwrap();
  let text = run(&d, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("cluster_se_slope") && text.contains("asymptotic_normal"));
  assert_eq!(fs::read(&path).unwrap(), bytes);
  fs::remove_file(&path).unwrap();
  let original = String::from_utf8(input).unwrap();
  let mut lines = original.lines();
  let header = lines.next().unwrap();
  let mut rows: Vec<_> = lines.collect();
  rows.reverse();
  fs::write(
    d.path().join("patents.csv"),
    format!("{header}\n{}\n", rows.join("\n")),
  )
  .unwrap();
  let (fit, r) = report(run(&d, "--json"));
  check_report(&fit, &r);
  let reversed = fs::read_to_string(&path).unwrap();
  assert!(reversed.lines().nth(1).unwrap().starts_with("12,1979,"));
  let reversed = predictions(&reversed);
  for (key, values) in &reversed {
    for (x, y) in values.iter().zip(&expected[key]) {
      close(*x, *y);
    }
  }
}

fn rewrite<F: FnMut(&mut Vec<String>)>(input: &str, mut f: F) -> String {
  let mut lines = input.lines();
  let mut out = lines.next().unwrap().to_owned() + "\n";
  for line in lines {
    let mut row: Vec<String> = line.split(',').map(str::to_owned).collect();
    f(&mut row);
    out.push_str(&row.join(","));
    out.push('\n');
  }
  out
}

#[test]
fn invalid_samples_and_fits_preserve_input_and_existing_file() {
  let d = TestDir::new();
  prepare(&d);
  let path = d.path().join("patents.csv");
  let input = fs::read_to_string(&path).unwrap();
  let export = d.path().join("exponential_nls_predictions.csv");
  fs::write(&export, b"sentinel").unwrap();
  let mut duplicate: Vec<_> = input.lines().map(str::to_owned).collect();
  duplicate[2] = duplicate[1].clone();
  let one_firm = input.lines().take(6).collect::<Vec<_>>().join("\n") + "\n";
  let collision = input
    .lines()
    .enumerate()
    .map(|(i, line)| format!("{line},{}", if i == 0 { "nls_mu" } else { "3" }))
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  let cases = [
    duplicate.join("\n") + "\n",
    one_firm,
    collision,
    rewrite(&input, |r| r[7] = "1".into()),
    rewrite(&input, |r| r[8] = "0".into()),
    rewrite(&input, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "".into();
      }
    }),
    input.replacen(",1975,", ",1980,", 1),
  ];
  for case in cases {
    fs::write(&path, &case).unwrap();
    let out = run(&d, "--json");
    assert!(!out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("export target already exists"));
    assert_eq!(fs::read_to_string(&path).unwrap(), case);
    assert_eq!(fs::read(&export).unwrap(), b"sentinel");
  }
}

#[test]
fn zero_and_cancellation_dominated_cluster_meat_rejected() {
  let d = TestDir::new();
  prepare(&d);
  let path = d.path().join("patents.csv");
  let original = fs::read_to_string(&path).unwrap();
  let input = rewrite(&original, |r| {
    let year: i32 = r[1].parse().unwrap();
    r[7] = (year - 1977).to_string();
    r[8] = match year {
      1975 | 1979 => "1",
      1976 | 1978 => "3",
      _ => "2",
    }
    .into();
  });
  let script = include_str!("../examples/mma23p1pannonlin/exponential_nls.td");
  let body = script
    .split_once("sql \"\"\"\nWITH base AS (")
    .unwrap()
    .1
    .split_once("\n\"\"\"")
    .unwrap()
    .0;
  let query = format!("sql \"\"\"WITH base AS ({body}\"\"\"");
  let control = tabdat_bin()
    .current_dir(d.path())
    .args([
      "--json",
      "-c",
      "use patents.csv",
      "-c",
      "sql \"\"\"SELECT *,2.0::DOUBLE AS nls_mu,pat-2 AS nls_residual FROM active\"\"\" into probe",
      "-c",
      &query,
    ])
    .output()
    .unwrap();
  assert!(
    control.status.success(),
    "{}",
    String::from_utf8_lossy(&control.stderr)
  );
  fs::write(&path, input).unwrap();
  for epsilon in ["0.0", "0.000000000000001"] {
    let prepare = format!(
      "sql \"\"\"SELECT *,2.0::DOUBLE AS nls_mu,pat-2+CASE WHEN year=1975 THEN {epsilon} ELSE 0 END AS nls_residual FROM active\"\"\" into probe"
    );
    let out = tabdat_bin()
      .current_dir(d.path())
      .args([
        "--json",
        "-c",
        "use patents.csv",
        "-c",
        &prepare,
        "-c",
        &query,
      ])
      .output()
      .unwrap();
    assert!(
      !out.status.success(),
      "cancellation-dominated meat was accepted"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("sql failed"));
  }
}
