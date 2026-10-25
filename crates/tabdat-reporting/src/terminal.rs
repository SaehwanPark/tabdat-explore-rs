use std::fmt::Write;

use tabdat_domain::{CellValue, DatasetInfo, ExecutionResult, SummaryRow};

fn binary_inference_note(robust: bool, clustered: bool) -> &'static str {
  if clustered {
    "Independent clusters; asymptotic standard-normal coefficient inference, not G-1 Student t. LR p-value unavailable under robust inference."
  } else if robust {
    "Independent observations; asymptotic standard-normal coefficient inference. LR p-value unavailable under robust inference."
  } else {
    "Independent Bernoulli model; asymptotic standard-normal coefficient inference, no N-K correction."
  }
}

fn binary_wald_note(test: Option<&tabdat_domain::BinaryOmnibusWald>) -> String {
  let Some(test) = test else {
    return String::new();
  };
  if let (Some(statistic), Some(p_value)) = (test.statistic, test.p_value) {
    format!(
      "\nOmnibus Wald chi2({}) = {statistic}; p-value = {p_value}\nH0: all non-intercept coefficients = 0; selected covariance, asymptotic chi-square (not F).",
      test.degrees_of_freedom
    )
  } else {
    format!(
      "\nOmnibus Wald unavailable: {}; requested df = {}. H0: all non-intercept coefficients = 0. No reduced-rank test supplied.",
      test.unavailable_reason.as_deref().unwrap_or("unavailable"),
      test.degrees_of_freedom
    )
  }
}

fn binary_covariance_note(
  robust: bool,
  cluster: Option<&tabdat_domain::BinaryCluster>,
  model_based: &str,
) -> String {
  if let Some(cluster) = cluster {
    format!(
      "one-way clustered score sandwich on {}; G = {}; G/(G-1) correction; independent clusters, normal approximation may be unreliable with few clusters",
      cluster.variable.escape_debug(),
      cluster.group_count
    )
  } else if robust {
    "robust score sandwich, N/(N-1) correction".to_owned()
  } else {
    model_based.to_owned()
  }
}

fn cell(value: &CellValue) -> String {
  match value {
    CellValue::Null => "NA".to_owned(),
    CellValue::Boolean(value) => value.to_string(),
    CellValue::SignedInteger(value) => value.to_string(),
    CellValue::UnsignedInteger(value) => value.to_string(),
    CellValue::Float(value) => value.to_string(),
    CellValue::Text(value) => format!("{value:?}"),
    CellValue::Bytes(value) => {
      let mut text = String::from("0x");
      for byte in value {
        write!(text, "{byte:02x}").unwrap();
      }
      text
    }
    CellValue::Decimal { scale, value, .. } => {
      let mut digits = value.unsigned_abs().to_string();
      let scale = usize::from(*scale);
      if scale > 0 {
        if digits.len() <= scale {
          digits = format!("{}{}", "0".repeat(scale + 1 - digits.len()), digits);
        }
        digits.insert(digits.len() - scale, '.');
      }
      if *value < 0 {
        format!("-{digits}")
      } else {
        digits
      }
    }
  }
}

fn optional_number(value: Option<f64>) -> String {
  value
    .map(|value| value.to_string())
    .unwrap_or_else(|| "NA".to_owned())
}

fn poisson_covariance_note(
  robust: bool,
  cluster: Option<&tabdat_domain::PoissonCluster>,
) -> &'static str {
  if cluster.is_some() {
    "one-way cluster score sandwich, G/(G-1), not OLS HC1"
  } else if robust {
    "independent-row score sandwich, N/(N-1), not OLS HC1"
  } else {
    "model-based inverse Poisson Fisher information, no N-K scaling"
  }
}

fn count_cluster_note(cluster: Option<&tabdat_domain::BinaryCluster>) -> String {
  cluster.map_or_else(String::new, |c| format!("Cluster variable: \"{}\"; retained independent clusters G = {}; finite-sample multiplier = {}\nAllows within-cluster dependence, assumes independence ACROSS clusters. Normal inference, NOT G-1 Student t or a few-cluster correction.\n",c.variable.escape_debug(),c.group_count,c.group_count as f64/(c.group_count-1) as f64))
}

fn poisson_estimates(estimates: &[tabdat_domain::PoissonEstimate]) -> String {
  let mut output = String::from(
    "Variable\tEstimate\tStd. error\tz\tP(two-sided)\tCI lower (95%)\tCI upper (95%)\n",
  );
  for c in estimates {
    writeln!(
      output,
      "{}\t{}\t{}\t{}\t{}\t{}\t{}",
      c.name.escape_debug(),
      c.value,
      c.standard_error,
      optional_number(c.statistic),
      optional_number(c.p_value),
      optional_number(c.confidence_lower),
      optional_number(c.confidence_upper)
    )
    .unwrap();
  }
  output.push_str(
    "NA = unavailable or undefined; zero SE does not imply an exact confidence interval.\n",
  );
  output
}

fn poisson_report(r: &tabdat_domain::PoissonResult) -> String {
  let mut output = format!(
    "Poisson log-link regression ({})\nOutcome: {}; N = {}; count total (float64) = {}; zeros = {}\n",
    poisson_covariance_note(r.robust, r.cluster.as_ref()),
    r.outcome.escape_debug(),
    r.observation_count,
    r.count_sum,
    r.zero_count
  );
  writeln!(
    output,
    "Candidate rows = {}; excluded missing/nonfinite = {}",
    r.sample
      .as_ref()
      .map_or(r.observation_count, |s| s.total_observations),
    r.sample.as_ref().map_or(0, |s| s.dropped_observations)
  )
  .unwrap();
  writeln!(output,"Converged in {} outer Newton iterations; max absolute unit-column-norm score = {}; full Newton step = {}",
    r.iterations,r.max_abs_scaled_score,r.max_abs_scaled_newton_step).unwrap();
  writeln!(
    output,
    "Log likelihood = {}; null log likelihood = {}; null model: {}",
    r.log_likelihood,
    r.null_log_likelihood,
    if r.offset_variable.is_some() {
      if r.include_intercept {
        "intercept-only with same fixed offset"
      } else {
        "all coefficients zero with same fixed offset"
      }
    } else if r.include_intercept {
      "intercept-only estimated mean"
    } else {
      "unit mean (all coefficients zero), not a fitted intercept"
    }
  )
  .unwrap();
  writeln!(
    output,
    "{}({}) = {}; p-value = {}",
    if r.likelihood_ratio_p_value.is_some() {
      "LR chi2"
    } else {
      "descriptive LR difference"
    },
    r.likelihood_ratio_df,
    r.likelihood_ratio,
    optional_number(r.likelihood_ratio_p_value)
  )
  .unwrap();
  output.push_str(&binary_wald_note(r.omnibus_wald.as_ref()));
  writeln!(
    output,
    "\nMcFadden pseudo R2 = {}; residual deviance = {}",
    r.mcfadden_r_squared, r.residual_deviance
  )
  .unwrap();
  writeln!(
    output,
    "Pearson chi-square = {}; residual df (N-K) = {}; Pearson dispersion = {}",
    r.pearson_chi_square, r.df_residual, r.pearson_dispersion
  )
  .unwrap();
  if r.cluster.is_some() {
    output.push_str("Dispersion is descriptive, not a panel/specification test. Conditional mean = exp(xb), not an event probability.\nNormal 95% coefficient inference using selected cluster covariance; no Pearson-dispersion rescaling.\n");
    output.push_str(&count_cluster_note(r.cluster.as_ref()));
    output.push_str("LR p-value unavailable under clustered QMLE inference.\n");
  } else {
    output.push_str("Dispersion is descriptive, not a panel/specification test. Conditional mean = exp(xb), not an event probability.\nNormal 95% coefficient inference; model-based Poisson assumes variance=mean. Robust corrects variance misspecification under independent rows, NOT repeated-person/panel dependence.\n");
    if r.robust {
      output.push_str("LR p-value unavailable under robust QMLE inference.\n");
    }
  }
  if let Some(variable) = &r.offset_variable {
    writeln!(output, "Fixed offset: {} (coefficient 1, not estimated); xb = X beta + offset. Predictions use current offset values; effects hold retained offsets fixed.", variable.escape_debug()).unwrap();
  }
  output.push_str(&poisson_estimates(&r.coefficients));
  output
}

fn nb2_covariance_note(robust: bool, cluster: Option<&tabdat_domain::Nb2Cluster>) -> &'static str {
  if cluster.is_some() {
    "one-way FULL joint score sandwich, G/(G-1), inverse JOINT observed information bread"
  } else if robust {
    "independent-row full joint score sandwich, N/(N-1), not OLS HC1"
  } else {
    "model-based inverse JOINT observed information (beta and lnalpha)"
  }
}
fn nb2_report(r: &tabdat_domain::Nb2Result) -> String {
  let mut output = format!(
    "Negative binomial NB2 log-link regression ({})\nOutcome: {}; N = {}; count total (float64) = {}; zeros = {}\nVariance = mu + alpha*mu^2; size = 1/alpha, alpha = exp(lnalpha)\n",
    nb2_covariance_note(r.robust, r.cluster.as_ref()),
    r.outcome.escape_debug(),
    r.observation_count,
    r.count_sum,
    r.zero_count
  );
  writeln!(
    output,
    "Candidate rows = {}; excluded missing/nonfinite = {}",
    r.sample
      .as_ref()
      .map_or(r.observation_count, |s| s.total_observations),
    r.sample.as_ref().map_or(0, |s| s.dropped_observations)
  )
  .unwrap();
  writeln!(output,"Converged in {} full-model outer iterations; max absolute scaled score = {}; full UNDAMPED Newton step = {}",r.iterations,r.max_abs_scaled_score,r.max_abs_scaled_newton_step).unwrap();
  writeln!(
    output,
    "Log likelihood = {}; NB null log likelihood = {}; null alpha = {}",
    r.log_likelihood, r.null_log_likelihood, r.null_alpha
  )
  .unwrap();
  writeln!(
    output,
    "Null: {}, with nuisance dispersion reestimated",
    if r.offset_variable.is_some() {
      if r.include_intercept {
        "intercept-only with same fixed offset"
      } else {
        "all beta zero with same fixed offset"
      }
    } else if r.include_intercept {
      "intercept-only estimated mean"
    } else {
      "unit mean (all beta zero), not a fitted intercept"
    }
  )
  .unwrap();
  writeln!(
    output,
    "{}({}) = {}; p-value = {}",
    if r.likelihood_ratio_p_value.is_some() {
      "LR chi2"
    } else {
      "descriptive LR difference"
    },
    r.likelihood_ratio_df,
    r.likelihood_ratio,
    optional_number(r.likelihood_ratio_p_value)
  )
  .unwrap();
  output.push_str(&binary_wald_note(r.omnibus_wald.as_ref()));
  writeln!(
    output,
    "\nMcFadden pseudo R2 = {}; NB deviance at fitted alpha = {}",
    r.mcfadden_r_squared, r.residual_deviance
  )
  .unwrap();
  writeln!(
    output,
    "Pearson chi-square = {}; residual df (N-mean parameters-1) = {}; Pearson dispersion = {}",
    r.pearson_chi_square, r.df_residual, r.pearson_dispersion
  )
  .unwrap();
  output.push_str("Normal 95% coefficient inference; dispersion diagnostics are descriptive, not panel/specification tests.\n");
  if r.cluster.is_some() {
    output.push_str(&count_cluster_note(r.cluster.as_ref()));
    output.push_str("Group FULL joint beta/lnalpha scores before outer products; selected V includes cross curvature, not conditional/Fisher or Pearson rescaling.\nLR p-value unavailable under clustered inference.\n");
  } else {
    output.push_str("Robust assumes independent rows, NOT repeated-person/panel dependence; it does not rescale V by Pearson dispersion.\n");
    if r.robust {
      output.push_str("LR p-value unavailable under robust inference.\n");
    }
  }
  if let Some(variable) = &r.offset_variable {
    writeln!(output, "Fixed offset: {} (coefficient 1, not estimated); xb = X beta + offset. Predictions use current offset values; effects hold retained offsets fixed.", variable.escape_debug()).unwrap();
  }
  output.push_str(&poisson_estimates(&r.coefficients));
  let d = &r.dispersion;
  writeln!(output,"Dispersion\tEstimate\tStd. error\tCI lower (95%)\tCI upper (95%)\nlnalpha\t{}\t{}\t{}\t{}\nalpha\t{}\t{}\t{}\t{}",d.log_alpha,d.log_alpha_standard_error,optional_number(d.log_alpha_confidence_lower),optional_number(d.log_alpha_confidence_upper),d.alpha,d.alpha_standard_error,optional_number(d.alpha_confidence_lower),optional_number(d.alpha_confidence_upper)).unwrap();
  if let Some(test) = &r.dispersion_test {
    writeln!(
      output,
      "Poisson-boundary LR statistic (H0 alpha=0 vs H1 alpha>0) = {}; p-value = {}; reference = {}",
      test.statistic,
      optional_number(test.p_value),
      test
        .reference_distribution
        .as_deref()
        .unwrap_or("unavailable")
    )
    .unwrap();
    if let Some(reason) = &test.unavailable_reason {
      writeln!(output, "Boundary LR p-value unavailable: {reason}.").unwrap();
    }
  }
  output.push_str("Alpha SE is delta-method; alpha CI exponentiates the log-alpha interval. The Poisson-boundary LR compares NB2 and Poisson likelihoods on the same sample/design/offset; only model-based inference reports the asymptotic 50:50 chi-square(0)/chi-square(1) mixture p-value. No normal-Wald alpha=0 test is implied.\nNumerical bounds: exact loaded counts 0..100000, lnalpha strictly (-12,12); Poisson-limit/boundary or nonconverged fits fail, not clip alpha.\n");
  output
}

