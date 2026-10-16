use tabdat_stats::StatsError;
use tabdat_stats::matrix::{
  multiply, multiply_vector, qr_decompose_with_response, solve_upper_triangular, xtx_inverse_from_r,
};

fn assert_dimension_error<T: std::fmt::Debug>(result: Result<T, StatsError>) {
  assert!(
    matches!(result, Err(StatsError::DimensionMismatch(_))),
    "{result:?}"
  );
}

#[test]
fn matrix_products_reject_short_and_long_rows_even_when_zeroes_hide_them() {
  let rectangular = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
  for ragged in [
    vec![vec![1.0, 2.0], vec![3.0]],
    vec![vec![1.0, 2.0], vec![3.0, 4.0, 5.0]],
  ] {
    assert_dimension_error(multiply(&ragged, &rectangular));
    assert_dimension_error(multiply(&rectangular, &ragged));
    assert_dimension_error(multiply_vector(&ragged, &[1.0, 2.0]));
    assert_dimension_error(multiply(&vec![vec![0.0, 0.0]; 2], &ragged));
  }
}

#[test]
fn factorization_and_triangular_utilities_reject_nonrectangular_or_nonsquare_rows() {
  for malformed in [
    vec![vec![1.0, 2.0], vec![3.0]],
    vec![vec![1.0, 2.0], vec![3.0, 4.0, 5.0]],
    vec![vec![1.0], vec![2.0]],
    vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]],
  ] {
    assert_dimension_error(solve_upper_triangular(&malformed, &[1.0, 2.0]));
    assert_dimension_error(xtx_inverse_from_r(&malformed));
  }
  for ragged in [
    vec![vec![1.0, 2.0], vec![3.0]],
    vec![vec![1.0, 2.0], vec![3.0, 4.0, 5.0]],
  ] {
    assert_dimension_error(qr_decompose_with_response(&ragged, &[1.0, 2.0]));
  }
}

#[test]
fn rectangular_products_and_existing_empty_shapes_remain_unchanged() {
  let left = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
  assert_eq!(
    multiply(&left, &left).unwrap(),
    vec![vec![7.0, 10.0], vec![15.0, 22.0]]
  );
  assert_eq!(
    multiply_vector(&left, &[2.0, 3.0]).unwrap(),
    vec![8.0, 18.0]
  );
  assert_eq!(
    multiply_vector(&[vec![], vec![]], &[]).unwrap(),
    vec![0.0, 0.0]
  );
  assert_eq!(
    multiply(&[vec![1.0]], &[vec![]]).unwrap(),
    vec![Vec::<f64>::new()]
  );
  assert!(solve_upper_triangular(&[], &[]).unwrap().is_empty());
  assert!(xtx_inverse_from_r(&[]).unwrap().is_empty());
}
