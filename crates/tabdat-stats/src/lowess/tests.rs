use super::*;

fn rows(text: &str) -> Vec<Vec<f64>> {
  text
    .lines()
    .skip(1)
    .map(|line| line.split(',').map(|v| v.parse().unwrap()).collect())
    .collect()
}
fn close(actual: f64, expected: f64) {
  assert!(
    actual.is_finite() && (actual - expected).abs() <= 1e-11 + 1e-10 * expected.abs(),
    "actual {actual:.17e}, expected {expected:.17e}"
  );
}
fn check_fixture(input: &[Vec<f64>], expected: &str, metadata: &str) {
  let expected = rows(expected);
  let metadata = rows(metadata);
  let x: Vec<_> = input.iter().map(|r| r[1]).collect();
  let y: Vec<_> = input.iter().map(|r| r[2]).collect();
  assert_eq!(input.len(), expected.len());
  for (i, span) in [0.8, 0.4, 0.1].into_iter().enumerate() {
    let result = smooth_lowess(&x, &y, span).unwrap();
    assert_eq!(result.half_window, metadata[i][2] as usize);
    assert_eq!(result.local_constant_windows, metadata[i][3] as usize);
    assert_eq!(result.values.len(), metadata[i][1] as usize);
    assert!(metadata[i][4] < 1e-12);
    for ((original, reference), actual) in input.iter().zip(&expected).zip(result.values) {
      assert_eq!(original[0], reference[0]);
      close(original[1], reference[1]);
      close(original[2], reference[2]);
      close(actual, reference[3 + i]);
    }
  }
}
#[test]
fn all_unordered_tied_fits_match_independent_r_weighted_qr() {
  check_fixture(
    &rows(include_str!("../../../../tests/fixtures/lowess/input.csv")),
    include_str!("../../../../tests/fixtures/lowess/expected.csv"),
    include_str!("../../../../tests/fixtures/lowess/metadata.csv"),
  );
  let differences = rows(include_str!(
    "../../../../tests/fixtures/lowess/nearest_window_difference.csv"
  ));
  assert!(
    differences[0][1] > 0.01,
    "R nearest-neighbour LOWESS must remain a different reference"
  );
}
#[test]
fn signed_zero_ties_and_coincident_window_means_match_r_qr() {
  check_fixture(
    &rows(include_str!(
      "../../../../tests/fixtures/lowess/all_tied.csv"
    )),
    include_str!("../../../../tests/fixtures/lowess/all_tied_expected.csv"),
    include_str!("../../../../tests/fixtures/lowess/all_tied_metadata.csv"),
  );
}
#[test]
fn source_preparation_analogue_matches_all_r_qr_fits() {
  let sample = rows(include_str!(
    "../../../../tests/fixtures/lowess/source_control_sample.csv"
  ));
  let input: Vec<_> = sample
    .iter()
    .map(|r| vec![r[0], r[4], (r[5].min(100000.0) / r[6]).ln()])
    .collect();
  check_fixture(
    &input,
    include_str!("../../../../tests/fixtures/lowess/source_control_expected.csv"),
    include_str!("../../../../tests/fixtures/lowess/source_control_metadata.csv"),
  );
}
#[test]
fn reproduces_lines_including_asymmetric_boundary_windows() {
  let x = vec![4., -4., 7., 0.1, 23., 2., 31., -1.5, 15., 0., 11.];
  let y: Vec<_> = x.iter().map(|x| 1.2 - 0.7 * x).collect();
  let fit = smooth_lowess(&x, &y, 0.8).unwrap();
  assert_eq!(fit.half_window, 4);
  assert_eq!(fit.local_constant_windows, 0);
  for (actual, expected) in fit.values.iter().zip(&y) {
    close(*actual, *expected);
  }
}
#[test]
fn positive_predictor_and_outcome_affine_transforms_preserve_fits() {
  let data = rows(include_str!("../../../../tests/fixtures/lowess/input.csv"));
  let x: Vec<_> = data.iter().map(|r| r[1]).collect();
  let y: Vec<_> = data.iter().map(|r| r[2]).collect();
  let base = smooth_lowess(&x, &y, 0.4).unwrap();
  let transformed_x: Vec<_> = x.iter().map(|v| 1e9 + 3.5 * v).collect();
  let transformed_y: Vec<_> = y.iter().map(|v| 7. - 3. * v).collect();
  let transformed = smooth_lowess(&transformed_x, &transformed_y, 0.4).unwrap();
  assert_eq!(
    base.local_constant_windows,
    transformed.local_constant_windows
  );
  for (actual, expected) in transformed.values.iter().zip(base.values) {
    close(*actual, 7. - 3. * expected);
  }
}
#[test]
fn within_tie_row_permutations_change_the_defined_row_smoother() {
  let y = vec![-2., 0., 6., 4., 12., 8., 14.];
  let original = smooth_lowess(&[0.; 7], &y, 0.8).unwrap();
  let signed = smooth_lowess(&[0., -0., 0., -0., 0., -0., 0.], &y, 0.8).unwrap();
  assert_eq!(original, signed);
  let mut permuted = y;
  permuted.swap(0, 3);
  let reordered = smooth_lowess(&[0.; 7], &permuted, 0.8).unwrap();
  assert!((original.values[0] - reordered.values[3]).abs() > 1.);
  assert_eq!(original.local_constant_windows, 7);
}
#[test]
fn normalized_coordinates_avoid_raw_square_and_response_sum_overflow() {
  let x: Vec<_> = (0..7).map(|i| 1e150 + i as f64 * 1e135).collect();
  let result = smooth_lowess(&x, &[f64::MAX; 7], 0.8).unwrap();
  assert_eq!(result.values, vec![f64::MAX; 7]);
  assert_eq!(result.local_constant_windows, 0);
  let tiny: Vec<_> = (0..7).map(|i| i as f64 * 1e-300).collect();
  let linear: Vec<_> = (0..7).map(|i| 1. + i as f64 * 0.25).collect();
  for (actual, expected) in smooth_lowess(&tiny, &linear, 0.8)
    .unwrap()
    .values
    .iter()
    .zip(linear)
  {
    close(*actual, expected);
  }
}
#[test]
fn validates_shape_finite_pairs_options_and_nonnegative_rank_window() {
  for n in [0, 1] {
    assert!(smooth_lowess(&vec![0.; n], &vec![0.; n], 0.8).is_err());
  }
  assert!(smooth_lowess(&[1., 2.], &[1.], 0.8).is_err());
  for span in [
    f64::NAN,
    f64::INFINITY,
    f64::NEG_INFINITY,
    -1.,
    -0.,
    0.,
    1.,
    2.,
  ] {
    assert!(smooth_lowess(&[1., 2.], &[1., 2.], span).is_err());
  }
  assert!(smooth_lowess(&[1., f64::NAN], &[1., 2.], 0.8).is_err());
  assert!(smooth_lowess(&[1., 2.], &[1., f64::INFINITY], 0.8).is_err());
  assert!(smooth_lowess(&[1., 2.], &[1., 2.], 0.1).is_err());
  let identity = smooth_lowess(&[f64::MAX, -f64::MAX], &[1.2345678901234567, -6.75], 0.25).unwrap();
  assert_eq!(identity.values, vec![1.2345678901234567, -6.75]);
  assert_eq!(identity.half_window, 0);
  assert_eq!(identity.local_constant_windows, 2);
}
#[test]
fn enforces_work_and_observation_limits_and_rejects_unrepresentable_arithmetic() {
  assert!(
    smooth_lowess(
      &vec![0.; MAX_LOWESS_OBSERVATIONS + 1],
      &vec![0.; MAX_LOWESS_OBSERVATIONS + 1],
      0.8
    )
    .is_err()
  );
  let full = vec![0.; MAX_LOWESS_OBSERVATIONS];
  assert!(
    smooth_lowess(&full, &full, 0.8)
      .unwrap_err()
      .to_string()
      .contains("window-work")
  );
  assert_eq!(smooth_lowess(&full, &full, 0.00005).unwrap().values, full);
  assert!(
    smooth_lowess(
      &[-f64::MAX, -f64::MAX, f64::MAX, f64::MAX, f64::MAX],
      &[1., 2., 3., 4., 5.],
      0.8
    )
    .unwrap_err()
    .to_string()
    .contains("difference")
  );
  let x: Vec<_> = (0..7).map(|i| i as f64).collect();
  assert!(
    smooth_lowess(&x, &[f64::MAX, f64::MAX, -f64::MAX, 0., 0., 0., 0.], 0.8)
      .unwrap_err()
      .to_string()
      .contains("fitted value")
  );
}
