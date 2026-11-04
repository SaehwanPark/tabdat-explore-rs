mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn script() -> &'static str {
  include_str!("../examples/mma23p1pannonlin/exchangeable_gee_bootstrap.td")
}
fn prepare(d: &TestDir, source: &str) {
  fs::write(d.path().join("source.asc"), source).unwrap();
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
    .arg(root().join("examples/mma23p1pannonlin/exchangeable_gee_bootstrap.td"))
    .output()
    .unwrap()
}
fn run_variant(d: &TestDir, text: &str) -> Output {
  fs::write(d.path().join("variant.td"), text).unwrap();
  tabdat_bin()
    .current_dir(d.path())
    .args(["--json", "-f", "variant.td"])
    .output()
    .unwrap()
}
fn events(out: &Output) -> Vec<Value> {
  String::from_utf8_lossy(&out.stdout)
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}
fn close(actual: f64, expected: f64) {
  assert!(
    actual.is_finite() && (actual - expected).abs() <= 1e-9 + 1e-10 * expected.abs(),
    "{actual} != {expected}"
  );
}
fn report(ev: &[Value]) -> Value {
  let e = ev
    .iter()
    .find(|e| {
      e["result_type"] == "TableResult"
        && e["data"]["headers"]
          .as_array()
          .unwrap()
          .contains(&Value::from("covariance_aa"))
    })
    .unwrap();
  Value::Object(
    e["data"]["headers"]
      .as_array()
      .unwrap()
      .iter()
      .zip(e["data"]["rows"][0].as_array().unwrap())
      .map(|(k, v)| (k.as_str().unwrap().into(), v.clone()))
      .collect(),
  )
}
fn check_report(r: &Value) {
  let lines = include_str!("fixtures/mma23_gee_bootstrap/reference.csv")
    .lines()
    .skip(1);
  let mut n = 0;
  for line in lines {
    let (key, value) = line.split_once(',').unwrap();
    close(r[key].as_f64().unwrap(), value.parse().unwrap());
    n += 1;
  }
  assert_eq!(n, 23);
  let a = r["estimate_intercept"].as_f64().unwrap();
  let b = r["estimate_slope"].as_f64().unwrap();
  let mut predictions = 0;
  for line in include_str!("fixtures/mma23_gee_bootstrap/predictions.csv")
    .lines()
    .skip(1)
  {
    let cols: Vec<_> = line.split(',').collect();
    let x: f64 = cols[2].parse().unwrap();
    let y: f64 = cols[3].parse().unwrap();
    let mu = (a + b * x).exp();
    close(mu, cols[4].parse().unwrap());
    close(y - mu, cols[5].parse().unwrap());
    predictions += 1;
  }
  assert_eq!(predictions, 60);
  assert_eq!(r["method"], "whole_firm_pairs_distinct_copies");
  assert_eq!(r["interval_method"], "type7_percentile_95");
  assert_eq!(r["base_seed"], 10001);
  assert_eq!(
    r["rng"],
    "xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1"
  );
}
fn check_draws(text: &str, r: &Value) {
  let mut rows = text.lines();
  assert_eq!(
    rows.next().unwrap(),
    "replicate,position,step,firm_id,replicate_intercept,replicate_slope,replicate_rho,replicate_iterations,replicate_last_step,estimate_a,estimate_b,estimate_rho,original_mu_mean,original_mu_min,original_mu_max,original_residual_mean,replications,firms_per_draw,candidate_rows"
  );
  let fits: Vec<Vec<f64>> = include_str!("fixtures/mma23_gee_bootstrap/fits.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').map(|x| x.parse().unwrap()).collect())
    .collect();
  assert_eq!(fits.len(), 500);
  let mut count = 0;
  for (i, line) in include_str!("fixtures/mma23_pooled_bootstrap/draws.csv")
    .lines()
    .skip(1)
    .enumerate()
  {
    let (rep, indices) = line.split_once(',').unwrap();
    assert_eq!(rep.parse::<usize>().unwrap(), i + 1);
    for (position, index) in indices.split(':').enumerate() {
      let row: Vec<_> = rows.next().unwrap().split(',').collect();
      assert_eq!(row[0].parse::<usize>().unwrap(), i + 1);
      assert_eq!(row[1].parse::<usize>().unwrap(), position + 1);
      assert_eq!(row[2].parse::<usize>().unwrap(), position + 1);
      assert_eq!(
        row[3].parse::<usize>().unwrap(),
        index.parse::<usize>().unwrap() + 1
      );
      for j in 0..3 {
        close(row[4 + j].parse().unwrap(), fits[i][1 + j]);
      }
      assert!((1..=100).contains(&row[7].parse::<usize>().unwrap()));
      assert!((0.0..=1e-12).contains(&row[8].parse::<f64>().unwrap()));
      for (j, key) in [
        "estimate_intercept",
        "estimate_slope",
        "estimate_rho",
        "original_mu_mean",
        "original_mu_min",
        "original_mu_max",
        "original_residual_mean",
      ]
      .iter()
      .enumerate()
      {
        close(row[9 + j].parse().unwrap(), r[*key].as_f64().unwrap());
      }
      assert_eq!(&row[16..], &["500", "12", "60"]);
      count += 1;
    }
  }
  assert_eq!(count, 6000);
  assert!(rows.next().is_none());
}
#[test]
fn independent_refits_draws_reporting_and_order() {
  let d = TestDir::new();
  prepare(&d, include_str!("fixtures/mma23_conditional_fe/source.asc"));
  let input = fs::read(d.path().join("patents.csv")).unwrap();
  let out = run(&d, "--json");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let ev = events(&out);
  assert!(ev.iter().any(|e| e["result_type"] == "PoissonResult"));
  let r = report(&ev);
  check_report(&r);
  let path = d.path().join("exchangeable_gee_bootstrap_draws.csv");
  let bytes = fs::read(&path).unwrap();
  check_draws(std::str::from_utf8(&bytes).unwrap(), &r);
  assert_eq!(fs::read(d.path().join("patents.csv")).unwrap(), input);
  let fail = run(&d, "--json");
  assert!(!fail.status.success());
  assert!(String::from_utf8_lossy(&fail.stderr).contains("export target already exists"));
  assert_eq!(fs::read(&path).unwrap(), bytes);
  fs::remove_file(&path).unwrap();
  let text = run(&d, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let output = String::from_utf8(text.stdout).unwrap();
  assert!(output.contains("covariance_aa") && output.contains("whole_firm_pairs_distinct_copies"));
  assert_eq!(fs::read(&path).unwrap(), bytes);
  fs::remove_file(&path).unwrap();
  let input = String::from_utf8(input).unwrap();
  let mut lines = input.lines();
  let header = lines.next().unwrap();
  let mut data: Vec<_> = lines.collect();
  data.reverse();
  fs::write(
    d.path().join("patents.csv"),
    format!("{header}\n{}\n", data.join("\n")),
  )
  .unwrap();
  let out = run(&d, "--json");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let reversed_report = report(&events(&out));
  check_report(&reversed_report);
  let reversed = fs::read_to_string(&path).unwrap();
  check_draws(&reversed, &reversed_report);
}
fn rewrite<F: FnMut(&mut Vec<String>)>(input: &str, mut f: F) -> String {
  let mut lines = input.lines();
  let mut out = lines.next().unwrap().to_owned() + "\n";
  for line in lines {
    let mut row: Vec<_> = line.split(',').map(str::to_owned).collect();
    f(&mut row);
    out.push_str(&row.join(","));
    out.push('\n');
  }
  out
}
fn query() -> String {
  let body = script()
    .split_once("sql \"\"\"\nWITH RECURSIVE\n")
    .unwrap()
    .1
    .split_once("\n\"\"\" into exchangeable_gee_bootstrap_draws")
    .unwrap()
    .0;
  format!("sql \"\"\"WITH RECURSIVE\n{body}\"\"\" into exchangeable_gee_bootstrap_draws")
}
fn exec(s: &mut Session, text: &str) -> tabdat_domain::ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}
#[test]
fn invalid_data_and_actual_singular_replicate_preserve_files_and_state() {
  let d = TestDir::new();
  prepare(&d, include_str!("fixtures/mma23_conditional_fe/source.asc"));
  let path = d.path().join("patents.csv");
  let source = fs::read_to_string(&path).unwrap();
  let export = d.path().join("exchangeable_gee_bootstrap_draws.csv");
  fs::write(&export, b"sentinel").unwrap();
  let mut dup: Vec<_> = source.lines().map(str::to_owned).collect();
  dup[2] = dup[1].clone();
  let cases = [
    dup.join("\n") + "\n",
    source.lines().take(6).collect::<Vec<_>>().join("\n") + "\n",
    source
      .lines()
      .enumerate()
      .map(|(i, line)| format!("{line},{}", if i == 0 { "pooled_xb" } else { "1" }))
      .collect::<Vec<_>>()
      .join("\n")
      + "\n",
    rewrite(&source, |r| r[7] = "1".into()),
    rewrite(&source, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "".into()
      }
    }),
    rewrite(&source, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "NaN".into()
      }
    }),
    rewrite(&source, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "21".into()
      }
    }),
    rewrite(&source, |r| {
      if r[0] == "1" {
        r[0] = "347".into()
      }
    }),
    rewrite(&source, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[8] = "-1".into()
      }
    }),
    rewrite(&source, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[8] = "1001".into()
      }
    }),
    rewrite(&source, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[8] = "1.5".into()
      }
    }),
    source.replacen(",1975,", ",1980,", 1),
  ];
  for case in cases {
    fs::write(&path, &case).unwrap();
    let out = run(&d, "--json");
    assert!(!out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("export target already exists"));
    assert_eq!(fs::read_to_string(&path).unwrap(), case);
    assert_eq!(fs::read(&export).unwrap(), b"sentinel");
  }
  // The pooled model is valid, but constant within-firm residuals force rho=1.
  let boundary = rewrite(&source, |r| {
    let id: usize = r[0].parse().unwrap();
    let year: i32 = r[1].parse().unwrap();
    r[7] = ((year - 1977) as f64 * 0.2).to_string();
    r[8] = if id <= 6 { "0" } else { "2" }.into();
  });
  fs::write(&path, &boundary).unwrap();
  let out = run(&d, "--json");
  assert!(!out.status.success());
  assert!(
    events(&out)
      .iter()
      .any(|e| e["result_type"] == "PoissonResult")
  );
  assert_eq!(fs::read_to_string(&path).unwrap(), boundary);
  assert_eq!(fs::read(&export).unwrap(), b"sentinel");
  fs::remove_file(&path).unwrap();
  prepare(
    &d,
    include_str!("fixtures/mma23_exchangeable_gee/negative/source.asc"),
  );
  let input = fs::read(&path).unwrap();
  let out = run(&d, "--json");
  assert!(!out.status.success());
  let ev = events(&out);
  assert!(ev.iter().any(|e| e["result_type"] == "PoissonResult"));
  assert!(ev.iter().any(|e| e["result_type"] == "PredictionResult"));
  assert!(!ev.iter().any(|e| e["result_type"] == "SqlCreateResult"));
  assert_eq!(fs::read(&path).unwrap(), input);
  assert_eq!(fs::read(&export).unwrap(), b"sentinel");
  let mut s = Session::new();
  exec(&mut s, &format!("use \"{}\"", path.display()));
  exec(&mut s, "poisson pat logr");
  exec(&mut s, "predict pooled_xb, xb");
  let before = s.state();
  let rows = exec(&mut s, "head 100");
  assert!(s.execute(parse_command(&query()).unwrap()).is_err());
  assert_eq!(s.state(), before);
  assert_eq!(exec(&mut s, "head 100"), rows);
  exec(&mut s, "predict still_available, xb");
}
#[test]
fn bounded_iterations_and_rng_fail_without_export() {
  let pooled = include_str!("../examples/mma23p1pannonlin/pooled_poisson_bootstrap.td");
  let rng = |s: &str| {
    s.split_once("-- Each replicate starts its own SplitMix64-expanded xoshiro256++ stream.")
      .unwrap()
      .1
      .split_once("), frequencies AS MATERIALIZED (")
      .unwrap()
      .0
      .to_owned()
  };
  assert_eq!(rng(script()), rng(pooled));
  let d = TestDir::new();
  prepare(&d, include_str!("fixtures/mma23_conditional_fe/source.asc"));
  let input = fs::read(d.path().join("patents.csv")).unwrap();
  for (old, new) in [
    (
      "WHERE iter<100 AND (iter=0 OR last_step>1e-12)",
      "WHERE iter<1 AND (iter=0 OR last_step>1e-12)",
    ),
    ("step<g+1024", "step<0"),
    (
      "SELECT rep,0::BIGINT,a,b,1e100::DOUBLE",
      "SELECT rep,0::BIGINT,21::DOUBLE,b,1e100::DOUBLE",
    ),
    (">=1e-9+1e-10*ABS(a)", ">=0"),
  ] {
    assert!(script().contains(old));
    let out = run_variant(&d, &script().replace(old, new));
    assert!(!out.status.success(), "forced guard unexpectedly succeeded");
    assert!(
      !d.path()
        .join("exchangeable_gee_bootstrap_draws.csv")
        .exists()
    );
    assert_eq!(fs::read(d.path().join("patents.csv")).unwrap(), input);
  }
}

