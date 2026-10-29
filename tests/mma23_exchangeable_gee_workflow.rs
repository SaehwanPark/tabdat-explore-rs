mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  collections::BTreeMap,
  fs,
  path::PathBuf,
  process::{Command, Output},
};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn prepare(d: &TestDir, negative: bool) {
  let src = if negative {
    include_str!("fixtures/mma23_exchangeable_gee/negative/source.asc")
  } else {
    include_str!("fixtures/mma23_conditional_fe/source.asc")
  };
  fs::write(d.path().join("source.asc"), src).unwrap();
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
    .arg(root().join("examples/mma23p1pannonlin/exchangeable_gee.td"))
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
  let table = ev
    .iter()
    .find(|e| {
      e["result_type"] == "TableResult"
        && e["data"]["headers"]
          .as_array()
          .unwrap()
          .contains(&Value::from("working_aa"))
    })
    .unwrap();
  let keys = table["data"]["headers"].as_array().unwrap();
  let vals = table["data"]["rows"][0].as_array().unwrap();
  Value::Object(
    keys
      .iter()
      .zip(vals)
      .map(|(k, v)| (k.as_str().unwrap().into(), v.clone()))
      .collect(),
  )
}
fn check_report(r: &Value, reference: &str, negative: bool) {
  let mut n = 0;
  for line in reference.lines().skip(1) {
    let (field, val) = line.split_once(',').unwrap();
    close(r[field].as_f64().unwrap(), val.parse().unwrap());
    n += 1;
  }
  assert_eq!(n, 27);
  assert_eq!(r["observations"], 60);
  assert_eq!(r["firms"], 12);
  assert_eq!(
    r["correlation"],
    "exchangeable_pearson_legacy_moment_fixed_scale_1"
  );
  assert_eq!(r["working_covariance"], "gee_working_scale_1");
  assert_eq!(r["cluster_covariance"], "firm_score_G_over_G_minus_1");
  assert_eq!(r["inference"], "asymptotic_normal; p-values not computed");
  assert!((1..=100).contains(&r["iterations"].as_i64().unwrap()));
  assert!(r["last_step"].as_f64().unwrap() <= 1e-12);
  assert_eq!(r["rho"].as_f64().unwrap() < 0., negative);
}
fn csv_rows(csv: &str) -> (Vec<&str>, Vec<Vec<&str>>) {
  let mut lines = csv.lines();
  let headers = lines.next().unwrap().split(',').collect();
  (headers, lines.map(|l| l.split(',').collect()).collect())
}
fn check_predictions(actual: &str, expected: &str, scores: &str, r: &Value) {
  let (headers, rows) = csv_rows(actual);
  let (refheaders, refs) = csv_rows(expected);
  let col = |name| headers.iter().position(|h| *h == name).unwrap();
  let refcol = |name| refheaders.iter().position(|h| *h == name).unwrap();
  assert_eq!(rows.len(), 60);
  assert_eq!(rows.len(), refs.len());
  let mut groups: BTreeMap<usize, Vec<(f64, f64, f64)>> = BTreeMap::new();
  let mut previous = None;
  for (row, expected) in rows.iter().zip(refs) {
    let key = (
      row[col("id")].parse::<usize>().unwrap(),
      row[col("year")].parse::<usize>().unwrap(),
    );
    if let Some(last) = previous {
      assert!(last < key);
    }
    previous = Some(key);
    for name in ["id", "year", "logr", "pat", "gee_mu", "gee_residual"] {
      close(
        row[col(name)].parse().unwrap(),
        expected[refcol(name)].parse().unwrap(),
      );
    }
    groups.entry(key.0).or_default().push((
      row[col("logr")].parse().unwrap(),
      row[col("pat")].parse().unwrap(),
      row[col("gee_mu")].parse().unwrap(),
    ));
  }
  assert_eq!(groups.len(), 12);
  let rho = r["rho"].as_f64().unwrap();
  let k = 1. / (1. - rho);
  let c = rho / (1. + 4. * rho);
  for line in scores.lines().skip(1) {
    let v: Vec<_> = line.split(',').collect();
    let id = v[0].parse::<usize>().unwrap();
    let data = &groups[&id];
    assert_eq!(data.len(), 5);
    let mut h0 = 0.;
    let mut h1 = 0.;
    let mut e = 0.;
    let mut d0 = 0.;
    let mut d1 = 0.;
    for &(x, y, mu) in data {
      let s = mu.sqrt();
      h0 += s;
      h1 += s * x;
      e += (y - mu) / s;
      d0 += y - mu;
      d1 += x * (y - mu);
    }
    close(k * (d0 - c * h0 * e), v[1].parse().unwrap());
    close(k * (d1 - c * h1 * e), v[2].parse().unwrap());
  }
}

