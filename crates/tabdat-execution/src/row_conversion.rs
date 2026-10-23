use crate::*;

/// Check original integer/decimal identity before f64 rounding can turn an
/// invalid numeric code into a permitted one. Callers own exclusion policy.
pub(crate) fn exact_integer_code(cell: &CellValue) -> Option<i64> {
  let value = match cell {
    CellValue::Float(v) => return tabdat_stats::multinomial_outcome_code(*v).ok(),
    CellValue::SignedInteger(v) => i64::try_from(*v).ok()?,
    CellValue::UnsignedInteger(v) => i64::try_from(*v).ok()?,
    CellValue::Decimal { value, scale, .. } => {
      let divisor = 10_i128.checked_pow(u32::from(*scale))?;
      if value % divisor != 0 {
        return None;
      }
      i64::try_from(value / divisor).ok()?
    }
    _ => return None,
  };
  (value.unsigned_abs() <= 9_007_199_254_740_991).then_some(value)
}

pub(crate) fn cluster_key(cell: &CellValue) -> Result<Option<String>, &'static str> {
  Ok(match cell {
    CellValue::Null => None,
    CellValue::Float(value) if !value.is_finite() => None,
    CellValue::Float(value) => Some(if *value == 0.0 {
      "0".to_owned()
    } else {
      value.to_string()
    }),
    CellValue::Text(value) => Some(value.clone()),
    CellValue::SignedInteger(value) => Some(value.to_string()),
    CellValue::UnsignedInteger(value) => Some(value.to_string()),
    // Never round cluster identity through f64: adjacent wide DECIMAL IDs
    // can otherwise collapse into a single group. Scale is fixed per column.
    CellValue::Decimal { value, scale, .. } => Some(format!("{value}e-{scale}")),
    CellValue::Boolean(value) => Some(value.to_string()),
    CellValue::Bytes(_) => return Err("cluster variable must be numeric, text or boolean"),
  })
}

pub(crate) fn cell_to_f64(cell: &CellValue) -> Option<f64> {
  match cell {
    CellValue::Float(f) => Some(*f),
    CellValue::SignedInteger(i) => Some(*i as f64),
    CellValue::UnsignedInteger(u) => Some(*u as f64),
    CellValue::Decimal { value, scale, .. } => {
      let divisor = 10f64.powi(*scale as i32);
      Some(*value as f64 / divisor)
    }
    _ => None,
  }
}

pub(crate) fn read_owned_cell<'a>(
  row: &'a DataRow,
  index: usize,
  regression: bool,
  unsupported: &str,
) -> Result<&'a CellValue, RuntimeError> {
  row.cell(index).map_err(|error| {
    let message = match error {
      CellReadError::Backend(message) => message,
      CellReadError::Unsupported | CellReadError::InvalidIndex => unsupported.to_owned(),
    };
    if regression {
      RuntimeError::RegressFailed { message }
    } else {
      RuntimeError::TtestFailed { message }
    }
  })
}

pub(crate) fn read_owned_text(
  row: &DataRow,
  index: usize,
  message: &str,
) -> Result<String, RuntimeError> {
  match read_owned_cell(row, index, false, message)? {
    CellValue::Text(text) => Ok(text.clone()),
    _ => Err(RuntimeError::TtestFailed {
      message: message.to_owned(),
    }),
  }
}
