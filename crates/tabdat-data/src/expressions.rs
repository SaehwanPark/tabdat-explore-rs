use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionDomain {
  Numeric,
  String,
  Boolean,
  Null,
  Other,
}

pub fn expression_identifiers(expression: &AssertExpression) -> Vec<String> {
  match expression {
    AssertExpression::Identifier(name) => vec![name.clone()],
    AssertExpression::Number(_) | AssertExpression::String(_) | AssertExpression::Null => {
      Vec::new()
    }
    AssertExpression::UnaryMinus(operand) => expression_identifiers(operand),
    AssertExpression::Binary { left, right, .. } => {
      let mut identifiers = expression_identifiers(left);
      identifiers.extend(expression_identifiers(right));
      identifiers
    }
  }
}

pub fn generate_expression_identifiers(expression: &GenerateExpression) -> Vec<String> {
  match expression {
    GenerateExpression::Identifier(name) => vec![name.clone()],
    GenerateExpression::Number(_) | GenerateExpression::String(_) | GenerateExpression::Null => {
      Vec::new()
    }
    GenerateExpression::UnaryMinus(operand) => generate_expression_identifiers(operand),
    GenerateExpression::Binary { left, right, .. } => {
      let mut identifiers = generate_expression_identifiers(left);
      identifiers.extend(generate_expression_identifiers(right));
      identifiers
    }
    GenerateExpression::FunctionCall { arguments, .. } => arguments
      .iter()
      .flat_map(generate_expression_identifiers)
      .collect(),
  }
}

pub fn replace_expression_identifiers(expression: &GenerateExpression) -> Vec<String> {
  match expression {
    GenerateExpression::Identifier(name) => vec![name.clone()],
    GenerateExpression::Number(_) | GenerateExpression::String(_) | GenerateExpression::Null => {
      Vec::new()
    }
    GenerateExpression::UnaryMinus(operand) => replace_expression_identifiers(operand),
    GenerateExpression::Binary { left, right, .. } => {
      let mut identifiers = replace_expression_identifiers(left);
      identifiers.extend(replace_expression_identifiers(right));
      identifiers
    }
    GenerateExpression::FunctionCall { arguments, .. } => arguments
      .iter()
      .flat_map(replace_expression_identifiers)
      .collect(),
  }
}

pub fn replace_expression_to_assert(
  expression: &GenerateExpression,
) -> Result<AssertExpression, RuntimeError> {
  bounded_expression_to_assert(expression).ok_or_else(|| {
    RuntimeError::ReplaceUnsupportedExpression {
      message: "replace does not support function calls".to_owned(),
    }
  })
}

/// Share the installed AST subset without assigning a caller's diagnostic context.
pub fn bounded_expression_to_assert(expression: &GenerateExpression) -> Option<AssertExpression> {
  match expression {
    GenerateExpression::Identifier(name) => Some(AssertExpression::Identifier(name.clone())),
    GenerateExpression::Number(value) => Some(AssertExpression::Number(value.clone())),
    GenerateExpression::String(value) => Some(AssertExpression::String(value.clone())),
    GenerateExpression::Null => Some(AssertExpression::Null),
    GenerateExpression::UnaryMinus(operand) => Some(AssertExpression::UnaryMinus(Box::new(
      bounded_expression_to_assert(operand)?,
    ))),
    GenerateExpression::Binary {
      left,
      operator,
      right,
    } => {
      let operator = match operator {
        GenerateBinaryOperator::Add => AssertBinaryOperator::Add,
        GenerateBinaryOperator::Subtract => AssertBinaryOperator::Subtract,
        GenerateBinaryOperator::Multiply => AssertBinaryOperator::Multiply,
        GenerateBinaryOperator::Divide => AssertBinaryOperator::Divide,
        GenerateBinaryOperator::Equal => AssertBinaryOperator::Equal,
        GenerateBinaryOperator::NotEqual => AssertBinaryOperator::NotEqual,
        GenerateBinaryOperator::Less => AssertBinaryOperator::Less,
        GenerateBinaryOperator::LessOrEqual => AssertBinaryOperator::LessOrEqual,
        GenerateBinaryOperator::Greater => AssertBinaryOperator::Greater,
        GenerateBinaryOperator::GreaterOrEqual => AssertBinaryOperator::GreaterOrEqual,
      };
      Some(AssertExpression::Binary {
        left: Box::new(bounded_expression_to_assert(left)?),
        operator,
        right: Box::new(bounded_expression_to_assert(right)?),
      })
    }
    GenerateExpression::FunctionCall { .. } => None,
  }
}

