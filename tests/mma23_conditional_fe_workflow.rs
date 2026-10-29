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
    .arg(root().join("examples/mma23p1pannonlin/conditional_fe.td"))
    .output()
    .unwrap()
}
fn report(output: Output) -> (Vec<Value>, Value) {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  let table = events
    .iter()
    .find(|e| {
      e["result_type"] == "TableResult"
        && e["data"]["headers"]
          .as_array()
          .unwrap()
          .contains(&Value::from("conditional_ll"))
    })
    .unwrap();
  let names = table["data"]["headers"].as_array().unwrap();
  let values = table["data"]["rows"][0].as_array().unwrap();
  let result = names
    .iter()
    .zip(values)
    .map(|(n, v)| (n.as_str().unwrap().into(), v.clone()))
    .collect();
  (events, Value::Object(result))
}
fn close(a: f64, b: f64, label: &str) {
  assert!(
    a.is_finite() && (a - b).abs() <= 1e-9 + 1e-10 * b.abs(),
    "{label}: {a} != {b}"
  );
}

#[test]
fn synthetic_r_reference_exclusions_predictions_and_file_safety() {
  let d = TestDir::new();
  prepare(&d);
  let input = fs::read(d.path().join("patents.csv")).unwrap();
  let (events, r) = report(run(&d, "--json"));
  assert_eq!(
    r["estimator"],
    "conditional_poisson_fixed_effects_one_slope"
  );
  assert_eq!(
    r["covariance"],
    "model_based_conditional_information_nonrobust"
  );
  assert_eq!(r["p_reference"], "asymptotic_standard_normal");
  assert!(r["score"].as_f64().unwrap().abs() < 1e-8);
  assert_eq!(
    events
      .iter()
      .filter(|e| e["result_type"] == "SqlCreateResult")
      .count(),
    1
  );
  let mut fields = 0;
  for line in include_str!("fixtures/mma23_conditional_fe/reference.csv")
    .lines()
    .skip(1)
  {
    let (field, expected) = line.split_once(',').unwrap();
    close(r[field].as_f64().unwrap(), expected.parse().unwrap(), field);
    fields += 1;
  }
  assert_eq!(fields, 14);
  let path = d.path().join("conditional_fe_predictions.csv");
  let bytes = fs::read(&path).unwrap();
  let actual = String::from_utf8(bytes.clone()).unwrap();
  let expected = include_str!("fixtures/mma23_conditional_fe/predictions.csv");
  let mut actual = actual.lines();
  let heads: Vec<_> = actual.next().unwrap().split(',').collect();
  let mut expected = expected.lines();
  let expected_heads: Vec<_> = expected.next().unwrap().split(',').collect();
  let mut n = 0;
  let mut excluded = 0;
  for (a, e) in actual.zip(expected) {
    let a: Vec<_> = a.split(',').collect();
    let e: Vec<_> = e.split(',').collect();
    for (j, key) in expected_heads.iter().enumerate() {
      let i = heads.iter().position(|name| name == key).unwrap();
      if *key == "included_in_fe" {
        assert_eq!(a[i], if e[j] == "TRUE" { "true" } else { "false" });
        if e[j] == "FALSE" {
          excluded += 1;
        }
      } else if e[j].is_empty() {
        assert!(a[i].is_empty(), "{key}");
      } else {
        close(a[i].parse().unwrap(), e[j].parse().unwrap(), key);
      }
    }
    n += 1;
  }
  assert_eq!(n, 60);
  assert_eq!(excluded, 10);
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
  assert_eq!(fs::read(&path).unwrap(), bytes);
  fs::remove_file(&path).unwrap();
  let csv = String::from_utf8(input).unwrap();
  let mut lines = csv.lines();
  let header = lines.next().unwrap();
  let mut rows: Vec<_> = lines.collect();
  rows.reverse();
  fs::write(
    d.path().join("patents.csv"),
    format!("{header}\n{}\n", rows.join("\n")),
  )
  .unwrap();
  let _ = report(run(&d, "--json"));
  assert_eq!(fs::read(&path).unwrap(), bytes);
}

