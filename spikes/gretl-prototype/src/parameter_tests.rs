use crate::{GretlError, check_parameter_count};
#[test]
fn facade_parameter_rejection_is_owned_and_threshold_exact() {
  assert!(check_parameter_count(46340).is_ok());
  for count in [0, 46341, usize::MAX] {
    assert!(
      matches!(check_parameter_count(count),Err(GretlError::InvalidProblem(message)) if message.contains("native arithmetic"))
    );
  }
}
