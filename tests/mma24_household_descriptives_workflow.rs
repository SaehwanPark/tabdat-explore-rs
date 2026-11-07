mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
};
use tabdat_language::{parse_command, script::parse_script};
use tabdat_runtime::Session;

const SCRIPT: &str = include_str!("../examples/mma24p1olscluster/household_descriptives.td");
const PREPARED: &str = "household_descriptive_source.csv";
const REPORT: &str = "household_descriptives.csv";
const VARIABLES: &str = "lnexp12m age sex hhsize farm educ hhexp lnhhexp commune";
fn fixture(case: &str, file: &str) -> String {
  fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
    "tests/fixtures/mma24_household_descriptives/{case}/{file}.csv"
  )))
  .unwrap()
}
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma24p1olscluster/household_descriptives.td"),
    )
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let names: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let values: Vec<_> = line.split(',').collect();
      assert_eq!(values.len(), names.len());
      names.iter().copied().zip(values).collect()
    })
    .collect()
}
fn close(a: f64, b: f64) {
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9,
    "{a} != {b}"
  );
}
fn check(case: &str) {
  let dir = TestDir::new();
  let input = fixture(case, "input");
  fs::write(dir.path().join("vietnam.csv"), &input).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  assert_eq!(
    events
      .iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    vec![
      "LoadResult",
      "IsidResult",
      "TableResult",
      "SqlCreateResult",
      "TableResult",
      "SummarizeResult",
      "ExportResult",
      "SqlCreateResult",
      "ExportResult",
      "ActivateResult"
    ]
  );
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let reference = fixture(case, "summary");
  let expected = rows(&reference);
  let report = fs::read_to_string(dir.path().join(REPORT)).unwrap();
  let exported = rows(&report);
  let summary = events[5]["data"]["rows"].as_array().unwrap();
  assert_eq!(expected.len(), 9);
  assert_eq!(exported.len(), 9);
  assert_eq!(summary.len(), 9);
  let mut gates = 0;
  for (((a, b), c), name) in summary
    .iter()
    .zip(&expected)
    .zip(&exported)
    .zip(VARIABLES.split_whitespace())
  {
    assert_eq!(a["variable"], name);
    assert_eq!(b["variable"], name);
    assert_eq!(c["variable"], name);
    assert_eq!(
      a["count"].as_u64().unwrap(),
      b["count"].parse::<u64>().unwrap()
    );
    assert_eq!(b["count"], c["count"]);
    gates += 1;
    for field in ["mean", "std_dev", "minimum", "maximum"] {
      if b[field].is_empty() {
        assert!(a[field].is_null());
        assert!(c[field].is_empty());
      } else {
        close(a[field].as_f64().unwrap(), b[field].parse().unwrap());
        close(c[field].parse().unwrap(), b[field].parse().unwrap());
      }
      gates += 1;
    }
  }
  assert_eq!(gates, 45);
  let prepared = fs::read_to_string(dir.path().join(PREPARED)).unwrap();
  let reference = fixture(case, "prepared_source");
  let actual = rows(&prepared);
  let expected = rows(&reference);
  assert_eq!(actual.len(), expected.len());
  for (a, b) in actual.iter().zip(&expected) {
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for (key, value) in b {
      if value.is_empty() {
        assert!(a[key].is_empty());
      } else if *key == "hhexp" {
        close(a[key].parse().unwrap(), value.parse().unwrap());
      } else {
        assert_eq!(
          a[key].parse::<f64>().unwrap(),
          value.parse::<f64>().unwrap()
        );
      }
    }
  }
  assert!(actual.windows(2).all(
    |w| w[0]["source_row"].parse::<u64>().unwrap() < w[1]["source_row"].parse::<u64>().unwrap()
  ));
  assert_eq!(events[3]["data"], events[9]["data"]);
  assert_eq!(
    events[9]["data"]["table_name"],
    "household_descriptive_source"
  );
  assert_eq!(events[9]["data"]["dataset"]["row_count"], actual.len());
  for column in events[9]["data"]["dataset"]["columns"].as_array().unwrap() {
    assert_eq!(
      column["data_type"],
      if column["name"] == "source_row" || column["name"] == "commune" {
        "BIGINT"
      } else {
        "DOUBLE"
      }
    );
  }
  let text = TestDir::new();
  fs::write(text.path().join("vietnam.csv"), &input).unwrap();
  let output = run(&text, "--text");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let rendered = String::from_utf8(output.stdout).unwrap();
  for name in VARIABLES.split_whitespace() {
    assert!(rendered.contains(name), "missing {name}");
  }
  for name in [PREPARED, REPORT] {
    assert_eq!(
      fs::read(dir.path().join(name)).unwrap(),
      fs::read(text.path().join(name)).unwrap()
    );
  }
  assert_eq!(
    fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
    input
  );
  if case == "missing" {
    assert_eq!(summary[0]["count"], 4);
    assert_eq!(summary[1]["count"], 7);
    assert_eq!(summary[4]["count"], 1);
    assert!(summary[4]["std_dev"].is_null());
    assert_eq!(summary[5]["count"], 0);
    assert!(summary[5]["mean"].is_null());
    assert_eq!(
      actual
        .iter()
        .filter(|r| r["lnexp12m"].parse::<f64>().ok() == Some(0.0))
        .count(),
      1
    );
  } else {
    assert!(summary.iter().all(|r| r["std_dev"].is_null()));
    assert_eq!(summary[0]["count"], 0);
  }
}
#[test]
fn variable_wise_missingness_zero_all_missing_and_singleton_match_frozen_r() {
  check("missing");
}
#[test]
fn one_source_row_is_valid_without_panel_fit_sample_constraints() {
  check("singleton");
}
#[test]
fn raw_permutation_preserves_summaries_and_ordered_prepared_export() {
  let input = fixture("missing", "input");
  let lines: Vec<_> = input.lines().collect();
  let permuted = std::iter::once(lines[0])
    .chain(lines[1..].iter().rev().copied())
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  let mut exports = None;
  for source in [&input, &permuted] {
    let dir = TestDir::new();
    fs::write(dir.path().join("vietnam.csv"), source).unwrap();
    let output = run(&dir, "--json");
    assert!(
      output.status.success(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
    );
    let current = (
      fs::read(dir.path().join(PREPARED)).unwrap(),
      fs::read(dir.path().join(REPORT)).unwrap(),
    );
    if let Some(expected) = exports {
      assert_eq!(current, expected);
    }
    exports = Some(current);
  }
}
#[test]
fn read_only_summaries_and_export_failures_preserve_state_and_file_owners() {
  for collision in [None, Some(PREPARED), Some(REPORT)] {
    let dir = TestDir::new();
    let input = fixture("missing", "input");
    let source = dir.path().join("vietnam.csv");
    fs::write(&source, &input).unwrap();
    if let Some(name) = collision {
      fs::write(dir.path().join(name), "owner").unwrap();
    }
    let mut session = Session::new();
    let mut failures = 0;
    for command in parse_script(SCRIPT, Path::new("household_descriptives.td")).unwrap() {
      let text = if command.text == "use vietnam.csv" {
        format!("use \"{}\"", source.display())
      } else if let Some(name) = command.text.strip_prefix("export ") {
        format!("export \"{}\"", dir.path().join(name).display())
      } else {
        command.text.clone()
      };
      let before = session.state();
      let result = session.execute(parse_command(&text).unwrap());
      if collision.is_some_and(|name| command.text == format!("export {name}")) {
        assert!(result.is_err());
        assert_eq!(session.state(), before);
        failures += 1;
        break;
      }
      result.unwrap();
      if command.text.starts_with("summarize ") {
        assert_eq!(session.state(), before);
      }
    }
    assert_eq!(failures, usize::from(collision.is_some()));
    assert!(session.state().estimation.is_none());
    if let Some(name) = collision {
      assert_eq!(fs::read_to_string(dir.path().join(name)).unwrap(), "owner");
      if name == PREPARED {
        assert!(!dir.path().join(REPORT).exists());
      } else {
        assert!(dir.path().join(PREPARED).exists());
      }
    } else {
      session
        .execute(parse_command("regress lnexp12m age").unwrap())
        .unwrap();
      session
        .execute(parse_command("label variable age \"Age years\"").unwrap())
        .unwrap();
      let before = session.state();
      assert!(before.estimation.is_some());
      session
        .execute(parse_command(&format!("summarize {VARIABLES}")).unwrap())
        .unwrap();
      assert_eq!(session.state(), before);
      session.execute(parse_command("sql \"\"\"SELECT CASE WHEN COUNT(*)=8 AND COUNT(lnexp12m)=4 AND COUNT(educ)=0 AND MIN(source_row)=1 AND MAX(source_row)=8 THEN 1 ELSE error('wrong reactivated source') END AS valid FROM active\"\"\"").unwrap()).unwrap();
      assert_eq!(session.state(), before);
    }
    assert_eq!(fs::read_to_string(source).unwrap(), input);
  }
}
#[test]
fn invalid_inputs_fail_before_summaries_exports_and_owner_overwrite() {
  let input = fixture("missing", "input");
  let base: Vec<Vec<String>> = input
    .lines()
    .map(|s| s.split(',').map(str::to_owned).collect())
    .collect();
  let mut cases = vec![
    format!("{input}{}\n", input.lines().nth(1).unwrap()),
    format!("{}\n", input.lines().next().unwrap()),
  ];
  for (row, col, value) in [
    (1, 0, "0"),
    (1, 0, "-1"),
    (1, 0, "1.5"),
    (1, 0, "9007199254740992"),
    (1, 8, ""),
    (1, 8, "NaN"),
    (1, 8, "9007199254740992"),
    (2, 2, "oops"),
    (2, 2, "NaN"),
    (1, 3, "Inf"),
    (1, 1, "Inf"),
    (1, 7, "1000"),
    (1, 7, "-1000"),
  ] {
    let mut data = base.clone();
    data[row][col] = value.into();
    cases.push(
      data
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  cases.push(
    format!("{}\n", base[0].join(","))
      + (1..=40001)
        .map(|id| format!("{id},0,20,1,1,0,4,0,1\n"))
        .collect::<String>()
        .as_str(),
  );
  assert_eq!(cases.len(), 16);
  for (index, input) in cases.iter().enumerate() {
    let dir = TestDir::new();
    let source = dir.path().join("vietnam.csv");
    fs::write(&source, input).unwrap();
    for name in [PREPARED, REPORT] {
      fs::write(dir.path().join(name), "owner").unwrap();
    }
    let output = run(&dir, "--json");
    assert!(!output.status.success(), "invalid case {index} succeeded");
    let events: Vec<Value> = String::from_utf8(output.stdout)
      .unwrap()
      .lines()
      .map(|s| serde_json::from_str(s).unwrap())
      .collect();
    assert!(
      !events
        .iter()
        .any(|e| e["result_type"] == "SummarizeResult" || e["result_type"] == "ExportResult"),
      "case {index} reached summary/export: {events:?}"
    );
    for name in [PREPARED, REPORT] {
      assert_eq!(fs::read_to_string(dir.path().join(name)).unwrap(), "owner");
    }
    assert_eq!(fs::read_to_string(source).unwrap(), *input);
  }
}
