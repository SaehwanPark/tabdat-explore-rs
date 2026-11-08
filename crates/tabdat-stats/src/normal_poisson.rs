//! Adaptive shared-normal-intercept integration; native fit integration is WIP.
mod nodes;
use crate::{StatsError, matrix::add_compensated};
use nodes::{GH80, GH96};

#[derive(Debug, Clone)]
struct Posterior {
  log_integral: f64,
  mean_exp_u: f64,
  mean_u2: f64,
  var_exp_u: f64,
  cov_exp_u_u2: f64,
  var_u2: f64,
  mean_u: f64,
  cov_exp_u_u: f64,
  mode: f64,
  curvature: f64,
  max_relative_difference: f64,
}
fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("normal Poisson RE: {message}"))
}
fn sum(values: impl Iterator<Item = f64>) -> f64 {
  let (mut total, mut correction) = (0., 0.);
  for value in values {
    add_compensated(&mut total, &mut correction, value);
  }
  total
}
fn mode(total: f64, mean_sum: f64, variance: f64) -> Result<(f64, f64), StatsError> {
  let score = |u: f64| total - mean_sum * u.exp() - u / variance;
  let (mut lo, mut hi) = (-60., 60.);
  if score(lo) <= 0. || score(hi) >= 0. {
    return Err(invalid("posterior mode is not bracketed"));
  }
  for _ in 0..55 {
    let mid = (lo + hi) / 2.;
    if score(mid) > 0. {
      lo = mid;
    } else {
      hi = mid;
    }
  }
  let center = (lo + hi) / 2.;
  let lambda = mean_sum * center.exp();
  if score(center).abs() > 1e-10 * (1. + total + lambda + (center / variance).abs()) {
    return Err(invalid("posterior mode root check failed"));
  }
  Ok((center, lambda + 1. / variance))
}
fn integrate_rule(
  total: f64,
  mean_sum: f64,
  variance: f64,
  center: f64,
  curvature: f64,
  rule: &[(f64, f64)],
) -> Result<[f64; 8], StatsError> {
  let scale = (2. / curvature).sqrt();
  let mut values = Vec::with_capacity(rule.len());
  for &(z, weight) in rule {
    let u = center + scale * z;
    if !u.is_finite() || u.abs() > 100. {
      return Err(invalid("quadrature random intercept exceeds100"));
    }
    let exp_u = u.exp();
    let log_weight = weight.ln() + z * z + total * u - mean_sum * exp_u - u * u / (2. * variance);
    if !log_weight.is_finite() {
      return Err(invalid("nonfinite quadrature integrand"));
    }
    values.push((log_weight, exp_u, u * u, u));
  }
  let anchor = values.iter().map(|v| v.0).fold(f64::NEG_INFINITY, f64::max);
  let weights: Vec<_> = values.iter().map(|v| (v.0 - anchor).exp()).collect();
  let denominator = sum(weights.iter().copied());
  let mean_exp = sum(values.iter().zip(&weights).map(|(v, w)| w * v.1)) / denominator;
  let mean_u2 = sum(values.iter().zip(&weights).map(|(v, w)| w * v.2)) / denominator;
  let mean_u = sum(values.iter().zip(&weights).map(|(v, w)| w * v.3)) / denominator;
  let cov_u = sum(
    values
      .iter()
      .zip(&weights)
      .map(|(v, w)| w * (v.1 - mean_exp) * (v.3 - mean_u)),
  ) / denominator;
  // Centered products retain small posterior covariances at high count totals.
  let var_exp = sum(
    values
      .iter()
      .zip(&weights)
      .map(|(v, w)| w * (v.1 - mean_exp).powi(2)),
  ) / denominator;
  let cov = sum(
    values
      .iter()
      .zip(&weights)
      .map(|(v, w)| w * (v.1 - mean_exp) * (v.2 - mean_u2)),
  ) / denominator;
  let var_u2 = sum(
    values
      .iter()
      .zip(&weights)
      .map(|(v, w)| w * (v.2 - mean_u2).powi(2)),
  ) / denominator;
  let log_integral =
    scale.ln() + anchor + denominator.ln() - (2. * std::f64::consts::PI * variance).ln() / 2.;
  let result = [
    log_integral,
    mean_exp,
    mean_u2,
    var_exp,
    cov,
    var_u2,
    mean_u,
    cov_u,
  ];
  if result.iter().any(|v| !v.is_finite()) {
    return Err(invalid("nonfinite posterior moments"));
  }
  Ok(result)
}
fn posterior(total: f64, mean_sum: f64, log_variance: f64) -> Result<Posterior, StatsError> {
  if !total.is_finite()
    || total < 0.
    || total > 100000.
    || total.fract() != 0.
    || !mean_sum.is_finite()
    || mean_sum <= 0.
    || !log_variance.is_finite()
    || log_variance.abs() >= 8.
  {
    return Err(invalid("invalid integrated group parameters"));
  }
  let variance = log_variance.exp();
  let (mode, curvature) = mode(total, mean_sum, variance)?;
  let high = integrate_rule(total, mean_sum, variance, mode, curvature, &GH96)?;
  let low = integrate_rule(total, mean_sum, variance, mode, curvature, &GH80)?;
  // Check intensity and prior-score units, not tiny unscaled exp(u) moments:
  // regression/information multiply these by M and inverse variance.
  let units = [
    1.,
    mean_sum,
    1. / variance,
    mean_sum.powi(2),
    mean_sum / variance,
    1. / variance.powi(2),
    1. / variance,
    mean_sum / variance,
  ];
  let discrepancy = high
    .iter()
    .zip(low)
    .zip(units)
    .map(|((a, b), scale)| ((a - b) * scale).abs() / (1. + (a * scale).abs()))
    .fold(0., f64::max);
  if discrepancy > 1e-9 {
    return Err(invalid("80/96-node quadrature disagreement exceeds1e-9"));
  }
  Ok(Posterior {
    log_integral: high[0],
    mean_exp_u: high[1],
    mean_u2: high[2],
    var_exp_u: high[3],
    cov_exp_u_u2: high[4],
    var_u2: high[5],
    mean_u: high[6],
    cov_exp_u_u: high[7],
    mode,
    curvature,
    max_relative_difference: discrepancy,
  })
}

