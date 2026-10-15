#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::{CellValue, ExecutionResult, Session};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-grouped-mean-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "x,y,g,h\n1,2,b,1\n2,5,a,1\n3,5,b,2\n4,9,b,1\n5,8,,1\n6,,a,\n",
    )
    .unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn commands(&self, commands: &[&str]) -> Vec<Value> {
    let mut args = vec!["--json"];
    for source in commands {
      args.extend(["-c", source]);
    }
    success(self.batch(&args))
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn lines(bytes: &[u8]) -> Vec<Value> {
  std::str::from_utf8(bytes)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn success(output: Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  lines(&output.stdout)
}
fn execute(session: &mut Session, source: &str) -> ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}
fn number_sql(value: &Value) -> String {
  match value {
    Value::Null => "cast(null as double)".into(),
    Value::Number(value) => value.to_string(),
    Value::String(tag) => match tag.as_str() {
      "nan" => "cast('NaN' as double)",
      "positive_infinity" => "cast('Infinity' as double)",
      "negative_infinity" => "cast('-Infinity' as double)",
      _ => panic!("unknown reference tag"),
    }
    .into(),
    _ => panic!("invalid numeric fixture"),
  }
}

#[test]
fn native_means_match_independent_statistics_reference_before_and_after_json_encoding() {
  let fixture = Fixture::new();
  let reference: Value =
    serde_json::from_str(include_str!("fixtures/grouped-mean-reference.json")).unwrap();
  let values: Vec<_> = reference["inputs"]
    .as_array()
    .unwrap()
    .iter()
    .map(|row| {
      let key = row[0]
        .as_str()
        .map(|key| format!("'{}'", key.replace('\'', "''")))
        .unwrap_or_else(|| "cast(null as varchar)".into());
      format!(
        "({key},{},{},{})",
        number_sql(&row[1]),
        number_sql(&row[2]),
        number_sql(&row[3])
      )
    })
    .collect();
  let source = format!(
    "sql select * from (values {}) as t(g,x,y,bad) into edge",
    values.join(",")
  );
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use '{}'", fixture.0.join("data.csv").display()),
  );
  execute(&mut session, &source);
  let result = execute(&mut session, "by g: summarize");
  let ExecutionResult::By(table) = &result else {
    panic!("expected ByResult");
  };
  assert_eq!(table.headers, ["g", "mean_x", "mean_y", "mean_bad"]);
  let expected = reference["rows"].as_array().unwrap();
  assert_eq!(table.rows.len(), expected.len());
  for (actual, expected) in table.rows.iter().zip(expected) {
    assert_eq!(
      actual[0],
      expected["key"]
        .as_str()
        .map(|key| CellValue::Text(key.into()))
        .unwrap_or(CellValue::Null)
    );
    for (cell, variable) in actual[1..].iter().zip(["x", "y", "bad"]) {
      let mean = &expected["samples"][variable]["mean"];
      if mean.is_null() {
        assert_eq!(cell, &CellValue::Null);
        continue;
      }
      let CellValue::Float(value) = cell else {
        panic!("expected native AVG float");
      };
      match mean.as_str() {
        Some("nan") => assert!(value.is_nan()),
        Some("positive_infinity") => assert_eq!(*value, f64::INFINITY),
        Some("negative_infinity") => assert_eq!(*value, f64::NEG_INFINITY),
        None => {
          let mean = mean.as_f64().unwrap();
          let tolerance = 1e-10_f64.max(1e-9 * mean.abs());
          assert!((value - mean).abs() <= tolerance, "{value} vs {mean}");
        }
        _ => panic!("unknown mean tag"),
      }
    }
  }
  let encoded: Value = serde_json::from_str(
    &tabdat_reporting::json_results::serialize_execution_result(&result).unwrap(),
  )
  .unwrap();
  let expected_rows: Vec<Value> = expected
    .iter()
    .map(|row| {
      let mut cells = vec![row["key"].clone()];
      cells.extend(["x", "y", "bad"].map(|name| {
        let value = &row["samples"][name]["mean"];
        if value.is_string() {
          Value::Null
        } else {
          value.clone()
        }
      }));
      json!(cells)
    })
    .collect();
  assert_eq!(encoded["result_type"], "ByResult");
  assert_eq!(
    encoded["data"],
    json!({"headers":table.headers,"rows":expected_rows})
  );
  let finite = execute(&mut session, "summarize x if g == 'C'");
  let ExecutionResult::Summarize(finite) = finite else {
    panic!("expected finite summary");
  };
  assert_eq!(finite.rows[0].count, 1);
  assert_eq!(finite.rows[0].mean, Some(3.0));
  assert_eq!(
    success(fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      &source,
      "-c",
      "by g: summarize"
    ]))[2],
    encoded
  );
  execute(
    &mut session,
    "sql select * from active where false into empty",
  );
  let empty = execute(&mut session, "by g: summarize");
  let empty: Value = serde_json::from_str(
    &tabdat_reporting::json_results::serialize_execution_result(&empty).unwrap(),
  )
  .unwrap();
  assert_eq!(
    empty["data"],
    json!({"headers":["g","mean_x","mean_y","mean_bad"],"rows":[]})
  );
}