#[test]
fn isolated_correlation_bread_and_start_guards() {
  let d = TestDir::new();
  prepare(&d, include_str!("fixtures/mma23_conditional_fe/source.asc"));
  let path = d.path().join("patents.csv");
  // Fractional outcomes isolate numerical guards, not public count-domain inputs.
  let probes = [
    "SELECT * EXCLUDE(pat),CASE WHEN id<=6 THEN 0 ELSE 2 END AS pat,0.0 AS pooled_xb FROM active",
    "SELECT * EXCLUDE(pat),2.0+CASE WHEN year=1975 THEN 1e-7 ELSE 0 END AS pat,0.0 AS pooled_xb FROM active",
    "SELECT * EXCLUDE(pat),CASE WHEN year IN (1975,1976) THEN 1 WHEN year=1977 THEN 2 ELSE 3 END AS pat,LN(2.0) AS pooled_xb FROM active",
    "SELECT * EXCLUDE(pat),CASE WHEN year IN (1975,1976) THEN 1 WHEN year=1977 THEN 2 ELSE 3 END+CASE WHEN year=1975 THEN 1e-7 ELSE 0 END AS pat,LN(2.0) AS pooled_xb FROM active",
    "SELECT * EXCLUDE(pat),2.0 AS pat,LN(2.0) AS pooled_xb FROM active",
    "SELECT * EXCLUDE(logr),1.0+(year-1975)*1e-9 AS logr,0.0 AS pooled_xb FROM active",
    "SELECT *,50.0+logr AS pooled_xb FROM active",
    "SELECT *,logr*logr AS pooled_xb FROM active",
  ];
  for probe in probes {
    let mut s = Session::new();
    exec(&mut s, &format!("use \"{}\"", path.display()));
    exec(&mut s, &format!("sql \"\"\"{probe}\"\"\" into probe"));
    let before = s.state();
    let rows = exec(&mut s, "head 100");
    assert!(
      s.execute(parse_command(&query()).unwrap()).is_err(),
      "guard unexpectedly accepted: {probe}"
    );
    assert_eq!(s.state(), before);
    assert_eq!(exec(&mut s, "head 100"), rows);
  }
}
