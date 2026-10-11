use crate::error::StatsError;
use crate::matrix::{mean, sample_variance, student_t_pvalue, student_t_quantile};

/// Descriptive statistics for one sample used in a Student t-test.
#[derive(Debug, Clone, PartialEq)]
pub struct TtestSampleStatistics {
  /// Number of finite observations.
  pub observations: usize,
  /// Arithmetic sample mean.
  pub mean: f64,
  /// Sample standard error (`sample standard deviation / sqrt(n)`), or NaN when `n == 1`.
  pub standard_error: f64,
  /// Unbiased sample standard deviation (denominator `n - 1`), or NaN when `n == 1`.
  pub standard_deviation: f64,
  /// Lower endpoint of the two-sided 95% Student-t interval, or NaN when `n == 1`.
  pub ci_lower: f64,
  /// Upper endpoint of the two-sided 95% Student-t interval, or NaN when `n == 1`.
  pub ci_upper: f64,
}

/// Numerical output shared by one-sample, paired, and independent-sample t-tests.
#[derive(Debug, Clone, PartialEq)]
pub struct TtestStatistics {
  /// Statistics for the first sample.
  pub group1: TtestSampleStatistics,
  /// Statistics for the second sample in paired or independent tests.
  pub group2: Option<TtestSampleStatistics>,
  /// Pooled descriptive statistics for independent-sample tests.
  pub combined: Option<TtestSampleStatistics>,
  /// Statistics for the tested sample mean or, for paired/independent tests, mean difference.
  pub difference: TtestSampleStatistics,
  /// Student t statistic; NaN when its standard error is zero or undefined.
  pub statistic: f64,
  /// Reference-distribution degrees of freedom, including zero when undefined for one observation.
  pub degrees_of_freedom: f64,
  /// Lower-tail probability under the reference Student-t distribution.
  pub p_left: f64,
  /// Two-sided probability under the reference Student-t distribution.
  pub p_two: f64,
  /// Upper-tail probability under the reference Student-t distribution.
  pub p_right: f64,
}

/// Compute a one-sample Student t-test against a numeric null mean.
pub fn one_sample_ttest(
  observations: &[f64],
  null_mean: f64,
) -> Result<TtestStatistics, StatsError> {
  validate_sample(observations, "one-sample observations")?;

  let group1 = summarize_sample(observations)?;
  let degrees_of_freedom = observations.len().saturating_sub(1) as f64;
  let inference = infer_mean(
    group1.mean - null_mean,
    group1.standard_error,
    degrees_of_freedom,
  );
  Ok(TtestStatistics {
    difference: group1.clone(),
    group1,
    group2: None,
    combined: None,
    statistic: inference.0,
    degrees_of_freedom,
    p_left: inference.1,
    p_two: inference.2,
    p_right: inference.3,
  })
}

/// Compute a paired Student t-test on finite, equally sized samples.
pub fn paired_ttest(first: &[f64], second: &[f64]) -> Result<TtestStatistics, StatsError> {
  validate_sample(first, "first paired sample")?;
  validate_sample(second, "second paired sample")?;
  if first.len() != second.len() {
    return Err(StatsError::DimensionMismatch(format!(
      "paired t-test samples must have the same length (got {} and {})",
      first.len(),
      second.len()
    )));
  }

  let differences = first
    .iter()
    .zip(second)
    .map(|(left, right)| left - right)
    .collect::<Vec<_>>();
  let group1 = summarize_sample(first)?;
  let group2 = summarize_sample(second)?;
  let difference = summarize_sample(&differences)?;
  let degrees_of_freedom = first.len().saturating_sub(1) as f64;
  let inference = infer_mean(
    difference.mean,
    difference.standard_error,
    degrees_of_freedom,
  );
  Ok(TtestStatistics {
    group1,
    group2: Some(group2),
    combined: None,
    difference,
    statistic: inference.0,
    degrees_of_freedom,
    p_left: inference.1,
    p_two: inference.2,
    p_right: inference.3,
  })
}

/// Compute a pooled or Welch two-sample Student t-test on finite samples.
pub fn independent_ttest(
  first: &[f64],
  second: &[f64],
  welch: bool,
) -> Result<TtestStatistics, StatsError> {
  validate_sample(first, "first independent sample")?;
  validate_sample(second, "second independent sample")?;

  let group1 = summarize_sample(first)?;
  let group2 = summarize_sample(second)?;
  let mut combined_observations = Vec::with_capacity(first.len() + second.len());
  combined_observations.extend_from_slice(first);
  combined_observations.extend_from_slice(second);
  let combined = summarize_sample(&combined_observations)?;
  let mean_difference = group1.mean - group2.mean;
  let (standard_error, degrees_of_freedom) = if welch {
    welch_inference(&group1, &group2)
  } else {
    pooled_inference(first.len(), second.len(), &group1, &group2)
  };
  let (ci_lower, ci_upper) =
    inference_interval(mean_difference, standard_error, degrees_of_freedom);
  let difference = TtestSampleStatistics {
    observations: if welch {
      0
    } else {
      first.len().saturating_sub(1) + second.len().saturating_sub(1)
    },
    mean: mean_difference,
    standard_error,
    standard_deviation: f64::NAN,
    ci_lower,
    ci_upper,
  };
  let inference = infer_mean(mean_difference, standard_error, degrees_of_freedom);
  Ok(TtestStatistics {
    group1,
    group2: Some(group2),
    combined: Some(combined),
    difference,
    statistic: inference.0,
    degrees_of_freedom,
    p_left: inference.1,
    p_two: inference.2,
    p_right: inference.3,
  })
}