#[test]
fn normal_tail_sql_matches_r_grid_across_declared_domain() {
  let d = TestDir::new();
  prepare(&d);
  let script = include_str!("../examples/mma23p1pannonlin/conditional_fe.td");
  let start = "tails(iter,term,partial) AS (";
  let body = script
    .split_once(start)
    .unwrap()
    .1
    .split_once("),\nreport AS (")
    .unwrap()
    .0;
  let mut command = tabdat_bin();
  command
    .current_dir(d.path())
    .args(["--json", "-c", "use patents.csv"]);
  let reference = include_str!("fixtures/mma23_conditional_fe/normal_tail.csv");
  let mut expected = Vec::new();
  for line in reference.lines().skip(1) {
    let (z, p) = line.split_once(',').unwrap();
    expected.push(p.parse::<f64>().unwrap());
    let sql = format!(
      "sql \"\"\"WITH RECURSIVE stats AS (SELECT CAST({z} AS DOUBLE) AS z), {start}{body}) SELECT p_value FROM tail\"\"\""
    );
    command.arg("-c").arg(sql);
  }
  let out = command.output().unwrap();
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
  assert_eq!(expected.len(), 61);
  assert_eq!(events.len(), 62);
  for (event, p) in events.iter().skip(1).zip(expected) {
    assert_eq!(event["result_type"], "TableResult");
    let a = event["data"]["rows"][0][0].as_f64().unwrap();
    assert!((a - p).abs() < 1e-13, "tail: {a} != {p}");
  }
}

fn changed_csv<F: FnMut(&mut Vec<String>)>(csv: &str, mut f: F) -> String {
  let mut lines = csv.lines();
  let mut output = lines.next().unwrap().to_owned() + "\n";
  for line in lines {
    let mut row: Vec<_> = line.split(',').map(str::to_owned).collect();
    f(&mut row);
    output.push_str(&row.join(","));
    output.push('\n');
  }
  output
}

#[test]
fn failures_do_not_publish_and_extreme_tail_is_explicitly_withheld() {
  let d = TestDir::new();
  prepare(&d);
  let path = d.path().join("patents.csv");
  let original = fs::read_to_string(&path).unwrap();
  let mut duplicate: Vec<_> = original.lines().map(str::to_owned).collect();
  duplicate[2] = duplicate[1].clone();
  let cases = [
    duplicate.join("\n") + "\n",
    changed_csv(&original, |r| r[8] = "0".into()),
    changed_csv(&original, |r| r[7] = r[0].clone()),
    changed_csv(&original, |r| {
      r[7] = (r[1].parse::<i32>().unwrap() - 1975).to_string();
      r[8] = if r[1] == "1979" {
        "10".into()
      } else {
        "0".into()
      };
    }),
    changed_csv(&original, |r| {
      if r[1] == "1979" {
        r[7] = "100".into();
      }
    }),
    changed_csv(&original, |r| {
      if r[1] == "1975" {
        r[8] = "1001".into();
      }
    }),
    changed_csv(&original, |r| {
      if r[1] == "1975" {
        r[8] = "".into();
      }
    }),
  ];
  for csv in cases {
    fs::write(&path, &csv).unwrap();
    let out = run(&d, "--json");
    assert!(!out.status.success(), "unexpected success");
    assert!(!d.path().join("conditional_fe_predictions.csv").exists());
    assert_eq!(fs::read_to_string(&path).unwrap(), csv);
  }
  let strong = changed_csv(&original, |r| {
    let t = (r[1].parse::<usize>().unwrap() - 1975) as usize;
    r[7] = t.to_string();
    r[8] = [1, 1, 2, 4, 8][t].to_string();
  });
  fs::write(&path, strong).unwrap();
  let (_, r) = report(run(&d, "--json"));
  assert!(r["z"].as_f64().unwrap() > 3.0);
  assert_eq!(r["p_reference"], "unavailable_abs_z_exceeds_3");
  assert!(r["p_value"].is_null());
}
