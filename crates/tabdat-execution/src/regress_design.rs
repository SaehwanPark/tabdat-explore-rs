use crate::*;

/// One parsed `regress` predictor term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RegressTerm {
  Continuous {
    name: String,
  },
  Factor {
    variable: String,
    base: Option<String>,
  },
}

impl RegressTerm {
  pub(crate) fn variable(&self) -> &str {
    match self {
      Self::Continuous { name } => name,
      Self::Factor { variable, .. } => variable,
    }
  }
}

/// One explicitly categorical observation after type-preserving extraction.
///
/// The exact source label is retained as level identity, so wide integer or
/// DECIMAL codes that collapse to the same `f64` remain distinct categories.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ObservedFactor {
  Number { label: String, value: f64 },
  Text(String),
  Boolean(bool),
}

impl ObservedFactor {
  pub(crate) fn from_cell(cell: &CellValue) -> Result<Option<Self>, RuntimeError> {
    Ok(match cell {
      CellValue::Null => None,
      CellValue::Boolean(value) => Some(Self::Boolean(*value)),
      CellValue::Text(value) => Some(Self::Text(value.clone())),
      CellValue::SignedInteger(value) => Some(Self::Number {
        label: value.to_string(),
        value: *value as f64,
      }),
      CellValue::UnsignedInteger(value) => Some(Self::Number {
        label: value.to_string(),
        value: *value as f64,
      }),
      CellValue::Decimal { value, scale, .. } => cell_to_f64(cell)
        .filter(|number| number.is_finite())
        .map(|number| Self::Number {
          label: decimal_label(*value, *scale),
          value: number,
        }),
      CellValue::Float(value) => value.is_finite().then(|| Self::Number {
        label: format_factor_number(*value),
        value: *value,
      }),
      CellValue::Bytes(_) => {
        return Err(RuntimeError::RegressFailed {
          message: "factor variables must be text, boolean, or finite numeric codes".into(),
        });
      }
    })
  }

  /// Stable level identity. Exact labels keep adjacent wide codes distinct.
  fn key(&self) -> String {
    match self {
      Self::Number { label, .. } => label.clone(),
      Self::Text(value) => format!("t{value}"),
      Self::Boolean(value) => format!("b{value}"),
    }
  }

  fn label(&self) -> String {
    match self {
      Self::Number { label, .. } => label.clone(),
      Self::Text(value) => value.clone(),
      Self::Boolean(value) => value.to_string(),
    }
  }

  fn same_level(&self, other: &Self) -> bool {
    self.key() == other.key()
  }

  fn as_number(&self) -> Option<f64> {
    match self {
      Self::Number { value, .. } => Some(*value),
      _ => None,
    }
  }
}

fn factor_level_from_observed(value: &ObservedFactor) -> LinearFactorLevel {
  match value {
    ObservedFactor::Number { label, value } => LinearFactorLevel::Number {
      label: label.clone(),
      value: *value,
    },
    ObservedFactor::Text(value) => LinearFactorLevel::Text {
      label: value.clone(),
    },
    ObservedFactor::Boolean(value) => LinearFactorLevel::Boolean {
      label: value.to_string(),
      value: *value,
    },
  }
}

/// Whether any retained design term is an explicitly categorical factor.
pub(crate) fn has_factor_terms(design_terms: &[LinearDesignTerm]) -> bool {
  design_terms
    .iter()
    .any(|term| matches!(term, LinearDesignTerm::Factor { .. }))
}

/// Parse the bounded inline factor syntax accepted by `regress`.
pub(crate) fn parse_regress_terms(predictors: &[String]) -> Result<Vec<RegressTerm>, RuntimeError> {
  predictors
    .iter()
    .map(|predictor| parse_regress_term(predictor))
    .collect()
}

