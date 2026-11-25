use duckdb::Connection;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs, path::PathBuf};
use tabdat_language::parse_command;
use tabdat_runtime::{CellValue, ExecutionResult, RuntimeError, Session};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn parquet(query: &str) -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-xtdes-{}-{}",
      std::process::id(),
      NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let path = root.join("panel.parquet");
    let connection = Connection::open_in_memory().unwrap();
    connection
      .execute(
        &format!("COPY ({query}) TO ? (FORMAT PARQUET)"),
        [path.to_string_lossy().as_ref()],
      )
      .unwrap();
    Self(path)
  }

  fn load(&self, session: &mut Session) {
    session
      .execute(parse_command(&format!("use {}", self.0.display())).unwrap())
      .unwrap();
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    if let Some(parent) = self.0.parent() {
      let _ = fs::remove_dir_all(parent);
    }
  }
}

#[test]
fn xtdes_matches_uneven_duplicate_panel_counts_and_stable_pattern_ties() {
  let fixture = Fixture::parquet(
    "SELECT * FROM (VALUES ('B',2000),('A',2004),('C',2010),('A',2000),('A',2000),('B',2010)) AS t(firm,year)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);
  let before = session.active_dataset().cloned();
  let ExecutionResult::XtDes(result) = session
    .execute(parse_command("xtdes, i(firm) t(year)").unwrap())
    .unwrap()
  else {
    panic!("xtdes should return its typed result");
  };
  assert_eq!(result.id_variable, "firm");
  assert_eq!(result.time_variable, "year");
  assert_eq!(
    (
      result.observation_count,
      result.panel_count,
      result.period_count,
      result.duplicate_key_pairs,
      result.extra_pair_observations,
      result.missing_panel_periods,
      result.balanced,
    ),
    (6, 3, 3, 1, 1, 4, Some(false))
  );
  assert_eq!(result.panel_min, Some(CellValue::Text("A".into())));
  assert_eq!(result.panel_max, Some(CellValue::Text("C".into())));
  assert_eq!(
    result.time_values,
    vec![
      CellValue::SignedInteger(2000),
      CellValue::SignedInteger(2004),
      CellValue::SignedInteger(2010)
    ]
  );
  assert_eq!(result.time_min, Some(CellValue::SignedInteger(2000)));
  assert_eq!(result.time_max, Some(CellValue::SignedInteger(2010)));
  assert_eq!(
    result.observations_per_panel,
    tabdat_runtime::XtDesCountDistribution {
      minimum: Some(1),
      p5: Some(1.1),
      p25: Some(1.5),
      p50: Some(2.0),
      p75: Some(2.5),
      p95: Some(2.9),
      maximum: Some(3),
    }
  );
  assert_eq!(
    result.periods_per_panel,
    tabdat_runtime::XtDesCountDistribution {
      minimum: Some(1),
      p5: Some(1.1),
      p25: Some(1.5),
      p50: Some(2.0),
      p75: Some(2.0),
      p95: Some(2.0),
      maximum: Some(2),
    }
  );
  assert_eq!(result.distinct_pattern_count, 3);
  assert_eq!(result.omitted_pattern_count, 0);
  assert_eq!(
    result
      .patterns
      .iter()
      .map(|pattern| pattern.counts_by_period.clone())
      .collect::<Vec<_>>(),
    [vec![0, 0, 1], vec![1, 0, 1], vec![2, 1, 0]]
  );
  for pattern in &result.patterns {
    assert_eq!(pattern.panel_count, 1);
    assert_eq!(pattern.percent, 100.0 / 3.0);
  }
  assert_eq!(result.patterns[0].cumulative_percent, 100.0 / 3.0);
  assert_eq!(result.patterns[1].cumulative_percent, 200.0 / 3.0);
  assert_eq!(result.patterns[2].cumulative_percent, 100.0);
  assert_eq!(session.active_dataset(), before.as_ref());

  let ExecutionResult::XtDes(truncated) = session
    .execute(parse_command("xtdescribe, i(firm) t(year) patterns(2)").unwrap())
    .unwrap()
  else {
    panic!("xtdescribe alias should return its typed result");
  };
  assert_eq!(truncated.patterns.len(), 2);
  assert_eq!(truncated.requested_patterns, 2);
  assert_eq!(truncated.omitted_pattern_count, 1);
  assert_eq!(truncated.patterns[1].cumulative_percent, 200.0 / 3.0);
}

#[test]
fn xtdes_accepts_numeric_panel_keys_and_defines_empty_structure() {
  let fixture =
    Fixture::parquet("SELECT * FROM (VALUES (20,1.5),(10,3.5),(10,1.5)) AS t(id,period)");
  let mut session = Session::new();
  fixture.load(&mut session);
  let ExecutionResult::XtDes(result) = session
    .execute(parse_command("xtdes, i(id) t(period)").unwrap())
    .unwrap()
  else {
    panic!("numeric panel keys should be supported");
  };
  assert_eq!(result.panel_min, Some(CellValue::SignedInteger(10)));
  assert_eq!(result.panel_max, Some(CellValue::SignedInteger(20)));
  assert_eq!(
    result.time_values,
    vec![
      CellValue::Decimal {
        width: 2,
        scale: 1,
        value: 15
      },
      CellValue::Decimal {
        width: 2,
        scale: 1,
        value: 35
      }
    ]
  );
  assert_eq!(result.balanced, Some(false));

  let empty =
    Fixture::parquet("SELECT CAST(NULL AS BIGINT) AS id, CAST(NULL AS DOUBLE) AS time WHERE false");
  let mut empty_session = Session::new();
  empty.load(&mut empty_session);
  let ExecutionResult::XtDes(result) = empty_session
    .execute(parse_command("xtdes, i(id) t(time)").unwrap())
    .unwrap()
  else {
    panic!("empty data should return a typed result");
  };
  assert_eq!(
    (
      result.observation_count,
      result.panel_count,
      result.period_count
    ),
    (0, 0, 0)
  );
  assert_eq!(result.panel_min, None);
  assert_eq!(result.time_min, None);
  assert_eq!(result.balanced, None);
  assert_eq!(result.observations_per_panel.minimum, None);
  assert_eq!(result.periods_per_panel.p95, None);
  assert!(result.patterns.is_empty());
}

#[test]
fn xtdes_rejects_invalid_keys_and_types_without_mutating_data() {
  let fixture = Fixture::parquet(
    "SELECT * FROM (VALUES (1,1.0,'A',true), (2,2.0,'B',false)) AS t(id,time,label,flag)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);
  let before = session.active_dataset().cloned();
  let cases = [
    (
      "xtdes, i(absent) t(time)",
      RuntimeError::XtDesUnknownPanelVariable {
        variable: "absent".into(),
      },
    ),
    (
      "xtdes, i(id) t(absent)",
      RuntimeError::XtDesUnknownTimeVariable {
        variable: "absent".into(),
      },
    ),
    (
      "xtdes, i(flag) t(time)",
      RuntimeError::XtDesPanelIdentifierType {
        variable: "flag".into(),
      },
    ),
    (
      "xtdes, i(id) t(label)",
      RuntimeError::XtDesTimeVariableType {
        variable: "label".into(),
      },
    ),
  ];
  for (command, expected) in cases {
    assert_eq!(
      session
        .execute(parse_command(command).unwrap())
        .unwrap_err(),
      expected
    );
    assert_eq!(session.active_dataset(), before.as_ref());
  }
  assert_eq!(
    session
      .execute(tabdat_runtime::Command::XtDes {
        command: tabdat_runtime::XtDesCommand {
          id_variable: "id".into(),
          time_variable: "time".into(),
          patterns: 0,
        },
      })
      .unwrap_err(),
    RuntimeError::XtDesInvalidPatternLimit
  );
  assert_eq!(session.active_dataset(), before.as_ref());

  for (query, expected) in [
    (
      "SELECT * FROM (VALUES (1,NULL::DOUBLE)) AS t(id,time)",
      RuntimeError::XtDesInvalidTimeValue {
        variable: "time".into(),
      },
    ),
    (
      "SELECT * FROM (VALUES (1,'NaN'::DOUBLE)) AS t(id,time)",
      RuntimeError::XtDesInvalidTimeValue {
        variable: "time".into(),
      },
    ),
    (
      "SELECT * FROM (VALUES (NULL::BIGINT,1.0)) AS t(id,time)",
      RuntimeError::XtDesInvalidPanelIdentifier {
        variable: "id".into(),
      },
    ),
    (
      "SELECT * FROM (VALUES ('NaN'::DOUBLE,1.0)) AS t(id,time)",
      RuntimeError::XtDesInvalidPanelIdentifier {
        variable: "id".into(),
      },
    ),
  ] {
    let invalid = Fixture::parquet(query);
    let mut invalid_session = Session::new();
    invalid.load(&mut invalid_session);
    let before = invalid_session.active_dataset().cloned();
    assert_eq!(
      invalid_session
        .execute(parse_command("xtdes, i(id) t(time)").unwrap())
        .unwrap_err(),
      expected
    );
    assert_eq!(invalid_session.active_dataset(), before.as_ref());
  }

  let too_many = Fixture::parquet("SELECT 1 AS id, period AS time FROM range(513) AS t(period)");
  let mut too_many_session = Session::new();
  too_many.load(&mut too_many_session);
  let before = too_many_session.active_dataset().cloned();
  assert_eq!(
    too_many_session
      .execute(parse_command("xtdes, i(id) t(time)").unwrap())
      .unwrap_err(),
    RuntimeError::XtDesTooManyPeriods { limit: 512 }
  );
  assert_eq!(too_many_session.active_dataset(), before.as_ref());
}