fn table(columns: &[String], rows: &[Vec<CellValue>]) -> String {
  let mut output = columns
    .iter()
    .map(|name| name.escape_debug().to_string())
    .collect::<Vec<_>>()
    .join("\t");
  output.push('\n');
  for row in rows {
    writeln!(
      output,
      "{}",
      row.iter().map(cell).collect::<Vec<_>>().join("\t")
    )
    .unwrap();
  }
  writeln!(output, "({} rows shown)", rows.len()).unwrap();
  output
}

fn dataset_outcome(action: &str, dataset: &DatasetInfo) -> String {
  format!(
    "{action}: {} rows, {} columns\n",
    dataset.row_count,
    dataset.columns.len()
  )
}

fn inference_number(value: f64) -> String {
  if value.is_nan() {
    "NA".to_owned()
  } else {
    value.to_string()
  }
}

fn optional_inference(value: Option<f64>) -> String {
  value
    .map(inference_number)
    .unwrap_or_else(|| "NA".to_owned())
}

fn logit_number(value: f64) -> String {
  if value != 0.0 && value.abs() < 1e-6 {
    format!("{value:.6e}")
  } else {
    inference_number(value)
  }
}

fn logit_estimates(estimates: &[tabdat_domain::LogitEstimate]) -> String {
  let mut output = String::from(
    "Variable\tEstimate\tStd. error\tz\tP(two-sided)\tCI lower (95%)\tCI upper (95%)\n",
  );
  for c in estimates {
    writeln!(
      output,
      "{}\t{}\t{}\t{}\t{}\t{}\t{}",
      c.name.escape_debug(),
      logit_number(c.value),
      logit_number(c.standard_error),
      logit_number(c.statistic),
      logit_number(c.p_value),
      logit_number(c.confidence_lower),
      logit_number(c.confidence_upper)
    )
    .unwrap();
  }
  output
}

fn tobit_estimates(estimates: &[tabdat_domain::TobitEstimate]) -> String {
  // Presentation-only conversion: both rows use the same normal-inference layout.
  let rows = estimates
    .iter()
    .map(|c| tabdat_domain::LogitEstimate {
      name: c.name.clone(),
      value: c.value,
      standard_error: c.standard_error,
      statistic: c.statistic,
      p_value: c.p_value,
      confidence_lower: c.confidence_lower,
      confidence_upper: c.confidence_upper,
    })
    .collect::<Vec<_>>();
  logit_estimates(&rows)
}

fn multinomial_estimates(estimates: &[tabdat_domain::MultinomialEstimate]) -> String {
  let mut output = String::from(
    "Category\tVariable\tEstimate\tStd. error\tz\tP(two-sided)\tCI lower (95%)\tCI upper (95%)\n",
  );
  for row in estimates {
    let c = &row.estimate;
    writeln!(
      output,
      "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
      row.category,
      c.name.escape_debug(),
      logit_number(c.value),
      logit_number(c.standard_error),
      logit_number(c.statistic),
      logit_number(c.p_value),
      logit_number(c.confidence_lower),
      logit_number(c.confidence_upper)
    )
    .unwrap();
  }
  output
}

fn random_effects(state: &tabdat_domain::RandomEffectsState) -> String {
  let r = &state.result;
  let fit = &r.gls;
  let clustered = matches!(
    fit.covariance.covariance_type,
    tabdat_domain::CovarianceType::Cluster(_)
  );
  let unbalanced = r.group_size == 0;
  let group_size = if unbalanced {
    let (lo, hi) = r.group_size_range().unwrap_or((0, 0));
    format!("{lo}..{hi}")
  } else {
    r.group_size.to_string()
  };
  let theta_label = if unbalanced {
    "theta@minimum T"
  } else {
    "theta"
  };
  let boundary = r.ml.as_ref().is_some_and(|ml| ml.unit_variance_boundary());
  let covariance = if boundary {
    "Constrained observed-information covariance (unit variance fixed at zero); normal coefficient inference"
  } else if r.ml.is_some() && clustered {
    "Full joint ML unit-score sandwich with observed-information bread; normal inference"
  } else if r.ml.is_some() {
    "Joint observed-information covariance; normal inference"
  } else if clustered {
    "Unit-cluster covariance: grouped transformed scores; normal inference"
  } else {
    "Model-based covariance: transformed RSS/(N-K); normal inference"
  };
  let mut output = format!(
    "{} one-way random-effects {}: {}\n{}; entity = {}; time = {}; N = {}; groups = {}; T = {}\n{covariance}\nsigma_e squared = {}; sigma_u squared = {}; raw sigma_u squared = {}; {theta_label} = {}; rho = {}\nSlope Wald chi-square({}) = {}; p = {}\n",
    if unbalanced { "Unbalanced" } else { "Balanced" },
    if r.ml.is_some() { "ML" } else { "GLS" },
    r.outcome.escape_debug(),
    if boundary {
      "Gaussian marginal maximum likelihood (zero unit variance boundary)"
    } else if r.ml.is_some() {
      "Gaussian marginal maximum likelihood (interior unit variance)"
    } else {
      "Swamy-Arora"
    },
    r.entity_variable.escape_debug(),
    state.time_variable.escape_debug(),
    fit.fit_statistics.observation_count,
    r.group_count,
    group_size,
    inference_number(r.sigma_e_squared),
    inference_number(r.sigma_u_squared),
    inference_number(r.raw_sigma_u_squared),
    inference_number(r.theta),
    inference_number(r.rho),
    r.predictor_names.len(),
    optional_inference(r.wald_statistic),
    optional_inference(r.wald_p_value)
  );
  if unbalanced {
    let (lo, hi) = r.theta_range().unwrap_or((f64::NAN, f64::NAN));
    if r.ml.is_some() {
      writeln!(output, "Unequal retained counts: theta range = {}..{}; eta uses minimum T. Interior information parameters are beta, log(sigma_e squared), log(sigma_u squared).",inference_number(lo),inference_number(hi)).unwrap();
    } else {
      writeln!(output, "Unequal retained counts: unit-specific Swamy-Arora theta ranges from {} to {}; singleton units are allowed when identified.",inference_number(lo),inference_number(hi)).unwrap();
    }
  }
  if let Some(ml) = &r.ml {
    if ml.cluster.is_some() {
      writeln!(output, "Working marginal log likelihood = {}; intercept-only null = {}; descriptive slope LR = {}; reference withheld",
        inference_number(ml.log_likelihood), inference_number(ml.null_log_likelihood), inference_number(ml.likelihood_ratio)).unwrap();
    } else {
      writeln!(output, "Marginal log likelihood = {}; intercept-only null = {}; slope LR chi-square({}) = {}; p = {}",
        inference_number(ml.log_likelihood), inference_number(ml.null_log_likelihood),
        ml.likelihood_ratio_df, inference_number(ml.likelihood_ratio), optional_inference(ml.likelihood_ratio_p_value)).unwrap();
    }
    if let Some(unit_variance_test) = &ml.unit_variance_test {
      writeln!(
        output,
        "Unit-variance LR test: H0 sigma_u squared = 0; LR = {}; p = {} (asymptotic 50:50 chi-square(0,1) mixture)",
        inference_number(unit_variance_test.statistic),
        inference_number(unit_variance_test.p_value),
      )
      .unwrap();
    }
    if let Some(reason) = &ml.unit_variance_test_unavailable_reason {
      writeln!(
        output,
        "Unit-variance LR reference unavailable: {}",
        reason.escape_debug()
      )
      .unwrap();
    }
    match r.variance_component_inference() {
      Ok(Some(inference)) => {
        output.push_str(if ml.cluster.is_some() {
          "Working variance-component inference: full joint unit-score sandwich; delta-method SEs; exponentiated log-Wald 95% intervals\n"
        } else {
          "Variance-component inference: joint observed information; delta-method SEs; exponentiated log-Wald 95% intervals\n"
        });
        let residual = inference.sigma_e_squared;
        writeln!(
          output,
          "sigma_e squared SE = {}; 95% CI = [{}, {}]",
          inference_number(residual.standard_error),
          inference_number(residual.confidence_interval[0]),
          inference_number(residual.confidence_interval[1]),
        )
        .unwrap();
        if let Some(unit) = inference.sigma_u_squared {
          writeln!(
            output,
            "sigma_u squared SE = {}; 95% CI = [{}, {}]",
            inference_number(unit.standard_error),
            inference_number(unit.confidence_interval[0]),
            inference_number(unit.confidence_interval[1]),
          )
          .unwrap();
        } else if let Some(reason) = inference.sigma_u_squared_unavailable_reason {
          writeln!(
            output,
            "sigma_u squared uncertainty unavailable: {}",
            reason.escape_debug()
          )
          .unwrap();
        }
      }
      Ok(None) => {}
      Err(error) => {
        writeln!(
          output,
          "Variance-component inference unavailable: {}",
          error.to_string().escape_debug()
        )
        .unwrap();
      }
    }
    if let Some(reason) = &ml.likelihood_ratio_unavailable_reason {
      writeln!(
        output,
        "Slope LR reference unavailable: {}",
        reason.escape_debug()
      )
      .unwrap();
    }
    if boundary {
      output.push_str("Boundary selected: theta = rho = 0; information parameters are beta and log(sigma_e squared). No regular normal-Wald unit-variance SE.\n");
    }
    output.push_str("Profile eta search [0,24], grid 0.5 and polished stationary maxima; no global-optimality guarantee for arbitrary designs.\n");
  }
  if clustered {
    let n = fit.fit_statistics.observation_count;
    let g = r.group_count;
    let k = fit.parameter_names.len();
    if let Some(cluster) = r.ml.as_ref().and_then(|ml| ml.cluster.as_ref()) {
      writeln!(output, "Unit-score multiplier G/(G-1) = {}; full joint beta/log-variance scores, independent units, many-group asymptotics; no few-cluster guarantee",
        inference_number(cluster.finite_sample_multiplier)).unwrap();
    } else {
      writeln!(
        output,
        "Unit-cluster multiplier G/(G-1)*(N-1)/(N-K) = {} (plm Arellano sss; independent units)",
        inference_number(g as f64 / (g - 1) as f64 * (n - 1) as f64 / (n - k) as f64)
      )
      .unwrap();
    }
  }
  if let Some(reason) = &r.wald_unavailable_reason {
    writeln!(output, "Slope Wald unavailable: {}", reason.escape_debug()).unwrap();
  }
  if let Some(s) = &fit.sample {
    writeln!(
      output,
      "Candidate rows = {}; excluded = {}",
      s.total_observations, s.dropped_observations
    )
    .unwrap();
  }
  output.push_str(
    "Parameter\tCoefficient\tStd. error\tz\tP(two-sided)\tCI lower (95%)\tCI upper (95%)\n",
  );
  for (c, ci) in fit.coefficients.iter().zip(&r.confidence_intervals) {
    writeln!(
      output,
      "{}\t{}\t{}\t{}\t{}\t{}\t{}",
      c.name.escape_debug(),
      inference_number(c.value),
      optional_inference(c.standard_error),
      optional_inference(c.statistic),
      optional_inference(c.p_value),
      inference_number(ci[0]),
      inference_number(ci[1])
    )
    .unwrap();
  }
  output.push_str("Assumes exogenous unit effects and independent units. Predictions are population xb/residuals. No within/between/overall R-squared.\n");
  if r.ml.is_none() {
    output.push_str("No panel likelihood.\n");
  }
  output
}