struct Evaluation {
  log_likelihood: f64,
  score: Vec<f64>,
  information: Vec<Vec<f64>>,
  max_quadrature_difference: f64,
}

fn evaluate(
  x: &[Vec<f64>],
  y: &[f64],
  groups: &[Vec<usize>],
  parameters: &[f64],
) -> Result<Evaluation, StatsError> {
  let k = parameters
    .len()
    .checked_sub(1)
    .ok_or_else(|| invalid("empty parameters"))?;
  if !(2..=15).contains(&k)
    || x.len() != y.len()
    || y.is_empty()
    || groups.is_empty()
    || x
      .iter()
      .any(|row| row.len() != k || row[0] != 1. || row.iter().any(|v| !v.is_finite()))
    || y
      .iter()
      .any(|v| !v.is_finite() || *v < 0. || *v > 100000. || v.fract() != 0.)
  {
    return Err(invalid("inconsistent complete-case group design"));
  }
  let t = parameters[k];
  if !t.is_finite() || t.abs() >= 8. {
    return Err(invalid("log variance must be strictly inside (-8,8)"));
  }
  let variance = t.exp();
  let mut seen = vec![false; y.len()];
  for group in groups {
    if group.is_empty() {
      return Err(invalid("empty group"));
    }
    for &row in group {
      if row >= y.len() || seen[row] {
        return Err(invalid("group frame must partition rows exactly"));
      }
      seen[row] = true;
    }
  }
  if seen.iter().any(|v| !*v) {
    return Err(invalid("group frame omitted rows"));
  }
  let eta: Vec<_> = x
    .iter()
    .map(|row| sum(row.iter().zip(parameters).map(|(a, b)| a * b)))
    .collect();
  if eta.iter().any(|v| !v.is_finite() || *v < -40. || *v > 40.) {
    return Err(invalid("fixed linear predictor exceeds [-40,40]"));
  }
  let mu: Vec<_> = eta.iter().map(|v| v.exp()).collect();
  let mut score = vec![0.; k + 1];
  let mut sc = score.clone();
  let mut information = vec![vec![0.; k + 1]; k + 1];
  let mut ic = information.clone();
  let (mut ll, mut lc, mut max_difference) = (0., 0., 0.0_f64);
  for group in groups {
    let total = sum(group.iter().map(|&i| y[i]));
    let m = sum(group.iter().map(|&i| mu[i]));
    let post = posterior(total, m, t)?;
    max_difference = max_difference.max(post.max_relative_difference);
    let mut center: Vec<_> = (0..k)
      .map(|j| sum(group.iter().map(|&i| mu[i] / m * x[i][j])))
      .collect();
    center[0] = 1.;
    let lambda_mean = m * post.mean_exp_u;
    // Stein identities stabilize count-total score/curvature. In particular,
    // Cov(lambda,u)/v replaces E(lambda)-Var(lambda), avoiding large-count subtraction.
    let eta_score = post.mean_u / variance;
    let eta_information = m * post.cov_exp_u_u / variance;
    for j in 0..k {
      let conditional_score = sum(group.iter().map(|&i| y[i] * (x[i][j] - center[j])));
      add_compensated(
        &mut score[j],
        &mut sc[j],
        conditional_score + center[j] * eta_score,
      );
      for l in j..k {
        let within = sum(
          group
            .iter()
            .map(|&i| mu[i] / m * (x[i][j] - center[j]) * (x[i][l] - center[l])),
        );
        add_compensated(
          &mut information[j][l],
          &mut ic[j][l],
          lambda_mean * within + eta_information * center[j] * center[l],
        );
      }
      add_compensated(
        &mut information[j][k],
        &mut ic[j][k],
        center[j] * m * post.cov_exp_u_u2 / (2. * variance),
      );
    }
    add_compensated(
      &mut score[k],
      &mut sc[k],
      post.mean_u2 / (2. * variance) - 0.5,
    );
    add_compensated(
      &mut information[k][k],
      &mut ic[k][k],
      post.mean_u2 / (2. * variance) - post.var_u2 / (4. * variance * variance),
    );
    let constant = sum(
      group
        .iter()
        .map(|&i| y[i] * eta[i] - crate::matrix::lgamma(y[i] + 1.)),
    );
    add_compensated(&mut ll, &mut lc, constant + post.log_integral);
  }
  let g = groups.len() as f64;
  for value in &mut score {
    *value /= g;
  }
  for row in &mut information {
    for value in row {
      *value /= g;
    }
  }
  for j in 0..information.len() {
    let (earlier, remaining) = information.split_at_mut(j);
    for (l, row) in earlier.iter().enumerate() {
      remaining[0][l] = row[j];
    }
  }
  if !ll.is_finite()
    || score.iter().any(|v| !v.is_finite())
    || information.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(invalid("nonfinite integrated likelihood derivatives"));
  }
  Ok(Evaluation {
    log_likelihood: ll,
    score,
    information,
    max_quadrature_difference: max_difference,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn fixed_rules_have_positive_tail_weights_and_independent_gaussian_moments() {
    for rule in [&GH80[..], &GH96[..]] {
      assert!(
        rule
          .iter()
          .all(|(z, w)| z.is_finite() && w.is_finite() && *w > 0.)
      );
      for pair in rule.windows(2) {
        assert!(pair[0].0 < pair[1].0);
      }
      let mut exact = std::f64::consts::PI.sqrt();
      for power in 0..=12 {
        if power > 0 {
          exact *= (2 * power - 1) as f64 / 2.;
        }
        let actual = sum(rule.iter().map(|(z, w)| w * z.powi(2 * power)));
        assert!(
          (actual / exact - 1.).abs() < 2e-13,
          "power{power}: {actual} vs {exact}"
        );
        let odd = sum(rule.iter().map(|(z, w)| w * z.powi(2 * power + 1)));
        assert!(odd.abs() < 2e-12 * (1. + exact), "odd power{power}: {odd}");
      }
    }
  }
  #[test]
  fn posterior_moments_match_direct_r_improper_integration() {
    let fixture = include_str!("../tests/fixtures/normal_poisson/posterior.csv");
    let cases: Vec<_> = fixture.lines().skip(1).collect();
    assert_eq!(cases.len(), 6);
    for line in cases {
      let v: Vec<f64> = line.split(',').map(|s| s.parse().unwrap()).collect();
      assert_eq!(v.len(), 11);
      let p = posterior(v[0], v[1], v[2]).unwrap();
      let actual = [
        p.log_integral,
        p.mean_exp_u,
        p.mean_u2,
        p.var_exp_u,
        p.cov_exp_u_u2,
        p.var_u2,
        p.mean_u,
        p.cov_exp_u_u,
      ];
      for (&a, &b) in actual.iter().zip(&v[3..]) {
        assert!((a - b).abs() <= 1e-9 + 1e-10 * b.abs(), "{v:?}: {a} vs {b}");
      }
      assert!(p.max_relative_difference <= 1e-9);
      let variance = v[2].exp();
      let score = v[0] - v[1] * p.mode.exp() - p.mode / variance;
      assert!(score.abs() <= 1e-10 * (1. + v[0] + v[1] * p.mode.exp() + (p.mode / variance).abs()));
      assert_eq!(p.curvature, v[1] * p.mode.exp() + 1. / variance);
    }
  }
  #[test]
  fn joint_multivariate_scores_and_information_match_richardson_differences() {
    let x = vec![
      vec![1., -1., 0.2],
      vec![1., 0.3, -0.8],
      vec![1., 1.2, 0.1],
      vec![1., -0.4, 0.7],
      vec![1., 0.2, -1.],
      vec![1., 1.1, 0.8],
      vec![1., -0.9, 0.5],
    ];
    let y = vec![0., 3., 5., 1., 4., 2., 8.];
    let groups = vec![vec![0, 1, 2], vec![3], vec![4, 5, 6]];
    let parameters = vec![0.2, 0.4, -0.3, 0.6];
    let e = evaluate(&x, &y, &groups, &parameters).unwrap();
    assert!(e.max_quadrature_difference <= 1e-9);
    for j in 0..4 {
      let difference = |h: f64| {
        let mut plus = parameters.clone();
        plus[j] += h;
        let a = evaluate(&x, &y, &groups, &plus).unwrap();
        let mut minus = parameters.clone();
        minus[j] -= h;
        let b = evaluate(&x, &y, &groups, &minus).unwrap();
        (
          (a.log_likelihood - b.log_likelihood) / (2. * h * groups.len() as f64),
          a.score
            .iter()
            .zip(b.score)
            .map(|(u, v)| -(u - v) / (2. * h))
            .collect::<Vec<_>>(),
        )
      };
      let (d1, i1) = difference(2e-4);
      let (d2, i2) = difference(1e-4);
      let derivative = (4. * d2 - d1) / 3.;
      assert!(
        (e.score[j] - derivative).abs() <= 1e-8 * (1. + e.score[j].abs()),
        "score{j}: {} vs {derivative}",
        e.score[j]
      );
      for i in 0..4 {
        let numeric = (4. * i2[i] - i1[i]) / 3.;
        assert!(
          (e.information[i][j] - numeric).abs() <= 1e-8 * (1. + e.information[i][j].abs()),
          "information{i},{j}: {} vs {numeric}",
          e.information[i][j]
        );
        assert_eq!(e.information[i][j], e.information[j][i]);
      }
    }
  }
  #[test]
  fn invalid_group_frames_and_fixed_predictors_reject_without_panics() {
    let x = vec![vec![1., 0.], vec![1., 1.]];
    let y = vec![0., 2.];
    let p = [0., 1., 0.];
    for groups in [
      vec![],
      vec![vec![]],
      vec![vec![0]],
      vec![vec![0, 1, 2]],
      vec![vec![0, 0, 1]],
    ] {
      assert!(evaluate(&x, &y, &groups, &p).is_err());
    }
    assert!(evaluate(&x, &y, &[vec![0, 1]], &[0., 41., 0.]).is_err());
    assert!(evaluate(&[vec![], vec![]], &y, &[vec![0, 1]], &p).is_err());
  }
  #[test]
  fn stein_score_and_information_avoid_high_count_subtraction() {
    for line in include_str!("../tests/fixtures/normal_poisson/posterior.csv")
      .lines()
      .skip(1)
    {
      let v: Vec<f64> = line.split(',').map(|s| s.parse().unwrap()).collect();
      let (s, m, t) = (v[0], v[1], v[2]);
      let variance = t.exp();
      let p = posterior(s, m, t).unwrap();
      // Integration by parts: S-E(lambda)=E(u)/v and
      // E(lambda)-Var(lambda)=Cov(lambda,u)/v. The latter avoids O(S) cancellation.
      let score = p.mean_u / variance;
      assert!(
        (score - (s - m * p.mean_exp_u)).abs() <= 1e-9 + 1e-10 * (s + (m * p.mean_exp_u).abs())
      );
      let information = m * p.cov_exp_u_u / variance;
      assert!(information > 0. && information.is_finite());
      let difference = |h: f64| {
        let plus = posterior(s, m * h.exp(), t).unwrap();
        let minus = posterior(s, m * (-h).exp(), t).unwrap();
        -(plus.mean_u - minus.mean_u) / (2. * h * variance)
      };
      let derivative = (4. * difference(1e-4) - difference(2e-4)) / 3.;
      assert!(
        (information - derivative).abs() <= 1e-8 * (1. + information.abs()),
        "{v:?}: {information} vs {derivative}"
      );
    }
  }
  #[test]
  fn high_variance_control_start_rejects_actual_intensity_moment_disagreement() {
    let source = include_str!("../../../tests/fixtures/mma23_gamma_re_bootstrap/source.asc");
    let mut rejected = 0;
    for line in source.lines().filter(|line| !line.trim().is_empty()) {
      let tokens: Vec<_> = line.split_whitespace().collect();
      let total = tokens[20..25]
        .iter()
        .map(|v| v.parse::<f64>().unwrap())
        .sum();
      // Independent R Poisson GLM start on the existing 200-record control.
      let m = tokens[10..15]
        .iter()
        .map(|v| (1.2479608264839344 + 0.5673452307402028 * v.parse::<f64>().unwrap()).exp())
        .sum();
      if let Err(error) = posterior(total, m, 2.) {
        assert!(
          error.to_string().contains("quadrature disagreement"),
          "{error}"
        );
        rejected += 1;
      }
    }
    assert!(
      rejected > 0,
      "high-variance start must expose actual 80/96 disagreement"
    );
  }
  #[test]
  fn invalid_parameters_and_unbracketed_modes_are_errors() {
    for (s, m, t) in [
      (0., 0., 0.),
      (-1., 1., 0.),
      (1.5, 1., 0.),
      (100001., 1., 0.),
      (1., 1., 8.),
      (1., 1., -8.),
      (f64::NAN, 1., 0.),
      (1., f64::INFINITY, 0.),
      (100000., 1e-100, 0.),
    ] {
      assert!(posterior(s, m, t).is_err(), "{s},{m},{t}");
    }
  }
}
