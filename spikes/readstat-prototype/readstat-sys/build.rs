use std::path::PathBuf;
use std::process::Command;

/// Builds the pinned ReadStat v1.0.0 release into a static library.
///
/// The release tarball is vendored and hash-pinned in
/// `docs/feasibility/readstat.md`; this script reproduces the documented
/// platform workarounds (macOS `-liconv` / `-Wno-strict-prototypes`, Linux
/// warning demotions) so the spike builds identically on both targets.
fn main() {
  let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
  let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
  let tarball = manifest_dir.join("vendor/readstat-1.0.0.tar.gz");
  let source = out_dir.join("readstat");

  println!("cargo:rerun-if-changed=vendor/readstat-1.0.0.tar.gz");
  println!("cargo:rerun-if-changed=build.rs");

  if source.exists() {
    remove_recursive(&source);
  }
  std::fs::create_dir_all(&source).expect("create ReadStat source directory");

  run(
    "tar",
    &["--extract", "--gzip", "--file", tarball.to_str().unwrap(), "--strip-components=1", "--directory", source.to_str().unwrap()],
    &[],
    &manifest_dir,
  );

  let is_macos = cfg!(target_os = "macos");
  let mut configure_env: Vec<(&str, &str)> = vec![];
  if is_macos {
    // The configure probe does not add macOS's system iconv automatically.
    configure_env.push(("LIBS", "-liconv"));
  }
  run_with_env("./configure", &["--disable-shared", "--enable-static"], &configure_env, &source);

  // ReadStat ships `-Werror -pedantic-errors`; demote only the diagnostics the
  // pinned release triggers on each platform's compiler (see feasibility report).
  let cflags = if is_macos {
    // `-Wno-implicit-const-int-float-conversion` is required by newer Apple
    // Clang releases for `src/sas/readstat_sas.c` (see feasibility report).
    "-Wno-strict-prototypes -Wno-implicit-const-int-float-conversion"
  } else {
    "-Wno-error=stringop-truncation -Wno-error=use-after-free -Wno-error=format-truncation"
  };
  let jobs = std::thread::available_parallelism()
    .map(|n| n.to_string())
    .unwrap_or_else(|_| "2".to_string());
  run(
    "make",
    &["-j", &jobs, "libreadstat.la", &format!("CFLAGS={cflags}")],
    &[],
    &source,
  );

  let libs = source.join(".libs");
  assert!(libs.join("libreadstat.a").exists(), "static ReadStat library missing after build");
  println!("cargo:rustc-link-search=native={}", libs.display());
  println!("cargo:rustc-link-lib=static=readstat");
  // ReadStat uses zlib for compressed DTA sections and iconv for text conversion.
  println!("cargo:rustc-link-lib=z");
  if is_macos {
    println!("cargo:rustc-link-lib=iconv");
  }
}

fn run_with_env(program: &str, args: &[&str], env: &[(&str, &str)], dir: &PathBuf) {
  let mut command = Command::new(program);
  command.args(args).current_dir(dir);
  for (key, value) in env {
    command.env(key, value);
  }
  let status = command.status().unwrap_or_else(|error| {
    panic!("failed to run {program} in {}: {error}", dir.display())
  });
  assert!(status.success(), "{program} {} failed", args.join(" "));
}

fn run(program: &str, args: &[&str], env: &[(&str, &str)], dir: &PathBuf) {
  if env.is_empty() {
    let status = Command::new(program)
      .args(args)
      .current_dir(dir)
      .status()
      .unwrap_or_else(|error| {
        panic!("failed to run {program} in {}: {error}", dir.display())
      });
    assert!(status.success(), "{program} {} failed", args.join(" "));
  } else {
    run_with_env(program, args, env, dir);
  }
}

fn remove_recursive(path: &PathBuf) {
  let entry = std::fs::read_dir(path).expect("read source directory");
  for child in entry {
    let child = child.unwrap().path();
    if child.is_dir() {
      remove_recursive(&child);
    } else {
      std::fs::remove_file(&child).expect("remove file");
    }
  }
  std::fs::remove_dir(path).expect("remove source directory");
}
