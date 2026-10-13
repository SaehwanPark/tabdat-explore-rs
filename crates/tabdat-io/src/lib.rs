#![forbid(unsafe_code)]
//! Local filesystem and script-path effects, returning owned contracts.

mod output;
pub use output::{OutputFormat, prepare_output_path};

use std::path::{Path, PathBuf};
use tabdat_domain::{RuntimeError, ScriptError};

pub fn resolve_script_path(path: &Path, base_dir: Option<&Path>) -> PathBuf {
  let path_str = path.to_string_lossy();
  let clean_str = path_str.trim_matches(['"', '\'']);
  let clean_path = Path::new(clean_str);
  let candidate = if clean_path.is_absolute() {
    clean_path.to_path_buf()
  } else if let Some(dir) = base_dir {
    dir.join(clean_path)
  } else {
    std::env::current_dir()
      .unwrap_or_else(|_| PathBuf::from("."))
      .join(clean_path)
  };
  if let Ok(canonical) = candidate.canonicalize() {
    canonical
  } else {
    normalize_path(&candidate)
  }
}

fn normalize_path(path: &Path) -> PathBuf {
  use std::path::Component;
  let mut components = Vec::new();
  for component in path.components() {
    match component {
      Component::CurDir => {}
      Component::ParentDir => {
        if let Some(Component::Normal(_)) = components.last() {
          components.pop();
        } else {
          components.push(component);
        }
      }
      c => components.push(c),
    }
  }
  components.into_iter().collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalInputFormat {
  Parquet,
  Csv,
}

pub fn local_input_format(path: &Path) -> Result<LocalInputFormat, RuntimeError> {
  match path.extension().and_then(|extension| extension.to_str()) {
    Some(extension) if extension.eq_ignore_ascii_case("parquet") => Ok(LocalInputFormat::Parquet),
    Some(extension) if extension.eq_ignore_ascii_case("csv") => Ok(LocalInputFormat::Csv),
    _ => Err(RuntimeError::UnsupportedFormat {
      path: path.to_owned(),
    }),
  }
}

pub fn validate_local_input_path(path: &Path) -> Result<(), RuntimeError> {
  if !path.exists() {
    return Err(RuntimeError::FileNotFound {
      path: path.to_owned(),
    });
  }
  if !path.is_file() {
    return Err(RuntimeError::NotAFile {
      path: path.to_owned(),
    });
  }
  Ok(())
}

pub fn read_script_text(path: &Path) -> Result<String, ScriptError> {
  if !path.exists() {
    return Err(ScriptError::file(path, 1, "script file not found"));
  }
  if path.is_dir() {
    return Err(ScriptError::file(path, 1, "script path is a directory"));
  }
  let bytes = match std::fs::read(path) {
    Ok(b) => b,
    Err(e) => {
      if e.kind() == std::io::ErrorKind::NotFound {
        return Err(ScriptError::file(path, 1, "script file not found"));
      }
      return Err(ScriptError::file(
        path,
        1,
        format!("could not read script: {e}"),
      ));
    }
  };
  let text = match std::str::from_utf8(&bytes) {
    Ok(s) => s,
    Err(e) => {
      let valid_up_to = e.valid_up_to();
      let line = bytes[..valid_up_to].iter().filter(|&&b| b == b'\n').count() + 1;
      return Err(ScriptError::file(
        path,
        line,
        "script file must be UTF-8 text",
      ));
    }
  };
  Ok(text.to_owned())
}