fn parse_regress_term(predictor: &str) -> Result<RegressTerm, RuntimeError> {
  if let Some(variable) = predictor.strip_prefix("i.") {
    validate_factor_variable(variable)?;
    return Ok(RegressTerm::Factor {
      variable: variable.to_owned(),
      base: None,
    });
  }
  if let Some(rest) = predictor.strip_prefix("ib(") {
    let Some(close) = rest.find(").") else {
      return Err(RuntimeError::RegressFailed {
        message: "malformed factor term; expected ib(level).variable".into(),
      });
    };
    let base = rest[..close].trim();
    if base.is_empty() {
      return Err(RuntimeError::RegressFailed {
        message: "factor term ib(level).variable requires a reference level".into(),
      });
    }
    let variable = &rest[close + 2..];
    validate_factor_variable(variable)?;
    return Ok(RegressTerm::Factor {
      variable: variable.to_owned(),
      base: Some(unquote_factor_base(base)),
    });
  }
  Ok(RegressTerm::Continuous {
    name: predictor.to_owned(),
  })
}

fn validate_factor_variable(variable: &str) -> Result<(), RuntimeError> {
  let mut characters = variable.chars();
  let Some(first) = characters.next() else {
    return Err(RuntimeError::RegressFailed {
      message: "factor term requires a variable after the dot".into(),
    });
  };
  if !(first.is_alphabetic() || first == '_')
    || !characters.all(|character| character.is_alphanumeric() || character == '_')
  {
    return Err(RuntimeError::RegressFailed {
      message: format!("factor variable name '{variable}' is not a plain identifier"),
    });
  }
  Ok(())
}

fn unquote_factor_base(base: &str) -> String {
  let base = base.trim();
  for quote in ['\'', '"', '`'] {
    if base.len() >= 2 && base.starts_with(quote) && base.ends_with(quote) {
      let inner = &base[quote.len_utf8()..base.len() - quote.len_utf8()];
      return inner.replace(&format!("{quote}{quote}"), &quote.to_string());
    }
  }
  base.to_owned()
}

fn format_factor_number(value: f64) -> String {
  if value == 0.0 {
    "0".into()
  } else {
    value.to_string()
  }
}

/// Canonical exact text for a scaled DECIMAL code: `(120, 2) -> "1.2"`.
fn decimal_label(value: i128, scale: u8) -> String {
  let digits = value.unsigned_abs().to_string();
  let scale = usize::from(scale);
  let mut text = if scale == 0 {
    digits
  } else {
    let padded = if digits.len() <= scale {
      format!("{}{digits}", "0".repeat(scale + 1 - digits.len()))
    } else {
      digits
    };
    let split = padded.len() - scale;
    format!("{}.{}", &padded[..split], &padded[split..])
  };
  if text.contains('.') {
    while text.ends_with('0') {
      text.pop();
    }
    if text.ends_with('.') {
      text.pop();
    }
  }
  if value < 0 {
    text.insert(0, '-');
  }
  text
}

/// Expanded ordered design after factor levels are learned from the retained sample.
#[derive(Debug)]
pub(crate) struct ExpandedDesign {
  pub parameter_names: Vec<String>,
  pub design_matrix: Vec<Vec<f64>>,
  pub design_terms: Vec<LinearDesignTerm>,
}

