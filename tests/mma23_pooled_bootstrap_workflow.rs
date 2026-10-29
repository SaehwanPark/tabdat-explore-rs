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
    .arg(root().join("examples/mma23p1pannonlin/pooled_poisson_bootstrap.td"))
    .output()
    .unwrap()
}
fn close(a: f64, b: f64) {
  assert!(a.is_finite() && (a - b).abs() <= 1e-10, "{a} != {b}");
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
fn summary(events: &[Value]) -> Value {
  let e = events
    .iter()
    .find(|e| {
      e["result_type"] == "TableResult"
        && e["data"]["headers"]
          .as_array()
          .unwrap()
          .contains(&Value::from("covariance_aa"))
    })
    .unwrap();
  let headers = e["data"]["headers"].as_array().unwrap();
  let vals = e["data"]["rows"][0].as_array().unwrap();
  Value::Object(
    headers
      .iter()
      .zip(vals)
      .map(|(k, v)| (k.as_str().unwrap().into(), v.clone()))
      .collect(),
  )
}
fn reference() -> BTreeMap<String, f64> {
  include_str!("fixtures/mma23_pooled_bootstrap/reference.csv")
    .lines()
    .skip(1)
    .map(|l| {
      let (key, val) = l.split_once(',').unwrap();
      (key.to_owned(), val.parse().unwrap())
    })
    .collect()
}
fn check_summary(r: &Value) {
  let reference = reference();
  assert_eq!(reference.len(), 18);
  for (key, value) in reference {
    close(r[&key].as_f64().unwrap(), value);
  }
  assert_eq!(r["base_seed"], 10001);
  assert_eq!(r["method"], "whole_firm_pairs");
  assert_eq!(r["interval_method"], "type7_percentile_95");
  assert_eq!(
    r["rng"],
    "xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1"
  );
}

#[test]
fn r_oracle_all_draws_covariance_reporting_and_file_invariants() {
  let d = TestDir::new();
  prepare(&d);
  let input = fs::read(d.path().join("patents.csv")).unwrap();
  let ev = events(run(&d, "--json"));
  let r = summary(&ev);
  check_summary(&r);
  let poisson = &ev
    .iter()
    .find(|e| e["result_type"] == "PoissonResult")
    .unwrap()["data"];
  assert_eq!(poisson["observation_count"], 60);
  assert_eq!(poisson["sample"]["dropped_observations"], 0);
  close(
    poisson["coefficients"][0]["value"].as_f64().unwrap(),
    r["estimate_intercept"].as_f64().unwrap(),
  );
  close(
    poisson["coefficients"][1]["value"].as_f64().unwrap(),
    r["estimate_slope"].as_f64().unwrap(),
  );
  let predictions = ev
    .iter()
    .find(|e| e["result_type"] == "SummarizeResult")
    .unwrap()["data"]["rows"]
    .as_array()
    .unwrap();
  let expected = include_str!("fixtures/mma23_pooled_bootstrap/prediction_summary.csv");
  for (row, line) in predictions.iter().zip(expected.lines().skip(1)) {
    let cols: Vec<_> = line.split(',').collect();
    assert_eq!(row["variable"], cols[0]);
    for (i, key) in ["count", "minimum", "maximum", "mean", "std_dev"]
      .iter()
      .enumerate()
    {
      close(row[key].as_f64().unwrap(), cols[i + 1].parse().unwrap());
    }
  }
  assert_eq!(predictions.len(), 2);
  let export = d.path().join("pooled_poisson_bootstrap_draws.csv");
  let bytes = fs::read(&export).unwrap();
  let csv = String::from_utf8(bytes.clone()).unwrap();
  let mut rows = csv.lines();
  assert_eq!(
    rows.next().unwrap(),
    "replicate,position,step,firm_id,replicate_intercept,replicate_slope,estimate_a,estimate_b,replications,firms_per_draw,candidate_rows"
  );
  let betas: Vec<(f64, f64)> = include_str!("fixtures/mma23_pooled_bootstrap/replicates.csv")
    .lines()
    .skip(1)
    .map(|s| {
      let cols: Vec<_> = s.split(',').collect();
      (cols[1].parse().unwrap(), cols[2].parse().unwrap())
    })
    .collect();
  let mut count = 0;
  let mut zero_firms_seen = [false; 2];
  for (i, line) in include_str!("fixtures/mma23_pooled_bootstrap/draws.csv")
    .lines()
    .skip(1)
    .enumerate()
  {
    let (rep, indices) = line.split_once(',').unwrap();
    assert_eq!(rep.parse::<usize>().unwrap(), i + 1);
    for (pos, index) in indices.split(':').enumerate() {
      let row: Vec<_> = rows.next().unwrap().split(',').collect();
      assert_eq!(row[0].parse::<usize>().unwrap(), i + 1);
      assert_eq!(row[1].parse::<usize>().unwrap(), pos + 1);
      assert_eq!(row[2].parse::<usize>().unwrap(), pos + 1);
      let firm = row[3].parse::<usize>().unwrap();
      assert_eq!(firm, index.parse::<usize>().unwrap() + 1);
      if firm == 3 {
        zero_firms_seen[0] = true;
      }
      if firm == 9 {
        zero_firms_seen[1] = true;
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
      count += 1;
    }
  }
  assert_eq!(count, 6000);
  assert_eq!(zero_firms_seen, [true, true]);
  assert!(rows.next().is_none());
  assert_eq!(fs::read(d.path().join("patents.csv")).unwrap(), input);
  let out = run(&d, "--json");
  assert!(!out.status.success());
  assert!(String::from_utf8_lossy(&out.stderr).contains("export target already exists"));
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
  let mut rows: Vec<_> = lines.collect();
  rows.reverse();
  fs::write(
    d.path().join("patents.csv"),
    format!("{header}\n{}\n", rows.join("\n")),
  )
  .unwrap();
  let r = summary(&events(run(&d, "--json")));
  check_summary(&r);
  assert_eq!(fs::read(&export).unwrap(), bytes);
}

fn rewrite<F: FnMut(&mut Vec<String>)>(input: &str, mut f: F) -> String {
  let mut lines = input.lines();
  let mut out = lines.next().unwrap().to_owned() + "\n";
  for line in lines {
    let mut r: Vec<String> = line.split(',').map(str::to_owned).collect();
    f(&mut r);
    out.push_str(&r.join(","));
    out.push('\n');
  }
  out
}
fn failed_without_publication(d: &TestDir, case: &str, sentinel: bool) -> Output {
  fs::write(d.path().join("patents.csv"), case).unwrap();
  let out = run(d, "--json");
  assert!(!out.status.success());
  assert!(!String::from_utf8_lossy(&out.stderr).contains("export target already exists"));
  assert_eq!(
    fs::read_to_string(d.path().join("patents.csv")).unwrap(),
    case
  );
  let path = d.path().join("pooled_poisson_bootstrap_draws.csv");
  if sentinel {
    assert_eq!(fs::read(path).unwrap(), b"sentinel");
  } else {
    assert!(!path.exists());
  }
  out
}

#[test]
fn invalid_and_failed_replicate_leave_files_unchanged() {
  let d = TestDir::new();
  prepare(&d);
  let input = fs::read_to_string(d.path().join("patents.csv")).unwrap();
  let export = d.path().join("pooled_poisson_bootstrap_draws.csv");
  fs::write(&export, b"sentinel").unwrap();
  let mut duplicate: Vec<_> = input.lines().map(str::to_owned).collect();
  duplicate[2] = duplicate[1].clone();
  let one_firm = input.lines().take(6).collect::<Vec<_>>().join("\n") + "\n";
  let collision = input
    .lines()
    .enumerate()
    .map(|(i, l)| format!("{l},{}", if i == 0 { "pooled_mu" } else { "3" }))
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
        r[7] = "NaN".into();
      }
    }),
    rewrite(&input, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[8] = "-1".into();
      }
    }),
    rewrite(&input, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "".into();
      }
    }),
    input.replacen(",1975,", ",1980,", 1),
  ];
  for case in cases {
    failed_without_publication(&d, &case, true);
  }
  fs::remove_file(&export).unwrap();
  // The original fit has a finite interior slope; replicate 54 draws only
  // zero-total firms 7..12, so its Poisson intercept does not exist.
  let replicate_54 = include_str!("fixtures/mma23_pooled_bootstrap/draws.csv")
    .lines()
    .nth(54)
    .unwrap()
    .split_once(',')
    .unwrap();
  assert_eq!(replicate_54.0, "54");
  assert!(
    replicate_54
      .1
      .split(':')
      .all(|i| i.parse::<usize>().unwrap() >= 6)
  );
  let separated = rewrite(&input, |r| {
    r[8] = if r[0].parse::<usize>().unwrap() <= 6 {
      "1"
    } else {
      "0"
    }
    .into()
  });
  let out = failed_without_publication(&d, &separated, false);
  assert!(String::from_utf8_lossy(&out.stderr).contains("sql failed"));
  let ev: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let fit = ev
    .iter()
    .find(|e| e["result_type"] == "PoissonResult")
    .unwrap();
  assert_eq!(fit["data"]["converged"], true);
  assert_eq!(
    ev.iter()
      .filter(|e| e["result_type"] == "PredictionResult")
      .count(),
    2
  );
  assert!(
    fit["data"]["coefficients"][1]["value"]
      .as_f64()
      .unwrap()
      .abs()
      < 10.
  );
  assert!(!ev.iter().any(|e| {
    e["result_type"] == "TableResult"
      && e["data"]["headers"]
        .as_array()
        .unwrap()
        .contains(&Value::from("covariance_aa"))
  }));
  // The failed SQL publication leaves the earlier fit and predictions intact;
  // it does not roll back the commands that preceded it in the script.
  let mut session = Session::new();
  let execute = |s: &mut Session, text: &str| s.execute(parse_command(text).unwrap()).unwrap();
  execute(
    &mut session,
    &format!("use \"{}\"", d.path().join("patents.csv").display()),
  );
  for command in [
    "isid id year",
    "poisson pat logr",
    "predict pooled_mu, n",
    "predict pooled_xb, xb",
  ] {
    execute(&mut session, command);
  }
  let before = session.state();
  let rows_before = execute(&mut session, "head 100");
  let script = include_str!("../examples/mma23p1pannonlin/pooled_poisson_bootstrap.td");
  let body = script
    .split_once("sql \"\"\"\nWITH RECURSIVE\n")
    .unwrap()
    .1
    .split_once("\n\"\"\" into pooled_poisson_bootstrap_draws")
    .unwrap()
    .0;
  let query = format!("sql \"\"\"WITH RECURSIVE\n{body}\"\"\" into pooled_poisson_bootstrap_draws");
  assert!(session.execute(parse_command(&query).unwrap()).is_err());
  assert_eq!(session.state(), before);
  assert_eq!(execute(&mut session, "head 100"), rows_before);
  execute(&mut session, "predict still_available, n");
}