fn fixed_effects(state: &tabdat_domain::FixedEffectsState) -> String {
  let result = &state.result;
  let fit = &result.within;
  let stats = &fit.fit_statistics;
  let g = result.effects.len();
  let p = fit.parameter_names.len();
  let n = stats.observation_count;
  let clustered = matches!(
    fit.covariance.covariance_type,
    tabdat_domain::CovarianceType::Cluster(_)
  );
  let inference_df = if clustered {
    g.saturating_sub(1)
  } else {
    stats.degrees_of_freedom
  };
  let covariance = if clustered {
    format!("cluster({})", result.entity_variable.escape_debug())
  } else {
    "nonrobust".into()
  };
  let mut output = format!(
    "Within fixed-effects regression: {} ~ {}\nEntity = {}; time = {}; N = {n}; groups = {g}; slopes = {p}\nAbsorbed unit intercepts = {g}; residual DF (N-G-p) = {}\nCovariance = {covariance}; slope Student-t / test F denominator DF = {inference_df}\n",
    result.outcome.escape_debug(),
    fit
      .parameter_names
      .iter()
      .map(|name| name.escape_debug().to_string())
      .collect::<Vec<_>>()
      .join(" "),
    result.entity_variable.escape_debug(),
    state.time_variable.escape_debug(),
    stats.degrees_of_freedom
  );
  if clustered {
    writeln!(
      output,
      "Unit-cluster multiplier G/(G-1)*(N-1)/(N-p) = {} (plm Arellano sss; independent units)",
      inference_number(g as f64 / g.saturating_sub(1) as f64 * (n - 1) as f64 / (n - p) as f64)
    )
    .unwrap();
  }
  if let Some(sample) = &fit.sample {
    writeln!(
      output,
      "Candidate rows = {}; excluded = {}",
      sample.total_observations, sample.dropped_observations
    )
    .unwrap();
  }
  writeln!(
    output,
    "Group size min = {}; max = {}; singleton groups = {}",
    result.minimum_group_size, result.maximum_group_size, result.singleton_groups
  )
  .unwrap();
  writeln!(
    output,
    "Within R-squared = {}; conditional level R-squared = {}; root MSE = {}",
    optional_inference(stats.r_squared),
    optional_inference(result.conditional_r_squared),
    optional_inference(stats.root_mse)
  )
  .unwrap();
  if let Some(test) = &result.unit_effects_f_test {
    writeln!(
      output,
      "F test all unit effects = 0 (classical pooled-vs-within): F({}, {}) = {}; p = {}",
      test.df_num,
      test.df_denom,
      inference_number(test.f_statistic),
      inference_number(test.p_value)
    )
    .unwrap();
  } else if let Some(reason) = &result.unit_effects_f_test_unavailable_reason {
    writeln!(
      output,
      "Unit-effects F test unavailable: {}",
      reason.escape_debug()
    )
    .unwrap();
  }
  writeln!(output,"Common intercept (normalization point only) = {}; unit deviations have observation-weighted mean zero",inference_number(result.common_intercept)).unwrap();
  output.push_str("Slope\tCoefficient\tStd. error\tStatistic\tP(two-sided)\n");
  for c in &fit.coefficients {
    writeln!(
      output,
      "{}\t{}\t{}\t{}\t{}",
      c.name.escape_debug(),
      inference_number(c.value),
      optional_inference(c.standard_error),
      optional_inference(c.statistic),
      optional_inference(c.p_value)
    )
    .unwrap();
  }
  output.push_str(
    "No standard errors or intervals for the common normalization point or individual unit-intercept coefficients; no panel likelihood. NA = unavailable or undefined.\n",
  );
  output
}

fn regression(result: &tabdat_domain::RegressionResult) -> String {
  let mut output = format!(
    "Regression: {} on {}\nEstimator: {}; covariance: {}; intercept: {}\nN = {}; residual df = {}\n",
    result.outcome.escape_debug(),
    result
      .predictors
      .iter()
      .map(|name| name.escape_debug().to_string())
      .collect::<Vec<_>>()
      .join(" "),
    result.estimator.escape_debug(),
    result.covariance.escape_debug(),
    result.include_intercept,
    result.observation_count,
    result.least_squares.fit_statistics.degrees_of_freedom
  );
  if let Some(sample) = &result.least_squares.sample {
    writeln!(
      output,
      "Candidate rows = {}; excluded = {}",
      sample.total_observations, sample.dropped_observations
    )
    .unwrap();
  }
  writeln!(
    output,
    "R-squared = {}; adjusted R-squared = {}; root MSE = {}",
    optional_inference(result.r_squared),
    optional_inference(result.adjusted_r_squared),
    optional_inference(result.root_mse)
  )
  .unwrap();
  if let Some(log_likelihood) = result.gaussian_working_log_likelihood() {
    writeln!(output,
      "Gaussian working-model log likelihood = {} (ML variance RSS/N; not Bernoulli likelihood or robust/cross-family LR inference)",
      inference_number(log_likelihood)
    ).unwrap();
  }
  output.push_str("Parameter\tCoefficient\tStd. error\tStatistic\tP(two-sided)\n");
  for coefficient in &result.coefficients {
    writeln!(
      output,
      "{}\t{}\t{}\t{}\t{}",
      coefficient.name.escape_debug(),
      inference_number(coefficient.value),
      optional_inference(coefficient.standard_error),
      optional_inference(coefficient.statistic),
      optional_inference(coefficient.p_value)
    )
    .unwrap();
  }
  if let Some(first_stage) = &result.first_stage {
    output.push_str(
      "Warning: coefficient p-values use conventional t inference, not weak-IV-robust inference. A separate joint test is available via estat ar (classical F or robust/cluster asymptotic chi-square).\n",
    );
    output.push_str(
      "First stage: conventional excluded-instrument partial F tests (relevance diagnostics only)\n",
    );
    output.push_str("Endogenous\tF(q, df)\tPartial R-squared\tShea partial R-squared\tP-value\n");
    for diagnostic in first_stage {
      writeln!(
        output,
        "{}\tF({}, {}) = {}\t{}\t{}\t{}",
        diagnostic.endogenous.escape_debug(),
        diagnostic.numerator_df,
        diagnostic.denominator_df,
        diagnostic
          .f_statistic
          .map(inference_number)
          .unwrap_or_else(|| "NA".to_owned()),
        optional_inference(diagnostic.partial_r_squared),
        optional_inference(diagnostic.shea_partial_r_squared),
        optional_inference(diagnostic.p_value)
      )
      .unwrap();
    }
  }
  output.push_str("NA = unavailable or undefined.\n");
  output
}

fn ttest_sample_row(output: &mut String, sample: &tabdat_domain::TtestGroupStats, n: &str) {
  writeln!(
    output,
    "{}\t{n}\t{}\t{}\t{}\t{}\t{}",
    sample.name.escape_debug(),
    inference_number(sample.mean),
    inference_number(sample.standard_error),
    inference_number(sample.standard_deviation),
    inference_number(sample.ci_lower),
    inference_number(sample.ci_upper)
  )
  .unwrap();
}

fn ttest(result: &tabdat_domain::TtestResult) -> String {
  let independent = result.by_variable.is_some();
  let mode = if result.is_paired {
    "paired"
  } else if independent {
    if result.is_welch {
      "Welch unequal-variance"
    } else {
      "pooled equal-variance"
    }
  } else {
    "one-sample"
  };
  let mut output = format!(
    "T test: {mode}\nVariable: {}",
    result.varname1.escape_debug()
  );
  if let Some(variable) = &result.varname2 {
    write!(output, "; paired variable: {}", variable.escape_debug()).unwrap();
  }
  if let Some(variable) = &result.by_variable {
    write!(output, "; by: {}", variable.escape_debug()).unwrap();
  }
  output.push('\n');
  output.push_str("Sample\tN\tMean\tStd. error\tStd. dev.\tCI lower (95%)\tCI upper (95%)\n");
  for sample in std::iter::once(&result.group1_stats)
    .chain(result.group2_stats.iter())
    .chain(result.combined_stats.iter())
  {
    ttest_sample_row(&mut output, sample, &sample.observations.to_string());
  }
  // Independent difference counts are legacy residual-df/zero values, not N.
  let difference_n = if independent {
    "NA".to_owned()
  } else {
    result.difference_stats.observations.to_string()
  };
  ttest_sample_row(&mut output, &result.difference_stats, &difference_n);
  if independent {
    writeln!(
      output,
      "Contrast: {} - {}",
      result.group1_stats.name.escape_debug(),
      result
        .group2_stats
        .as_ref()
        .map(|group| group.name.escape_debug().to_string())
        .unwrap_or_else(|| "NA".to_owned())
    )
    .unwrap();
    output.push_str("Difference N is not an observation count; use group N and reference df.\n");
  } else if result.is_paired {
    writeln!(
      output,
      "Contrast: {} - {}",
      result.varname1.escape_debug(),
      result.varname2.as_deref().unwrap_or("NA").escape_debug()
    )
    .unwrap();
  }
  writeln!(
    output,
    "Null {} = {}",
    if independent || result.is_paired {
      "difference"
    } else {
      "mean"
    },
    inference_number(result.null_value)
  )
  .unwrap();
  writeln!(
    output,
    "t reference; df = {}; t = {}",
    inference_number(result.degrees_of_freedom),
    inference_number(result.t_statistic)
  )
  .unwrap();
  writeln!(
    output,
    "P(left-tail) = {}; P(two-sided) = {}; P(right-tail) = {}",
    inference_number(result.p_left),
    inference_number(result.p_two),
    inference_number(result.p_right)
  )
  .unwrap();
  output.push_str("NA = unavailable, undefined, or not an observation count.\n");
  output
}

fn label_key(key: &tabdat_domain::LabelValue) -> String {
  match key {
    tabdat_domain::LabelValue::Integer(value) => format!("integer {value}"),
    tabdat_domain::LabelValue::Number(value) => format!("number {value:?}"),
    tabdat_domain::LabelValue::Text(value) => format!("text {value:?}"),
  }
}

fn label(result: &tabdat_domain::LabelResult) -> String {
  use tabdat_domain::LabelResultAction;
  let action = match result.action {
    LabelResultAction::Variable => "variable",
    LabelResultAction::Define => "define",
    LabelResultAction::Values => "values",
    LabelResultAction::List => "list",
    LabelResultAction::Drop => "drop",
  };
  let mut output = format!("Label {action}: {}\n", result.message.escape_debug());
  let Some(metadata) = &result.metadata else {
    output.push_str("No label metadata.\n");
    return output;
  };
  output.push_str("Variable labels:\n");
  if metadata.variable_labels.is_empty() {
    output.push_str("  (none)\n");
  }
  for (variable, text) in &metadata.variable_labels {
    writeln!(output, "  {variable:?} -> {text:?}").unwrap();
  }
  output.push_str("Attachments:\n");
  if metadata.attachments.is_empty() {
    output.push_str("  (none)\n");
  }
  for (variable, set) in &metadata.attachments {
    writeln!(output, "  {variable:?} -> {set:?}").unwrap();
  }
  output.push_str("Value-label sets:\n");
  if metadata.value_sets.is_empty() {
    output.push_str("  (none)\n");
  }
  for set in &metadata.value_sets {
    writeln!(output, "  {:?}:", set.name).unwrap();
    if set.mappings.is_empty() {
      output.push_str("    (none)\n");
    }
    for (key, text) in &set.mappings {
      writeln!(output, "    {} -> {text:?}", label_key(key)).unwrap();
    }
  }
  output
}

/// Present core workflow guidance or a packaged reference topic without execution.
pub fn render_interactive_help(topic: Option<&str>) -> Result<String, String> {
  match topic {
    None => Ok(concat!(
      "Interactive core workflow\n",
      "  Load local eager CSV/Parquet: use <path>\n",
      "  Inspect: status, describe, codebook [variables], summarize [variables], missing [variables], tabulate, count, head/tail\n",
      "  Transform: generate, replace, rename, sort/gsort, keep/drop/select, recode, encode/decode, label\n",
      "  Cohorts/layouts: sql ... into <name>, use <name>, join, append, reshape, collapse\n",
      "  Grouped EDA: by <keys>: count; by <keys>: summarize uses native means\n",
      "  Linear analysis: regress, predict, lincom, test\n",
      "  Mean comparisons: ttest x == 0; ttest x == y; ttest x, by(group) [welch]\n",
      "  Scripts/files: run <script.td>, save <file.parquet>, export <file.csv>\n",
      "  Help: help [topic] or ?[topic]; example: help summarize\n",
      "  Leave: exit, quit, or EOF\n",
      "Supported forms/options are bounded; reference topics may include parser-only or future syntax.\n",
      "See SPEC.md for current limits. In capable terminals, Tab completes core commands/help topics, local use/run paths, bare use/append/join registered names, and basic bare/backtick EDA variable lists. Open a backtick to complete headers with spaces; embedded-backtick/control names are excluded.\n",
      "Path/variable prefixes are literal and case-sensitive; open a compatible quote for spaced paths. Piped input is not completed.\n",
      "Ctrl-C cancels pending input; executing-command cancellation is not supported.\n"
    ).to_owned()),
    Some(topic) => crate::catalog::help_topic_result(topic).map(|topic| format!(
      "Reference topic: supported forms/options are bounded; some syntax is parser-only or future. See SPEC.md.\n\n{}",
      topic.text)),
  }
}

fn render_summary_rows(rows: &[SummaryRow]) -> String {
  let mut output = String::from("Variable\tN\tMean\tStd. dev.\tMin\tMax\n");
  for row in rows {
    writeln!(
      output,
      "{}\t{}\t{}\t{}\t{}\t{}",
      row.variable.escape_debug(),
      row.count,
      optional_number(row.mean),
      optional_number(row.std_dev),
      row
        .minimum
        .as_ref()
        .map(cell)
        .unwrap_or_else(|| "NA".to_owned()),
      row
        .maximum
        .as_ref()
        .map(cell)
        .unwrap_or_else(|| "NA".to_owned()),
    )
    .unwrap();
  }
  output
}