/// Expand raw retained term values into treatment-coded columns. Levels are
/// learned only from the same finite sample used by the fit, so excluded rows
/// cannot create empty dummy columns.
pub(crate) fn expand_design(
  terms: &[RegressTerm],
  raw_terms: &[Vec<ObservedTermValue>],
  include_intercept: bool,
) -> Result<ExpandedDesign, RuntimeError> {
  let mut used_names: std::collections::HashSet<String> = terms
    .iter()
    .filter_map(|term| match term {
      RegressTerm::Continuous { name } => Some(name.clone()),
      RegressTerm::Factor { .. } => None,
    })
    .collect();
  let mut parameter_names = Vec::new();
  let mut design_terms = Vec::new();
  let mut factor_plans: Vec<Option<FactorPlan>> = Vec::with_capacity(terms.len());

  for (term_index, term) in terms.iter().enumerate() {
    match term {
      RegressTerm::Continuous { name } => {
        parameter_names.push(name.clone());
        design_terms.push(LinearDesignTerm::Continuous { name: name.clone() });
        factor_plans.push(None);
      }
      RegressTerm::Factor { variable, base } => {
        let values = raw_terms
          .iter()
          .map(|row| match &row[term_index] {
            ObservedTermValue::Factor(value) => Ok(value.clone()),
            ObservedTermValue::Continuous(_) => Err(RuntimeError::RegressFailed {
              message: "internal factor design mismatch".into(),
            }),
          })
          .collect::<Result<Vec<_>, _>>()?;
        let levels = sorted_factor_levels(&values);
        let reference = select_reference(variable, base.as_deref(), &levels)?;
        if include_intercept && levels.len() < 2 {
          return Err(RuntimeError::RegressFailed {
            message: format!(
              "factor variable '{variable}' has only one level in the estimation sample; a reference contrast requires at least two"
            ),
          });
        }
        if !include_intercept && base.is_some() {
          return Err(RuntimeError::RegressFailed {
            message: format!(
              "factor variable '{variable}' cannot use ib(level) with noconstant; use i.{variable} to include every level"
            ),
          });
        }
        // Without an intercept no level is omitted, so no reference is recorded.
        let reference_label = include_intercept.then(|| levels[reference].label());
        let mut included = Vec::new();
        for (index, level) in levels.iter().enumerate() {
          if include_intercept && index == reference {
            continue;
          }
          let mut name = factor_parameter_name(variable, &level.label());
          let base_name = name.clone();
          let mut suffix = 2usize;
          while !used_names.insert(name.clone()) {
            name = format!("{base_name}_{suffix}");
            suffix += 1;
          }
          parameter_names.push(name);
          included.push(level.clone());
        }
        design_terms.push(LinearDesignTerm::Factor {
          variable: variable.clone(),
          reference: reference_label,
          levels: levels.iter().map(factor_level_from_observed).collect(),
        });
        factor_plans.push(Some(FactorPlan { included }));
      }
    }
  }

  let mut design_matrix = Vec::with_capacity(raw_terms.len());
  for row in raw_terms {
    let mut design_row = Vec::with_capacity(parameter_names.len());
    for (index, value) in row.iter().enumerate() {
      match value {
        ObservedTermValue::Continuous(value) => design_row.push(*value),
        ObservedTermValue::Factor(value) => {
          let plan = factor_plans[index]
            .as_ref()
            .ok_or_else(|| RuntimeError::RegressFailed {
              message: "internal factor design mismatch".into(),
            })?;
          for level in &plan.included {
            design_row.push(f64::from(value.same_level(level)));
          }
        }
      }
    }
    design_matrix.push(design_row);
  }

  Ok(ExpandedDesign {
    parameter_names,
    design_matrix,
    design_terms,
  })
}

/// One retained raw term value before dummy expansion.
#[derive(Debug, Clone)]
pub(crate) enum ObservedTermValue {
  Continuous(f64),
  Factor(ObservedFactor),
}

/// Expand one current-data row into the retained design columns, in the same
/// order as the fitted parameter names. Missing cells and levels unseen at fit
/// time yield `None` (NULL predictions) rather than inventing a category.
pub(crate) fn expand_current_row(
  design_terms: &[LinearDesignTerm],
  include_intercept: bool,
  cells: &[&CellValue],
) -> Result<Option<Vec<f64>>, RuntimeError> {
  if design_terms.len() != cells.len() {
    return Err(RuntimeError::PredictFailed);
  }
  let mut columns = Vec::new();
  for (term, cell) in design_terms.iter().zip(cells) {
    match term {
      LinearDesignTerm::Continuous { .. } => {
        let Some(value) = cell_to_f64(cell).filter(|value| value.is_finite()) else {
          return Ok(None);
        };
        columns.push(value);
      }
      LinearDesignTerm::Factor {
        reference, levels, ..
      } => {
        let Some(observed) = ObservedFactor::from_cell(cell)? else {
          return Ok(None);
        };
        let Some(position) = levels
          .iter()
          .position(|level| same_level_as_observed(level, &observed))
        else {
          // A level unseen at fit time has no retained contrast; predict NULL.
          return Ok(None);
        };
        let reference_position = include_intercept
          .then(|| {
            reference.as_ref().and_then(|reference| {
              levels
                .iter()
                .position(|level| level.label() == reference.as_str())
            })
          })
          .flatten();
        if include_intercept && reference_position.is_none() {
          // A retained intercept design must name its omitted level.
          return Err(RuntimeError::PredictFailed);
        }
        for (index, _) in levels.iter().enumerate() {
          if Some(index) == reference_position {
            continue;
          }
          columns.push(f64::from(index == position));
        }
      }
    }
  }
  Ok(Some(columns))
}

