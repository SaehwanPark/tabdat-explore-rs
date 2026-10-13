use duckdb::Connection;
use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::{CellValue, ExecutionResult, Session};
static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

#[test]
fn finite_predicate_preserves_wide_exact_extrema_and_filters_real_without_narrowing_values() {
  let root = std::env::temp_dir().join(format!(
    "tabdat-finite-summary-types-{}-{}",
    std::process::id(),
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
  ));
  fs::create_dir(&root).unwrap();
  let fixture = Fixture(root);
  let path = fixture.0.join("types.parquet");
  let decimal = format!("{}.99", "9".repeat(36));
  let connection = Connection::open_in_memory().unwrap();
  connection.execute(&format!("COPY (SELECT CAST(i AS BIGINT) AS \"wide\"\"signed\", CAST(u AS UBIGINT) AS unsigned, CAST(d AS DECIMAL(38,2)) AS exact_decimal, CAST(f AS REAL) AS real_value FROM (VALUES ({}, {}, '-{decimal}', 'Infinity'), ({}, {}, '{decimal}', '1.25'), (NULL, NULL, NULL, 'NaN')) t(i,u,d,f)) TO ? (FORMAT PARQUET)", i64::MIN, u64::MAX-1, i64::MAX, u64::MAX), [path.to_string_lossy().as_ref()]).unwrap();
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", path.display())).unwrap())
    .unwrap();
  let metadata = session.active_dataset().cloned();
  let result = session
    .execute(parse_command("summarize").unwrap())
    .unwrap();
  let ExecutionResult::Summarize(result) = result else {
    panic!("expected summary");
  };
  assert_eq!(result.rows.len(), 4);
  assert_eq!(result.rows[0].variable, "wide\"signed");
  assert_eq!(
    result.rows[0].minimum,
    Some(CellValue::SignedInteger(i128::from(i64::MIN)))
  );
  assert_eq!(
    result.rows[0].maximum,
    Some(CellValue::SignedInteger(i128::from(i64::MAX)))
  );
  assert_eq!(
    result.rows[1].minimum,
    Some(CellValue::UnsignedInteger(u128::from(u64::MAX - 1)))
  );
  assert_eq!(
    result.rows[1].maximum,
    Some(CellValue::UnsignedInteger(u128::from(u64::MAX)))
  );
  let limit = 10_i128.pow(38) - 1;
  assert_eq!(
    result.rows[2].minimum,
    Some(CellValue::Decimal {
      width: 38,
      scale: 2,
      value: -limit
    })
  );
  assert_eq!(
    result.rows[2].maximum,
    Some(CellValue::Decimal {
      width: 38,
      scale: 2,
      value: limit
    })
  );
  for row in &result.rows[..3] {
    assert_eq!(row.count, 2);
    assert!(row.mean.unwrap().is_finite());
    assert!(row.std_dev.unwrap().is_finite());
  }
  assert_eq!(result.rows[3].count, 1);
  assert_eq!(result.rows[3].mean, Some(1.25));
  assert_eq!(result.rows[3].std_dev, None);
  assert_eq!(result.rows[3].minimum, Some(CellValue::Float(1.25)));
  assert_eq!(result.rows[3].maximum, Some(CellValue::Float(1.25)));
  assert_eq!(session.active_dataset(), metadata.as_ref());
}
