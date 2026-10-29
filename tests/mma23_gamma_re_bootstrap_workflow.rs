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
  include_str!("../examples/mma23p1pannonlin/gamma_re_bootstrap.td")
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
    .arg(root().join("examples/mma23p1pannonlin/gamma_re_bootstrap.td"))
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
          .contains(&Value::from("covariance_tt"))
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
  let mut n = 0;
  for line in include_str!("fixtures/mma23_gamma_re_bootstrap/reference.csv")
    .lines()
    .skip(1)
  {
    let (key, value) = line.split_once(',').unwrap();
    close(r[key].as_f64().unwrap(), value.parse().unwrap());
    n += 1;
  }
  assert_eq!(n, 31);
  assert_eq!(r["base_seed"], 10001);
  assert_eq!(r["method"], "whole_firm_pairs_distinct_gamma_copies");
  assert_eq!(r["interval_method"], "type7_percentile_95");
  assert_eq!(
    r["rng"],
    "xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1"
  );
  let a = r["estimate_intercept"].as_f64().unwrap();
  let b = r["estimate_slope"].as_f64().unwrap();
  let mut n = 0;
  for line in include_str!("fixtures/mma23_gamma_re_bootstrap/predictions.csv")
    .lines()
    .skip(1)
  {
    let c: Vec<_> = line.split(',').collect();
    let x: f64 = c[2].parse().unwrap();
    let y: f64 = c[3].parse().unwrap();
    let mu = (a + b * x).exp();
    close(mu, c[4].parse().unwrap());
    close(y - mu, c[5].parse().unwrap());
    n += 1;
  }
  assert_eq!(n, 200);
}
fn check_draws(text: &str, r: &Value) {
  let mut rows = text.lines();
  assert_eq!(
    rows.next().unwrap(),
    "replicate,position,step,firm_id,replicate_intercept,replicate_slope,replicate_lnalpha,replicate_iterations,replicate_last_step,estimate_a,estimate_b,estimate_t,original_mu_mean,original_mu_min,original_mu_max,original_residual_mean,replications,firms_per_draw,candidate_rows"
  );
  let fits: Vec<Vec<f64>> = include_str!("fixtures/mma23_gamma_re_bootstrap/fits.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').map(|x| x.parse().unwrap()).collect())
    .collect();
  assert_eq!(fits.len(), 500);
  let mut n = 0;
  let mut zero = [false; 8];
  for (i, line) in include_str!("fixtures/mma23_gamma_re_bootstrap/draws.csv")
    .lines()
    .skip(1)
    .enumerate()
  {
    let (rep, indices) = line.split_once(',').unwrap();
    assert_eq!(rep.parse::<usize>().unwrap(), i + 1);
    for (position, index) in indices.split(':').enumerate() {
      let v: Vec<_> = rows.next().unwrap().split(',').collect();
      assert_eq!(v[0].parse::<usize>().unwrap(), i + 1);
      assert_eq!(v[1].parse::<usize>().unwrap(), position + 1);
      assert_eq!(v[2].parse::<usize>().unwrap(), position + 1);
      let firm = v[3].parse::<usize>().unwrap();
      assert_eq!(firm, index.parse::<usize>().unwrap() + 1);
      if firm % 5 == 1 {
        zero[(firm - 1) / 5] = true;
      }
      for j in 0..3 {
        close(v[4 + j].parse().unwrap(), fits[i][1 + j]);
      }
      assert!((1..=100).contains(&v[7].parse::<usize>().unwrap()));
      assert!((0.0..=1e-12).contains(&v[8].parse::<f64>().unwrap()));
      for (j, key) in [
        "estimate_intercept",
        "estimate_slope",
        "estimate_lnalpha",
        "original_mu_mean",
        "original_mu_min",
        "original_mu_max",
        "original_residual_mean",
      ]
      .iter()
      .enumerate()
      {
        close(v[9 + j].parse().unwrap(), r[*key].as_f64().unwrap());
      }
      assert_eq!(&v[16..], &["500", "40", "200"]);
      n += 1;
    }
  }
  assert_eq!(n, 20000);
  assert_eq!(zero, [true; 8]);
  assert!(rows.next().is_none());
}
#[test]
fn independent_refits_draws_reporting_and_order() {
  let d = TestDir::new();
  prepare(
    &d,
    include_str!("fixtures/mma23_gamma_re_bootstrap/source.asc"),
  );
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
  let path = d.path().join("gamma_re_bootstrap_draws.csv");
  let bytes = fs::read(&path).unwrap();
  check_draws(std::str::from_utf8(&bytes).unwrap(), &r);
  assert_eq!(fs::read(d.path().join("patents.csv")).unwrap(), input);
  let fail = run(&d, "--json");
  assert!(!fail.status.success());
  assert!(String::from_utf8_lossy(&fail.stderr).contains("export target already exists"));
  assert_eq!(fs::read(&path).unwrap(), bytes);
  fs::remove_file(&path).unwrap();
  let out = run(&d, "--text");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let output = String::from_utf8(out.stdout).unwrap();
  assert!(
    output.contains("covariance_tt") && output.contains("whole_firm_pairs_distinct_gamma_copies")
  );
  check_draws(&fs::read_to_string(&path).unwrap(), &r);
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
  let r2 = report(&events(&out));
  check_report(&r2);
  check_draws(&fs::read_to_string(&path).unwrap(), &r2);
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
    .split_once("\n\"\"\" into gamma_re_bootstrap_draws")
    .unwrap()
    .0;
  format!("sql \"\"\"WITH RECURSIVE\n{body}\"\"\" into gamma_re_bootstrap_draws")
}
fn search_probe(rep: usize) -> String {
  let body = script()
    .split_once("sql \"\"\"\nWITH RECURSIVE\n")
    .unwrap()
    .1
    .split_once("), ranked AS MATERIALIZED (")
    .unwrap()
    .0;
  let body = body.replace(
    "(SELECT DISTINCT rep FROM frequencies) r",
    &format!("(SELECT DISTINCT rep FROM frequencies WHERE rep={rep}) r"),
  );
  format!(
    "sql \"\"\"WITH RECURSIVE\n{body}) SELECT rep,MAX(iter) AS iterations FROM search GROUP BY rep\"\"\""
  )
}
fn exec(s: &mut Session, text: &str) -> tabdat_domain::ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}
#[test]
fn invalid_input_actual_failed_replicate_and_prior_fit_state() {
  let d = TestDir::new();
  prepare(
    &d,
    include_str!("fixtures/mma23_gamma_re_bootstrap/source.asc"),
  );
  let path = d.path().join("patents.csv");
  let source = fs::read_to_string(&path).unwrap();
  let export = d.path().join("gamma_re_bootstrap_draws.csv");
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
  // Identical firm panels have no between-firm variation and no regular interior dispersion fit.
  let boundary = rewrite(&source, |r| {
    let t = r[1].parse::<usize>().unwrap() - 1975;
    r[7] = ((t as f64 - 2.0) * 0.2).to_string();
    r[8] = [1, 2, 2, 3, 4][t].to_string();
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
  prepare(&d, include_str!("fixtures/mma23_conditional_fe/source.asc"));
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
  // The original converges, while prescribed draw 13 has non-positive full-Newton information at its second iterate.
  let reference = include_str!("fixtures/mma23_gamma_re_bootstrap/failure_reference.csv")
    .lines()
    .last()
    .unwrap();
  let values: Vec<_> = reference.split(',').collect();
  assert_eq!(&values[..2], &["13", "1"]);
  close(values[6].parse().unwrap(), -0.153077776588151);
  exec(&mut s, &search_probe(0));
  assert!(
    s.execute(parse_command(&search_probe(13)).unwrap())
      .is_err()
  );
  let before = s.state();
  let rows = exec(&mut s, "head 100");
  assert!(s.execute(parse_command(&query()).unwrap()).is_err());
  assert_eq!(s.state(), before);
  assert_eq!(exec(&mut s, "head 100"), rows);
  exec(&mut s, "predict still_available, xb");
}
#[test]
fn singular_design_and_reconstructed_start_guards_preserve_state() {
  let d = TestDir::new();
  prepare(
    &d,
    include_str!("fixtures/mma23_gamma_re_bootstrap/source.asc"),
  );
  for probe in [
    "SELECT * EXCLUDE(logr),1.0+(year-1975)*1e-9 AS logr,0.0 AS pooled_xb FROM active",
    "SELECT *,50.0+logr AS pooled_xb FROM active",
    "SELECT *,logr*logr AS pooled_xb FROM active",
  ] {
    let mut s = Session::new();
    exec(
      &mut s,
      &format!("use \"{}\"", d.path().join("patents.csv").display()),
    );
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
#[test]
fn successful_composition_retains_pooled_fit_not_gamma_state() {
  let d = TestDir::new();
  prepare(
    &d,
    include_str!("fixtures/mma23_gamma_re_bootstrap/source.asc"),
  );
  let mut s = Session::new();
  exec(
    &mut s,
    &format!("use \"{}\"", d.path().join("patents.csv").display()),
  );
  exec(&mut s, "poisson pat logr");
  exec(&mut s, "predict pooled_xb, xb");
  let fit = s.state().estimation;
  exec(&mut s, &query());
  assert_eq!(s.state().estimation, fit);
  // Draw output contains no LOGR; retained native prediction cannot run on it.
  assert!(
    s.execute(parse_command("predict pooled_after, n").unwrap())
      .is_err()
  );
}
#[test]
fn rng_iteration_predictor_and_incomplete_sample_guards() {
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
  prepare(
    &d,
    include_str!("fixtures/mma23_gamma_re_bootstrap/source.asc"),
  );
  let input = fs::read(d.path().join("patents.csv")).unwrap();
  for (old, new) in [
    (
      "WHERE iter<100 AND (iter=0 OR last_step>1e-12)",
      "WHERE iter<1 AND (iter=0 OR last_step>1e-12)",
    ),
    ("step<g+1024", "step<0"),
    ("ABS(a+b*s.logr)>40", "ABS(a+b*s.logr)>0"),
    (
      "i11-i01*i01/p00<=2.842170943040401e-14",
      "i11-i01*i01/p00<=1e99",
    ),
    (
      "i22-i02*i02/p00-POWER(i12-i01*i02/p00,2)/p11<=2.842170943040401e-14",
      "i22-i02*i02/p00-POWER(i12-i01*i02/p00,2)/p11<=1e99",
    ),
    (
      "SELECT rep,0::BIGINT,start.a,start.b,0.0::DOUBLE,g",
      "SELECT rep,0::BIGINT,start.a,start.b,8.0::DOUBLE,g",
    ),
    (
      "WHERE f.rep=search.rep",
      "WHERE f.rep=search.rep AND (search.iter<>1 OR f.rep<>13 OR s.id<>1)",
    ),
    (">=1e-9+1e-10*ABS(intercept)", ">=0"),
  ] {
    assert!(script().contains(old));
    let out = run_variant(&d, &script().replace(old, new));
    assert!(!out.status.success(), "guard unexpectedly succeeded: {old}");
    assert!(!d.path().join("gamma_re_bootstrap_draws.csv").exists());
    assert_eq!(fs::read(d.path().join("patents.csv")).unwrap(), input);
  }
}