/// Compare a retained report level with a freshly read cell value.
pub(crate) fn same_level_as_observed(level: &LinearFactorLevel, observed: &ObservedFactor) -> bool {
  match (level, observed) {
    (LinearFactorLevel::Number { label, .. }, ObservedFactor::Number { label: other, .. }) => {
      label == other
    }
    (LinearFactorLevel::Text { label }, ObservedFactor::Text(other)) => label == other,
    (LinearFactorLevel::Boolean { value, .. }, ObservedFactor::Boolean(other)) => value == other,
    _ => false,
  }
}

struct FactorPlan {
  included: Vec<ObservedFactor>,
}

fn sorted_factor_levels(values: &[ObservedFactor]) -> Vec<ObservedFactor> {
  let mut seen = std::collections::HashSet::new();
  let mut levels = Vec::new();
  for value in values {
    if seen.insert(value.key()) {
      levels.push(value.clone());
    }
  }
  // A column has one physical type, so mixed-type levels are only a defensive
  // fallback; text labels sort before numeric codes there.
  levels.sort_by(|left, right| match (left.as_number(), right.as_number()) {
    (Some(left), Some(right)) => left
      .partial_cmp(&right)
      .unwrap_or(std::cmp::Ordering::Equal)
      .then_with(|| left.to_string().cmp(&right.to_string())),
    (None, Some(_)) => std::cmp::Ordering::Less,
    (Some(_), None) => std::cmp::Ordering::Greater,
    (None, None) => left.label().cmp(&right.label()),
  });
  levels
}

/// Resolve the omitted reference level, accepting an exact label or a numeric
/// spelling such as `ib(1.20)` for a DECIMAL `1.2` level.
fn select_reference(
  variable: &str,
  base: Option<&str>,
  levels: &[ObservedFactor],
) -> Result<usize, RuntimeError> {
  let Some(base) = base else {
    return Ok(0);
  };
  if let Some(index) = levels
    .iter()
    .position(|level| level.label().as_str() == base)
  {
    return Ok(index);
  }
  if let Ok(number) = base.parse::<f64>()
    && let Some(index) = levels
      .iter()
      .position(|level| level.as_number().is_some_and(|value| value == number))
  {
    return Ok(index);
  }
  match base.to_ascii_lowercase().as_str() {
    "true"
      if levels
        .iter()
        .any(|level| matches!(level, ObservedFactor::Boolean(true))) =>
    {
      Ok(
        levels
          .iter()
          .position(|level| matches!(level, ObservedFactor::Boolean(true)))
          .expect("checked above"),
      )
    }
    "false"
      if levels
        .iter()
        .any(|level| matches!(level, ObservedFactor::Boolean(false))) =>
    {
      Ok(
        levels
          .iter()
          .position(|level| matches!(level, ObservedFactor::Boolean(false)))
          .expect("checked above"),
      )
    }
    _ => Err(RuntimeError::RegressFailed {
      message: format!(
        "reference level '{base}' for factor variable '{variable}' is not present in the estimation sample"
      ),
    }),
  }
}

fn factor_parameter_name(variable: &str, label: &str) -> String {
  let variable = sanitize_parameter_part(variable);
  let label = sanitize_parameter_part(label);
  // `sanitize_parameter_part` prefixes labels that would start a column name
  // with a digit; avoid doubling that separator (`region__1`).
  match label.strip_prefix('_') {
    Some(rest) => format!("{variable}_{rest}"),
    None => format!("{variable}_{label}"),
  }
}

