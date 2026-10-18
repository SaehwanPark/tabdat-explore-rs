//! Atomic no-clobber publication of already generated SVG, not an SVG input API.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use tabdat_domain::RuntimeError;

static STAGE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct OwnedStage(PathBuf);
impl Drop for OwnedStage {
  fn drop(&mut self) {
    let _ = std::fs::remove_file(&self.0);
  }
}

fn failure(path: &Path, message: impl Into<String>) -> RuntimeError {
  RuntimeError::PlotWriteFailed {
    path: path.to_owned(),
    message: message.into(),
  }
}

pub fn validate_svg_path(path: &Path) -> Result<(), RuntimeError> {
  if !path
    .extension()
    .and_then(|e| e.to_str())
    .is_some_and(|e| e.eq_ignore_ascii_case("svg"))
  {
    return Err(failure(
      path,
      "only .svg files are supported; choose a new SVG path",
    ));
  }
  Ok(())
}

fn create_stage(parent: &Path, target: &Path) -> Result<(OwnedStage, File), RuntimeError> {
  for _ in 0..100 {
    let serial = STAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let path = parent.join(format!(".tabdat-svg-{}-{serial}.tmp", std::process::id()));
    match OpenOptions::new().write(true).create_new(true).open(&path) {
      Ok(file) => return Ok((OwnedStage(path), file)),
      Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
      Err(e) => {
        return Err(failure(
          target,
          format!("could not create sibling stage: {e}"),
        ));
      }
    }
  }
  Err(failure(
    target,
    "could not allocate an exclusive sibling stage",
  ))
}

/// Render before calling. A sibling stage is fully written and closed before
/// hard-link publication, which is atomic and never replaces any existing entry
/// (including a symlink). This requires a local hard-link-capable filesystem;
/// it is not a crash-durability guarantee. Cleanup owns only the created stage.
pub fn publish_svg(path: &Path, svg: &str) -> Result<(), RuntimeError> {
  validate_svg_path(path)?;
  let parent = path
    .parent()
    .filter(|p| !p.as_os_str().is_empty())
    .unwrap_or(Path::new("."));
  std::fs::create_dir_all(parent)
    .map_err(|e| failure(path, format!("could not create parent directories: {e}")))?;
  let (stage, mut file) = create_stage(parent, path)?;
  file
    .write_all(svg.as_bytes())
    .and_then(|()| file.flush())
    .map_err(|e| failure(path, format!("could not write SVG stage: {e}")))?;
  drop(file);
  std::fs::hard_link(&stage.0, path).map_err(|e| {
    if e.kind() == std::io::ErrorKind::AlreadyExists {
      failure(path, "target already exists; choose a new SVG path")
    } else {
      failure(
        path,
        format!("could not publish SVG; a local hard-link-capable filesystem is required: {e}"),
      )
    }
  })?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  struct Directory(PathBuf);
  impl Directory {
    fn new() -> Self {
      let path = std::env::temp_dir().join(format!(
        "tabdat-svg-io-{}-{}",
        std::process::id(),
        STAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
      ));
      std::fs::create_dir(&path).unwrap();
      Self(path)
    }
    fn entries(&self) -> Vec<PathBuf> {
      std::fs::read_dir(&self.0)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect()
    }
  }
  impl Drop for Directory {
    fn drop(&mut self) {
      let _ = std::fs::remove_dir_all(&self.0);
    }
  }

  #[test]
  fn publishes_complete_bytes_preserves_existing_entries_and_cleans_stages() {
    let directory = Directory::new();
    let path = directory.0.join("figure.svg");
    publish_svg(&path, "<svg>complete</svg>").unwrap();
    assert_eq!(
      std::fs::read_to_string(&path).unwrap(),
      "<svg>complete</svg>"
    );
    assert!(
      publish_svg(&path, "replacement")
        .unwrap_err()
        .to_string()
        .contains("already exists")
    );
    assert_eq!(
      std::fs::read_to_string(&path).unwrap(),
      "<svg>complete</svg>"
    );
    assert_eq!(directory.entries(), [path]);
    let existing = directory.0.join("folder.svg");
    std::fs::create_dir(&existing).unwrap();
    assert!(publish_svg(&existing, "replacement").is_err());
    assert!(existing.is_dir());
    assert_eq!(directory.entries().len(), 2);
    assert!(publish_svg(&directory.0.join("image.png"), "no").is_err());
    assert_eq!(directory.entries().len(), 2);
  }

  #[test]
  fn concurrent_publishers_cannot_replace_one_another_or_leave_stages() {
    let directory = Directory::new();
    let path = directory.0.join("figure.svg");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles: Vec<_> = ["<svg>first</svg>", "<svg>second</svg>"]
      .into_iter()
      .map(|content| {
        let path = path.clone();
        let barrier = barrier.clone();
        std::thread::spawn(move || {
          barrier.wait();
          publish_svg(&path, content)
        })
      })
      .collect();
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert!(
      ["<svg>first</svg>", "<svg>second</svg>"]
        .contains(&std::fs::read_to_string(&path).unwrap().as_str())
    );
    assert_eq!(directory.entries(), [path]);
  }

  #[test]
  fn parent_failure_does_not_touch_existing_parent_file() {
    let directory = Directory::new();
    let parent = directory.0.join("not-a-directory");
    std::fs::write(&parent, "original").unwrap();
    assert!(publish_svg(&parent.join("out.svg"), "no").is_err());
    assert_eq!(std::fs::read_to_string(&parent).unwrap(), "original");
    assert_eq!(directory.entries(), [parent]);
  }

  #[cfg(unix)]
  #[test]
  fn existing_and_dangling_symlinks_are_not_replaced_or_followed() {
    let directory = Directory::new();
    let original = directory.0.join("original");
    std::fs::write(&original, "unchanged").unwrap();
    let link = directory.0.join("link.svg");
    std::os::unix::fs::symlink(&original, &link).unwrap();
    assert!(publish_svg(&link, "no").is_err());
    assert!(
      std::fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink()
    );
    assert_eq!(std::fs::read_to_string(&original).unwrap(), "unchanged");
    std::fs::remove_file(&original).unwrap();
    assert!(publish_svg(&link, "no").is_err());
    assert!(
      std::fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink()
    );
    assert_eq!(directory.entries(), [link]);
  }
}
