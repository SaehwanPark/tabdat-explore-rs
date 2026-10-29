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
  include_str!("../examples/mma23p1pannonlin/normal_re_poisson.td")
}
fn prepare(d: &TestDir) {
  fs::write(
    d.path().join("source.asc"),
    include_str!("fixtures/mma23_gamma_re_bootstrap/source.asc"),
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
    .arg(root().join("examples/mma23p1pannonlin/normal_re_poisson.td"))
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
fn close(a: f64, b: f64) {
  assert!(
    a.is_finite() && (a - b).abs() <= 1e-9 + 1e-10 * b.abs(),
    "{a} != {b}"
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
  for line in include_str!("fixtures/mma23_normal_re/reference.csv")
    .lines()
    .skip(1)
  {
    let (k, v) = line.split_once(',').unwrap();
    let expected: f64 = v.parse().unwrap();
    let actual = r[k].as_f64().unwrap();
    if k.starts_with("p_") && expected > 0.0 {
      assert!(
        actual.is_finite() && (actual - expected).abs() <= 1e-9 * expected,
        "{k}: {actual} != {expected}"
      );
    } else {
      close(actual, expected);
    }
    n += 1;
  }
  assert_eq!(n, 42);
  assert_eq!(r["inference"], "normal_asymptotic");
  assert_eq!(r["lr_reference"], "chi_bar_square_0_1_equal_mixture");
  assert!((1..=100).contains(&r["iterations"].as_u64().unwrap()));
  assert!((0.0..=1e-12).contains(&r["last_step"].as_f64().unwrap()));
  assert!((0.0..=1e-9).contains(&r["quad_diff"].as_f64().unwrap()));
}
fn check_predictions(text: &str, input: &str, r: &Value) {
  let mut rows = text.lines();
  assert_eq!(
    rows.next().unwrap(),
    "id,year,cusip,ardssic,scisect,logk,sumpat,logr,pat,pooled_xb,marginal_mu,marginal_residual,estimate_intercept,estimate_slope,estimate_lnvariance,sigma,covariance_aa,covariance_ab,covariance_at,covariance_bb,covariance_bt,covariance_tt,score_intercept,score_slope,score_lnvariance,log_likelihood,poisson_log_likelihood,lr_variance_zero,iterations,last_step,quad_diff,candidate_rows,firms,zero_total_firms"
  );
  let mut source: Vec<Vec<&str>> = input
    .lines()
    .skip(1)
    .map(|s| s.split(',').collect())
    .collect();
  source.sort_by_key(|v| {
    (
      v[0].parse::<usize>().unwrap(),
      v[1].parse::<usize>().unwrap(),
    )
  });
  let refs = include_str!("fixtures/mma23_normal_re/predictions.csv")
    .lines()
    .skip(1);
  let mut n = 0;
  for (line, (src, reference)) in rows.by_ref().zip(source.iter().zip(refs)) {
    let v: Vec<_> = line.split(',').collect();
    let x: Vec<_> = reference.split(',').collect();
    for i in 0..9 {
      if src[i].is_empty() {
        assert!(v[i].is_empty())
      } else {
        close(v[i].parse().unwrap(), src[i].parse().unwrap());
      }
    }
    assert_eq!(v[0], x[0]);
    assert_eq!(v[1], x[1]);
    close(v[7].parse().unwrap(), x[2].parse().unwrap());
    close(v[8].parse().unwrap(), x[3].parse().unwrap());
    close(v[10].parse().unwrap(), x[4].parse().unwrap());
    close(v[11].parse().unwrap(), x[5].parse().unwrap());
    for (i, k) in [
      "estimate_intercept",
      "estimate_slope",
      "estimate_lnvariance",
      "sigma",
      "covariance_aa",
      "covariance_ab",
      "covariance_at",
      "covariance_bb",
      "covariance_bt",
      "covariance_tt",
      "score_intercept",
      "score_slope",
      "score_lnvariance",
      "log_likelihood",
      "poisson_log_likelihood",
      "lr_variance_zero",
      "iterations",
      "last_step",
      "quad_diff",
    ]
    .iter()
    .enumerate()
    {
      close(v[12 + i].parse().unwrap(), r[*k].as_f64().unwrap());
    }
    assert_eq!(&v[31..], &["200", "40", "8"]);
    n += 1;
  }
  assert_eq!(n, 200);
  assert!(rows.next().is_none());
}
#[test]
fn independent_source_covariance_predictions_text_json_and_order() {
  let d = TestDir::new();
  prepare(&d);
  let input = fs::read_to_string(d.path().join("patents.csv")).unwrap();
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
  let path = d.path().join("normal_re_poisson_predictions.csv");
  let bytes = fs::read(&path).unwrap();
  check_predictions(std::str::from_utf8(&bytes).unwrap(), &input, &r);
  assert_eq!(
    fs::read_to_string(d.path().join("patents.csv")).unwrap(),
    input
  );
  let out = run(&d, "--json");
  assert!(!out.status.success());
  assert!(String::from_utf8_lossy(&out.stderr).contains("export target already exists"));
  assert_eq!(fs::read(&path).unwrap(), bytes);
  fs::remove_file(&path).unwrap();
  let out = run(&d, "--text");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  assert!(String::from_utf8_lossy(&out.stdout).contains("covariance_tt"));
  check_predictions(&fs::read_to_string(&path).unwrap(), &input, &r);
  fs::remove_file(&path).unwrap();
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
  check_predictions(&fs::read_to_string(&path).unwrap(), &input, &r2);
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
    .split_once("\n\"\"\" into normal_re_poisson_predictions")
    .unwrap()
    .0;
  format!("sql \"\"\"WITH RECURSIVE\n{body}\"\"\" into normal_re_poisson_predictions")
}
fn exec(s: &mut Session, text: &str) -> tabdat_domain::ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}
#[test]
fn invalid_input_actual_boundary_and_retained_state() {
  let d = TestDir::new();
  prepare(&d);
  let path = d.path().join("patents.csv");
  let source = fs::read_to_string(&path).unwrap();
  let export = d.path().join("normal_re_poisson_predictions.csv");
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
  for input in cases {
    fs::write(&path, &input).unwrap();
    let out = run(&d, "--json");
    assert!(!out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("export target already exists"));
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
    assert_eq!(fs::read(&export).unwrap(), b"sentinel");
  }
  // Repeated identical panels have no between-firm variation and no interior RE fit.
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
fn successful_composition_retains_pooled_not_normal_re_fit() {
  let d = TestDir::new();
  prepare(&d);
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
  exec(&mut s, "predict pooled_after, n");
  exec(
    &mut s,
    "sql \"\"\"SELECT CASE WHEN MAX(ABS(pooled_after-EXP(pooled_xb)))>1e-10 OR MAX(ABS(pooled_after-marginal_mu))<1e-6 THEN error('retained model is not the pooled fit') ELSE 1 END AS correct FROM active\"\"\"",
  );
}
#[test]
fn quadrature_mode_iteration_information_and_sample_guards() {
  let d = TestDir::new();
  prepare(&d);
  let input = fs::read(d.path().join("patents.csv")).unwrap();
  for (old, new) in [
    (
      "WHERE iter<100 AND (iter=0 OR last_step>1e-12)",
      "WHERE iter<1 AND (iter=0 OR last_step>1e-12)",
    ),
    ("WHERE k<55", "WHERE k<1"),
    ("quad_diff>1e-9", "quad_diff>0"),
    (
      "ABS(c.center+SQRT(2)*c.scale*q.z)>100",
      "ABS(c.center+SQRT(2)*c.scale*q.z)>0",
    ),
    (
      "ABS(f.intercept+f.slope*s.logr+EXP(f.old_t)/2)>700",
      "ABS(f.intercept+f.slope*s.logr+EXP(f.old_t)/2)>0",
    ),
    (
      "ELSE EXP(a+b*s.logr) END AS mu FROM source s",
      "ELSE EXP(a+b*s.logr) END AS mu FROM source s WHERE iter<>1 OR id<>1",
    ),
    (
      "i11-i01*i01/p00<=2.842170943040401e-14",
      "i11-i01*i01/p00<=1e99",
    ),
    (">=1e-9+1e-10*ABS(intercept)", ">=0"),
  ] {
    assert!(script().contains(old));
    let out = run_variant(&d, &script().replace(old, new));
    assert!(!out.status.success(), "guard unexpectedly succeeded: {old}");
    assert!(!d.path().join("normal_re_poisson_predictions.csv").exists());
    assert_eq!(fs::read(d.path().join("patents.csv")).unwrap(), input);
  }
}
#[test]
fn quadrature_nodes_match_normal_moments() {
  let d = TestDir::new();
  prepare(&d);
  let mut s = Session::new();
  exec(
    &mut s,
    &format!("use \"{}\"", d.path().join("patents.csv").display()),
  );
  let values = script()
    .split_once("quadrature(level,node,z,weight) AS (VALUES\n")
    .unwrap()
    .1
    .split_once("\n),\nsearch(")
    .unwrap()
    .0;
  for (level, n) in [(80, 80), (96, 96)] {
    let q = format!(
      "sql \"\"\"WITH quadrature(level,node,z,weight) AS (VALUES\n{values}) SELECT COUNT(*) AS n,SUM(weight)/SQRT(PI()) AS m0,SUM(weight*z)/SQRT(PI()) AS m1,SUM(weight*z*z)/SQRT(PI()) AS m2,SUM(weight*POWER(z,4))/SQRT(PI()) AS m4,SUM(weight*POWER(z,6))/SQRT(PI()) AS m6 FROM quadrature WHERE level={level}\"\"\""
    );
    let tabdat_domain::ExecutionResult::Table(t) = exec(&mut s, &q) else {
      panic!("expected table")
    };
    let expected = [n as f64, 1.0, 0.0, 0.5, 0.75, 1.875];
    for (v, e) in t.rows[0].iter().zip(expected) {
      let a = match v {
        tabdat_domain::CellValue::SignedInteger(n) => *n as f64,
        tabdat_domain::CellValue::UnsignedInteger(n) => *n as f64,
        tabdat_domain::CellValue::Float(x) => *x,
        _ => panic!("expected number"),
      };
      close(a, e);
    }
  }
}
