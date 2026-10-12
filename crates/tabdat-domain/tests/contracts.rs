#![forbid(unsafe_code)]

use tabdat_domain::{Command, CountResult, ExecutionResult, RowLimit, RuntimeError};

#[test]
fn contracts_are_constructible_without_a_backend() {
  let command = Command::Count;
  assert_eq!(command, Command::Count);
  assert_eq!(
    ExecutionResult::Count(CountResult { row_count: 7 }),
    ExecutionResult::Count(CountResult { row_count: 7 })
  );
  assert_eq!(
    RuntimeError::NoActiveDataset { command: "count" }.to_string(),
    "count requires an active dataset; run use <path> first"
  );
}

#[test]
fn row_limits_validate_and_canonicalize_without_integer_overflow() {
  assert_eq!(RowLimit::new("0005").unwrap().as_decimal(), "5");
  assert_eq!(RowLimit::new("000").unwrap().as_decimal(), "0");
  let huge = "9".repeat(1000);
  assert_eq!(RowLimit::new(&huge).unwrap().as_decimal(), huge);
  for invalid in ["", "-1", "+2", "1.0", " 1", "１", "1e3"] {
    assert!(RowLimit::new(invalid).is_err(), "{invalid}");
  }
}