#[test]
fn positive_and_negative_rho_r_fits_predictions_and_order() {
  for negative in [false, true] {
    let d = TestDir::new();
    prepare(&d, negative);
    let original = fs::read(d.path().join("patents.csv")).unwrap();
    let ev = events(run(&d, "--json"));
    let r = report(&ev);
    let (reference, predictions, scores) = if negative {
      (
        include_str!("fixtures/mma23_exchangeable_gee/negative/reference.csv"),
        include_str!("fixtures/mma23_exchangeable_gee/negative/predictions.csv"),
        include_str!("fixtures/mma23_exchangeable_gee/negative/scores.csv"),
      )
    } else {
      (
        include_str!("fixtures/mma23_exchangeable_gee/positive/reference.csv"),
        include_str!("fixtures/mma23_exchangeable_gee/positive/predictions.csv"),
        include_str!("fixtures/mma23_exchangeable_gee/positive/scores.csv"),
      )
    };
    check_report(&r, reference, negative);
    assert_eq!(
      ev.iter()
        .filter(|e| e["result_type"] == "PoissonResult")
        .count(),
      1
    );
    let export = d.path().join("exchangeable_gee_predictions.csv");
    let bytes = fs::read(&export).unwrap();
    check_predictions(
      &String::from_utf8(bytes.clone()).unwrap(),
      predictions,
      scores,
      &r,
    );
    assert_eq!(fs::read(d.path().join("patents.csv")).unwrap(), original);
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
    assert!(text.contains("working_aa") && text.contains("firm_score_G_over_G_minus_1"));
    assert_eq!(fs::read(&export).unwrap(), bytes);
    fs::remove_file(&export).unwrap();
    let input = String::from_utf8(original).unwrap();
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
    check_report(&r, reference, negative);
    check_predictions(
      &fs::read_to_string(&export).unwrap(),
      predictions,
      scores,
      &r,
    );
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
fn failed_without_export(d: &TestDir, source: &str) -> Output {
  fs::write(d.path().join("patents.csv"), source).unwrap();
  let out = run(d, "--json");
  assert!(!out.status.success());
  assert!(!String::from_utf8_lossy(&out.stderr).contains("export target already exists"));
  assert_eq!(
    fs::read_to_string(d.path().join("patents.csv")).unwrap(),
    source
  );
  assert_eq!(
    fs::read(d.path().join("exchangeable_gee_predictions.csv")).unwrap(),
    b"sentinel"
  );
  out
}
fn gee_query() -> String {
  let script = include_str!("../examples/mma23p1pannonlin/exchangeable_gee.td");
  let body = script
    .split_once("sql \"\"\"\nWITH RECURSIVE\n")
    .unwrap()
    .1
    .split_once("\n\"\"\" into exchangeable_gee_predictions")
    .unwrap()
    .0;
  format!("sql \"\"\"WITH RECURSIVE\n{body}\"\"\" into exchangeable_gee_predictions")
}
fn exec(s: &mut Session, text: &str) -> tabdat_domain::ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}

#[test]
fn invalid_inputs_and_late_failure_preserve_data_and_prior_fit() {
  let d = TestDir::new();
  prepare(&d, false);
  let path = d.path().join("patents.csv");
  let original = fs::read_to_string(&path).unwrap();
  fs::write(
    d.path().join("exchangeable_gee_predictions.csv"),
    b"sentinel",
  )
  .unwrap();
  let mut duplicate: Vec<_> = original.lines().map(str::to_owned).collect();
  duplicate[2] = duplicate[1].clone();
  let one_firm = original.lines().take(6).collect::<Vec<_>>().join("\n") + "\n";
  let collision = original
    .lines()
    .enumerate()
    .map(|(i, l)| format!("{l},{}", if i == 0 { "pooled_xb" } else { "3" }))
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  let cases = [
    duplicate.join("\n") + "\n",
    one_firm,
    collision,
    rewrite(&original, |r| r[7] = "1".into()),
    rewrite(&original, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "".into();
      }
    }),
    rewrite(&original, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "NaN".into();
      }
    }),
    rewrite(&original, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[8] = "-1".into();
      }
    }),
    original.replacen(",1975,", ",1980,", 1),
  ];
  for case in cases {
    failed_without_export(&d, &case);
  }
  // Each firm's residual vector is constant, so rho=1 after a valid pooled fit.
  let boundary = rewrite(&original, |r| {
    let id: usize = r[0].parse().unwrap();
    let year: i32 = r[1].parse().unwrap();
    r[7] = ((year - 1977) as f64 * 0.2).to_string();
    r[8] = if id <= 6 { "0" } else { "2" }.into();
  });
  let out = failed_without_export(&d, &boundary);
  assert!(String::from_utf8_lossy(&out.stderr).contains("sql failed"));
  let ev: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  assert!(
    ev.iter()
      .any(|e| e["result_type"] == "PoissonResult" && e["data"]["converged"] == true)
  );
  assert_eq!(
    ev.iter()
      .filter(|e| e["result_type"] == "PredictionResult")
      .count(),
    1
  );
  assert!(!ev.iter().any(|e| e["result_type"] == "SqlCreateResult"));
  let mut session = Session::new();
  exec(&mut session, &format!("use \"{}\"", path.display()));
  exec(&mut session, "poisson pat logr");
  exec(&mut session, "predict pooled_xb, xb");
  let before = session.state();
  let rows = exec(&mut session, "head 100");
  assert!(
    session
      .execute(parse_command(&gee_query()).unwrap())
      .is_err()
  );
  assert_eq!(session.state(), before);
  assert_eq!(exec(&mut session, "head 100"), rows);
  exec(&mut session, "predict still_available, n");
}