pub fn map_replace_error(error: RuntimeError) -> RuntimeError {
  match error {
    RuntimeError::AssertUnknownVariable { variables } => {
      RuntimeError::ReplaceUnknownVariable { variables }
    }
    RuntimeError::AssertTypeMismatch { message } => RuntimeError::ReplaceTypeMismatch { message },
    other => other,
  }
}

pub fn generate_expression_to_assert(
  expression: &GenerateExpression,
) -> Result<AssertExpression, RuntimeError> {
  match expression {
    GenerateExpression::Identifier(name) => Ok(AssertExpression::Identifier(name.clone())),
    GenerateExpression::Number(value) => Ok(AssertExpression::Number(value.clone())),
    GenerateExpression::String(_) => Err(RuntimeError::GenerateUnsupportedExpression {
      message: "generate does not support string expressions".to_owned(),
    }),
    GenerateExpression::Null => Err(RuntimeError::GenerateUnsupportedExpression {
      message: "generate does not support NULL expressions".to_owned(),
    }),
    GenerateExpression::UnaryMinus(operand) => Ok(AssertExpression::UnaryMinus(Box::new(
      generate_expression_to_assert(operand)?,
    ))),
    GenerateExpression::Binary {
      left,
      operator,
      right,
    } => {
      let operator = match operator {
        GenerateBinaryOperator::Add => AssertBinaryOperator::Add,
        GenerateBinaryOperator::Subtract => AssertBinaryOperator::Subtract,
        GenerateBinaryOperator::Multiply => AssertBinaryOperator::Multiply,
        GenerateBinaryOperator::Divide => AssertBinaryOperator::Divide,
        GenerateBinaryOperator::Equal
        | GenerateBinaryOperator::NotEqual
        | GenerateBinaryOperator::Less
        | GenerateBinaryOperator::LessOrEqual
        | GenerateBinaryOperator::Greater
        | GenerateBinaryOperator::GreaterOrEqual => {
          return Err(RuntimeError::GenerateUnsupportedExpression {
            message: "generate does not support comparison expressions".to_owned(),
          });
        }
      };
      Ok(AssertExpression::Binary {
        left: Box::new(generate_expression_to_assert(left)?),
        operator,
        right: Box::new(generate_expression_to_assert(right)?),
      })
    }
    GenerateExpression::FunctionCall { .. } => Err(RuntimeError::GenerateUnsupportedExpression {
      message: "generate does not support function calls".to_owned(),
    }),
  }
}

