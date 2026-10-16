use tabdat_data::{DataError, validate_recode_number};

#[test]
fn recode_number_validation_requires_a_finite_number() {
  for value in ["0", "-0.0", "1.25", "-2", "1e3", "1e-300"] {
    assert_eq!(validate_recode_number(value), Ok(()), "{value}");
  }
  for value in ["", "not a number", "NaN", "inf", "-inf", "1e999", " 1 "] {
    assert_eq!(validate_recode_number(value), Err(DataError), "{value}");
  }
}
