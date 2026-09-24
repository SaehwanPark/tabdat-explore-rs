#![forbid(unsafe_code)]
#![allow(clippy::needless_range_loop)]

use crate::error::StatsError;

/// Compute the arithmetic mean of a slice of floats.
pub fn mean(values: &[f64]) -> Result<f64, StatsError> {
  if values.is_empty() {
    return Err(StatsError::EmptySample);
  }
  let sum: f64 = values.iter().sum();
  Ok(sum / (values.len() as f64))
}

/// Compute the unbiased sample variance (denominator N - 1).
pub fn sample_variance(values: &[f64]) -> Result<f64, StatsError> {
  if values.len() < 2 {
    return Err(StatsError::DimensionMismatch(
      "sample variance requires at least two values".into(),
    ));
  }
  let val_mean = mean(values)?;
  let sum_sq_diff: f64 = values.iter().map(|&v| (v - val_mean).powi(2)).sum();
  Ok(sum_sq_diff / ((values.len() - 1) as f64))
}

/// Compute the sample covariance between two equal-length slices (denominator N - 1).
pub fn sample_covariance(left: &[f64], right: &[f64]) -> Result<f64, StatsError> {
  if left.len() != right.len() {
    return Err(StatsError::DimensionMismatch(format!(
      "sample covariance inputs must have the same length (got {} and {})",
      left.len(),
      right.len()
    )));
  }
  if left.len() < 2 {
    return Err(StatsError::DimensionMismatch(
      "sample covariance requires at least two paired values".into(),
    ));
  }
  let left_mean = mean(left)?;
  let right_mean = mean(right)?;
  let sum_cross: f64 = left
    .iter()
    .zip(right.iter())
    .map(|(&l, &r)| (l - left_mean) * (r - right_mean))
    .sum();
  Ok(sum_cross / ((left.len() - 1) as f64))
}

