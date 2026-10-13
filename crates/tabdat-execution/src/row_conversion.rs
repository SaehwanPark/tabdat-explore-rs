use crate::*;

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