fn plot_artifact(result: &tabdat_domain::PlotArtifactResult) -> String {
  let mut output = format!(
    "SVG scatter saved to {} ({} x {})\nCandidate rows = {}; scatter {} versus {}: finite points = {}; omitted missing/nonfinite = {}\n",
    result.path.to_string_lossy().escape_debug(),
    result.width,
    result.height,
    result.candidate_rows,
    result.scatter.variable.escape_debug(),
    result.x_variable.escape_debug(),
    result.scatter.point_count,
    result.scatter.omitted_rows,
  );
  for line in &result.lines {
    writeln!(
      output,
      "Overlaid line {}: finite vertices = {}; omitted missing/nonfinite = {}",
      line.variable.escape_debug(),
      line.point_count,
      line.omitted_rows
    )
    .unwrap();
  }
  output.push_str("Line order: ascending x, then original candidate-row index; missing line values break paths.\nCoordinates: float64 conversions, not lossless wide-integer/decimal identities; no [0,1] clipping.\n");
  writeln!(output,"Display-only y jitter amplitude = {} outcome units; deterministic row-index pattern, not RNG state.",result.jitter_y).unwrap();
  output
}

fn bootstrap_report(result: &tabdat_domain::BootstrapOlsResult, whole_cluster: bool) -> String {
  let kind = if whole_cluster {
    "Whole-cluster"
  } else {
    "IID"
  };
  let mut text = format!(
    "{kind} pairs-bootstrap OLS: {}\nN = {}; replications = {}; seed = {}; RNG = {}\n95% percentile intervals (R type7); no analytic t/normal p-values\n",
    result.outcome.escape_debug(),
    result.observation_count,
    result.replications,
    result.seed,
    result.rng
  );
  for b in &result.coefficients {
    writeln!(
      text,
      "{}: estimate = {}; bootstrap mean = {}; bias = {}; SE = {}; 95% percentile CI = [{}, {}]",
      b.name.escape_debug(),
      inference_number(b.estimate),
      inference_number(b.bootstrap_mean),
      inference_number(b.bias),
      inference_number(b.standard_error),
      inference_number(b.ci_lower),
      inference_number(b.ci_upper)
    )
    .unwrap();
  }
  text.push_str(if whole_cluster {
    "Resamples whole clusters with replacement; retains within-cluster rows. Not residual, wild or automatic FE/RE bootstrap. "
  } else {
    "Resamples complete rows independently; not a cluster or residual bootstrap. "
  });
  text.push_str("Data and the retained fit are unchanged.\n");
  text
}

