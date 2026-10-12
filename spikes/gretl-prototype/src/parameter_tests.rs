use crate::{GretlError, check_parameter_count, new_dataset};
#[test]
fn facade_parameter_rejection_is_owned_and_threshold_exact() {
  assert!(check_parameter_count(46340).is_ok());
  for count in [0, 46341, usize::MAX] {
    assert!(
      matches!(check_parameter_count(count),Err(GretlError::InvalidProblem(message)) if message.contains("native arithmetic"))
    );
  }
}

#[test]
fn facade_dataset_budget_returns_owned_errors_before_native_allocation() {
  for (variables, observations) in [(1025, 1), (2, usize::MAX), (1000, 10_001)] {
    assert!(
      matches!(new_dataset(variables, observations), Err(GretlError::InvalidProblem(message))
      if message.contains("budget"))
    );
  }
  let dataset = new_dataset(2, 8).unwrap();
  assert_eq!(dataset.variable_count(), 2);
}