#[test]
fn r_glm_near_boundary_and_exact_endpoint_rejection() {
  let d = TestDir::new();
  prepare(&d);
  let input = fs::read_to_string(d.path().join("patents.csv")).unwrap();
  let export = d.path().join("pooled_poisson_bootstrap_draws.csv");
  for slope in [9.9_f64, -9.9_f64, 10., -10., 10. - 1e-13, -10. + 1e-13] {
    let case = rewrite(&input, |r| {
      let high = r[1].parse::<usize>().unwrap() >= 1978;
      r[7] = if high {
        (2_f64.ln() / slope.abs()).to_string()
      } else {
        "0".into()
      };
      r[8] = if (slope > 0. && high) || (slope < 0. && !high) {
        "2"
      } else {
        "1"
      }
      .into();
    });
    if slope.abs() > 9.999999999 {
      let out = failed_without_publication(&d, &case, false);
      assert!(String::from_utf8_lossy(&out.stderr).contains("sql failed"));
      let ev: Vec<Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
      assert!(
        ev.iter().any(
          |event| event["result_type"] == "PoissonResult" && event["data"]["converged"] == true
        )
      );
    } else {
      fs::write(d.path().join("patents.csv"), &case).unwrap();
      let r = summary(&events(run(&d, "--json")));
      let expected = include_str!("fixtures/mma23_pooled_bootstrap/boundaries.csv")
        .lines()
        .skip(1)
        .find_map(|line| {
          let row: Vec<_> = line.split(',').collect();
          (row[0].parse::<f64>().unwrap() == slope).then(|| {
            (
              row[1].parse::<f64>().unwrap(),
              row[2].parse::<f64>().unwrap(),
            )
          })
        })
        .unwrap();
      close(r["estimate_intercept"].as_f64().unwrap(), expected.0);
      close(r["estimate_slope"].as_f64().unwrap(), expected.1);
      fs::remove_file(&export).unwrap();
    }
  }
}

#[test]
fn rng_matches_validated_fe_stream_and_work_exhaustion_fails() {
  let pooled = include_str!("../examples/mma23p1pannonlin/pooled_poisson_bootstrap.td");
  let fe = include_str!("../examples/mma23p1pannonlin/conditional_fe_bootstrap.td");
  let extract = |s: &'static str| {
    s.split_once("expansion AS (")
      .unwrap()
      .1
      .split_once("), draws AS MATERIALIZED (")
      .unwrap()
      .0
  };
  assert_eq!(extract(pooled), extract(fe));
  let fragment = extract(pooled).replace("step<g+1024", "step<1");
  let sql = format!(
    "sql \"\"\"WITH RECURSIVE params AS (SELECT 12::BIGINT AS g,10001::UHUGEINT AS seed,18446744073709551615::UHUGEINT AS mask), expansion AS ({fragment}) SELECT valid_streams FROM rng_health\"\"\""
  );
  let d = TestDir::new();
  prepare(&d);
  let out = tabdat_bin()
    .current_dir(d.path())
    .args(["--json", "-c", "use patents.csv", "-c", &sql])
    .output()
    .unwrap();
  assert!(!out.status.success());
  assert!(String::from_utf8_lossy(&out.stderr).contains("sql failed"));
}