/// Render owned execution results without recomputing data or statistics.
/// Exhaustive matching makes a new result variant require an explicit display decision.
pub fn render_execution_result(result: &ExecutionResult) -> String {
  match result {
    ExecutionResult::EstimateStore(result) => format!(
      "Stored fit report {} ({}) — {} reports; active fit unchanged.\n",
      result.name.escape_debug(),
      result.estimator.escape_debug(),
      result.stored_count
    ),
    ExecutionResult::EstimateDrop(result) => format!(
      "Dropped fit reports {}; {} remain; active fit unchanged.\n",
      result.names.join(", ").escape_debug(),
      result.stored_count
    ),
    ExecutionResult::EstimateTable(result) => crate::estimate_table::render_estimate_table(result),
    ExecutionResult::Quantile(result) => {
      let mut text = format!(
        "Conditional quantile regression: {} at tau={}\nN = {}; check loss = {}; pseudo R2 = {}\nIID location errors assumed; Hall-Sheather residual-spacing covariance, asymptotic normal 95% intervals.\n",
        result.outcome.escape_debug(),
        result.quantile,
        result.observation_count,
        inference_number(result.check_loss),
        result
          .pseudo_r_squared
          .map(inference_number)
          .unwrap_or_else(|| "unavailable".into())
      );
      for b in &result.coefficients {
        writeln!(
          text,
          "{}: estimate = {}; SE = {}; z = {}; p = {}; 95% CI = [{}, {}]",
          b.name.escape_debug(),
          inference_number(b.value),
          inference_number(b.standard_error),
          inference_number(b.statistic),
          inference_number(b.p_value),
          inference_number(b.ci_lower),
          inference_number(b.ci_upper)
        )
        .unwrap();
      }
      writeln!(text,"Sparsity = {}; local residuals = {}; bandwidth probability = {}. No heteroskedastic/cluster/survey or cross-quantile inference.",inference_number(result.sparsity),result.sparsity_observation_count,inference_number(result.bandwidth_probability)).unwrap();
      text
    }
    ExecutionResult::QuantileBootstrap(result) => {
      let mut text = format!(
        "Joint IID pairs-bootstrap quantiles: {}\nN = {}; quantiles = {:?}; replications = {}; seed = {}; RNG = {}\nFull joint covariance uses common row draws and B-1 denominator.\n95% marginal percentile intervals (R type7); no simultaneous coverage or marginal coefficient p-values\n",
        result.outcome,
        result.observation_count,
        result.quantiles,
        result.replications,
        result.seed,
        result.rng
      );
      for b in &result.coefficients {
        let _ = writeln!(
          text,
          "{}: estimate = {}; bootstrap mean = {}; bias = {}; SE = {}; 95% percentile CI = [{}, {}]",
          b.name,
          inference_number(b.estimate),
          inference_number(b.bootstrap_mean),
          inference_number(b.bias),
          inference_number(b.standard_error),
          inference_number(b.ci_lower),
          inference_number(b.ci_upper)
        );
      }
      if let Some(c) = &result.contrast {
        let _ = writeln!(
          text,
          "Contrast {}: q{} minus q{}; difference = {}; bootstrap SE = {}",
          c.parameter,
          c.upper_quantile,
          c.lower_quantile,
          inference_number(c.estimate),
          c.standard_error
            .map_or_else(|| "unavailable".into(), inference_number)
        );
        if let Some(reason) = &c.inference_unavailable_reason {
          let _ = writeln!(text, "Contrast normal inference unavailable: {reason}");
        } else {
          let _ = writeln!(
            text,
            "H0: difference = 0; asymptotic normal z = {}; two-sided p = {}; 95% normal CI = [{}, {}]. B-1 is the bootstrap covariance divisor, not Student-t degrees of freedom.",
            inference_number(c.statistic.unwrap()),
            inference_number(c.p_value.unwrap()),
            inference_number(c.ci_lower.unwrap()),
            inference_number(c.ci_upper.unwrap())
          );
        }
      }
      text.push_str("IID complete-row pairs; no cluster/survey inference. Failed draws are not discarded or retried. Data and retained fit unchanged.");
      text
    }
    ExecutionResult::Bootstrap(result) => bootstrap_report(result, false),
    ExecutionResult::ClusterBootstrap(result) => {
      let text = bootstrap_report(&result.bootstrap, true);
      format!(
        "{text}Clusters = {}; first-seen sample order; replicate N range = {}..{}; few-cluster/nominal-coverage guarantees are unavailable.\n",
        result.cluster_count,
        result.replicate_observation_counts.iter().min().unwrap(),
        result.replicate_observation_counts.iter().max().unwrap()
      )
    }
    ExecutionResult::RobustEndogeneity(result) => format!(
      "Robust joint endogeneity test (augmented-regression Wald)\nTested endogenous regressors: {:?}\nH0: all declared endogenous regressors are exogenous, given valid instruments and the maintained model.\nN = {}; asymptotic chi-square({}) = {}; p-value = {}\nCovariance = {:?}; auxiliary K = {}; residual DF = {}; correction = {}; clusters = {:?}\nConventional strong-identification reference; independent rows or independent growing clusters required, no few-cluster guarantee.\nFailure to reject does not prove exogeneity or instrument validity; not weak-IV-robust coefficient inference.\n",
      result.endogenous,
      result.observation_count,
      result.degrees_of_freedom,
      inference_number(result.statistic),
      inference_number(result.p_value),
      result.covariance_type,
      result.auxiliary_parameter_count,
      result.auxiliary_residual_df,
      inference_number(result.covariance_correction),
      result.cluster_group_count,
    ),
    ExecutionResult::WuHausman(result) => format!(
      "Wu-Hausman joint endogeneity test (homoskedastic augmented-regression F)\nTested endogenous regressors: {:?}\nH0: all declared endogenous regressors are exogenous, given valid instruments and the maintained model.\nN = {}; F({}, {}) = {}; p-value = {}\nConventional strong-identification F reference; exact finite-sample F additionally requires conditional normality.\nFailure to reject does not prove exogeneity or instrument validity; not weak-IV-robust coefficient inference.\n",
      result.endogenous,
      result.observation_count,
      result.numerator_df,
      result.denominator_df,
      inference_number(result.statistic),
      inference_number(result.p_value),
    ),
    ExecutionResult::RobustOveridScore(result) => format!(
      "Heteroskedasticity-robust overidentification score test (Wooldridge residualized instruments)\nN = {}; asymptotic chi-square({}) = {}; p-value = {}\nFull-instrument rank L = {}; structural rank K = {}; DF = L-K; score covariance correction = {} (no N-K/HC1 adjustment).\nH0: overidentifying restrictions hold, given the maintained model. Independent rows and conventional strong identification required.\nNot Hansen J, clustered inference or weak-IV-robust inference. Failure to reject does not prove instrument validity.\n",
      result.observation_count,
      result.degrees_of_freedom,
      inference_number(result.statistic),
      inference_number(result.p_value),
      result.instrument_parameter_count,
      result.structural_parameter_count,
      inference_number(result.covariance_correction),
    ),
    ExecutionResult::Sargan(result) => format!(
      "Sargan overidentification test (homoskedastic N*R²)\nN = {}; chi-square({}) = {}; p-value = {}\nH0: overidentifying restrictions hold, given the maintained model.\nAsymptotic chi-square under homoskedasticity and strong identification; not robust Hansen J or a finite-sample F test.\nFailure to reject does not prove instrument validity or resolve weak identification.\n",
      result.observation_count,
      result.degrees_of_freedom,
      inference_number(result.statistic),
      inference_number(result.p_value),
    ),
    ExecutionResult::AndersonRubinConfidenceSet(result) => {
      let mut output = format!(
        "Anderson-Rubin confidence set (homoskedastic, weak-IV-robust)\n{}; level = {}%; N = {}; critical F({}, {}) = {}\n",
        result.endogenous.escape_debug(),
        inference_number(result.confidence_level),
        result.observation_count,
        result.numerator_df,
        result.denominator_df,
        inference_number(result.critical_f)
      );
      if result.intervals.is_empty() {
        output.push_str("Empty set\n");
      }
      for (index, interval) in result.intervals.iter().enumerate() {
        if index > 0 {
          output.push_str(" union ");
        }
        write!(
          output,
          "{}{}, {}{}",
          if interval.lower.is_some() { "[" } else { "(" },
          interval
            .lower
            .map(inference_number)
            .unwrap_or_else(|| "-inf".into()),
          interval
            .upper
            .map(inference_number)
            .unwrap_or_else(|| "+inf".into()),
          if interval.upper.is_some() { "]" } else { ")" }
        )
        .unwrap();
      }
      output.push_str("\nAssumes valid instruments and homoskedastic errors; exact F requires conditional normality.\nIncluded exogenous coefficients are unrestricted nuisance parameters. Exactly one endogenous coefficient; no endogenous nuisance/subvector inference.\n");
      output
    }
    ExecutionResult::ProjectedAndersonRubinConfidenceSet(result) => {
      let mut output = format!(
        "Projected Anderson-Rubin confidence set (homoskedastic, weak-IV-robust, at least nominal coverage)\n{}; level = {}%; N = {}; critical F({}, {}) = {}\nNuisance endogenous coefficients left unrestricted: {:?}\n",
        result.coefficient.escape_debug(),
        inference_number(result.confidence_level),
        result.observation_count,
        result.numerator_df,
        result.denominator_df,
        inference_number(result.critical_f),
        result.nuisance_endogenous
      );
      if result.intervals.is_empty() {
        output.push_str("Empty set\n");
      }
      for (index, interval) in result.intervals.iter().enumerate() {
        if index > 0 {
          output.push_str(" union ");
        }
        write!(
          output,
          "{}{}, {}{}",
          if interval.lower.is_some() { "[" } else { "(" },
          interval
            .lower
            .map(inference_number)
            .unwrap_or_else(|| "-inf".into()),
          interval
            .upper
            .map(inference_number)
            .unwrap_or_else(|| "+inf".into()),
          if interval.upper.is_some() { "]" } else { ")" }
        )
        .unwrap();
      }
      output.push_str("\nAssumes valid instruments and homoskedastic errors; exact F requires conditional normality. The scalar set projects the joint region over every other endogenous coefficient and therefore has at-least-nominal coverage.\n");
      if result.all_real_conservative_fallback {
        output.push_str("The nuisance profile was not safely positive definite, so the conservative all-real set is reported.\n");
      }
      output
    }
    ExecutionResult::RobustAndersonRubin(result) => {
      let mut output = String::from(
        "Anderson-Rubin joint Wald test (HC1/one-way cluster, weak-IV-robust)\nH0: all endogenous coefficients jointly, in fit order\n",
      );
      for (name, value) in result.endogenous.iter().zip(&result.null_values) {
        writeln!(
          output,
          "{} = {}",
          name.escape_debug(),
          inference_number(*value)
        )
        .unwrap();
      }
      writeln!(
        output,
        "N = {}; asymptotic chi-square({}) = {}; p-value = {}",
        result.observation_count,
        result.degrees_of_freedom,
        inference_number(result.statistic),
        inference_number(result.p_value)
      )
      .unwrap();
      writeln!(
        output,
        "Auxiliary covariance: {:?}; K = {}; residual DF = {}; correction = {}; clusters = {:?}",
        result.covariance_type,
        result.auxiliary_parameter_count,
        result.auxiliary_residual_df,
        inference_number(result.covariance_correction),
        result.cluster_group_count
      )
      .unwrap();
      output.push_str("Assumes valid instruments and a correctly specified linear model; independent rows for HC1 or independent clusters with many groups. No finite-sample or few-cluster guarantee.\nIncluded exogenous coefficients are unrestricted nuisance parameters; no endogenous-subvector inference, confidence set or instrument-validity test.\n");
      output
    }
    ExecutionResult::AndersonRubin(result) => {
      let mut output = String::from("Anderson-Rubin joint test (homoskedastic, weak-IV-robust)\n");
      output.push_str("H0: all endogenous coefficients jointly, in fit order\n");
      for (name, value) in result.endogenous.iter().zip(&result.null_values) {
        writeln!(
          output,
          "{} = {}",
          name.escape_debug(),
          inference_number(*value)
        )
        .unwrap();
      }
      writeln!(
        output,
        "N = {}; F({}, {}) = {}; p-value = {}",
        result.observation_count,
        result.numerator_df,
        result.denominator_df,
        inference_number(result.statistic),
        inference_number(result.p_value)
      )
      .unwrap();
      output.push_str("Assumes valid instruments and homoskedastic errors; exact F requires conditional normality.\nIncluded exogenous coefficients are unrestricted nuisance parameters. No confidence set or subvector inference is included in this joint-test result.\n");
      output
    }
    ExecutionResult::Status(status) => {
      let mut output = format!(
        "Session status\nData engine initialized: {}\n",
        status.data_initialized
      );
      if let Some(dataset) = &status.dataset {
        writeln!(
          output,
          "Active source: {:?}\nActive dimensions: {} rows, {} columns",
          dataset.source.to_string_lossy(),
          dataset.row_count,
          dataset.columns.len()
        )
        .unwrap();
        output.push_str("Current columns:\n");
        for column in &dataset.columns {
          writeln!(output, "  {:?}: {:?}", column.name, column.data_type).unwrap();
        }
      } else {
        output.push_str("Active dataset: none\n");
      }
      writeln!(
        output,
        "Active named table: {}\nRegistered named tables: {:?}",
        status
          .active_table_name
          .as_ref()
          .map_or_else(|| "none".to_owned(), |name| format!("{name:?}")),
        status.named_tables
      )
      .unwrap();
      writeln!(
        output,
        "Retained estimation family: {}",
        status
          .estimation_family
          .as_ref()
          .map_or_else(|| "none".to_owned(), |family| format!("{family:?}"))
      )
      .unwrap();
      if let Some(model) = &status.linear_model {
        writeln!(output, "Retained linear model: outcome={:?}; predictors={:?}; intercept={}\nRetained N = {}; residual df = {}\nOriginal candidate rows = {}; excluded = {}",
          model.outcome, model.predictors, model.include_intercept, model.observation_count,
          model.degrees_of_freedom,
          model.sample_total_observations.map_or_else(|| "NA".to_owned(), |value| value.to_string()),
          model.sample_dropped_observations.map_or_else(|| "NA".to_owned(), |value| value.to_string())).unwrap();
        output.push_str(
          "Retained fit describes its original sample, not current-data prediction eligibility.\n",
        );
      } else {
        output.push_str("Retained linear model: none\n");
      }
      output
    }
    ExecutionResult::Load(load) => format!(
      "Loaded {} rows, {} columns from {}\n",
      load.dataset.row_count,
      load.dataset.columns.len(),
      load.dataset.source.to_string_lossy().escape_debug()
    ),
    ExecutionResult::Describe(describe) => {
      let mut output = format!(
        "{} rows, {} columns\nVariable\tType\n",
        describe.dataset.row_count,
        describe.dataset.columns.len()
      );
      for column in &describe.dataset.columns {
        writeln!(
          output,
          "{}\t{}",
          column.name.escape_debug(),
          column.data_type.escape_debug()
        )
        .unwrap();
      }
      output
    }
    ExecutionResult::Count(count) => format!("{} rows\n", count.row_count),
    ExecutionResult::Summarize(summary) => render_summary_rows(&summary.rows),
    ExecutionResult::SummarizeDetail(result) => {
      let mut output = String::from("Detailed finite summary: type-7 linear DOUBLE percentiles\n");
      for row in &result.rows {
        output.push_str(&render_summary_rows(std::slice::from_ref(&row.summary)));
        output.push_str("Percentile\tValue\n");
        for percentile in &row.percentiles {
          writeln!(
            output,
            "{}\t{}",
            percentile.percentile,
            optional_number(percentile.value)
          )
          .unwrap();
        }
      }
      output
    }
    ExecutionResult::Codebook(result) => {
      let mut output = String::from(
        "Codebook: SQL NULL missingness; native non-NULL distinct counts. Examples are up to three non-NULL scan values, not necessarily unique.\nVariable\tType\tNonmissing\tMissing\tDistinct\tExamples\n",
      );
      for row in &result.rows {
        writeln!(
          output,
          "{}\t{}\t{}\t{}\t{}\t[{}]",
          row.variable.escape_debug(),
          row.data_type.escape_debug(),
          row.nonmissing,
          row.missing,
          row.distinct,
          row.examples.iter().map(cell).collect::<Vec<_>>().join(", ")
        )
        .unwrap();
      }
      output
    }
    ExecutionResult::Missing(result) => {
      let mut output = String::from(
        "Missingness: SQL NULL only (stored NaN, infinity, empty strings and sentinels are not missing).\nVariable\tType\tTotal\tMissing\tNonmissing\tMissing (%)\n",
      );
      for row in &result.rows {
        writeln!(
          output,
          "{}\t{}\t{}\t{}\t{}\t{}",
          row.variable.escape_debug(),
          row.data_type.escape_debug(),
          row.total,
          row.missing,
          row.nonmissing,
          inference_number(row.missing_percent)
        )
        .unwrap();
      }
      output.push_str("Empty relations report 0% by convention; NA = unavailable or undefined.\n");
      output
    }
    ExecutionResult::Duplicates(result) => format!(
      "Duplicate key report\nKey variables: {}\nRows checked: {}\nDistinct key groups: {}\nDuplicate groups: {}\nRows in duplicate groups: {}\nSurplus rows: {}\nMaximum copies: {}\nSQL NULL key components are grouped; this report does not remove rows.\n",
      result
        .variables
        .iter()
        .map(|name| format!("{name:?}"))
        .collect::<Vec<_>>()
        .join(", "),
      result.total_rows,
      result.unique_groups,
      result.duplicate_groups,
      result.duplicate_rows,
      result.extra_rows,
      result.max_copies
    ),
    ExecutionResult::Isid(result) => format!(
      "isid passed\nKey variables: {}\nRows checked: {}\nDistinct key groups: {}\nRows with SQL NULL keys: {}\nMissing keys allowed: {}\n",
      result
        .variables
        .iter()
        .map(|name| format!("{name:?}"))
        .collect::<Vec<_>>()
        .join(", "),
      result.total_rows,
      result.unique_groups,
      result.missing_key_rows,
      if result.missok { "yes" } else { "no" }
    ),
    ExecutionResult::Assert(result) => format!(
      "assertion passed: {} rows\nFailed rows: {}\n",
      result.checked, result.failed
    ),
    ExecutionResult::Datasignature(result) => format!(
      "Data signature\nAlgorithm: {}\nRows: {}\nColumns: {}\nSignature: {}\n",
      result.algorithm.escape_debug(),
      result.row_count,
      result.column_count,
      result.signature.escape_debug()
    ),
    ExecutionResult::Head(preview) | ExecutionResult::Tail(preview) => {
      table(&preview.columns, &preview.rows)
    }
    ExecutionResult::SqlCreate(result) => dataset_outcome(
      &format!("Created and activated table {:?}", result.table_name),
      &result.dataset,
    ),
    ExecutionResult::Activate(result) => dataset_outcome(
      &format!("Activated table {:?}", result.table_name),
      &result.dataset,
    ),
    ExecutionResult::Join(result) => dataset_outcome("Joined", &result.dataset),
    ExecutionResult::Append(result) => dataset_outcome("Appended", &result.dataset),
    ExecutionResult::Reshape(result) => dataset_outcome("Reshaped", &result.dataset),
    ExecutionResult::Recode(result) => dataset_outcome("Recoded", &result.dataset),
    ExecutionResult::By(result) => format!(
      "Grouped results (native; no finite-sample reinterpretation)\n{}",
      table(&result.headers, &result.rows)
    ),
    ExecutionResult::Table(result) => table(&result.headers, &result.rows),
    ExecutionResult::Tabulate(result) => table(&result.headers, &result.rows),
    ExecutionResult::Collapse(result) => dataset_outcome("Collapsed", &result.dataset),
    ExecutionResult::Encode(result) => dataset_outcome("Encoded", &result.dataset),
    ExecutionResult::Decode(result) => dataset_outcome("Decoded", &result.dataset),
    ExecutionResult::Label(result) => label(result),
    ExecutionResult::Generate(result) => dataset_outcome("Generated", &result.dataset),
    ExecutionResult::Replace(result) => dataset_outcome("Replaced", &result.dataset),
    ExecutionResult::Rename(result) => dataset_outcome("Renamed", &result.dataset),
    ExecutionResult::Sort(result) => dataset_outcome("Sorted", &result.dataset),
    ExecutionResult::Gsort(result) => dataset_outcome("Sorted", &result.dataset),
    ExecutionResult::Keep(result) => dataset_outcome("Kept", &result.dataset),
    ExecutionResult::Drop(result) => dataset_outcome("Dropped", &result.dataset),
    ExecutionResult::Select(result) => dataset_outcome("Selected", &result.dataset),
    ExecutionResult::Save(result) => format!(
      "Saved {} rows, {} columns to {}\n",
      result.dataset.row_count,
      result.dataset.columns.len(),
      result.path.to_string_lossy().escape_debug()
    ),
    ExecutionResult::Plot(result) => plot_artifact(result),
    ExecutionResult::Export(result) => format!(
      "Exported {} rows, {} columns to {}\n",
      result.dataset.row_count,
      result.dataset.columns.len(),
      result.path.to_string_lossy().escape_debug()
    ),
    ExecutionResult::Logit(result) => format!(
      "Binary logit maximum likelihood ({})\nOutcome: {}; N = {}; events = {}; converged in {} information evaluations\nCandidate rows = {}; excluded missing/nonfinite = {}\nLog likelihood = {}; null log likelihood = {}; {lr_label}({}) = {}; p-value = {}\nMcFadden pseudo R2 = {}\n{inference_note}\n{}",
      binary_covariance_note(
        result.robust,
        result.cluster.as_ref(),
        "model-based inverse Fisher information"
      ),
      result.outcome.escape_debug(),
      result.observation_count,
      result.event_count,
      result.iterations,
      result
        .sample
        .as_ref()
        .map_or(result.observation_count, |sample| sample.total_observations),
      result
        .sample
        .as_ref()
        .map_or(0, |sample| sample.dropped_observations),
      inference_number(result.log_likelihood),
      inference_number(result.null_log_likelihood),
      result.likelihood_ratio_df,
      inference_number(result.likelihood_ratio),
      result
        .likelihood_ratio_p_value
        .map(logit_number)
        .unwrap_or_else(|| "NA".to_owned()),
      inference_number(result.mcfadden_r_squared),
      logit_estimates(&result.coefficients),
      lr_label = if result.robust {
        "descriptive LR difference"
      } else {
        "LR chi2"
      },
      inference_note = format_args!(
        "{}{}",
        binary_inference_note(result.robust, result.cluster.is_some()),
        binary_wald_note(result.omnibus_wald.as_ref())
      ),
    ),
    ExecutionResult::LogitMarginalEffects(result) => format!(
      "Logit continuous average partial effects; retained estimation N = {}\nPredictor-column derivatives, not discrete changes or original-variable interaction chain rules.\nDelta-method uncertainty conditional on observed estimation X; {}, asymptotic normal 95% intervals.\n{}{}",
      result.observation_count,
      binary_covariance_note(
        result.robust,
        result.cluster.as_ref(),
        "model-based covariance"
      ),
      logit_estimates(&result.effects),
      if result.effects.is_empty() {
        "No predictor effects in an intercept-only model.\n"
      } else {
        ""
      }
    ),
    ExecutionResult::NormalizedCommonNestedLogit(result) => format!(
      "Normalized common-scale nested choice maximum likelihood\nOutcome: {}; independent choice sets = {}; alternative rows = {}; alternatives per set = {}\nGroup: {}; alternative: {}; nest: {}; alternative/nest map = {:?}/{:?}\nCandidate sets = {}; whole-set exclusions = {}; selected fit iterations = {}; converged starts = {}/2\nPreferred common-utility normalization with ONE shared positive scale; not unrestricted nest-specific scales.\nFull joint natural-utility observed-Hessian covariance, correction1, asymptotic normal95; no uniform-null omnibus chi-square.\nLog likelihood = {}; fitted scale=1 conditional restriction = {}; independence LR chi2({}) = {}; p-value = {}\nSlope/upper coefficients (H0=0):\n{}\nShared inclusive-value diagnostic (H0=1; natural-scale Wald interval may cross zero):\n{}\nFitted IV bounds (0,1] satisfied: {}{}",
      result.outcome.escape_debug(),
      result.choice_set_count,
      result.observation_count,
      result.choice_set_size,
      result.group_variable.escape_debug(),
      result.alternative_variable.escape_debug(),
      result.nest_variable.escape_debug(),
      result.alternative_levels,
      result.alternative_nests,
      result.candidate_choice_set_count,
      result.excluded_choice_set_count,
      result.iterations,
      result.converged_starts,
      inference_number(result.log_likelihood),
      inference_number(result.restricted_log_likelihood),
      result.independence_df,
      inference_number(result.independence_likelihood_ratio),
      logit_number(result.independence_p_value),
      logit_estimates(&result.coefficients),
      logit_estimates(std::slice::from_ref(&result.dissimilarity)),
      result.random_utility_iv_bounds_satisfied,
      if result.random_utility_iv_bounds_satisfied {
        ""
      } else {
        "\nWARNING: fitted common inclusive value violates RUM bounds; preferred normalization does not make this fit random-utility consistent."
      },
    ),
    ExecutionResult::NonNormalizedNestedLogit(result) => format!(
      "Non-normalized two-level nested choice maximum likelihood\nOutcome: {}; independent choice sets = {}; alternative rows = {}; alternatives per set = {}\nGroup: {}; alternative: {}; nest: {}; alternative/nest map = {:?}/{:?}\nCandidate sets = {}; whole-set exclusions = {}; selected fit iterations = {}; converged starts = {}/2\nLower utilities retain levels: NOT the preferred common-utility RUM normalization.\nFull joint observed-Hessian covariance, correction1, asymptotic normal95; no uniform-null omnibus chi-square.\nLog likelihood = {}; fitted scale=1 conditional restriction = {}; independence LR chi2({}) = {}; p-value = {}\nSlope/upper coefficients (H0=0):\n{}\nInclusive-value diagnostics (H0=1; natural-scale Wald intervals may cross zero):\n{}\nFitted IV bounds (0,1] satisfied: {}{}",
      result.outcome.escape_debug(),
      result.choice_set_count,
      result.observation_count,
      result.choice_set_size,
      result.group_variable.escape_debug(),
      result.alternative_variable.escape_debug(),
      result.nest_variable.escape_debug(),
      result.alternative_levels,
      result.alternative_nests,
      result.candidate_choice_set_count,
      result.excluded_choice_set_count,
      result.iterations,
      result.converged_starts,
      inference_number(result.log_likelihood),
      inference_number(result.restricted_log_likelihood),
      result.independence_df,
      inference_number(result.independence_likelihood_ratio),
      logit_number(result.independence_p_value),
      logit_estimates(&result.coefficients),
      logit_estimates(
        &result
          .dissimilarities
          .iter()
          .map(|d| d.estimate.clone())
          .collect::<Vec<_>>()
      ),
      result.random_utility_iv_bounds_satisfied,
      if result.random_utility_iv_bounds_satisfied {
        ""
      } else {
        "\nWARNING: fitted inclusive values violate RUM bounds; higher likelihood is not random-utility consistency."
      },
    ),
    ExecutionResult::ConditionalLogit(result) => format!(
      "Single-choice conditional logit maximum likelihood\nOutcome: {}; independent choice sets = {}; alternative rows = {}; alternatives per set = {}\nGroup: {}; alternative: {}; available levels = {:?}; chosen counts = {:?}\nCandidate choice sets = {}; whole-set exclusions = {}; converged in {} information evaluations\nCommon group intercepts cancel; joint model-based Fisher covariance without N-K/G-K correction, normal95 inference.\nLog likelihood = {}; equal-probability null = {}; LR chi2({}) = {}; p-value = {}; McFadden pseudo R2 = {}\n{}",
      result.outcome.escape_debug(),
      result.choice_set_count,
      result.observation_count,
      result.choice_set_size,
      result.group_variable.escape_debug(),
      result.alternative_variable.escape_debug(),
      result.alternative_levels,
      result.chosen_counts,
      result.candidate_choice_set_count,
      result.excluded_choice_set_count,
      result.iterations,
      inference_number(result.log_likelihood),
      inference_number(result.null_log_likelihood),
      result.likelihood_ratio_df,
      inference_number(result.likelihood_ratio),
      logit_number(result.likelihood_ratio_p_value),
      inference_number(result.mcfadden_r_squared),
      logit_estimates(&result.coefficients),
    ),
    ExecutionResult::MultinomialLogit(result) => format!(
      "Multinomial logit maximum likelihood (model-based joint Fisher information)\nOutcome: {}; N = {}; base outcome = {}; converged in {} information evaluations\nCategory levels = {:?}; counts = {:?}\nCandidate rows = {}; excluded missing/nonfinite = {}\nLog likelihood = {}; null log likelihood = {}; LR chi2({}) = {}; p-value = {}\nMcFadden pseudo R2 = {}\nJoint covariance without N-K correction; asymptotic normal 95% intervals.\n{}",
      result.outcome.escape_debug(),
      result.observation_count,
      result.base_outcome,
      result.iterations,
      result.category_levels,
      result.category_counts,
      result
        .sample
        .as_ref()
        .map_or(result.observation_count, |s| s.total_observations),
      result.sample.as_ref().map_or(0, |s| s.dropped_observations),
      inference_number(result.log_likelihood),
      inference_number(result.null_log_likelihood),
      result.likelihood_ratio_df,
      inference_number(result.likelihood_ratio),
      result
        .likelihood_ratio_p_value
        .map(logit_number)
        .unwrap_or_else(|| "NA".into()),
      inference_number(result.mcfadden_r_squared),
      multinomial_estimates(&result.coefficients),
    ),
    ExecutionResult::MultinomialMarginalEffects(result) => format!(
      "Multinomial logit continuous average partial effects; retained estimation N = {}\nAll category probabilities; base outcome = {}. Predictor-column derivatives, not at-means effects or discrete changes.\nDelta-method uncertainty conditional on observed estimation X; full joint model-based covariance, asymptotic normal 95% intervals.\n{}{}",
      result.observation_count,
      result.base_outcome,
      multinomial_estimates(&result.effects),
      if result.effects.is_empty() {
        "No predictor effects in an intercept-only model.\n"
      } else {
        ""
      },
    ),
    ExecutionResult::Probit(result) => format!(
      "Binary probit maximum likelihood ({})\nOutcome: {}; N = {}; events = {}; converged in {} information evaluations\nCandidate rows = {}; excluded missing/nonfinite = {}\nLog likelihood = {}; null log likelihood = {}; {lr_label}({}) = {}; p-value = {}\nMcFadden pseudo R2 = {}\n{inference_note}\n{}",
      binary_covariance_note(
        result.robust,
        result.cluster.as_ref(),
        "model-based inverse observed information"
      ),
      result.outcome.escape_debug(),
      result.observation_count,
      result.event_count,
      result.iterations,
      result
        .sample
        .as_ref()
        .map_or(result.observation_count, |sample| sample.total_observations),
      result
        .sample
        .as_ref()
        .map_or(0, |sample| sample.dropped_observations),
      inference_number(result.log_likelihood),
      inference_number(result.null_log_likelihood),
      result.likelihood_ratio_df,
      inference_number(result.likelihood_ratio),
      result
        .likelihood_ratio_p_value
        .map(logit_number)
        .unwrap_or_else(|| "NA".to_owned()),
      inference_number(result.mcfadden_r_squared),
      logit_estimates(&result.coefficients),
      lr_label = if result.robust {
        "descriptive LR difference"
      } else {
        "LR chi2"
      },
      inference_note = format_args!(
        "{}{}",
        binary_inference_note(result.robust, result.cluster.is_some()),
        binary_wald_note(result.omnibus_wald.as_ref())
      ),
    ),
    ExecutionResult::ProbitMarginalEffects(result) => format!(
      "Probit continuous average partial effects; retained estimation N = {}\nPredictor-column derivatives, not discrete changes or original-variable interaction chain rules.\nDelta-method uncertainty conditional on observed estimation X; {}, asymptotic normal 95% intervals.\n{}{}",
      result.observation_count,
      binary_covariance_note(
        result.robust,
        result.cluster.as_ref(),
        "model-based covariance"
      ),
      logit_estimates(&result.effects),
      if result.effects.is_empty() {
        "No predictor effects in an intercept-only model.\n"
      } else {
        ""
      }
    ),
    ExecutionResult::HeckmanTwoStep(result) => format!(
      "Heckman two-step sample selection (IID Gaussian model; not joint ML)\nOutcome: {}; selection: {}; eligible N = {}; selected = {}; unselected = {}\nCandidate rows = {}; excluded = {}\nSelection equation (observed probit ML covariance):\n{}Augmented outcome equation (corrected generated-regressor covariance):\n{}Sigma = {}; rho = {} (moment points only; no scale/correlation SEs)\nSelection-stage log likelihood ONLY = {}\nSeparate covariance blocks, correction1, asymptotic normal95; no joint ML/LR or cross-equation inference.\n{}Exclusion validity is a user assumption, not tested. Coefficients are latent-mean associations, not causal effects.\nUse predict ..., xb / pr / mills / ycond / yexpected for point projections.\n",
      result.outcome.escape_debug(),
      result.selection.outcome.escape_debug(),
      result.selection.observation_count,
      result.selected_count,
      result.unselected_count,
      result
        .selection
        .sample
        .as_ref()
        .map_or(result.selection.observation_count, |s| s.total_observations),
      result
        .selection
        .sample
        .as_ref()
        .map_or(0, |s| s.dropped_observations),
      logit_estimates(&result.selection.coefficients),
      logit_estimates(&result.coefficients),
      inference_number(result.sigma),
      inference_number(result.rho),
      inference_number(result.selection.log_likelihood),
      if result.excluded_selection_predictors.is_empty() {
        "WARNING: No excluded selection predictor; identification relies on Gaussian functional form and may be weak.\n"
      } else {
        ""
      },
    ),
    ExecutionResult::TruncatedGaussian(result) => format!(
      "Lower-truncated Gaussian maximum likelihood (IID homoskedastic model)\nOutcome: {}; known lower limit = {}; retained N = {}\nCandidate rows = {}; excluded joint missing/nonfinite = {}; Newton iterations = {}\nEvery retained outcome is strictly above the limit; conditional density, not censoring.\nLog likelihood = {}; refitted same-sample null log likelihood = {}\nLR chi2({}) = {}; p-value = {}\nFull joint beta/lnsigma inverse observed information; correction1; asymptotic normal95, not Student t.\n{}Sigma = {}; delta SE = {}; log-Wald95 interval = [{}, {}]\nNo sigma=0 Wald test. Coefficients describe the underlying latent mean, not the conditional observed mean.\nUse predict ..., xb for latent location; pr for population P(Y*>lower|X), NOT conditional probability1; mean for E[Y*|Y*>lower,X].\nUse estat margins for retained-X continuous conditional-mean effects, not causal effects.\n",
      result.outcome.escape_debug(),
      inference_number(result.lower_limit),
      result.observation_count,
      result
        .sample
        .as_ref()
        .map_or(result.observation_count, |s| s.total_observations),
      result.sample.as_ref().map_or(0, |s| s.dropped_observations),
      result.iterations,
      inference_number(result.log_likelihood),
      inference_number(result.null_log_likelihood),
      result.likelihood_ratio_df,
      inference_number(result.likelihood_ratio),
      result
        .likelihood_ratio_p_value
        .map(logit_number)
        .unwrap_or_else(|| "NA".into()),
      tobit_estimates(&result.coefficients),
      inference_number(result.scale.value),
      inference_number(result.scale.standard_error),
      inference_number(result.scale.confidence_lower),
      inference_number(result.scale.confidence_upper)
    ),
    ExecutionResult::TruncatedGaussianMarginalEffects(result) => format!(
      "Truncated Gaussian continuous conditional-mean average partial effects; retained estimation N = {}; lower limit = {}\nDerivatives of E[Y*|Y*>lower,X], not latent slopes, censored means, discrete changes or causal effects.\nConditional-on-X full joint beta/lnsigma delta covariance; correction1, asymptotic normal95.\n{}{}",
      result.observation_count,
      inference_number(result.lower_limit),
      tobit_estimates(&result.effects),
      if result.effects.is_empty() {
        "No predictor effects in an intercept-only model.\n"
      } else {
        ""
      }
    ),
    ExecutionResult::Cox(r) => {
      let mut table = String::from(
        "Variable\tPH coefficient\tStd. error\tz\tP(two-sided)\tCI lower (95%)\tCI upper (95%)\n",
      );
      for c in &r.coefficients {
        writeln!(
          table,
          "{}\t{}\t{}\t{}\t{}\t{}\t{}",
          c.name.escape_debug(),
          c.value,
          c.standard_error,
          optional_number(c.statistic),
          optional_number(c.p_value),
          optional_number(c.confidence_lower),
          optional_number(c.confidence_upper)
        )
        .unwrap();
      }
      let mut baseline =
        String::from("Baseline reference (all retained covariate means, including dummy means):\n");
      for (name, value) in r.predictor_names.iter().zip(&r.baseline_reference_values) {
        writeln!(baseline, "{}\t{}", name.escape_debug(), value).unwrap();
      }
      baseline.push_str("Failure time\tAt risk\tFailures\tlog increment\tIncrement\tlog cumulative H\tCumulative H\tNet survival\n");
      for b in &r.baseline {
        writeln!(
          baseline,
          "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
          b.time,
          b.risk_count,
          b.event_count,
          b.log_increment,
          optional_number(b.increment),
          b.log_cumulative_hazard,
          optional_number(b.cumulative_hazard),
          b.survival
        )
        .unwrap();
      }
      format!(
        "Cox proportional hazards (Breslow ties; entry-zero, single spell; no intercept)\nTime: {}; event: {}; N = {}; failures = {}; non-events = {}\nCandidate rows = {}; excluded joint missing/nonfinite = {}; Newton iterations = {}\nPartial log likelihood = {}; same-sample zero-beta null = {}\nSlope LR chi2({}) = {}; p-value = {}{}\nSlope Wald chi2({}) = {}; p-value = {}; availability = {}\nCovariance: {}; multiplier = {}; asymptotic normal95, not Student t.\n{}{}Baseline uncertainty unavailable. NA = unavailable or unrepresentable natural scale; finite log baseline is retained.\nxb uses current X; cumhaz/survival use current X/time with right-continuous baseline steps, H0/S1 before first failure and last-step hold after final failure, not validated tail extrapolation.\nBreslow jumps do not identify instantaneous hazard. Net cause-specific survival is NOT all-cause survival or competing-risk incidence; associations are NOT causal effects. PH adequacy is not tested.\n",
        r.time_variable.escape_debug(),
        r.failure_variable.escape_debug(),
        r.observation_count,
        r.failure_count,
        r.censored_count,
        r.sample
          .as_ref()
          .map_or(r.observation_count, |s| s.total_observations),
        r.sample.as_ref().map_or(0, |s| s.dropped_observations),
        r.iterations,
        r.log_likelihood,
        r.null_log_likelihood,
        r.likelihood_ratio_df,
        r.likelihood_ratio,
        optional_number(r.likelihood_ratio_p_value),
        if r.robust {
          " (robust LR reference withheld)"
        } else {
          ""
        },
        r.omnibus_wald.degrees_of_freedom,
        optional_number(r.omnibus_wald.statistic),
        optional_number(r.omnibus_wald.p_value),
        r.omnibus_wald
          .unavailable_reason
          .as_deref()
          .unwrap_or("available"),
        if r.robust {
          "full efficient martingale-score sandwich N/(N-1), independent spells"
        } else {
          "inverse partial-likelihood observed information"
        },
        r.covariance_correction,
        table,
        baseline
      )
    }
    ExecutionResult::Weibull(r) => {
      let mut coefficients = String::from(
        "Variable\tPH coefficient\tStd. error\tz\tP(two-sided)\tCI lower (95%)\tCI upper (95%)\n",
      );
      for c in &r.coefficients {
        writeln!(
          coefficients,
          "{}\t{}\t{}\t{}\t{}\t{}\t{}",
          c.name.escape_debug(),
          c.value,
          c.standard_error,
          optional_number(c.statistic),
          optional_number(c.p_value),
          optional_number(c.confidence_lower),
          optional_number(c.confidence_upper)
        )
        .unwrap();
      }
      format!(
        "Weibull proportional-hazards maximum likelihood (entry-zero, single spell)\nTime: {}; event: {}; N = {}; failures = {}; non-events = {}\nCandidate rows = {}; excluded joint missing/nonfinite = {}; Newton iterations = {}\nDensity log likelihood = {}; Stata-normalized log likelihood = {}\nSame-sample refitted null density log likelihood = {}; normalized = {}\nSlope LR chi2({}) = {}; p-value = {}{}\nSlope Wald chi2({}) = {}; p-value = {}; availability = {}\nFull joint PH beta/lnshape covariance: {}; multiplier = {}; asymptotic normal95, not Student t.\n{}Shape = {}; delta SE = {}; lnshape = {}; SE(lnshape) = {}; z(shape=1) = {}; p-value = {}; positive log-Wald95 interval = [{}, {}]\nNo shape=0 Wald test; shape=1 tests constant hazard within this family, not overall model adequacy.\nNA = unavailable or unrepresentable; zero SE does not imply exact inference.\nxb is the PH index; hazard/cumhaz/survival use current time and X. Net survival is NOT all-cause joblessness survival or competing-risk incidence; associations are NOT causal effects.\n",
        r.time_variable.escape_debug(),
        r.failure_variable.escape_debug(),
        r.observation_count,
        r.failure_count,
        r.censored_count,
        r.sample
          .as_ref()
          .map_or(r.observation_count, |s| s.total_observations),
        r.sample.as_ref().map_or(0, |s| s.dropped_observations),
        r.iterations,
        inference_number(r.log_likelihood),
        inference_number(r.normalized_log_likelihood),
        inference_number(r.null_log_likelihood),
        inference_number(r.null_normalized_log_likelihood),
        r.likelihood_ratio_df,
        inference_number(r.likelihood_ratio),
        optional_number(r.likelihood_ratio_p_value),
        if r.robust {
          " (robust LR reference withheld)"
        } else {
          ""
        },
        r.omnibus_wald.degrees_of_freedom,
        optional_number(r.omnibus_wald.statistic),
        optional_number(r.omnibus_wald.p_value),
        r.omnibus_wald
          .unavailable_reason
          .as_deref()
          .unwrap_or("available"),
        if r.robust {
          "independent-spell score sandwich N/(N-1), not OLS HC1"
        } else {
          "inverse observed information, no N-K scaling"
        },
        r.covariance_correction,
        coefficients,
        inference_number(r.shape.value),
        inference_number(r.shape.standard_error),
        inference_number(r.shape.log_value),
        inference_number(r.shape.log_standard_error),
        optional_number(r.shape.log_statistic),
        optional_number(r.shape.log_p_value),
        optional_number(r.shape.confidence_lower),
        optional_number(r.shape.confidence_upper)
      )
    }
    ExecutionResult::Tobit(result) => format!(
      "Lower-censored Gaussian Tobit maximum likelihood (IID homoskedastic model)\nOutcome: {}; lower limit = {}; N = {}; left-censored = {}; uncensored = {}\nCandidate rows = {}; excluded missing/nonfinite = {}; Newton iterations = {}\nLog likelihood = {}; refitted same-sample null log likelihood = {}\nLR chi2({}) = {}; p-value = {}\nFull joint beta/lnsigma inverse observed information; correction1; asymptotic normal95, not Student t.\n{}Sigma = {}; delta SE = {}; log-Wald95 interval = [{}, {}]\nNo sigma=0 Wald test. Coefficients describe the latent mean, not observed-mean effects.\nUse predict ..., pr for P(Y*>lower|X), predict ..., mean for E[max(lower,Y*)|X], estat margins for continuous observed-mean effects.\n",
      result.outcome.escape_debug(),
      inference_number(result.lower_limit),
      result.observation_count,
      result.left_censored_count,
      result.uncensored_count,
      result
        .sample
        .as_ref()
        .map_or(result.observation_count, |s| s.total_observations),
      result.sample.as_ref().map_or(0, |s| s.dropped_observations),
      result.iterations,
      inference_number(result.log_likelihood),
      inference_number(result.null_log_likelihood),
      result.likelihood_ratio_df,
      inference_number(result.likelihood_ratio),
      result
        .likelihood_ratio_p_value
        .map(logit_number)
        .unwrap_or_else(|| "NA".into()),
      tobit_estimates(&result.coefficients),
      inference_number(result.scale.value),
      inference_number(result.scale.standard_error),
      inference_number(result.scale.confidence_lower),
      inference_number(result.scale.confidence_upper),
    ),
    ExecutionResult::TobitMarginalEffects(result) => format!(
      "Tobit continuous observed-censored-mean average partial effects; retained estimation N = {}; lower limit = {}\nDerivatives of E[max(lower,Y*)|X], not latent slopes, truncated means, discrete changes or causal effects.\nConditional-on-X full joint beta/lnsigma delta covariance; correction1, asymptotic normal95.\n{}{}",
      result.observation_count,
      inference_number(result.lower_limit),
      tobit_estimates(&result.effects),
      if result.effects.is_empty() {
        "No predictor effects in an intercept-only model.\n"
      } else {
        ""
      },
    ),
    ExecutionResult::CountProbabilities(result) => {
      let law = match result.distribution {
        tabdat_domain::CountProbabilityLaw::Poisson => "Poisson".to_owned(),
        tabdat_domain::CountProbabilityLaw::Nb2 { alpha } => format!("NB2 (alpha = {alpha})"),
      };
      let mut text = format!(
        "Observed/fitted count frequencies: {law}\nOutcome: {}; retained estimation N = {}\nCount\tObserved N\tObserved fraction\tFitted fraction\tExpected N\n",
        result.outcome.escape_debug(),
        result.table.observation_count
      );
      for bin in &result.table.bins {
        let label = if bin.upper_bound.is_some() {
          bin.lower_bound.to_string()
        } else {
          format!(">={}", bin.lower_bound)
        };
        writeln!(
          text,
          "{label}\t{}\t{}\t{}\t{}",
          bin.observed_count, bin.observed_fraction, bin.fitted_fraction, bin.expected_count
        )
        .unwrap();
      }
      text.push_str("Fitted fractions average individual conditional probabilities over the ORIGINAL retained sample, not the probability at an average mean.\nDescriptive distributional assumption: Poisson/NB2 probabilities are NOT guaranteed by robust/cluster QMLE covariance. No uncertainty or formal goodness-of-fit test is computed.\n");
      text
    }
    ExecutionResult::Nbreg(result) => nb2_report(result),
    ExecutionResult::NbregMarginalEffects(result) => format!(
      "NB2 continuous average partial effects; retained estimation N = {}\nPredictor-column derivatives beta_j*mean(mu), not discrete changes or original-variable chain rules.\nDelta-method uncertainty conditional on observed X; {}, asymptotic normal 95% intervals. Uses beta block of FULL joint beta/lnalpha covariance.\n{}{}{}{}",
      result.observation_count,
      nb2_covariance_note(result.robust, result.cluster.as_ref()),
      count_cluster_note(result.cluster.as_ref()),
      result
        .offset_variable
        .as_ref()
        .map_or_else(String::new, |v| format!(
          "Fixed offset {} held at retained values in derivatives and uncertainty.\n",
          v.escape_debug()
        )),
      poisson_estimates(&result.effects),
      if result.effects.is_empty() {
        "No predictor effects in an intercept-only model.\n"
      } else {
        ""
      }
    ),
    ExecutionResult::Poisson(result) => poisson_report(result),
    ExecutionResult::PoissonMarginalEffects(result) => format!(
      "Poisson continuous average partial effects; retained estimation N = {}\nPredictor-column derivatives beta_j*mean(mu), not discrete changes or original-variable chain rules.\nDelta-method uncertainty conditional on observed estimation X; {}, asymptotic normal 95% intervals.\n{}{}{}{}",
      result.observation_count,
      poisson_covariance_note(result.robust, result.cluster.as_ref()),
      count_cluster_note(result.cluster.as_ref()),
      result
        .offset_variable
        .as_ref()
        .map_or_else(String::new, |v| format!(
          "Fixed offset {} held at retained values in derivatives and uncertainty.\n",
          v.escape_debug()
        )),
      poisson_estimates(&result.effects),
      if result.effects.is_empty() {
        "No predictor effects in an intercept-only model.\n"
      } else {
        ""
      }
    ),
    ExecutionResult::Regression(result) => regression(result),
    ExecutionResult::FixedEffects(result) => fixed_effects(result),
    ExecutionResult::RandomEffects(result) => random_effects(result),
    ExecutionResult::Ttest(result) => ttest(result),
    ExecutionResult::Prediction(result) => dataset_outcome("Predicted", &result.dataset),
    ExecutionResult::Lincom(result) => format!(
      "Linear combination: {}\nt reference; residual df = {}\nEstimate\tStd. error\tt\tP(two-sided)\tCI lower ({}%)\tCI upper ({}%)\n{}\t{}\t{}\t{}\t{}\t{}\nNA = unavailable or undefined.\n",
      result.label.escape_debug(),
      result
        .df_residual
        .map(|df| df.to_string())
        .unwrap_or_else(|| "NA".to_owned()),
      result.ci_level,
      result.ci_level,
      inference_number(result.estimate),
      inference_number(result.standard_error),
      inference_number(result.statistic),
      inference_number(result.p_value),
      inference_number(result.ci_lower),
      inference_number(result.ci_upper)
    ),
    ExecutionResult::Test(result) => {
      let mut output = String::from("Linear restrictions:\n");
      for constraint in &result.constraints {
        writeln!(output, "  {}", constraint.escape_debug()).unwrap();
      }
      let distribution = if result.is_chi2 {
        format!("chi-square({})", result.df)
      } else {
        format!(
          "F({}, {})",
          result.df,
          result
            .df_residual
            .map(|df| df.to_string())
            .unwrap_or_else(|| "NA".to_owned())
        )
      };
      writeln!(
        output,
        "{distribution} = {}; P(upper-tail) = {}",
        inference_number(result.statistic),
        inference_number(result.p_value)
      )
      .unwrap();
      output.push_str("NA = unavailable or undefined.\n");
      output
    }
    ExecutionResult::Run(run) => format!(
      "Executed {} commands from {}\n",
      run.executed_commands,
      run.path.to_string_lossy().escape_debug()
    ),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tabdat_domain::{SummarizeResult, SummaryRow};

  #[test]
  fn projected_ar_terminal_marks_the_conservative_all_real_fallback() {
    let output = render_execution_result(&ExecutionResult::ProjectedAndersonRubinConfidenceSet(
      tabdat_stats::ProjectedArConfidenceSet {
        coefficient: "grade76".into(),
        nuisance_endogenous: vec!["exp76".into(), "expsq76".into()],
        excluded_instruments: vec!["col4".into()],
        confidence_level: 95.0,
        observation_count: 3_010,
        numerator_df: 1,
        denominator_df: 3_004,
        critical_f: 3.84,
        shape: tabdat_stats::ArConfidenceSetShape::AllReal,
        intervals: vec![tabdat_stats::ArConfidenceInterval {
          lower: None,
          upper: None,
        }],
        all_real_conservative_fallback: true,
      },
    ));
    assert!(output.contains("Projected Anderson-Rubin confidence set"));
    assert!(output.contains("(-inf, +inf)"));
    assert!(output.contains("conservative all-real set"));
  }

  #[test]
  fn unbalanced_gls_terminal_describes_theta_range_without_ml_profile_claims() {
    let sizes = [1, 2, 3, 4, 5, 3, 2, 4];
    let mut y = Vec::new();
    let mut x = Vec::new();
    let mut groups = Vec::new();
    for (unit, &size) in sizes.iter().enumerate() {
      let id = unit + 1;
      for time in 0..size {
        let predictor = 0.3 * id as f64 + 0.2 * time as f64 + 0.07 * ((id * time + 2) % 5) as f64;
        y.push(
          1.0
            + 0.7 * predictor
            + 0.3 * ((id * id + 1) % 5) as f64
            + 0.11 * ((id * time + 3) % 7) as f64,
        );
        x.push(predictor);
        groups.push(id.to_string());
      }
    }
    let n = y.len();
    let problem = tabdat_stats::EstimationProblem::new(
      "y",
      vec!["x".into()],
      y,
      x.into_iter().map(|value| vec![value]).collect(),
    )
    .unwrap()
    .with_sample(
      tabdat_stats::EstimationSample::new((0..n).collect(), n, 0, None, Some(groups)).unwrap(),
    );
    let result = tabdat_stats::fit_random_effects_unbalanced(&problem, "id").unwrap();
    let output = random_effects(&tabdat_domain::RandomEffectsState {
      time_variable: "time".into(),
      entity_data_type: "BIGINT".into(),
      result,
    });
    assert!(output.contains("Unbalanced one-way random-effects GLS"));
    assert!(output.contains("theta@minimum T"));
    assert!(output.contains("unit-specific Swamy-Arora theta ranges"));
    assert!(!output.contains("eta uses minimum T"));
    assert!(!output.contains("Interior information parameters"));
  }

  #[test]
  fn key_guard_display_escapes_names_and_preserves_owned_counters() {
    let duplicates = ExecutionResult::Duplicates(tabdat_domain::DuplicatesResult {
      variables: vec!["key\n列".into()],
      total_rows: 5,
      unique_groups: 3,
      duplicate_groups: 1,
      duplicate_rows: 3,
      extra_rows: 2,
      max_copies: 3,
    });
    let text = render_execution_result(&duplicates);
    assert!(text.contains("Key variables: \"key\\n列\"\n"));
    assert!(text.contains("Distinct key groups: 3\nDuplicate groups: 1\nRows in duplicate groups: 3\nSurplus rows: 2\nMaximum copies: 3\n"));
    for missok in [false, true] {
      let result = ExecutionResult::Isid(tabdat_domain::IsidResult {
        variables: vec!["x\t".into()],
        total_rows: 0,
        unique_groups: 0,
        missing_key_rows: 0,
        missok,
      });
      let text = render_execution_result(&result);
      assert!(text.starts_with("isid passed\nKey variables: \"x\\t\"\nRows checked: 0\n"));
      assert!(text.contains(if missok {
        "Missing keys allowed: yes"
      } else {
        "Missing keys allowed: no"
      }));
    }
  }

  #[test]
  fn codebook_display_preserves_examples_and_escapes_controls() {
    let result = ExecutionResult::Codebook(tabdat_domain::CodebookResult {
      rows: vec![tabdat_domain::CodebookRow {
        variable: "x\tline\n\u{1b}[2J".to_owned(),
        data_type: "TYPE\nFAKE".to_owned(),
        nonmissing: 3,
        missing: 0,
        distinct: 2,
        examples: vec![
          CellValue::Text("a\t\n\u{1b}[2J".to_owned()),
          CellValue::Decimal {
            width: 38,
            scale: 2,
            value: -1,
          },
          CellValue::Bytes(vec![0, 255]),
        ],
      }],
    });
    let text = render_execution_result(&result);
    assert!(text.contains("x\\tline\\n\\u{1b}[2J\tTYPE\\nFAKE\t3\t0\t2\t["));
    assert!(text.contains("-0.01, 0x00ff]"));
    assert!(!text.contains('\u{1b}'));
    assert!(text.contains("not necessarily unique"));
    let empty =
      render_execution_result(&ExecutionResult::Codebook(tabdat_domain::CodebookResult {
        rows: vec![],
      }));
    assert!(empty.ends_with("Variable\tType\tNonmissing\tMissing\tDistinct\tExamples\n"));
  }

  #[test]
  fn missing_display_escapes_owned_names_and_types_without_fabricating_percent() {
    let result = ExecutionResult::Missing(tabdat_domain::MissingResult {
      rows: vec![tabdat_domain::MissingRow {
        variable: "x\tline\n\u{1b}[2J".to_owned(),
        data_type: "TYPE\nFAKE".to_owned(),
        total: 1,
        missing: 0,
        nonmissing: 1,
        missing_percent: f64::NAN,
      }],
    });
    let text = render_execution_result(&result);
    assert!(text.contains("x\\tline\\n\\u{1b}[2J\tTYPE\\nFAKE\t1\t0\t1\tNA\n"));
    assert!(!text.contains('\u{1b}'));
    assert!(text.starts_with("Missingness: SQL NULL only"));
  }

  #[test]
  fn ttest_display_escapes_labels_and_keeps_singleton_inference_undefined() {
    let sample = tabdat_domain::TtestGroupStats {
      name: "label\tline\n\u{1b}[2J".to_owned(),
      observations: 1,
      mean: 2.0,
      standard_error: f64::NAN,
      standard_deviation: f64::NAN,
      ci_lower: f64::NAN,
      ci_upper: f64::NAN,
    };
    let result = ExecutionResult::Ttest(Box::new(tabdat_domain::TtestResult {
      varname1: "x\nbad".to_owned(),
      varname2: None,
      by_variable: None,
      is_paired: false,
      is_welch: false,
      null_value: 2.0,
      group1_stats: sample.clone(),
      group2_stats: None,
      combined_stats: None,
      difference_stats: tabdat_domain::TtestGroupStats {
        name: "mean".to_owned(),
        ..sample
      },
      t_statistic: f64::NAN,
      degrees_of_freedom: 0.0,
      p_left: f64::NAN,
      p_two: f64::NAN,
      p_right: f64::NAN,
    }));
    let text = render_execution_result(&result);
    assert!(text.contains("Variable: x\\nbad\n"));
    assert!(text.contains("label\\tline\\n\\u{1b}[2J\t1\t2\tNA\tNA\tNA\tNA\n"));
    assert!(!text.contains('\u{1b}'));
    assert!(text.contains("t reference; df = 0; t = NA\n"));
    assert!(text.contains("P(left-tail) = NA; P(two-sided) = NA; P(right-tail) = NA\n"));
  }

  #[test]
  fn scalar_display_preserves_integer_decimal_and_missing_values() {
    assert_eq!(
      cell(&CellValue::SignedInteger(i128::MIN)),
      i128::MIN.to_string()
    );
    assert_eq!(
      cell(&CellValue::UnsignedInteger(u128::MAX)),
      u128::MAX.to_string()
    );
    for (scale, value, expected) in [
      (2, -1, "-0.01"),
      (2, 0, "0.00"),
      (0, 12, "12"),
      (2, 1234, "12.34"),
    ] {
      assert_eq!(
        cell(&CellValue::Decimal {
          width: 38,
          scale,
          value
        }),
        expected
      );
    }
    assert_eq!(cell(&CellValue::Null), "NA");
    assert_eq!(
      cell(&CellValue::Text("NA\t\n\u{1b}".to_owned())),
      "\"NA\\t\\n\\u{1b}\""
    );
    assert_eq!(cell(&CellValue::Bytes(vec![0, 255])), "0x00ff");
  }

  #[test]
  fn frequency_display_preserves_owned_order_labels_and_empty_tables() {
    let result = ExecutionResult::Tabulate(tabdat_domain::TabulateResult {
      headers: vec![
        "category".to_owned(),
        "Count".to_owned(),
        "Percent".to_owned(),
      ],
      rows: vec![
        vec![
          CellValue::Text("label\t10".to_owned()),
          CellValue::SignedInteger(2),
          CellValue::Float(50.0),
        ],
        vec![
          CellValue::SignedInteger(2),
          CellValue::SignedInteger(1),
          CellValue::Float(25.0),
        ],
        vec![
          CellValue::Null,
          CellValue::SignedInteger(1),
          CellValue::Float(25.0),
        ],
      ],
    });
    assert_eq!(
      render_execution_result(&result),
      "category\tCount\tPercent\n\"label\\t10\"\t2\t50\n2\t1\t25\nNA\t1\t25\n(3 rows shown)\n"
    );
    let empty = ExecutionResult::Tabulate(tabdat_domain::TabulateResult {
      headers: vec![
        "category".to_owned(),
        "Count".to_owned(),
        "Percent".to_owned(),
      ],
      rows: vec![],
    });
    assert_eq!(
      render_execution_result(&empty),
      "category\tCount\tPercent\n(0 rows shown)\n"
    );
  }

  #[test]
  fn post_estimation_names_the_reference_and_preserves_undefined_inference() {
    let result = ExecutionResult::Lincom(tabdat_domain::LincomResult {
      label: "constant".to_owned(),
      estimate: 5.0,
      standard_error: 0.0,
      statistic: f64::NAN,
      p_value: f64::NAN,
      ci_lower: 5.0,
      ci_upper: 5.0,
      ci_level: 95.0,
      df_residual: Some(4),
    });
    let text = render_execution_result(&result);
    assert!(text.contains("t reference; residual df = 4"));
    assert!(text.contains("CI lower (95%)\tCI upper (95%)\n5\t0\tNA\tNA\t5\t5\n"));
    assert!(!text.contains("NaN"));
    for (is_chi2, expected) in [(false, "F(2, 4)"), (true, "chi-square(2)")] {
      let test = ExecutionResult::Test(tabdat_domain::TestResult {
        constraints: vec!["x = 0".to_owned()],
        statistic: 3.0,
        p_value: 0.0,
        df: 2,
        df_residual: Some(4),
        is_chi2,
      });
      assert!(
        render_execution_result(&test).contains(&format!("{expected} = 3; P(upper-tail) = 0"))
      );
    }
  }

  #[test]
  fn summary_displays_absent_statistics_without_inventing_zeroes() {
    let result = ExecutionResult::Summarize(SummarizeResult {
      rows: vec![SummaryRow {
        variable: "empty".to_owned(),
        count: 0,
        mean: None,
        std_dev: None,
        minimum: None,
        maximum: None,
      }],
    });
    assert_eq!(
      render_execution_result(&result),
      "Variable\tN\tMean\tStd. dev.\tMin\tMax\nempty\t0\tNA\tNA\tNA\tNA\n"
    );
  }
}