#[test]
fn numerical_guards_reject_boundaries_singular_bread_and_nonconvergence() {
  let d = TestDir::new();
  prepare(&d, false);
  let path = d.path().join("patents.csv");
  let use_cmd = format!("use \"{}\"", path.display());
  let mut control = Session::new();
  exec(&mut control, &use_cmd);
  exec(&mut control, "poisson pat logr");
  exec(&mut control, "predict pooled_xb, xb");
  exec(&mut control, &gee_query());
  // Fractional outcomes below are intentional isolated mathematical guard probes,
  // not accepted inputs to the public workflow's count-domain preflight.
  let probes = [
    "SELECT * EXCLUDE(pat),CASE WHEN id<=6 THEN 0 ELSE 2 END AS pat,0.0 AS pooled_xb FROM active",
    "SELECT * EXCLUDE(pat),2.0+CASE WHEN year=1975 THEN 1e-7 ELSE 0 END AS pat,0.0 AS pooled_xb FROM active",
    "SELECT * EXCLUDE(pat),CASE WHEN year IN (1975,1976) THEN 1 WHEN year=1977 THEN 2 ELSE 3 END AS pat,LN(2.0) AS pooled_xb FROM active",
    "SELECT * EXCLUDE(pat),CASE WHEN year IN (1975,1976) THEN 1 WHEN year=1977 THEN 2 ELSE 3 END+CASE WHEN year=1975 THEN 1e-7 ELSE 0 END AS pat,LN(2.0) AS pooled_xb FROM active",
    "SELECT * EXCLUDE(pat),2.0 AS pat,LN(2.0) AS pooled_xb FROM active",
    "SELECT * EXCLUDE(logr),1.0+(year-1975)*1e-9 AS logr,0.0 AS pooled_xb FROM active",
    "SELECT *,50.0+logr AS pooled_xb FROM active",
  ];
  for probe in probes {
    let mut s = Session::new();
    exec(&mut s, &use_cmd);
    exec(&mut s, &format!("sql \"\"\"{probe}\"\"\" into probe"));
    let before = s.state();
    let rows = exec(&mut s, "head 100");
    assert!(
      s.execute(parse_command(&gee_query()).unwrap()).is_err(),
      "guard unexpectedly accepted: {probe}"
    );
    assert_eq!(s.state(), before);
    assert_eq!(exec(&mut s, "head 100"), rows);
  }
  let mut s = Session::new();
  exec(&mut s, &use_cmd);
  exec(&mut s, "poisson pat logr");
  exec(&mut s, "predict pooled_xb, xb");
  let before = s.state();
  let limited = gee_query().replace(
    "WHERE iter<100 AND (iter=0 OR last_step>1e-12)",
    "WHERE iter<1 AND (iter=0 OR last_step>1e-12)",
  );
  assert!(s.execute(parse_command(&limited).unwrap()).is_err());
  assert_eq!(s.state(), before);
}