#[test]
fn grouped_inspection_preserves_labels_named_relation_cached_fit_and_order_through_failures() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use '{}'", fixture.0.join("data.csv").display()),
  );
  execute(
    &mut session,
    "sql select * from active order by x into cohort",
  );
  execute(&mut session, "label variable g 'Group'");
  execute(&mut session, "label define groups 'a' 'Alpha' 'b' 'Beta'");
  execute(&mut session, "label values g groups");
  execute(&mut session, "regress y x");
  let state = session.state();
  let labels = session.active_label_metadata().cloned();
  let preview = execute(&mut session, "head 10");
  let inference = execute(&mut session, "lincom x");
  for source in [
    "by g: summarize x y",
    "by g h: summarize",
    "by g g: summarize y x y",
  ] {
    let result = execute(&mut session, source);
    assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
    assert_eq!(session.state(), state);
    assert_eq!(session.active_label_metadata(), labels.as_ref());
    assert_eq!(execute(&mut session, "head 10"), preview);
    assert_eq!(execute(&mut session, "lincom x"), inference);
  }
  for source in [
    "by absent: summarize x",
    "by g: summarize absent",
    "by g: summarize g",
  ] {
    assert!(session.execute(parse_command(source).unwrap()).is_err());
    assert_eq!(session.state(), state);
    assert_eq!(session.active_label_metadata(), labels.as_ref());
    assert_eq!(execute(&mut session, "head 10"), preview);
    assert_eq!(execute(&mut session, "lincom x"), inference);
  }
  execute(&mut session, "use cohort");
  assert_eq!(execute(&mut session, "head 10"), preview);
}

#[test]
fn grouped_means_nested_batch_run_fit_prediction_and_persistence_share_the_owned_stream() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "by g: summarize x y",
    "by g h: summarize x y",
    "by g g: summarize y x y",
    "regress y x",
    "predict fitted",
    "head 10",
    "export reported.csv, replace",
  ];
  let results = fixture.commands(&commands);
  assert_eq!(
    results[1]["data"],
    json!({"headers":["g","mean_x","mean_y"],"rows":[["a",4.0,5.0],["b",8.0/3.0,16.0/3.0],[null,5.0,8.0]]})
  );
  assert_eq!(
    results[3]["data"]["headers"],
    json!(["g", "g", "mean_y", "mean_x", "mean_y"])
  );
  let bytes = fs::read(fixture.0.join("reported.csv")).unwrap();
  fs::write(fixture.0.join("child.td"), commands[1..].join("\n")).unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  assert_eq!(
    success(fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
  assert_eq!(
    success(fixture.batch(&["--json", "-c", "run parent.td"])),
    results
  );
  assert_eq!(fs::read(fixture.0.join("reported.csv")).unwrap(), bytes);
  let mut ungrouped = vec![commands[0]];
  ungrouped.extend_from_slice(&commands[4..]);
  let expected = fixture.commands(&ungrouped);
  assert_eq!(results[4..], expected[1..]);
  assert_eq!(fs::read(fixture.0.join("reported.csv")).unwrap(), bytes);
}

#[test]
fn typed_quoted_group_keys_default_targets_and_no_numeric_failure_remain_explicit() {
  let fixture = Fixture::new();
  let results = fixture.commands(&[
    "use data.csv",
    "sql select cast(h as decimal(38,1)) as \"g key\", x as \"x quote\", y from active into typed",
    "by `g key`: summarize",
  ]);
  assert_eq!(
    results[2]["data"],
    json!({"headers":["g key","mean_x quote","mean_y"],"rows":[["1.0",3.0,6.0],["2.0",3.0,5.0],[null,6.0,null]]})
  );
  let wide = fixture.commands(&[
    "use data.csv",
    "sql select cast(9007199254740993 as bigint) as g, x from active into wide",
    "by g: summarize x",
  ]);
  assert_eq!(
    wide[2]["data"],
    json!({"headers":["g","mean_x"],"rows":[[9007199254740993_i64,3.5]]})
  );
  let failed = fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "sql select g from active into keys_only",
    "-c",
    "by g: summarize",
    "-c",
    "export never.csv",
  ]);
  assert_eq!(failed.status.code(), Some(1));
  let results = lines(&failed.stdout);
  assert_eq!(results.len(), 3);
  assert_eq!(results[2]["error"]["type"], "ExecutionError");
  assert!(!fixture.0.join("never.csv").exists());
  let missing = fixture.batch(&["--json", "-c", "by g: summarize x"]);
  assert_eq!(missing.status.code(), Some(1));
  assert_eq!(lines(&missing.stdout)[0]["error"]["type"], "ExecutionError");
}