pub fn validate_generate_expression(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> Result<(), RuntimeError> {
  match expression {
    AssertExpression::Identifier(name) => {
      let is_numeric = dataset
        .columns
        .iter()
        .find(|column| column.name == *name)
        .is_some_and(|column| is_numeric_data_type(&column.data_type));
      if !is_numeric {
        return Err(RuntimeError::GenerateTypeMismatch {
          message: "expression type mismatch: arithmetic requires numeric operands".to_owned(),
        });
      }
    }
    AssertExpression::Number(value) => {
      if !value.parse::<f64>().is_ok_and(f64::is_finite) {
        return Err(RuntimeError::GenerateTypeMismatch {
          message: "expression type mismatch: arithmetic requires numeric operands".to_owned(),
        });
      }
    }
    AssertExpression::UnaryMinus(operand) => validate_generate_expression(operand, dataset)?,
    AssertExpression::Binary { left, right, .. } => {
      validate_generate_expression(left, dataset)?;
      validate_generate_expression(right, dataset)?;
    }
    AssertExpression::String(_) | AssertExpression::Null => {
      return Err(RuntimeError::GenerateUnsupportedExpression {
        message: "generate does not support non-numeric expressions".to_owned(),
      });
    }
  }
  Ok(())
}

pub fn expression_domain(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> Result<ExpressionDomain, RuntimeError> {
  match expression {
    AssertExpression::Identifier(name) => dataset
      .columns
      .iter()
      .find(|column| column.name == *name)
      .map(|column| data_type_expression_domain(&column.data_type))
      .ok_or_else(|| RuntimeError::AssertUnknownVariable {
        variables: vec![name.clone()],
      }),
    AssertExpression::Number(_) => Ok(ExpressionDomain::Numeric),
    AssertExpression::UnaryMinus(operand) => {
      if expression_contains_unsigned_identifier(operand, dataset) {
        return Err(RuntimeError::AssertTypeMismatch {
          message:
            "expression type mismatch: unsigned numeric values do not support subtraction or unary minus"
              .to_owned(),
        });
      }
      let operand_domain = expression_domain(operand, dataset)?;
      if operand_domain == ExpressionDomain::Null {
        return Err(RuntimeError::AssertTypeMismatch {
          message: "null literal only supports equality and inequality comparisons".to_owned(),
        });
      }
      if operand_domain != ExpressionDomain::Numeric {
        return Err(RuntimeError::AssertTypeMismatch {
          message: "expression type mismatch: unary minus requires numeric operand".to_owned(),
        });
      }
      Ok(ExpressionDomain::Numeric)
    }
    AssertExpression::String(_) => Ok(ExpressionDomain::String),
    AssertExpression::Null => Ok(ExpressionDomain::Null),
    AssertExpression::Binary {
      left,
      operator,
      right,
    } => {
      if expression_has_unsafe_unsigned_arithmetic(expression, dataset) {
        return Err(RuntimeError::AssertTypeMismatch {
          message:
            "expression type mismatch: unsigned numeric values do not support subtraction or unary minus"
              .to_owned(),
        });
      }
      if expression_has_unsafe_unsigned_numeric_pair(expression, dataset) {
        return Err(RuntimeError::AssertTypeMismatch {
          message: "expression type mismatch: unsigned numeric values cannot be combined with negative numeric literals"
            .to_owned(),
        });
      }
      let left_domain = expression_domain(left, dataset)?;
      let right_domain = expression_domain(right, dataset)?;
      if operator.is_comparison() {
        if left_domain == ExpressionDomain::Null || right_domain == ExpressionDomain::Null {
          if !matches!(
            operator,
            AssertBinaryOperator::Equal | AssertBinaryOperator::NotEqual
          ) {
            return Err(RuntimeError::AssertTypeMismatch {
              message: "null literal only supports equality and inequality comparisons".to_owned(),
            });
          }
          return Ok(ExpressionDomain::Boolean);
        }
        let compatible = left_domain == right_domain
          || (left_domain == ExpressionDomain::Numeric
            && right_domain == ExpressionDomain::Numeric);
        if !compatible {
          return Err(RuntimeError::AssertTypeMismatch {
            message: format!(
              "expression type mismatch: cannot compare {} and {} values",
              expression_domain_name(left_domain),
              expression_domain_name(right_domain)
            ),
          });
        }
        return Ok(ExpressionDomain::Boolean);
      }
      if expression_contains_null_literal(left) || expression_contains_null_literal(right) {
        return Err(RuntimeError::AssertTypeMismatch {
          message: "null literal only supports equality and inequality comparisons".to_owned(),
        });
      }
      if left_domain != ExpressionDomain::Numeric || right_domain != ExpressionDomain::Numeric {
        return Err(RuntimeError::AssertTypeMismatch {
          message: "expression type mismatch: arithmetic requires numeric operands".to_owned(),
        });
      }
      Ok(ExpressionDomain::Numeric)
    }
  }
}

pub fn data_type_expression_domain(data_type: &str) -> ExpressionDomain {
  let normalized = data_type.trim().to_ascii_uppercase();
  let base = normalized.split('(').next().unwrap_or_default().trim();
  if is_numeric_data_type(data_type) {
    ExpressionDomain::Numeric
  } else if matches!(base, "BOOLEAN" | "BOOL") {
    ExpressionDomain::Boolean
  } else if matches!(base, "VARCHAR" | "TEXT" | "STRING" | "CHAR") {
    ExpressionDomain::String
  } else {
    ExpressionDomain::Other
  }
}

pub fn expression_domain_name(domain: ExpressionDomain) -> &'static str {
  match domain {
    ExpressionDomain::Numeric => "numeric",
    ExpressionDomain::String => "string",
    ExpressionDomain::Boolean => "boolean",
    ExpressionDomain::Null => "null",
    ExpressionDomain::Other => "unsupported",
  }
}

pub fn expression_contains_null_literal(expression: &AssertExpression) -> bool {
  match expression {
    AssertExpression::Null => true,
    AssertExpression::UnaryMinus(operand) => expression_contains_null_literal(operand),
    AssertExpression::Binary { left, right, .. } => {
      expression_contains_null_literal(left) || expression_contains_null_literal(right)
    }
    AssertExpression::Identifier(_) | AssertExpression::Number(_) | AssertExpression::String(_) => {
      false
    }
  }
}

pub fn assert_operator_sql_symbol(operator: AssertBinaryOperator) -> &'static str {
  match operator {
    AssertBinaryOperator::Add => "+",
    AssertBinaryOperator::Subtract => "-",
    AssertBinaryOperator::Multiply => "*",
    AssertBinaryOperator::Divide => "/",
    AssertBinaryOperator::Equal => "=",
    AssertBinaryOperator::NotEqual => "!=",
    AssertBinaryOperator::Less => "<",
    AssertBinaryOperator::LessOrEqual => "<=",
    AssertBinaryOperator::Greater => ">",
    AssertBinaryOperator::GreaterOrEqual => ">=",
  }
}

