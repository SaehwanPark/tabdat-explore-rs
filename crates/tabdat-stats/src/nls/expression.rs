use super::{
  MAX_NLS_DEPTH, MAX_NLS_NODES, MAX_NLS_PARAMETERS, failed, finite_product, finite_ratio, invalid,
};
use crate::StatsError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Bounded mathematical mean expression. Indices bind to the model's ordered
/// predictor/parameter names, independently of a language AST or SQL engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "arguments", rename_all = "snake_case")]
pub enum NlsExpression {
  Constant(f64),
  Predictor(usize),
  Parameter(usize),
  Negate(Box<Self>),
  Add(Box<Self>, Box<Self>),
  Subtract(Box<Self>, Box<Self>),
  Multiply(Box<Self>, Box<Self>),
  Divide(Box<Self>, Box<Self>),
  Exp(Box<Self>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NlsModel {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub parameter_names: Vec<String>,
  pub expression: NlsExpression,
}

/// First and second derivatives of the mean, NOT derivatives of residual loss.
#[derive(Debug, Clone, PartialEq)]
pub struct NlsRowEvaluation {
  pub mean: f64,
  pub gradient: Vec<f64>,
  pub hessian: Vec<Vec<f64>>,
}

pub(super) struct Jet {
  pub mean: f64,
  pub gradient: Vec<f64>,
  pub hessian: Vec<f64>,
}
impl Jet {
  fn constant(mean: f64, p: usize) -> Self {
    Self {
      mean,
      gradient: vec![0.0; p],
      hessian: vec![0.0; p * p],
    }
  }
  fn finite(self) -> Result<Self, StatsError> {
    if !self.mean.is_finite()
      || self
        .gradient
        .iter()
        .chain(&self.hessian)
        .any(|v| !v.is_finite())
    {
      return Err(failed("nonrepresentable mean or derivatives"));
    }
    Ok(self)
  }
  fn unary(self, mean: f64, first: f64, second: f64) -> Result<Self, StatsError> {
    let p = self.gradient.len();
    let mut result = Self::constant(mean, p);
    for i in 0..p {
      result.gradient[i] = finite_product(first, self.gradient[i])?;
      for j in 0..p {
        result.hessian[i * p + j] = finite_product(first, self.hessian[i * p + j])?
          + finite_product(finite_product(second, self.gradient[i])?, self.gradient[j])?;
      }
    }
    result.finite()
  }
  fn sum(self, other: Self, sign: f64) -> Result<Self, StatsError> {
    Self {
      mean: self.mean + sign * other.mean,
      gradient: self
        .gradient
        .iter()
        .zip(&other.gradient)
        .map(|(a, b)| a + sign * b)
        .collect(),
      hessian: self
        .hessian
        .iter()
        .zip(&other.hessian)
        .map(|(a, b)| a + sign * b)
        .collect(),
    }
    .finite()
  }
  fn product(self, other: Self) -> Result<Self, StatsError> {
    let p = self.gradient.len();
    let mut result = Self::constant(finite_product(self.mean, other.mean)?, p);
    for i in 0..p {
      result.gradient[i] = finite_product(self.gradient[i], other.mean)?
        + finite_product(self.mean, other.gradient[i])?;
      for j in 0..p {
        result.hessian[i * p + j] = finite_product(self.hessian[i * p + j], other.mean)?
          + finite_product(self.mean, other.hessian[i * p + j])?
          + finite_product(self.gradient[i], other.gradient[j])?
          + finite_product(other.gradient[i], self.gradient[j])?;
      }
    }
    result.finite()
  }
  fn reciprocal(self) -> Result<Self, StatsError> {
    if self.mean == 0.0 {
      return Err(failed("division by zero"));
    }
    let inverse = finite_ratio(1.0, self.mean)?;
    let p = self.gradient.len();
    let mut result = Self::constant(inverse, p);
    for i in 0..p {
      let first = finite_product(self.gradient[i], inverse)?;
      result.gradient[i] = finite_product(-first, inverse)?;
      for j in 0..p {
        let second = finite_product(self.gradient[j], inverse)?;
        let cross = finite_product(finite_product(2.0, first)?, second)?;
        let value = cross - finite_product(self.hessian[i * p + j], inverse)?;
        result.hessian[i * p + j] = finite_product(value, inverse)?;
      }
    }
    result.finite()
  }
}

fn exponential(value: f64) -> Result<f64, StatsError> {
  let result = value.exp();
  // A clipped zero mean can erase representable tail derivatives and falsely
  // pass a stationary/curvature check. This bounded evaluator rejects underflow.
  if !result.is_finite() || result == 0.0 {
    return Err(failed("exp overflow or underflow"));
  }
  Ok(result)
}

impl NlsModel {
  pub fn validate(&self) -> Result<usize, StatsError> {
    let p = self.parameter_names.len();
    if self.outcome.trim().is_empty()
      || !(1..=MAX_NLS_PARAMETERS).contains(&p)
      || self.predictor_names.len() > MAX_NLS_NODES
    {
      return Err(invalid("requires an outcome and 1..32 explicit parameters"));
    }
    let mut names = BTreeSet::new();
    for name in &self.parameter_names {
      if name.trim().is_empty() || name == &self.outcome || !names.insert(name) {
        return Err(invalid(
          "parameter names must be nonempty, unique and distinct from outcome",
        ));
      }
    }
    for name in &self.predictor_names {
      if name.trim().is_empty() || !names.insert(name) {
        return Err(invalid(
          "data names must be nonempty, unique and distinct from parameters",
        ));
      }
    }
    let mut parameters = vec![false; p];
    let mut predictors = vec![false; self.predictor_names.len()];
    let mut nodes = 0;
    validate_node(
      &self.expression,
      1,
      &mut nodes,
      &mut parameters,
      &mut predictors,
    )?;
    if parameters.contains(&false) || predictors.contains(&false) {
      return Err(invalid(
        "every declared parameter and predictor must occur in the expression",
      ));
    }
    Ok(nodes)
  }

  pub(super) fn validate_row(&self, parameters: &[f64], row: &[f64]) -> Result<(), StatsError> {
    if parameters.len() != self.parameter_names.len()
      || row.len() != self.predictor_names.len()
      || parameters.iter().chain(row).any(|v| !v.is_finite())
    {
      return Err(invalid(
        "parameter/data row dimensions or finite values do not match the model",
      ));
    }
    Ok(())
  }

  pub fn evaluate(&self, parameters: &[f64], row: &[f64]) -> Result<NlsRowEvaluation, StatsError> {
    self.validate()?;
    self.validate_row(parameters, row)?;
    let jet = jet(&self.expression, parameters, row)?;
    let p = parameters.len();
    Ok(NlsRowEvaluation {
      mean: jet.mean,
      gradient: jet.gradient,
      hessian: jet.hessian.chunks(p).map(<[f64]>::to_vec).collect(),
    })
  }

  pub fn predict(&self, parameters: &[f64], row: &[f64]) -> Result<f64, StatsError> {
    self.validate()?;
    self.validate_row(parameters, row)?;
    value(&self.expression, parameters, row)
  }
}

fn validate_node(
  node: &NlsExpression,
  depth: usize,
  nodes: &mut usize,
  parameters: &mut [bool],
  predictors: &mut [bool],
) -> Result<(), StatsError> {
  if depth > MAX_NLS_DEPTH || *nodes >= MAX_NLS_NODES {
    return Err(invalid("expression exceeds node/depth bounds"));
  }
  *nodes += 1;
  use NlsExpression::*;
  match node {
    Constant(v) if !v.is_finite() => return Err(invalid("numeric literals must be finite")),
    Constant(_) => {}
    Parameter(i) => {
      *parameters
        .get_mut(*i)
        .ok_or_else(|| invalid("parameter index out of bounds"))? = true
    }
    Predictor(i) => {
      *predictors
        .get_mut(*i)
        .ok_or_else(|| invalid("predictor index out of bounds"))? = true
    }
    Negate(a) | Exp(a) => validate_node(a, depth + 1, nodes, parameters, predictors)?,
    Add(a, b) | Subtract(a, b) | Multiply(a, b) | Divide(a, b) => {
      validate_node(a, depth + 1, nodes, parameters, predictors)?;
      validate_node(b, depth + 1, nodes, parameters, predictors)?;
    }
  }
  Ok(())
}

pub(super) fn jet(
  node: &NlsExpression,
  parameters: &[f64],
  row: &[f64],
) -> Result<Jet, StatsError> {
  use NlsExpression::*;
  let p = parameters.len();
  match node {
    Constant(v) => Ok(Jet::constant(*v, p)),
    Predictor(i) => Ok(Jet::constant(row[*i], p)),
    Parameter(i) => {
      let mut result = Jet::constant(parameters[*i], p);
      result.gradient[*i] = 1.0;
      Ok(result)
    }
    Negate(a) => {
      let a = jet(a, parameters, row)?;
      let mean = -a.mean;
      a.unary(mean, -1.0, 0.0)
    }
    Exp(a) => {
      let a = jet(a, parameters, row)?;
      let mean = exponential(a.mean)?;
      a.unary(mean, mean, mean)
    }
    Add(a, b) => jet(a, parameters, row)?.sum(jet(b, parameters, row)?, 1.0),
    Subtract(a, b) => jet(a, parameters, row)?.sum(jet(b, parameters, row)?, -1.0),
    Multiply(a, b) => jet(a, parameters, row)?.product(jet(b, parameters, row)?),
    Divide(a, b) => jet(a, parameters, row)?.product(jet(b, parameters, row)?.reciprocal()?),
  }
}

pub(super) fn value(
  node: &NlsExpression,
  parameters: &[f64],
  row: &[f64],
) -> Result<f64, StatsError> {
  use NlsExpression::*;
  let result = match node {
    Constant(v) => *v,
    Predictor(i) => row[*i],
    Parameter(i) => parameters[*i],
    Negate(a) => -value(a, parameters, row)?,
    Exp(a) => exponential(value(a, parameters, row)?)?,
    Add(a, b) => value(a, parameters, row)? + value(b, parameters, row)?,
    Subtract(a, b) => value(a, parameters, row)? - value(b, parameters, row)?,
    Multiply(a, b) => finite_product(value(a, parameters, row)?, value(b, parameters, row)?)?,
    Divide(a, b) => {
      let denominator = value(b, parameters, row)?;
      if denominator == 0.0 {
        return Err(failed("division by zero"));
      }
      finite_ratio(value(a, parameters, row)?, denominator)?
    }
  };
  if result.is_finite() {
    Ok(result)
  } else {
    Err(failed("nonrepresentable mean arithmetic"))
  }
}
