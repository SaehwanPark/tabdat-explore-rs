mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
};

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
    .current_dir(dir.path())
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
fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(root().join("examples/mma23p1pannonlin/conditional_fe_bootstrap.td"))
    .output()
    .unwrap()
}
fn close(actual: f64, expected: f64) {
  assert!(
    actual.is_finite() && (actual - expected).abs() <= 1e-10,
    "{actual} != {expected}"
  );
}
fn report(out: Output) -> Value {
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let events: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let table = events
    .iter()
    .find(|e| {
      e["result_type"] == "TableResult"
        && e["data"]["headers"]
          .as_array()
          .unwrap()
          .contains(&Value::from("bootstrap_se"))
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

#[test]
fn r_refits_all_draws_reporting_and_no_clobber() {
  let d = TestDir::new();
  prepare(&d);
  let input = fs::read(d.path().join("patents.csv")).unwrap();
  let r = report(run(&d, "--json"));
  for line in include_str!("fixtures/mma23_conditional_fe_bootstrap/reference.csv")
    .lines()
    .skip(1)
  {
    let (field, val) = line.split_once(',').unwrap();
    close(r[field].as_f64().unwrap(), val.parse().unwrap());
  }
  assert_eq!(r["base_seed"], 10001);
  assert_eq!(
    r["rng"],
    "xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1"
  );
  assert_eq!(r["method"], "whole_firm_pairs");
  assert_eq!(r["interval_method"], "type7_percentile_95");
  let path = d.path().join("conditional_fe_bootstrap_draws.csv");
  let bytes = fs::read(&path).unwrap();
  let csv = String::from_utf8(bytes.clone()).unwrap();
  let mut rows = csv.lines();
  assert_eq!(
    rows.next().unwrap(),
    "replicate,position,step,firm_id,replicate_beta,estimate,replications,firms_per_draw,candidate_rows,candidate_firms,retained_rows"
  );
  let groups: Vec<usize> = include_str!("fixtures/mma23_conditional_fe_bootstrap/groups.csv")
    .lines()
    .skip(1)
    .map(|line| line.split_once(',').unwrap().1.parse().unwrap())
    .collect();
  let slopes: Vec<f64> = include_str!("fixtures/mma23_conditional_fe_bootstrap/replicates.csv")
    .lines()
    .skip(1)
    .map(|line| line.split_once(',').unwrap().1.parse().unwrap())
    .collect();
  let mut count = 0;
  for (n, line) in include_str!("fixtures/mma23_conditional_fe_bootstrap/draws.csv")
    .lines()
    .skip(1)
    .enumerate()
  {
    let (rep, indices) = line.split_once(',').unwrap();
    assert_eq!(rep.parse::<usize>().unwrap(), n + 1);
    for (pos, index) in indices.split(':').enumerate() {
      let row: Vec<_> = rows.next().unwrap().split(',').collect();
      assert_eq!(row[0].parse::<usize>().unwrap(), n + 1);
      assert_eq!(row[1].parse::<usize>().unwrap(), pos + 1);
      assert_eq!(row[2].parse::<usize>().unwrap(), pos + 1);
      assert_eq!(
        row[3].parse::<usize>().unwrap(),
        groups[index.parse::<usize>().unwrap()]
      );
      close(row[4].parse().unwrap(), slopes[n]);
      count += 1;
    }
  }
  assert_eq!(count, 5000);
  assert!(rows.next().is_none());
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
  assert!(text.contains("bootstrap_se") && text.contains("whole_firm_pairs"));
  assert_eq!(fs::read(&path).unwrap(), bytes);
}

#[test]
fn rng_high_bits_and_forced_rejection_match_external_oracle() {
  let d = TestDir::new();
  prepare(&d);
  let script = include_str!("../examples/mma23p1pannonlin/conditional_fe_bootstrap.td");
  let rng = script
    .split_once("expansion AS (")
    .unwrap()
    .1
    .split_once("), rng_health AS (")
    .unwrap()
    .0
    .replace("range(1,501)", "range(1,4)")
    .replace("accepted<g AND step<g+1024", "accepted<8 AND step<1032");
  let sql = format!(
    "sql \"\"\"WITH RECURSIVE params AS (SELECT 4611686018427387905::BIGINT AS g, 18446744073709551614::UHUGEINT AS seed, 18446744073709551615::UHUGEINT AS mask), expansion AS ({rng}) SELECT rep,accepted,step,(v%g::UHUGEINT)::BIGINT AS index0 FROM rng CROSS JOIN params WHERE v IS NOT NULL AND v<mask-(mask%g::UHUGEINT) ORDER BY rep,accepted\"\"\""
  );
  let out = tabdat_bin()
    .current_dir(d.path())
    .args(["--json", "-c", "use patents.csv", "-c", &sql])
    .output()
    .unwrap();
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let events: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let rows = events.last().unwrap()["data"]["rows"].as_array().unwrap();
  let expected: Vec<_> =
    include_str!("fixtures/mma23_conditional_fe_bootstrap/rng_high_rejection.csv")
      .lines()
      .skip(1)
      .collect();
  assert_eq!(rows.len(), 24);
  let mut rejection_seen = false;
  for (row, line) in rows.iter().zip(expected) {
    let values: Vec<i64> = line.split(',').map(|s| s.parse().unwrap()).collect();
    for (value, expected) in row.as_array().unwrap().iter().zip(&values) {
      assert_eq!(value.as_i64().unwrap(), *expected);
    }
    rejection_seen |= values[2] > values[1];
  }
  assert!(rejection_seen);
}

#[test]
fn invalid_data_rng_budget_and_failed_replicate_never_export() {
  let d = TestDir::new();
  prepare(&d);
  let input = d.path().join("patents.csv");
  let original = fs::read_to_string(&input).unwrap();
  let output = d.path().join("conditional_fe_bootstrap_draws.csv");
  // Equal positive- and negative-separation firms have a finite original root,
  // but the fixed stream's replicate 54 draws only the latter six firms.
  let mut separated = String::new();
  for (i, line) in original.lines().enumerate() {
    if i == 0 {
      separated.push_str(line);
      separated.push('\n');
      continue;
    }
    let mut row: Vec<String> = line.split(',').map(str::to_owned).collect();
    let id: i32 = row[0].parse().unwrap();
    let year: i32 = row[1].parse().unwrap();
    row[7] = (year - 1977).to_string();
    row[8] = if (id <= 6 && year == 1979) || (id > 6 && year == 1975) {
      "1"
    } else {
      "0"
    }
    .into();
    separated.push_str(&row.join(","));
    separated.push('\n');
  }
  fs::write(&input, &separated).unwrap();
  fs::write(&output, b"sentinel").unwrap();
  let failed = run(&d, "--json");
  assert!(!failed.status.success());
  assert!(String::from_utf8_lossy(&failed.stderr).contains("sql failed"));
  assert_eq!(fs::read(&output).unwrap(), b"sentinel");
  assert_eq!(fs::read_to_string(&input).unwrap(), separated);
  fs::remove_file(&output).unwrap();
  let zero = separated
    .lines()
    .enumerate()
    .map(|(i, line)| {
      if i == 0 {
        return line.to_owned();
      }
      let mut row: Vec<_> = line.split(',').collect();
      row[8] = "0";
      row.join(",")
    })
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  fs::write(&input, &zero).unwrap();
  assert!(!run(&d, "--json").status.success());
  assert!(!output.exists());
  let bad = original.replacen(",1975,", ",1980,", 1);
  fs::write(&input, &bad).unwrap();
  assert!(!run(&d, "--json").status.success());
  assert!(!output.exists());
  assert_eq!(fs::read_to_string(&input).unwrap(), bad);

  let script = include_str!("../examples/mma23p1pannonlin/conditional_fe_bootstrap.td");
  let fragment = script
    .split_once("expansion AS (")
    .unwrap()
    .1
    .split_once("), draws AS MATERIALIZED (")
    .unwrap()
    .0
    .replace("step<g+1024", "step<1");
  let sql = format!(
    "sql \"\"\"WITH RECURSIVE params AS (SELECT 10::BIGINT AS g,10001::UHUGEINT AS seed,18446744073709551615::UHUGEINT AS mask), expansion AS ({fragment}) SELECT valid_streams FROM rng_health\"\"\""
  );
  let out = tabdat_bin()
    .current_dir(d.path())
    .args(["--json", "-c", "use patents.csv", "-c", &sql])
    .output()
    .unwrap();
  assert!(!out.status.success());
  assert!(String::from_utf8_lossy(&out.stderr).contains("sql failed"));
}
