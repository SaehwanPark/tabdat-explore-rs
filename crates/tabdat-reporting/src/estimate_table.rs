//! Presentation of owned reports only: no fitting or current-data access.

use std::fmt::Write;
use tabdat_domain::{EstimateSnapshot, EstimateTableResult};

fn number(value: Option<f64>) -> String {
  match value.filter(|v| v.is_finite()) {
    Some(value) => format!("{value:.6}"),
    None => "NA".into(),
  }
}

fn row(
  output: &mut String,
  label: &str,
  models: &[EstimateSnapshot],
  value: impl Fn(&EstimateSnapshot) -> String,
) {
  write!(output, "{}", label.escape_debug()).unwrap();
  for model in models {
    write!(output, "\t{}", value(model).escape_debug()).unwrap();
  }
  output.push('\n');
}

fn nls_expression(
  expression: &tabdat_stats::NlsExpression,
  model: &tabdat_stats::NlsModel,
) -> String {
  use tabdat_stats::NlsExpression as E;
  let render = |e: &E| nls_expression(e, model);
  match expression {
    E::Constant(v) => v.to_string(),
    E::Predictor(i) => model
      .predictor_names
      .get(*i)
      .cloned()
      .unwrap_or_else(|| "?".into()),
    E::Parameter(i) => model
      .parameter_names
      .get(*i)
      .cloned()
      .unwrap_or_else(|| "?".into()),
    E::Negate(e) => format!("(-{})", render(e)),
    E::Add(a, b) => format!("({}+{})", render(a), render(b)),
    E::Subtract(a, b) => format!("({}-{})", render(a), render(b)),
    E::Multiply(a, b) => format!("({}*{})", render(a), render(b)),
    E::Divide(a, b) => format!("({}/{})", render(a), render(b)),
    E::Exp(e) => format!("exp({})", render(e)),
  }
}

pub fn render_estimate_table(table: &EstimateTableResult) -> String {
  let models = &table.models;
  let mut output = String::from("Stored fit comparison (reporting snapshots, no refits)\n");
  row(&mut output, "Estimate", models, |m| m.name.clone());
  row(&mut output, "Estimator", models, |m| m.estimator.clone());
  row(&mut output, "Outcome", models, |m| m.outcome.clone());
  row(&mut output, "Covariance", models, |m| m.covariance.clone());
  if models.iter().any(|m| m.nls_model.is_some()) {
    row(&mut output, "NLS mean", models, |m| {
      m.nls_model
        .as_ref()
        .map(|model| nls_expression(&model.expression, model))
        .unwrap_or_else(|| "NA".into())
    });
  }
  row(&mut output, "Cluster variable", models, |m| {
    m.cluster_variable.clone().unwrap_or("NA".into())
  });
  row(&mut output, "Cluster groups", models, |m| {
    m.cluster_count.map_or("NA".into(), |g| g.to_string())
  });
  row(&mut output, "Coefficient reference", models, |m| {
    match m.inference_degrees_of_freedom {
      Some(df) => format!("{}({df})", m.reference_distribution),
      None => m.reference_distribution.clone(),
    }
  });
  for name in &table.parameter_names {
    row(&mut output, &format!("{name} coefficient"), models, |m| {
      number(
        m.coefficients
          .iter()
          .find(|c| c.name == *name)
          .map(|c| c.value),
      )
    });
    row(
      &mut output,
      &format!("{name} standard error"),
      models,
      |m| {
        number(
          m.coefficients
            .iter()
            .find(|c| c.name == *name)
            .and_then(|c| c.standard_error),
        )
      },
    );
    row(&mut output, &format!("{name} statistic"), models, |m| {
      number(
        m.coefficients
          .iter()
          .find(|c| c.name == *name)
          .and_then(|c| c.statistic),
      )
    });
  }
  row(&mut output, "N", models, |m| {
    m.observation_count.to_string()
  });
  row(&mut output, "Candidate rows", models, |m| {
    m.sample.total_observations.to_string()
  });
  row(&mut output, "Dropped rows", models, |m| {
    m.sample.dropped_observations.to_string()
  });
  row(&mut output, "Log likelihood (descriptive)", models, |m| {
    number(m.log_likelihood)
  });
  row(&mut output, "Likelihood convention", models, |m| {
    m.log_likelihood_convention.clone().unwrap_or("NA".into())
  });
  row(&mut output, "OLS R-squared", models, |m| {
    number(m.r_squared)
  });
  row(
    &mut output,
    "Binary McFadden pseudo-R-squared",
    models,
    |m| number(m.mcfadden_r_squared),
  );
  output.push_str("Statistics use each model's selected covariance/reference: binary normal z; OLS Student t (clustered: G-1); NLS approximate t. NA means unavailable/absent, not zero.\nSamples/outcomes can differ; equal N does not establish equal rows. JSON retains fit-time candidate positions, not current-data identities.\nGaussian working-model RSS/N and Bernoulli likelihoods are descriptive, not a cross-family or robust LR test. Reports cannot restore fits.\n");
  output
}
