use crate::*;

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
