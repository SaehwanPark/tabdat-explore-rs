#![forbid(unsafe_code)]

use tabdat_data::{CellReadError, DuckDbBackend};
use tabdat_domain::CellValue;

#[test]
fn owned_rows_execute_before_reading_metadata_and_handle_empty_queries() {
  let backend = DuckDbBackend::new().unwrap();
  let mut visited = 0;
  backend
    .visit_rows(
      "SELECT 7 AS n, 'λ' AS text",
      |row| {
        assert_eq!(row.cell(0).unwrap(), &CellValue::SignedInteger(7));
        assert_eq!(row.cell(1).unwrap(), &CellValue::Text("λ".into()));
        visited += 1;
        Ok::<(), String>(())
      },
      |message| message,
    )
    .unwrap();
  assert_eq!(visited, 1);
  backend
    .visit_rows(
      "SELECT 1 WHERE false",
      |_| {
        visited += 1;
        Ok::<(), String>(())
      },
      |message| message,
    )
    .unwrap();
  assert_eq!(visited, 1);
}

#[test]
fn unaccessed_unsupported_cells_do_not_change_sample_skip_semantics() {
  let backend = DuckDbBackend::new().unwrap();
  backend
    .visit_rows(
      "SELECT NULL AS y, [1,2] AS unsupported",
      |row| {
        assert_eq!(row.cell(0).unwrap(), &CellValue::Null);
        assert_eq!(row.cell(1).unwrap_err(), CellReadError::Unsupported);
        // A sample builder can skip this row without propagating column 1's error.
        Ok::<(), String>(())
      },
      |message| message,
    )
    .unwrap();
}

#[test]
fn visitor_errors_short_circuit_and_query_failure_recovery_is_owned() {
  let backend = DuckDbBackend::new().unwrap();
  let mut visited = 0;
  let error = backend
    .visit_rows(
      "SELECT * FROM range(10)",
      |_| {
        visited += 1;
        Err("visitor failure".to_owned())
      },
      |message| message,
    )
    .unwrap_err();
  assert_eq!(error, "visitor failure");
  assert_eq!(visited, 1);
  assert!(
    backend
      .visit_rows(
        "SELECT missing",
        |_| Ok::<(), String>(()),
        |message| message
      )
      .is_err()
  );
  assert_eq!(backend.query_i64("SELECT 7").unwrap(), 7);
}