fn sanitize_parameter_part(value: &str) -> String {
  let mut output = String::new();
  for character in value.chars() {
    if character.is_alphanumeric() || character == '_' {
      output.push(character);
    } else {
      output.push('_');
    }
  }
  if output.is_empty() {
    output.push_str("level");
  }
  if !output
    .chars()
    .next()
    .is_some_and(|character| character.is_alphabetic() || character == '_')
  {
    output.insert(0, '_');
  }
  output
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_inline_factor_terms() {
    assert_eq!(
      parse_regress_terms(&[
        "x".into(),
        "i.region".into(),
        "ib(North East).region".into(),
        "ib(2).code".into(),
      ])
      .unwrap(),
      vec![
        RegressTerm::Continuous { name: "x".into() },
        RegressTerm::Factor {
          variable: "region".into(),
          base: None
        },
        RegressTerm::Factor {
          variable: "region".into(),
          base: Some("North East".into())
        },
        RegressTerm::Factor {
          variable: "code".into(),
          base: Some("2".into())
        },
      ]
    );
  }

  #[test]
  fn factor_names_are_sanitized_and_disambiguated() {
    assert_eq!(
      factor_parameter_name("region", "North East"),
      "region_North_East"
    );
    assert_eq!(factor_parameter_name("region", ""), "region_level");
    assert_eq!(factor_parameter_name("region", "1"), "region_1");
    assert_eq!(factor_parameter_name("my region", "A-B"), "my_region_A_B");
  }

  #[test]
  fn numeric_factor_labels_do_not_expose_negative_zero() {
    assert_eq!(format_factor_number(-0.0), "0");
    assert_eq!(format_factor_number(2.5), "2.5");
  }

  #[test]
  fn decimal_labels_are_exact_and_canonical() {
    assert_eq!(decimal_label(120, 2), "1.2");
    assert_eq!(decimal_label(-120, 2), "-1.2");
    assert_eq!(decimal_label(100, 2), "1");
    assert_eq!(decimal_label(5, 0), "5");
    assert_eq!(decimal_label(5, 3), "0.005");
    assert_eq!(decimal_label(0, 4), "0");
  }

  #[test]
  fn wide_codes_stay_distinct_levels_after_f64_rounding() {
    let level = |value| {
      ObservedFactor::from_cell(&CellValue::Decimal {
        width: 38,
        scale: 0,
        value,
      })
      .unwrap()
      .unwrap()
    };
    let right = level(9_223_372_036_854_775_808);
    let left = level(9_223_372_036_854_775_809);
    assert_eq!(left.as_number(), right.as_number());
    assert!(!left.same_level(&right));
    assert_eq!(sorted_factor_levels(&[right, left]).len(), 2);
  }

  fn observed(values: &[ObservedFactor]) -> Vec<Vec<ObservedTermValue>> {
    values
      .iter()
      .map(|value| vec![ObservedTermValue::Factor(value.clone())])
      .collect()
  }

  fn text(value: &str) -> ObservedFactor {
    ObservedFactor::Text(value.into())
  }

  #[test]
  fn expansion_omits_the_sorted_reference_level_and_keeps_stable_names() {
    let terms = parse_regress_terms(&["i.region".into()]).unwrap();
    let raw = observed(&[text("West"), text("East"), text("West")]);
    let design = expand_design(&terms, &raw, true).unwrap();
    assert_eq!(design.parameter_names, ["region_West"]);
    assert_eq!(design.design_matrix, [[1.0], [0.0], [1.0]]);
    assert_eq!(
      design.design_terms,
      [LinearDesignTerm::Factor {
        variable: "region".into(),
        reference: Some("East".into()),
        levels: vec![
          LinearFactorLevel::Text {
            label: "East".into()
          },
          LinearFactorLevel::Text {
            label: "West".into()
          },
        ],
      }]
    );
  }

  #[test]
  fn explicit_reference_reorders_contrasts_and_noconstant_keeps_every_level() {
    let terms = parse_regress_terms(&["ib(West).region".into()]).unwrap();
    let raw = observed(&[text("West"), text("East")]);
    let design = expand_design(&terms, &raw, true).unwrap();
    assert_eq!(design.parameter_names, ["region_East"]);
    assert_eq!(design.design_matrix, [[0.0], [1.0]]);

    let error = expand_design(&terms, &raw, false).unwrap_err();
    assert!(
      error
        .to_string()
        .contains("cannot use ib(level) with noconstant"),
      "{error}"
    );

    let terms = parse_regress_terms(&["i.region".into()]).unwrap();
    let design = expand_design(&terms, &raw, false).unwrap();
    assert_eq!(design.parameter_names, ["region_East", "region_West"]);
    assert_eq!(design.design_matrix, [[0.0, 1.0], [1.0, 0.0]]);
    assert_eq!(
      design.design_terms,
      [LinearDesignTerm::Factor {
        variable: "region".into(),
        reference: None,
        levels: vec![
          LinearFactorLevel::Text {
            label: "East".into()
          },
          LinearFactorLevel::Text {
            label: "West".into()
          },
        ],
      }]
    );
  }

  #[test]
  fn single_level_contrast_and_unknown_reference_fail_explicitly() {
    let terms = parse_regress_terms(&["i.region".into()]).unwrap();
    let error = expand_design(&terms, &observed(&[text("East")]), true).unwrap_err();
    assert!(error.to_string().contains("only one level"), "{error}");

    let terms = parse_regress_terms(&["ib(North).region".into()]).unwrap();
    let error = expand_design(&terms, &observed(&[text("East")]), true).unwrap_err();
    assert!(error.to_string().contains("'North'"), "{error}");
  }

  #[test]
  fn numeric_and_boolean_codes_sort_and_resolve_references() {
    let terms = parse_regress_terms(&["ib(true).flag".into(), "i.code".into()]).unwrap();
    let raw = vec![
      vec![
        ObservedTermValue::Factor(ObservedFactor::Boolean(true)),
        ObservedTermValue::Factor(ObservedFactor::Number {
          label: "10".into(),
          value: 10.0,
        }),
      ],
      vec![
        ObservedTermValue::Factor(ObservedFactor::Boolean(false)),
        ObservedTermValue::Factor(ObservedFactor::Number {
          label: "2".into(),
          value: 2.0,
        }),
      ],
    ];
    let design = expand_design(&terms, &raw, true).unwrap();
    assert_eq!(design.parameter_names, ["flag_false", "code_10"]);
    assert_eq!(design.design_matrix, [[0.0, 1.0], [1.0, 0.0]]);
  }

  #[test]
  fn prediction_expansion_reuses_retained_contrast_order() {
    let terms = parse_regress_terms(&["i.region".into()]).unwrap();
    let raw = observed(&[text("West"), text("East"), text("North")]);
    let design = expand_design(&terms, &raw, true).unwrap();
    assert_eq!(design.parameter_names, ["region_North", "region_West"]);
    let cell = |value: &str| CellValue::Text(value.to_owned());
    assert_eq!(
      expand_current_row(&design.design_terms, true, &[&cell("East")]).unwrap(),
      Some(vec![0.0, 0.0])
    );
    assert_eq!(
      expand_current_row(&design.design_terms, true, &[&cell("North")]).unwrap(),
      Some(vec![1.0, 0.0])
    );
    assert_eq!(
      expand_current_row(&design.design_terms, true, &[&cell("West")]).unwrap(),
      Some(vec![0.0, 1.0])
    );
    // A level first seen after the fit has no retained contrast.
    assert_eq!(
      expand_current_row(&design.design_terms, true, &[&cell("South")]).unwrap(),
      None
    );
    assert_eq!(
      expand_current_row(&design.design_terms, true, &[&CellValue::Null]).unwrap(),
      None
    );
    // Mixed physical types cannot masquerade as a retained text level.
    assert_eq!(
      expand_current_row(&design.design_terms, true, &[&CellValue::SignedInteger(1)]).unwrap(),
      None
    );
  }

  #[test]
  fn generated_names_never_shadow_continuous_or_earlier_columns() {
    let terms = parse_regress_terms(&["a_b_x".into(), "i.a_b".into(), "i.a".into()]).unwrap();
    let raw = vec![
      vec![
        ObservedTermValue::Continuous(1.0),
        ObservedTermValue::Factor(text("m")),
        ObservedTermValue::Factor(text("a")),
      ],
      vec![
        ObservedTermValue::Continuous(2.0),
        ObservedTermValue::Factor(text("x")),
        ObservedTermValue::Factor(text("b_x")),
      ],
    ];
    let design = expand_design(&terms, &raw, true).unwrap();
    assert_eq!(
      design.parameter_names,
      ["a_b_x", "a_b_x_2", "a_b_x_3"],
      "continuous names are reserved before factor columns are generated"
    );
    assert_eq!(design.design_matrix, [[1.0, 0.0, 0.0], [2.0, 1.0, 1.0]]);
  }
}