/// Compute the empirical covariance matrix for a set of column vectors.
pub fn covariance_matrix(columns: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, StatsError> {
  if columns.is_empty() {
    return Err(StatsError::DimensionMismatch(
      "covariance matrix requires at least one column".into(),
    ));
  }
  let nobs = columns[0].len();
  if nobs < 2 {
    return Err(StatsError::DimensionMismatch(
      "covariance matrix requires at least two observations".into(),
    ));
  }
  for col in &columns[1..] {
    if col.len() != nobs {
      return Err(StatsError::DimensionMismatch(
        "covariance matrix requires columns of equal length".into(),
      ));
    }
  }

  let ncol = columns.len();
  let mut matrix = vec![vec![0.0; ncol]; ncol];
  for i in 0..ncol {
    for j in i..ncol {
      let cov = sample_covariance(&columns[i], &columns[j])?;
      matrix[i][j] = cov;
      matrix[j][i] = cov;
    }
  }
  Ok(matrix)
}

/// Transpose a row-major matrix.
pub fn transpose(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, StatsError> {
  if matrix.is_empty() {
    return Err(StatsError::DimensionMismatch(
      "cannot transpose empty matrix".into(),
    ));
  }
  let rows = matrix.len();
  let cols = matrix[0].len();
  if cols == 0 {
    return Err(StatsError::DimensionMismatch(
      "cannot transpose matrix with 0 columns".into(),
    ));
  }
  for row in matrix {
    if row.len() != cols {
      return Err(StatsError::DimensionMismatch(
        "matrix is ragged and cannot be transposed".into(),
      ));
    }
  }

  let mut transposed = vec![vec![0.0; rows]; cols];
  for (r, row) in matrix.iter().enumerate() {
    for (c, &val) in row.iter().enumerate() {
      transposed[c][r] = val;
    }
  }
  Ok(transposed)
}

/// Multiply two conformable matrices: C = A * B.
pub fn multiply(left: &[Vec<f64>], right: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, StatsError> {
  if left.is_empty() || right.is_empty() {
    return Err(StatsError::DimensionMismatch(
      "cannot multiply empty matrix".into(),
    ));
  }
  let left_rows = left.len();
  let left_cols = left[0].len();
  let right_rows = right.len();
  let right_cols = right[0].len();

  if left_cols != right_rows {
    return Err(StatsError::DimensionMismatch(format!(
      "matrix inner dimensions mismatch: {left_rows}x{left_cols} vs {right_rows}x{right_cols}"
    )));
  }

  let mut result = vec![vec![0.0; right_cols]; left_rows];
  for i in 0..left_rows {
    for k in 0..left_cols {
      let a_ik = left[i][k];
      if a_ik == 0.0 {
        continue;
      }
      for j in 0..right_cols {
        result[i][j] += a_ik * right[k][j];
      }
    }
  }
  Ok(result)
}

/// Multiply a matrix by a column vector: y = A * x.
pub fn multiply_vector(matrix: &[Vec<f64>], vector: &[f64]) -> Result<Vec<f64>, StatsError> {
  if matrix.is_empty() {
    return Err(StatsError::DimensionMismatch(
      "cannot multiply empty matrix".into(),
    ));
  }
  let rows = matrix.len();
  let cols = matrix[0].len();
  if cols != vector.len() {
    return Err(StatsError::DimensionMismatch(format!(
      "matrix columns ({cols}) must match vector length ({})",
      vector.len()
    )));
  }

  let mut result = vec![0.0; rows];
  for (r, row) in matrix.iter().enumerate() {
    let mut sum = 0.0;
    for (c, &x) in row.iter().enumerate() {
      sum += x * vector[c];
    }
    result[r] = sum;
  }
  Ok(result)
}

/// Scale all elements of a matrix by a scalar.
pub fn scale(matrix: &[Vec<f64>], scalar: f64) -> Vec<Vec<f64>> {
  matrix
    .iter()
    .map(|row| row.iter().map(|&val| val * scalar).collect())
    .collect()
}

/// Invert a square matrix using Gauss-Jordan elimination with partial pivoting.
///
/// Returns `StatsError::SingularMatrix` if any pivot is smaller than 1e-12 in magnitude.
pub fn invert(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, StatsError> {
  if matrix.is_empty() {
    return Err(StatsError::DimensionMismatch(
      "cannot invert empty matrix".into(),
    ));
  }
  let n = matrix.len();
  for row in matrix {
    if row.len() != n {
      return Err(StatsError::DimensionMismatch(format!(
        "matrix must be square for inversion (found row of length {} for {n} rows)",
        row.len()
      )));
    }
  }

  // Construct [A | I] augmented matrix
  let mut augmented = vec![vec![0.0; 2 * n]; n];
  for i in 0..n {
    for j in 0..n {
      augmented[i][j] = matrix[i][j];
    }
    augmented[i][n + i] = 1.0;
  }

  // Gauss-Jordan elimination
  for pivot_idx in 0..n {
    // Find pivot row with maximum absolute value in current column
    let mut max_val = augmented[pivot_idx][pivot_idx].abs();
    let mut max_row = pivot_idx;
    for r in (pivot_idx + 1)..n {
      let val = augmented[r][pivot_idx].abs();
      if val > max_val {
        max_val = val;
        max_row = r;
      }
    }

    if max_val <= 1e-12 {
      return Err(StatsError::SingularMatrix(format!(
        "pivot at index {pivot_idx} is near zero ({max_val:e})"
      )));
    }

    if max_row != pivot_idx {
      augmented.swap(pivot_idx, max_row);
    }

    let pivot = augmented[pivot_idx][pivot_idx];
    for col in 0..(2 * n) {
      augmented[pivot_idx][col] /= pivot;
    }

    for r in 0..n {
      if r == pivot_idx {
        continue;
      }
      let factor = augmented[r][pivot_idx];
      if factor.abs() > 1e-15 {
        for col in 0..(2 * n) {
          augmented[r][col] -= factor * augmented[pivot_idx][col];
        }
      }
    }
  }

  // Extract right half of augmented matrix
  let mut inverse = vec![vec![0.0; n]; n];
  for i in 0..n {
    for j in 0..n {
      inverse[i][j] = augmented[i][n + j];
    }
  }
  Ok(inverse)
}

/// Result of Householder QR decomposition of design matrix A and response y.
#[derive(Debug, Clone)]
pub struct QrDecomposition {
  /// Upper triangular matrix R (n x n).
  pub r: Vec<Vec<f64>>,
  /// First n elements of transformed response Q' * y.
  pub qty: Vec<f64>,
}

/// Compute Householder QR decomposition of A (m x n, m >= n) and simultaneously
/// transform vector y (m x 1) into Q' * y.
///
/// This avoids squaring the condition number kappa(A)^2 that occurs when forming A'A,
/// ensuring backward numerical stability on ill-conditioned benchmarks like Longley.
pub fn qr_decompose_with_response(
  a: &[Vec<f64>],
  y: &[f64],
) -> Result<QrDecomposition, StatsError> {
  if a.is_empty() {
    return Err(StatsError::EmptySample);
  }
  let m = a.len();
  let n = a[0].len();
  if m < n {
    return Err(StatsError::ZeroDegreesOfFreedom {
      observations: m,
      parameters: n,
    });
  }
  if y.len() != m {
    return Err(StatsError::DimensionMismatch(format!(
      "response length ({}) must match matrix rows ({m})",
      y.len()
    )));
  }

  let mut a_work: Vec<Vec<f64>> = a.to_vec();
  let mut qty = y.to_vec();

  for k in 0..n {
    let mut norm_sq = 0.0;
    for i in k..m {
      norm_sq += a_work[i][k] * a_work[i][k];
    }
    let norm = norm_sq.sqrt();
    if norm <= 1e-12 {
      return Err(StatsError::SingularMatrix(format!(
        "column {k} is linearly dependent or near zero"
      )));
    }

    let a_kk = a_work[k][k];
    let alpha = if a_kk >= 0.0 { -norm } else { norm };

    // v = [a_kk - alpha, a[k+1..m, k]]
    let mut v = Vec::with_capacity(m - k);
    v.push(a_kk - alpha);
    for i in (k + 1)..m {
      v.push(a_work[i][k]);
    }

    let mut v_norm_sq = 0.0;
    for &val in &v {
      v_norm_sq += val * val;
    }
    let v_norm = v_norm_sq.sqrt();
    if v_norm > 0.0 {
      for val in &mut v {
        *val /= v_norm;
      }
    }

    // Apply Householder reflection H = I - 2 * v * v' to remaining columns of A
    for j in k..n {
      let mut dot = 0.0;
      for i in k..m {
        dot += v[i - k] * a_work[i][j];
      }
      for i in k..m {
        a_work[i][j] -= 2.0 * v[i - k] * dot;
      }
    }

    // Apply Householder reflection to qty
    let mut dot_y = 0.0;
    for i in k..m {
      dot_y += v[i - k] * qty[i];
    }
    for i in k..m {
      qty[i] -= 2.0 * v[i - k] * dot_y;
    }
  }

  // Extract upper triangular R (n x n)
  let mut r = vec![vec![0.0; n]; n];
  for i in 0..n {
    for j in i..n {
      r[i][j] = a_work[i][j];
    }
  }

  qty.truncate(n);

  Ok(QrDecomposition { r, qty })
}

/// Solve upper triangular system R * beta = rhs by back-substitution.
pub fn solve_upper_triangular(r: &[Vec<f64>], rhs: &[f64]) -> Result<Vec<f64>, StatsError> {
  let n = r.len();
  if rhs.len() != n {
    return Err(StatsError::DimensionMismatch(format!(
      "rhs length ({}) must match R dimension ({n})",
      rhs.len()
    )));
  }

  let mut beta = vec![0.0; n];
  for i in (0..n).rev() {
    let diag = r[i][i];
    if diag.abs() <= 1e-12 {
      return Err(StatsError::SingularMatrix(format!(
        "diagonal element at index {i} is near zero ({diag:e})"
      )));
    }
    let mut sum = 0.0;
    for j in (i + 1)..n {
      sum += r[i][j] * beta[j];
    }
    beta[i] = (rhs[i] - sum) / diag;
  }
  Ok(beta)
}

/// Compute (X'X)^(-1) from upper triangular matrix R where X = Q * R.
///
/// Because X'X = R'Q'QR = R'R, (X'X)^(-1) = R^(-1) * (R^(-1))'.
pub fn xtx_inverse_from_r(r: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, StatsError> {
  let n = r.len();

  // Compute R^(-1), which is also upper triangular
  let mut r_inv = vec![vec![0.0; n]; n];
  for j in 0..n {
    let diag = r[j][j];
    if diag.abs() <= 1e-12 {
      return Err(StatsError::SingularMatrix(format!(
        "diagonal element at index {j} is near zero ({diag:e})"
      )));
    }
    r_inv[j][j] = 1.0 / diag;
    for i in (0..j).rev() {
      let mut sum = 0.0;
      for k in (i + 1)..=j {
        sum += r[i][k] * r_inv[k][j];
      }
      r_inv[i][j] = -sum / r[i][i];
    }
  }

  // (X'X)^(-1) = R_inv * R_inv'
  let mut xtx_inv = vec![vec![0.0; n]; n];
  for i in 0..n {
    for j in 0..n {
      let start_k = i.max(j);
      let mut sum = 0.0;
      for k in start_k..n {
        sum += r_inv[i][k] * r_inv[j][k];
      }
      xtx_inv[i][j] = sum;
    }
  }
  Ok(xtx_inv)
}

/// Compute sandwich quadratic form: V = A * B * A'.
pub fn sandwich(a: &[Vec<f64>], b: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, StatsError> {
  let ab = multiply(a, b)?;
  let a_t = transpose(a)?;
  multiply(&ab, &a_t)
}

/// Lanczos approximation for log-gamma ln(Gamma(z)) for z > 0.
#[allow(clippy::excessive_precision)]
pub fn lgamma(z: f64) -> f64 {
  const COEFFS: [f64; 9] = [
    0.99999999999980993,
    676.5203681218851,
    -1259.1392167224028,
    771.32342877765313,
    -176.61502916214059,
    12.507343278686905,
    -0.13857109583652625,
    9.9843695780195716e-6,
    1.5056327351493116e-7,
  ];

  if z < 0.5 {
    let pi = std::f64::consts::PI;
    (pi / (pi * z).sin()).ln() - lgamma(1.0 - z)
  } else {
    let z_adj = z - 1.0;
    let mut x = COEFFS[0];
    for (i, &c) in COEFFS[1..].iter().enumerate() {
      x += c / (z_adj + (i as f64) + 1.0);
    }
    let t = z_adj + 7.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (z_adj + 0.5) * t.ln() - t + x.ln()
  }
}

fn beta_continued_fraction(a: f64, b: f64, x: f64) -> f64 {
  let qab = a + b;
  let qap = a + 1.0;
  let qam = a - 1.0;
  let mut c = 1.0;
  let mut d = 1.0 - qab * x / qap;
  if d.abs() < 1e-30 {
    d = 1e-30;
  }
  d = 1.0 / d;
  let mut h = d;

  for m in 1..200 {
    let m_f = m as f64;
    let m2 = 2.0 * m_f;

    // Even step
    let aa_even = m_f * (b - m_f) * x / ((qam + m2) * (a + m2));
    d = 1.0 + aa_even * d;
    if d.abs() < 1e-30 {
      d = 1e-30;
    }
    c = 1.0 + aa_even / c;
    if c.abs() < 1e-30 {
      c = 1e-30;
    }
    d = 1.0 / d;
    h *= d * c;

    // Odd step
    let aa_odd = -(a + m_f) * (qab + m_f) * x / ((a + m2) * (qap + m2));
    d = 1.0 + aa_odd * d;
    if d.abs() < 1e-30 {
      d = 1e-30;
    }
    c = 1.0 + aa_odd / c;
    if c.abs() < 1e-30 {
      c = 1e-30;
    }
    d = 1.0 / d;
    let del_h = d * c;
    h *= del_h;

    if (del_h - 1.0).abs() < 1e-15 {
      break;
    }
  }
  h
}

/// Regularized incomplete beta function I_x(a, b).
pub fn regularized_incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
  if x <= 0.0 {
    return 0.0;
  }
  if x >= 1.0 {
    return 1.0;
  }
  let ln_beta = lgamma(a) + lgamma(b) - lgamma(a + b);
  if x < (a + 1.0) / (a + b + 2.0) {
    let factor = (a * x.ln() + b * (1.0 - x).ln() - ln_beta).exp() / a;
    (factor * beta_continued_fraction(a, b, x)).clamp(0.0, 1.0)
  } else {
    let factor = (b * (1.0 - x).ln() + a * x.ln() - ln_beta).exp() / b;
    (1.0 - factor * beta_continued_fraction(b, a, 1.0 - x)).clamp(0.0, 1.0)
  }
}

/// Two-tailed p-value for Student's t distribution with `df` degrees of freedom.
pub fn student_t_pvalue(t: f64, df: f64) -> f64 {
  if df <= 0.0 || t.is_nan() {
    return f64::NAN;
  }
  if t == 0.0 {
    return 1.0;
  }
  let x = df / (df + t * t);
  regularized_incomplete_beta(df / 2.0, 0.5, x)
}
