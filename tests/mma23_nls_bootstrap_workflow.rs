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
fn script() -> &'static str {
  include_str!("../examples/mma23p1pannonlin/exponential_nls_bootstrap.td")
}
fn run(d: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(root().join("examples/mma23p1pannonlin/exponential_nls_bootstrap.td"))
    .output()
    .unwrap()
}
fn run_variant(d: &TestDir, content: &str) -> Output {
  fs::write(d.path().join("variant.td"), content).unwrap();
  tabdat_bin()
    .current_dir(d.path())
    .args(["--json", "-f", "variant.td"])
    .output()
    .unwrap()
}
fn close(a: f64, b: f64) {
  assert!(
    a.is_finite() && (a - b).abs() <= 1e-9 + 1e-10 * b.abs(),
    "{a} != {b}"
  );
}
fn events(out: Output) -> Vec<Value> {
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
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
  let mut count = 0;
  for line in include_str!("fixtures/mma23_nls_bootstrap/reference.csv")
    .lines()
    .skip(1)
  {
    let (field, value) = line.split_once(',').unwrap();
    close(r[field].as_f64().unwrap(), value.parse().unwrap());
    count += 1;
  }
  assert_eq!(count, 18);
  assert_eq!(r["replications"], 500);
  assert_eq!(r["firms_per_draw"], 12);
  assert_eq!(r["base_seed"], 10001);
  assert_eq!(r["method"], "whole_firm_pairs");
  assert_eq!(r["interval_method"], "type7_percentile_95");
  assert_eq!(
    r["rng"],
    "xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1"
  );
}
fn check_native(ev: &[Value]) {
  let n = &ev.iter().find(|e| e["result_type"] == "NlsResult").unwrap()["data"];
  assert_eq!(n["observation_count"], 60);
  assert_eq!(n["sample"]["dropped_observations"], 0);
  let fitted = n["fitted_values"].as_array().unwrap();
  let residuals = n["residuals"].as_array().unwrap();
  let refs = include_str!("fixtures/mma23_nls_bootstrap/predictions.csv");
  assert_eq!(fitted.len(), 60);
  assert_eq!(residuals.len(), 60);
  for ((line, mu), res) in refs.lines().skip(1).zip(fitted).zip(residuals) {
    let cols: Vec<_> = line.split(',').collect();
    close(mu.as_f64().unwrap(), cols[4].parse().unwrap());
    close(res.as_f64().unwrap(), cols[5].parse().unwrap());
  }
}
fn check_draws(csv: &str, r: &Value) {
  let mut rows = csv.lines();
  assert_eq!(
    rows.next().unwrap(),
    "replicate,position,step,firm_id,replicate_intercept,replicate_slope,estimate_a,estimate_b,replications,firms_per_draw,candidate_rows"
  );
  let betas: Vec<(f64, f64)> = include_str!("fixtures/mma23_nls_bootstrap/fits.csv")
    .lines()
    .skip(1)
    .map(|line| {
      let cols: Vec<_> = line.split(',').collect();
      (cols[1].parse().unwrap(), cols[2].parse().unwrap())
    })
    .collect();
  assert_eq!(betas.len(), 500);
  let mut count = 0;
  let mut zero_firms = [false; 2];
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
      let firm = row[3].parse::<usize>().unwrap();
      assert_eq!(firm, index.parse::<usize>().unwrap() + 1);
      if firm == 3 {
        zero_firms[0] = true;
      }
      if firm == 9 {
        zero_firms[1] = true;
      }
      close(row[4].parse().unwrap(), betas[i].0);
      close(row[5].parse().unwrap(), betas[i].1);
      close(
        row[6].parse().unwrap(),
        r["estimate_intercept"].as_f64().unwrap(),
      );
      close(
        row[7].parse().unwrap(),
        r["estimate_slope"].as_f64().unwrap(),
      );
      assert_eq!(&row[8..], &["500", "12", "60"]);
      count += 1;
    }
  }
  assert_eq!(count, 6000);
  assert_eq!(zero_firms, [true, true]);
  assert!(rows.next().is_none());
}
#[test]
fn independent_refits_draws_predictions_reporting_and_order() {
  let d = TestDir::new();
  prepare(&d);
  let input = fs::read(d.path().join("patents.csv")).unwrap();
  let ev = events(run(&d, "--json"));
  let r = report(&ev);
  check_report(&r);
  check_native(&ev);
  let export = d.path().join("exponential_nls_bootstrap_draws.csv");
  let bytes = fs::read(&export).unwrap();
  check_draws(std::str::from_utf8(&bytes).unwrap(), &r);
  assert_eq!(fs::read(d.path().join("patents.csv")).unwrap(), input);
  let fail = run(&d, "--json");
  assert!(!fail.status.success());
  assert!(String::from_utf8_lossy(&fail.stderr).contains("export target already exists"));
  assert_eq!(fs::read(&export).unwrap(), bytes);
  fs::remove_file(&export).unwrap();
  let text = run(&d, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("covariance_aa") && text.contains("whole_firm_pairs"));
  assert_eq!(fs::read(&export).unwrap(), bytes);
  fs::remove_file(&export).unwrap();
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
  let r = report(&events(run(&d, "--json")));
  check_report(&r);
  assert_eq!(fs::read(&export).unwrap(), bytes);
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
fn fail(d: &TestDir, input: &str) -> Output {
  fs::write(d.path().join("patents.csv"), input).unwrap();
  let out = run(d, "--json");
  assert!(!out.status.success());
  assert!(!String::from_utf8_lossy(&out.stderr).contains("export target already exists"));
  assert_eq!(
    fs::read_to_string(d.path().join("patents.csv")).unwrap(),
    input
  );
  assert_eq!(
    fs::read(d.path().join("exponential_nls_bootstrap_draws.csv")).unwrap(),
    b"sentinel"
  );
  out
}
fn query() -> String {
  let body = script()
    .split_once("sql \"\"\"\nWITH RECURSIVE\n")
    .unwrap()
    .1
    .split_once("\n\"\"\" into exponential_nls_bootstrap_draws")
    .unwrap()
    .0;
  format!("sql \"\"\"WITH RECURSIVE\n{body}\"\"\" into exponential_nls_bootstrap_draws")
}
fn exec(s: &mut Session, text: &str) -> tabdat_domain::ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}
#[test]
fn invalid_input_failed_replicates_and_prior_fit_state() {
  let d = TestDir::new();
  prepare(&d);
  let path = d.path().join("patents.csv");
  let source = fs::read_to_string(&path).unwrap();
  fs::write(
    d.path().join("exponential_nls_bootstrap_draws.csv"),
    b"sentinel",
  )
  .unwrap();
  let mut dup: Vec<_> = source.lines().map(str::to_owned).collect();
  dup[2] = dup[1].clone();
  let one = source.lines().take(6).collect::<Vec<_>>().join("\n") + "\n";
  let collision = source
    .lines()
    .enumerate()
    .map(|(i, line)| format!("{line},{}", if i == 0 { "nls_mu" } else { "1" }))
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  let cases = [
    dup.join("\n") + "\n",
    one,
    collision,
    rewrite(&source, |r| r[7] = "1".into()),
    rewrite(&source, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "".into();
      }
    }),
    rewrite(&source, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "NaN".into();
      }
    }),
    rewrite(&source, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[8] = "-1".into();
      }
    }),
    rewrite(&source, |r| {
      if r[0] == "1" {
        r[0] = "347".into();
      }
    }),
    rewrite(&source, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[8] = "1001".into();
      }
    }),
    rewrite(&source, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[8] = "1.5".into();
      }
    }),
    rewrite(&source, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "21".into();
      }
    }),
    source.replacen(",1975,", ",1980,", 1),
  ];
  for case in cases {
    fail(&d, &case);
  }
  // The original has an interior valid fit, but deterministic draws include no positive firm.
  let zero = rewrite(&source, |r| {
    let t = r[1].parse::<i32>().unwrap() - 1975;
    r[7] = ((t - 2) as f64 * 0.2).to_string();
    r[8] = if r[0] == "1" {
      [2, 3, 4, 4, 5][t as usize].to_string()
    } else {
      "0".into()
    };
  });
  assert!(
    include_str!("fixtures/mma23_pooled_bootstrap/draws.csv")
      .lines()
      .skip(1)
      .any(|line| line
        .split_once(',')
        .unwrap()
        .1
        .split(':')
        .all(|index| index != "0"))
  );
  let out = fail(&d, &zero);
  let ev: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let n = &ev.iter().find(|e| e["result_type"] == "NlsResult").unwrap()["data"];
  let slope = n["coefficients"]
    .as_array()
    .unwrap()
    .iter()
    .find(|c| c["name"] == "b")
    .unwrap()["value"]
    .as_f64()
    .unwrap();
  assert!(slope.abs() < 2.);
  assert_eq!(
    ev.iter()
      .filter(|e| e["result_type"] == "PredictionResult")
      .count(),
    2
  );
  assert!(!ev.iter().any(|e| e["result_type"] == "SqlCreateResult"));
  // Both original firms have varying amplitudes; a one-firm resample has exact zero RSS.
  let two = source
    .lines()
    .filter(|line| line.starts_with("id,") || line.starts_with("1,") || line.starts_with("2,"))
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  let perfect = rewrite(&two, |r| {
    let t = r[1].parse::<i32>().unwrap() - 1975;
    let id = r[0].parse::<i32>().unwrap();
    r[7] = ((t as f64) * std::f64::consts::LN_2).to_string();
    r[8] = (id * (1 << t)).to_string();
  });
  let ev: Vec<Value> = String::from_utf8(fail(&d, &perfect).stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let n = &ev.iter().find(|e| e["result_type"] == "NlsResult").unwrap()["data"];
  assert!(n["residual_sum_of_squares"].as_f64().unwrap() > 0.);
  // Replicate 2 draws the first firm twice. Its exact profile has a flat root
  // at zero; rounding previously produced a false, apparently positive curvature.
  let mixed = rewrite(&two, |r| {
    let t = r[1].parse::<usize>().unwrap() - 1975;
    r[7] = [-1, 0, 0, 0, 1][t].to_string();
    r[8] = if r[0] == "1" {
      [4, 1, 0, 1, 4][t]
    } else {
      [1, 2, 2, 1, 3][t]
    }
    .to_string();
  });
  let ev: Vec<Value> = String::from_utf8(fail(&d, &mixed).stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let n = &ev.iter().find(|e| e["result_type"] == "NlsResult").unwrap()["data"];
  let b = n["coefficients"]
    .as_array()
    .unwrap()
    .iter()
    .find(|c| c["name"] == "b")
    .unwrap()["value"]
    .as_f64()
    .unwrap();
  assert!(b > 0.5 && b < 0.6);
  // Original native slope is outside the bounded bootstrap root interval.
  let outside = rewrite(&source, |r| {
    let t = r[1].parse::<usize>().unwrap() - 1975;
    r[7] = (t as f64 * 0.25).to_string();
    r[8] = ([2, 4, 9, 19, 40][t] + r[0].parse::<i32>().unwrap() % 3).to_string();
  });
  let ev: Vec<Value> = String::from_utf8(fail(&d, &outside).stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let n = &ev.iter().find(|e| e["result_type"] == "NlsResult").unwrap()["data"];
  assert!(
    n["coefficients"]
      .as_array()
      .unwrap()
      .iter()
      .find(|c| c["name"] == "b")
      .unwrap()["value"]
      .as_f64()
      .unwrap()
      > 2.
  );
  fs::write(&path, &zero).unwrap();
  let mut s = Session::new();
  exec(&mut s, &format!("use \"{}\"", path.display()));
  exec(&mut s, "nl pat = exp(a+b*logr), params(a b) start(2.5 0.5)");
  exec(&mut s, "predict nls_mu, mean");
  exec(&mut s, "predict nls_residual, residuals");
  let before = s.state();
  let rows = exec(&mut s, "head 100");
  assert!(s.execute(parse_command(&query()).unwrap()).is_err());
  assert_eq!(s.state(), before);
  assert_eq!(exec(&mut s, "head 100"), rows);
  exec(&mut s, "predict still_available, mean");
}
#[test]
fn exact_and_near_profile_endpoint_guards() {
  let d = TestDir::new();
  prepare(&d);
  let path = d.path().join("patents.csv");
  let source = fs::read_to_string(&path).unwrap();
  let two = source
    .lines()
    .filter(|line| line.starts_with("id,") || line.starts_with("1,") || line.starts_with("2,"))
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  let body = script()
    .split_once("sql \"\"\"\nWITH RECURSIVE\n")
    .unwrap()
    .1
    .split_once("), search(rep,iter,lo,hi) AS (")
    .unwrap()
    .0;
  let endpoint_query = format!(
    "sql \"\"\"WITH RECURSIVE\n{body}) SELECT COUNT(*) AS valid FROM bounds WHERE valid=1\"\"\""
  );
  for (slope, accepted) in [
    (-2., false),
    (-2. + 1e-14, false),
    (-1.9, true),
    (1.9, true),
    (2. - 1e-14, false),
    (2., false),
  ] {
    let input = rewrite(&two, |row| {
      let t = row[1].parse::<i32>().unwrap() - 1975;
      let id = row[0].parse::<i32>().unwrap();
      row[7] = ((t as f64) * std::f64::consts::LN_2 / slope).to_string();
      row[8] = (id * (1 << t)).to_string();
    });
    fs::write(&path, input).unwrap();
    let mut s = Session::new();
    exec(&mut s, &format!("use \"{}\"", path.display()));
    exec(
      &mut s,
      "sql \"\"\"SELECT *,1.0 AS nls_mu FROM active\"\"\" into endpoint_probe",
    );
    let before = s.state();
    let outcome = s.execute(parse_command(&endpoint_query).unwrap());
    if accepted {
      let tabdat_domain::ExecutionResult::Table(table) = outcome.unwrap() else {
        panic!("expected endpoint table")
      };
      assert_eq!(
        table.rows[0][0],
        tabdat_domain::CellValue::SignedInteger(501)
      );
    } else {
      assert!(outcome.is_err(), "endpoint {slope} unexpectedly accepted");
      assert_eq!(s.state(), before);
    }
  }
}

#[test]
fn cancellation_dominated_observed_curvature_is_rejected() {
  let d = TestDir::new();
  prepare(&d);
  let path = d.path().join("patents.csv");
  let source = fs::read_to_string(&path).unwrap();
  // At b=0, the two profile means are zero, Var_y(x)=0.8 and Var_w(x)=0.4;
  // hence the observed-loss curvature 2*Var_w-Var_y is exactly zero.
  let flat = rewrite(&source, |row| {
    let t = row[1].parse::<usize>().unwrap() - 1975;
    row[7] = [-1, 0, 0, 0, 1][t].to_string();
    row[8] = [4, 1, 0, 1, 4][t].to_string();
  });
  fs::write(&path, flat).unwrap();
  for beta in ["0.0", "1e-8", "1e-7"] {
    let mut s = Session::new();
    exec(&mut s, &format!("use \"{}\"", path.display()));
    exec(
      &mut s,
      "sql \"\"\"SELECT *,2.0 AS nls_mu FROM active\"\"\" into curvature_probe",
    );
    let before = s.state();
    let original = "SELECT rep,(lo+hi)/2 AS beta FROM search WHERE iter=80";
    assert!(query().contains(original));
    let injected = query().replace(
      original,
      &format!("SELECT rep,{beta}::DOUBLE AS beta FROM search WHERE iter=80"),
    );
    assert!(
      s.execute(parse_command(&injected).unwrap()).is_err(),
      "flat curvature at {beta} accepted"
    );
    assert_eq!(s.state(), before);
  }
}

#[test]
fn rng_identity_and_bounded_work_fail_before_export() {
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
  prepare(&d);
  let original = fs::read(d.path().join("patents.csv")).unwrap();
  let export = d.path().join("exponential_nls_bootstrap_draws.csv");
  fs::write(&export, b"sentinel").unwrap();
  for (from, to) in [
    ("step<g+1024", "step<1"),
    ("AS step WHERE iter<80", "AS step WHERE iter<1"),
  ] {
    assert!(script().contains(from));
    let out = run_variant(&d, &script().replace(from, to));
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("sql failed"));
    assert_eq!(fs::read(&export).unwrap(), b"sentinel");
    assert_eq!(fs::read(d.path().join("patents.csv")).unwrap(), original);
  }
}