pub fn compile_assert_expression(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> Result<String, DataError> {
  match expression {
    AssertExpression::Binary {
      left,
      operator,
      right,
    } if operator.is_comparison() => {
      let left_is_null = matches!(left.as_ref(), AssertExpression::Null);
      let right_is_null = matches!(right.as_ref(), AssertExpression::Null);
      if left_is_null || right_is_null {
        if left_is_null && right_is_null {
          return Ok(if *operator == AssertBinaryOperator::Equal {
            "TRUE".to_owned()
          } else if *operator == AssertBinaryOperator::NotEqual {
            "FALSE".to_owned()
          } else {
            return Err(DataError);
          });
        }
        let other = if left_is_null { right } else { left };
        let other_sql = compile_assert_expression_operand(other, dataset)?;
        return Ok(format!(
          "({other_sql} {})",
          if *operator == AssertBinaryOperator::Equal {
            "IS NULL"
          } else if *operator == AssertBinaryOperator::NotEqual {
            "IS NOT NULL"
          } else {
            return Err(DataError);
          }
        ));
      }
      let left_sql = compile_assert_expression_operand(left, dataset)?;
      let right_sql = compile_assert_expression_operand(right, dataset)?;
      Ok(format!(
        "({left_sql} {} {right_sql})",
        assert_operator_sql_symbol(*operator)
      ))
    }
    _ if expression_is_numeric_result(expression) => {
      let raw = compile_assert_expression_raw(expression, dataset)?;
      Ok(safe_numeric_sql(&raw))
    }
    _ => compile_assert_expression_raw(expression, dataset),
  }
}

pub fn compile_assert_expression_operand(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> Result<String, DataError> {
  if expression_is_numeric_result(expression) {
    let raw = compile_assert_expression_raw(expression, dataset)?;
    Ok(safe_numeric_sql(&raw))
  } else {
    compile_assert_expression_raw(expression, dataset)
  }
}

pub fn compile_assert_expression_raw(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> Result<String, DataError> {
  match expression {
    AssertExpression::Identifier(name) => Ok(quote_identifier(name)),
    AssertExpression::Number(value) => {
      if value.parse::<f64>().is_err() {
        return Err(DataError);
      }
      Ok(value.clone())
    }
    AssertExpression::String(value) => Ok(format!("'{}'", value.replace('\'', "''"))),
    AssertExpression::Null => Ok("NULL".to_owned()),
    AssertExpression::UnaryMinus(operand) => {
      let mut operand_sql = compile_assert_expression_raw(operand, dataset)?;
      if expression_is_integral(expression, dataset) {
        operand_sql = cast_exact_integer_sql(&operand_sql);
      }
      Ok(format!("-({operand_sql})"))
    }
    AssertExpression::Binary { operator, .. } if operator.is_comparison() => {
      compile_assert_expression(expression, dataset)
    }
    AssertExpression::Binary {
      left,
      operator,
      right,
    } => {
      let mut left_sql = compile_assert_expression_raw(left, dataset)?;
      let mut right_sql = compile_assert_expression_raw(right, dataset)?;
      if expression_is_integral(expression, dataset) {
        left_sql = cast_exact_integer_sql(&left_sql);
        right_sql = cast_exact_integer_sql(&right_sql);
      }
      Ok(format!(
        "({left_sql} {} {right_sql})",
        assert_operator_sql_symbol(*operator)
      ))
    }
  }
}

pub fn is_numeric_data_type(data_type: &str) -> bool {
  let normalized = data_type.trim().to_ascii_uppercase();
  let base = normalized.split('(').next().unwrap_or_default().trim();
  matches!(
    base,
    "TINYINT"
      | "SMALLINT"
      | "INTEGER"
      | "BIGINT"
      | "HUGEINT"
      | "UHUGEINT"
      | "UTINYINT"
      | "USMALLINT"
      | "UINTEGER"
      | "UBIGINT"
      | "INT8"
      | "INT16"
      | "INT32"
      | "INT64"
      | "UINT8"
      | "UINT16"
      | "UINT32"
      | "UINT64"
      | "UINT128"
      | "FLOAT32"
      | "FLOAT64"
      | "FLOAT"
      | "REAL"
      | "DOUBLE"
      | "DECIMAL"
  )
}

pub fn is_integer_data_type(data_type: &str) -> bool {
  let normalized = data_type.trim().to_ascii_uppercase();
  let base = normalized.split('(').next().unwrap_or_default().trim();
  matches!(
    base,
    "TINYINT"
      | "SMALLINT"
      | "INTEGER"
      | "BIGINT"
      | "HUGEINT"
      | "UHUGEINT"
      | "UTINYINT"
      | "USMALLINT"
      | "UINTEGER"
      | "UBIGINT"
      | "INT8"
      | "INT16"
      | "INT32"
      | "INT64"
      | "UINT8"
      | "UINT16"
      | "UINT32"
      | "UINT64"
      | "UINT128"
  )
}

pub fn is_unsigned_data_type(data_type: &str) -> bool {
  let normalized = data_type.trim().to_ascii_uppercase();
  let base = normalized.split('(').next().unwrap_or_default().trim();
  matches!(
    base,
    "UTINYINT"
      | "USMALLINT"
      | "UINTEGER"
      | "UBIGINT"
      | "UHUGEINT"
      | "UINT8"
      | "UINT16"
      | "UINT32"
      | "UINT64"
      | "UINT128"
  )
}

pub fn expression_contains_unsigned_identifier(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> bool {
  match expression {
    AssertExpression::Identifier(name) => dataset
      .columns
      .iter()
      .any(|column| column.name == *name && is_unsigned_data_type(&column.data_type)),
    AssertExpression::UnaryMinus(operand) => {
      expression_contains_unsigned_identifier(operand, dataset)
    }
    AssertExpression::Binary { left, right, .. } => {
      expression_contains_unsigned_identifier(left, dataset)
        || expression_contains_unsigned_identifier(right, dataset)
    }
    AssertExpression::Number(_) | AssertExpression::String(_) | AssertExpression::Null => false,
  }
}

pub fn expression_has_unsafe_unsigned_arithmetic(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> bool {
  match expression {
    AssertExpression::UnaryMinus(operand) => {
      expression_contains_unsigned_identifier(operand, dataset)
    }
    AssertExpression::Binary {
      left,
      operator,
      right,
    } => {
      (*operator == AssertBinaryOperator::Subtract
        && (expression_contains_unsigned_identifier(left, dataset)
          || expression_contains_unsigned_identifier(right, dataset)))
        || expression_has_unsafe_unsigned_arithmetic(left, dataset)
        || expression_has_unsafe_unsigned_arithmetic(right, dataset)
    }
    AssertExpression::Identifier(_)
    | AssertExpression::Number(_)
    | AssertExpression::String(_)
    | AssertExpression::Null => false,
  }
}

pub fn expression_contains_negative_numeric_literal(expression: &AssertExpression) -> bool {
  match expression {
    AssertExpression::Number(value) => value.parse::<f64>().is_ok_and(|value| value < 0.0),
    AssertExpression::UnaryMinus(operand) => matches!(
      operand.as_ref(),
      AssertExpression::Number(value) if value.parse::<f64>().is_ok_and(|value| value != 0.0)
    ),
    AssertExpression::Identifier(_)
    | AssertExpression::String(_)
    | AssertExpression::Null
    | AssertExpression::Binary { .. } => false,
  }
}

pub fn expression_has_unsafe_unsigned_numeric_pair(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> bool {
  let AssertExpression::Binary { left, right, .. } = expression else {
    return false;
  };
  (expression_contains_unsigned_identifier(left, dataset)
    && expression_contains_negative_numeric_literal(right))
    || (expression_contains_unsigned_identifier(right, dataset)
      && expression_contains_negative_numeric_literal(left))
}

pub fn expression_is_integral(expression: &AssertExpression, dataset: &DatasetInfo) -> bool {
  match expression {
    AssertExpression::Identifier(name) => dataset
      .columns
      .iter()
      .any(|column| column.name == *name && is_integer_data_type(&column.data_type)),
    AssertExpression::Number(value) => !value.contains('.'),
    AssertExpression::UnaryMinus(operand) => expression_is_integral(operand, dataset),
    AssertExpression::Binary {
      left,
      operator,
      right,
    } => {
      matches!(
        operator,
        AssertBinaryOperator::Add | AssertBinaryOperator::Subtract | AssertBinaryOperator::Multiply
      ) && expression_is_integral(left, dataset)
        && expression_is_integral(right, dataset)
    }
    AssertExpression::String(_) | AssertExpression::Null => false,
  }
}

pub fn expression_is_numeric_result(expression: &AssertExpression) -> bool {
  matches!(
    expression,
    AssertExpression::UnaryMinus(_)
      | AssertExpression::Binary {
        operator: AssertBinaryOperator::Add
          | AssertBinaryOperator::Subtract
          | AssertBinaryOperator::Multiply
          | AssertBinaryOperator::Divide,
        ..
      }
  )
}

pub fn cast_exact_integer_sql(expression: &str) -> String {
  format!("CAST({expression} AS DECIMAL(38,0))")
}

pub fn safe_numeric_sql(expression: &str) -> String {
  format!(
    "(SELECT CASE WHEN isfinite(CAST(__tabdat_numeric_value AS DOUBLE)) THEN __tabdat_numeric_value ELSE NULL END FROM (SELECT try({expression}) AS __tabdat_numeric_value) AS __tabdat_numeric_result)"
  )
}

pub fn finite_float_sql_literal(value: f64) -> Result<String, DataError> {
  if !value.is_finite() {
    return Err(DataError);
  }
  // Rust's shortest-round-trip representation preserves the fitted coefficient as a double.
  Ok(value.to_string())
}

pub fn quote_identifier(identifier: &str) -> String {
  format!("\"{}\"", identifier.replace('"', "\"\""))
}

pub fn quote_sql_literal(value: &str) -> String {
  format!("'{}'", value.replace('\'', "''"))
}

pub fn collapse_statistic_name(statistic: CollapseStatistic) -> &'static str {
  match statistic {
    CollapseStatistic::Count => "count",
    CollapseStatistic::Mean => "mean",
    CollapseStatistic::Sum => "sum",
    CollapseStatistic::Min => "min",
    CollapseStatistic::Max => "max",
  }
}

pub fn quote_recode_literal(value: &str) -> String {
  format!("'{}'", value.replace('\'', "''"))
}

pub fn validate_recode_number(value: &str) -> Result<(), DataError> {
  value
    .parse::<f64>()
    .is_ok_and(f64::is_finite)
    .ok_or(DataError)
}

pub fn compile_recode_case(source: &ColumnInfo, rules: &[RecodeRule]) -> Result<String, DataError> {
  let source_is_numeric = is_numeric_data_type(&source.data_type);
  let output_is_text = !source_is_numeric
    || rules
      .iter()
      .any(|rule| matches!(&rule.output, RecodeValue::Text(_)));
  let source_sql = quote_identifier(&source.name);
  let mut clauses = Vec::new();
  let mut else_sql = None;

  for rule in rules {
    let mut conditions = Vec::new();
    let mut is_else = false;
    for input in &rule.inputs {
      match input {
        RecodeInput::Value(value) => {
          conditions.push(compile_recode_value_condition(
            &source_sql,
            source_is_numeric,
            value,
          )?);
        }
        RecodeInput::Range { start, end } => {
          if !source_is_numeric {
            return Err(DataError);
          }
          conditions.push(compile_recode_range_condition(&source_sql, start, end)?);
        }
        RecodeInput::Missing => conditions.push(format!("{source_sql} IS NULL")),
        RecodeInput::NonMissing => conditions.push(format!("{source_sql} IS NOT NULL")),
        RecodeInput::Else => is_else = true,
      }
    }
    let output_sql = compile_recode_output(&rule.output, output_is_text)?;
    if is_else {
      else_sql = Some(output_sql);
    } else if !conditions.is_empty() {
      let condition_sql = conditions
        .into_iter()
        .map(|condition| format!("({condition})"))
        .collect::<Vec<_>>()
        .join(" OR ");
      clauses.push(format!("WHEN {condition_sql} THEN {output_sql}"));
    }
  }

  let fallback_sql = else_sql.unwrap_or_else(|| {
    if output_is_text {
      format!("CAST({source_sql} AS VARCHAR)")
    } else {
      source_sql.clone()
    }
  });
  if clauses.is_empty() {
    return Ok(fallback_sql);
  }
  Ok(format!(
    "CASE {} ELSE {fallback_sql} END",
    clauses.join(" ")
  ))
}

pub fn compile_recode_value_condition(
  source_sql: &str,
  source_is_numeric: bool,
  value: &RecodeValue,
) -> Result<String, DataError> {
  match value {
    RecodeValue::Number(value) => {
      validate_recode_number(value)?;
      if source_is_numeric {
        Ok(format!("{source_sql} = {value}"))
      } else {
        Ok(format!(
          "CAST({source_sql} AS VARCHAR) = {}",
          quote_recode_literal(value)
        ))
      }
    }
    RecodeValue::Text(value) => Ok(format!(
      "CAST({source_sql} AS VARCHAR) = {}",
      quote_recode_literal(value)
    )),
  }
}

pub fn compile_recode_range_condition(
  source_sql: &str,
  start: &RecodeRangeEndpoint,
  end: &RecodeRangeEndpoint,
) -> Result<String, DataError> {
  let start_sql = recode_range_endpoint_sql(start)?;
  let end_sql = recode_range_endpoint_sql(end)?;
  match (start_sql, end_sql) {
    (None, None) => Ok(format!("{source_sql} IS NOT NULL")),
    (None, Some(end)) => Ok(format!("{source_sql} <= {end}")),
    (Some(start), None) => Ok(format!("{source_sql} >= {start}")),
    (Some(start), Some(end)) => Ok(format!("{source_sql} >= {start} AND {source_sql} <= {end}")),
  }
}

pub fn recode_range_endpoint_sql(
  endpoint: &RecodeRangeEndpoint,
) -> Result<Option<String>, DataError> {
  match endpoint {
    RecodeRangeEndpoint::Min => Ok(None),
    RecodeRangeEndpoint::Max => Ok(None),
    RecodeRangeEndpoint::Number(value) => {
      validate_recode_number(value)?;
      Ok(Some(value.clone()))
    }
  }
}

pub fn compile_recode_output(
  value: &RecodeValue,
  output_is_text: bool,
) -> Result<String, DataError> {
  match value {
    RecodeValue::Number(value) => {
      validate_recode_number(value)?;
      if output_is_text {
        Ok(quote_recode_literal(value))
      } else {
        Ok(value.clone())
      }
    }
    RecodeValue::Text(value) => Ok(quote_recode_literal(value)),
  }
}