fn validate_sample(observations: &[f64], description: &str) -> Result<(), StatsError> {
  if observations.is_empty() {
    return Err(StatsError::EmptySample);
  }
  if observations.iter().any(|value| !value.is_finite()) {
    return Err(StatsError::NonFiniteValue(description.into()));
  }
  Ok(())
}

fn summarize_sample(observations: &[f64]) -> Result<TtestSampleStatistics, StatsError> {
  let sample_mean = mean(observations)?;
  let observations_count = observations.len();
  let (standard_deviation, standard_error, ci_lower, ci_upper) = if observations_count == 1 {
    (f64::NAN, f64::NAN, f64::NAN, f64::NAN)
  } else {
    let sample_std_dev = sample_variance(observations)?.sqrt();
    let standard_error = sample_std_dev / (observations_count as f64).sqrt();
    let (ci_lower, ci_upper) = if standard_error == 0.0 || standard_error.is_nan() {
      (sample_mean, sample_mean)
    } else {
      let critical_value = student_t_quantile(0.975, (observations_count - 1) as f64);
      (
        sample_mean - critical_value * standard_error,
        sample_mean + critical_value * standard_error,
      )
    };
    (sample_std_dev, standard_error, ci_lower, ci_upper)
  };
  Ok(TtestSampleStatistics {
    observations: observations_count,
    mean: sample_mean,
    standard_error,
    standard_deviation,
    ci_lower,
    ci_upper,
  })
}

fn pooled_inference(
  first_n: usize,
  second_n: usize,
  first: &TtestSampleStatistics,
  second: &TtestSampleStatistics,
) -> (f64, f64) {
  if first_n <= 1 || second_n <= 1 {
    return (f64::NAN, 0.0);
  }
  let df = (first_n + second_n - 2) as f64;
  let first_variance = first.standard_deviation.powi(2);
  let second_variance = second.standard_deviation.powi(2);
  let pooled_variance =
    (((first_n - 1) as f64) * first_variance + ((second_n - 1) as f64) * second_variance) / df;
  let standard_error = (pooled_variance * (1.0 / first_n as f64 + 1.0 / second_n as f64)).sqrt();
  (standard_error, df)
}

fn welch_inference(first: &TtestSampleStatistics, second: &TtestSampleStatistics) -> (f64, f64) {
  let first_term = if first.observations > 1 && !first.standard_deviation.is_nan() {
    first.standard_deviation.powi(2) / first.observations as f64
  } else {
    0.0
  };
  let second_term = if second.observations > 1 && !second.standard_deviation.is_nan() {
    second.standard_deviation.powi(2) / second.observations as f64
  } else {
    0.0
  };
  let sum_terms = first_term + second_term;
  if sum_terms == 0.0 {
    return (0.0, 0.0);
  }

  let mut denominator = 0.0;
  if first.observations > 1 && !first.standard_deviation.is_nan() && first_term > 0.0 {
    denominator += first_term.powi(2) / (first.observations - 1) as f64;
  }
  if second.observations > 1 && !second.standard_deviation.is_nan() && second_term > 0.0 {
    denominator += second_term.powi(2) / (second.observations - 1) as f64;
  }
  let df = if denominator == 0.0 {
    0.0
  } else {
    sum_terms.powi(2) / denominator
  };
  (sum_terms.sqrt(), df)
}

fn inference_interval(mean_difference: f64, standard_error: f64, df: f64) -> (f64, f64) {
  if !standard_error.is_nan() && standard_error > 0.0 && df > 0.0 {
    let critical_value = student_t_quantile(0.975, df);
    (
      mean_difference - critical_value * standard_error,
      mean_difference + critical_value * standard_error,
    )
  } else if standard_error == 0.0 {
    (mean_difference, mean_difference)
  } else {
    (f64::NAN, f64::NAN)
  }
}

fn infer_mean(estimate: f64, standard_error: f64, df: f64) -> (f64, f64, f64, f64) {
  if df <= 0.0 || standard_error.is_nan() || standard_error == 0.0 {
    return (f64::NAN, f64::NAN, f64::NAN, f64::NAN);
  }
  let statistic = estimate / standard_error;
  let p_left = student_t_cdf(statistic, df);
  let p_two = 2.0 * (1.0 - student_t_cdf(statistic.abs(), df));
  let p_right = 1.0 - p_left;
  (statistic, p_left, p_two, p_right)
}

fn student_t_cdf(statistic: f64, df: f64) -> f64 {
  if statistic >= 0.0 {
    1.0 - student_t_pvalue(statistic, df) / 2.0
  } else {
    student_t_pvalue(-statistic, df) / 2.0
  }
}
