use duckdb::Connection;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs, path::PathBuf};
use tabdat_language::parse_command;
use tabdat_runtime::{ExecutionResult, RuntimeError, Session};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn parquet(query: &str) -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-xtsum-{}-{}",
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

fn close(actual: Option<f64>, expected: f64) {
  let actual = actual.unwrap();
  assert!(
    (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
    "{actual} != {expected}"
  );
}

#[test]
fn xtsum_uses_variable_samples_and_pooled_within_center_for_unbalanced_panels() {
  let fixture = Fixture::parquet(
    "SELECT * FROM (VALUES ('A',0,1),('A',2,3),('A',4,NULL),('B',10,10),('B',NULL,14),('C',NULL,20)) AS t(firm,x,y)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);
  let before = session.active_dataset().cloned();
  let ExecutionResult::XtSum(result) = session
    .execute(parse_command("xtsum x y, i(firm)").unwrap())
    .unwrap()
  else {
    panic!("xtsum should return its typed result");
  };
  assert_eq!(result.id_variable, "firm");
  assert_eq!(
    result
      .rows
      .iter()
      .map(|r| r.variable.as_str())
      .collect::<Vec<_>>(),
    ["x", "y"]
  );

  let x = &result.rows[0];
  assert_eq!((x.overall_n, x.between_n, x.within_n), (4, 2, 4));
  close(x.overall_mean, 4.0);
  close(x.overall_sd, (56.0_f64 / 3.0).sqrt());
  close(x.overall_min, 0.0);
  close(x.overall_max, 10.0);
  close(x.between_sd, 32.0_f64.sqrt());
  close(x.between_min, 2.0);
  close(x.between_max, 10.0);
  close(x.within_sd, (8.0_f64 / 3.0).sqrt());
  close(x.within_min, 2.0);
  close(x.within_max, 6.0);
  close(x.mean_t, 2.0);

  let y = &result.rows[1];
  assert_eq!((y.overall_n, y.between_n, y.within_n), (5, 3, 5));
  close(y.overall_mean, 9.6);
  close(y.overall_sd, 61.3_f64.sqrt());
  close(y.between_sd, (732.0_f64 / 9.0).sqrt());
  close(y.within_sd, 2.5_f64.sqrt());
  close(y.within_min, 7.6);
  close(y.within_max, 11.6);
  close(y.mean_t, 5.0 / 3.0);
  assert_ne!(y.overall_mean, Some((2.0 + 12.0 + 20.0) / 3.0));
  assert_eq!(session.active_dataset(), before.as_ref());
}

#[test]
fn xtsum_default_varlist_keeps_numeric_panel_identifier_and_undefined_samples() {
  let fixture = Fixture::parquet(
    "SELECT * FROM (VALUES (1, NULL::DOUBLE, 5.0), (1, NULL::DOUBLE, NULL::DOUBLE), (2, NULL::DOUBLE, NULL::DOUBLE)) AS t(id,x,y)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);
  let ExecutionResult::XtSum(result) = session
    .execute(parse_command("xtsum, i(id)").unwrap())
    .unwrap()
  else {
    panic!("xtsum should return its typed result");
  };
  assert_eq!(
    result
      .rows
      .iter()
      .map(|row| row.variable.as_str())
      .collect::<Vec<_>>(),
    ["id", "x", "y"]
  );
  let id = &result.rows[0];
  assert_eq!((id.overall_n, id.between_n, id.within_n), (3, 2, 3));
  close(id.overall_mean, 4.0 / 3.0);
  close(id.within_sd, 0.0);
  close(id.within_min, 4.0 / 3.0);
  close(id.within_max, 4.0 / 3.0);
  close(id.mean_t, 1.5);
  let all_missing = &result.rows[1];
  assert_eq!(
    (
      all_missing.overall_n,
      all_missing.between_n,
      all_missing.within_n
    ),
    (0, 0, 0)
  );
  assert_eq!(all_missing.overall_mean, None);
  assert_eq!(all_missing.between_sd, None);
  assert_eq!(all_missing.within_min, None);
  assert_eq!(all_missing.mean_t, None);
  let singleton_panel = &result.rows[2];
  assert_eq!(
    (singleton_panel.overall_n, singleton_panel.between_n),
    (1, 1)
  );
  assert_eq!(singleton_panel.overall_sd, None);
  assert_eq!(singleton_panel.between_sd, None);
  assert_eq!(singleton_panel.within_sd, None);
  close(singleton_panel.within_min, 5.0);
  close(singleton_panel.mean_t, 1.0);
}

#[test]
fn xtsum_rejects_bad_identifiers_and_columns_without_changing_active_data() {
  let fixture = Fixture::parquet(
    "SELECT * FROM (VALUES (1,1.0,'A',true), (2,2.0,'B',false)) AS t(id,x,label,flag)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);
  let before = session.active_dataset().cloned();
  let cases = [
    ("xtsum x, i(absent)", "xtsum unknown panel variable: absent"),
    ("xtsum absent, i(id)", "xtsum unknown variable: absent"),
    (
      "xtsum label, i(id)",
      "xtsum requires numeric variables: label",
    ),
    (
      "xtsum x, i(flag)",
      "xtsum panel identifier must be numeric or string: flag",
    ),
  ];
  for (command, expected) in cases {
    assert_eq!(
      session
        .execute(parse_command(command).unwrap())
        .unwrap_err()
        .to_string(),
      expected
    );
    assert_eq!(session.active_dataset(), before.as_ref());
  }
  let ExecutionResult::XtSum(string_id) = session
    .execute(parse_command("xtsum x, i(label)").unwrap())
    .unwrap()
  else {
    panic!("string identifiers should be supported");
  };
  assert_eq!(string_id.rows[0].between_n, 2);
  assert_eq!(session.active_dataset(), before.as_ref());

  let invalid = Fixture::parquet("SELECT * FROM (VALUES (1,1.0), (NULL,2.0)) AS t(id,x)");
  let mut invalid_session = Session::new();
  invalid.load(&mut invalid_session);
  let before = invalid_session.active_dataset().cloned();
  assert_eq!(
    invalid_session
      .execute(parse_command("xtsum x, i(id)").unwrap())
      .unwrap_err(),
    RuntimeError::XtSumInvalidPanelIdentifier {
      variable: "id".to_owned()
    }
  );
  assert_eq!(invalid_session.active_dataset(), before.as_ref());

  let nonfinite =
    Fixture::parquet("SELECT * FROM (VALUES ('NaN'::DOUBLE,1.0), (2.0,2.0)) AS t(id,x)");
  let mut nonfinite_session = Session::new();
  nonfinite.load(&mut nonfinite_session);
  assert_eq!(
    nonfinite_session
      .execute(parse_command("xtsum x, i(id)").unwrap())
      .unwrap_err(),
    RuntimeError::XtSumInvalidPanelIdentifier {
      variable: "id".to_owned()
    }
  );

  let missing_string = Fixture::parquet("SELECT * FROM (VALUES ('A',1.0), (NULL,2.0)) AS t(id,x)");
  let mut missing_string_session = Session::new();
  missing_string.load(&mut missing_string_session);
  assert_eq!(
    missing_string_session
      .execute(parse_command("xtsum x, i(id)").unwrap())
      .unwrap_err(),
    RuntimeError::XtSumInvalidPanelIdentifier {
      variable: "id".to_owned()
    }
  );
}
