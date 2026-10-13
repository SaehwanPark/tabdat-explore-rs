use std::path::Path;
use tabdat_domain::RuntimeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
  Parquet,
  Csv,
}

/// Validate format/replacement and create parents before backend persistence.
/// This retains the existing save/export contract (not atomic file publication).
pub fn prepare_output_path(
  path: &Path,
  replace: bool,
  format: OutputFormat,
) -> Result<(), RuntimeError> {
  let extension = match format {
    OutputFormat::Parquet => "parquet",
    OutputFormat::Csv => "csv",
  };
  let error = |stage| {
    let path = path.to_owned();
    match (format, stage) {
      (OutputFormat::Parquet, OutputStage::Format) => RuntimeError::SaveUnsupportedFormat { path },
      (OutputFormat::Parquet, OutputStage::Exists) => RuntimeError::SaveTargetExists { path },
      (OutputFormat::Parquet, OutputStage::NotFile) => RuntimeError::SaveTargetNotAFile { path },
      (OutputFormat::Parquet, OutputStage::Parent) => RuntimeError::SaveParentFailed { path },
      (OutputFormat::Csv, OutputStage::Format) => RuntimeError::ExportUnsupportedFormat { path },
      (OutputFormat::Csv, OutputStage::Exists) => RuntimeError::ExportTargetExists { path },
      (OutputFormat::Csv, OutputStage::NotFile) => RuntimeError::ExportTargetNotAFile { path },
      (OutputFormat::Csv, OutputStage::Parent) => RuntimeError::ExportParentFailed { path },
    }
  };
  if !path
    .extension()
    .and_then(|value| value.to_str())
    .is_some_and(|value| value.eq_ignore_ascii_case(extension))
  {
    return Err(error(OutputStage::Format));
  }
  if path.exists() {
    if !replace {
      return Err(error(OutputStage::Exists));
    }
    if !path.is_file() {
      return Err(error(OutputStage::NotFile));
    }
  }
  if let Some(parent) = path
    .parent()
    .filter(|parent| !parent.as_os_str().is_empty())
  {
    std::fs::create_dir_all(parent).map_err(|_| error(OutputStage::Parent))?;
  }
  Ok(())
}

#[derive(Clone, Copy)]
enum OutputStage {
  Format,
  Exists,
  NotFile,
  Parent,
}
