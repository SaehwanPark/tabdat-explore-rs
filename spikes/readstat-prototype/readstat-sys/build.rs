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

  // Verify before extraction on every build-script run, including cached source.
  let mut hash = Command::new(if cfg!(target_os = "macos") {
    "shasum"
  } else {
    "sha256sum"
  });
  if cfg!(target_os = "macos") {
    hash.args(["-a", "256"]);
  }
  let result = hash
    .arg(&tarball)
    .output()
    .expect("run source SHA-256 verification");
  assert!(result.status.success(), "source SHA-256 tool failed");
  assert_eq!(
    String::from_utf8(result.stdout)
      .expect("SHA-256 output is text")
      .split_whitespace()
      .next(),
    Some("7635c73dd04532dbfa785c0485dbd6d52d86bc23030a1227e4a5a77a83824b84"),
    "ReadStat source archive checksum mismatch"
  );

  println!("cargo:rerun-if-changed=vendor/readstat-1.0.0.tar.gz");
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=value_bridge.c");
  println!("cargo:rerun-if-env-changed=CC");
  println!("cargo:rerun-if-env-changed=AR");

  if source.exists() {
    remove_recursive(&source);
  }
  std::fs::create_dir_all(&source).expect("create ReadStat source directory");

  run(
    "tar",
    &[
      "--extract",
      "--gzip",
      "--file",
      tarball.to_str().unwrap(),
      "--strip-components=1",
      "--directory",
      source.to_str().unwrap(),
    ],
    &[],
    &manifest_dir,
  );

  patch_readstat_boundary(&source);

  let is_macos = cfg!(target_os = "macos");
  let mut configure_env: Vec<(&str, &str)> = vec![];
  if is_macos {
    // The configure probe does not add macOS's system iconv automatically.
    configure_env.push(("LIBS", "-liconv"));
  }
  run_with_env(
    "./configure",
    &["--disable-shared", "--enable-static"],
    &configure_env,
    &source,
  );

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
  assert!(
    libs.join("libreadstat.a").exists(),
    "static ReadStat library missing after build"
  );
  // Compile the aggregate ABI bridge against the exact extracted header;
  // no Rust declaration needs to model ReadStat's union, bitfields or padding.
  let object = out_dir.join("value_bridge.o");
  let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".into());
  run(
    &compiler,
    &[
      "-std=c99",
      "-Wall",
      "-Wextra",
      "-Werror",
      "-I",
      source.join("src").to_str().unwrap(),
      "-c",
      manifest_dir.join("value_bridge.c").to_str().unwrap(),
      "-o",
      object.to_str().unwrap(),
    ],
    &[],
    &manifest_dir,
  );
  let archiver = std::env::var("AR").unwrap_or_else(|_| "ar".into());
  run(
    &archiver,
    &[
      "crs",
      out_dir
        .join("libtabdat_readstat_bridge.a")
        .to_str()
        .unwrap(),
      object.to_str().unwrap(),
    ],
    &[],
    &manifest_dir,
  );
  println!("cargo:rustc-link-search=native={}", out_dir.display());
  println!("cargo:rustc-link-search=native={}", libs.display());
  println!("cargo:rustc-link-lib=static=tabdat_readstat_bridge");
  println!("cargo:rustc-link-lib=static=readstat");
  // ReadStat uses zlib for compressed DTA sections and iconv for text conversion.
  println!("cargo:rustc-link-lib=z");
  if is_macos {
    println!("cargo:rustc-link-lib=iconv");
  }
}

// Pinned label-table reads compared signed read() errors with unsigned lengths,
// allowing -1 to bypass the short-read check and use unread memory. Preserve EOF
// only at a table boundary; errors/truncated table payloads must return READ.
fn patch_readstat_boundary(source: &std::path::Path) {
  for (file, old, new) in [
    (
      "src/readstat_parser.c",
      "readstat_parser_t *parser = calloc(1, sizeof(readstat_parser_t));\n    parser->io = calloc(1, sizeof(readstat_io_t));",
      "readstat_parser_t *parser = calloc(1, sizeof(readstat_parser_t));\n    if (parser == NULL) return NULL;\n    parser->io = calloc(1, sizeof(readstat_io_t));\n    if (parser->io == NULL) { free(parser); return NULL; }",
    ),
    (
      "src/readstat_io_unistd.c",
      "io_ctx = calloc(1, sizeof(unistd_io_ctx_t));\n    io_ctx->fd = -1;",
      "io_ctx = calloc(1, sizeof(unistd_io_ctx_t));\n    if (io_ctx == NULL) return READSTAT_ERROR_MALLOC;\n    io_ctx->fd = -1;",
    ),
  ] {
    let path = source.join(file);
    let text = std::fs::read_to_string(&path).expect("read pinned parser source");
    assert_eq!(
      text.matches(old).count(),
      1,
      "parser allocation patch anchor changed"
    );
    std::fs::write(path, text.replacen(old, new, 1)).expect("write parser allocation patch");
  }
  let path = source.join("src/stata/readstat_dta_read.c");
  let mut text = std::fs::read_to_string(&path).expect("read pinned DTA source");
  for (old, new) in [
    (
      "if (dta_read_tag(ctx, \"<lbl>\") != READSTAT_OK) {\n                break;\n            }",
      "retval = dta_read_tag(ctx, \"<lbl>\");\n            if (retval == READSTAT_ERROR_PARSE) { retval = READSTAT_OK; break; }\n            if (retval != READSTAT_OK) goto cleanup;",
    ),
    (
      "if (io->seek(ctx->value_label_table_padding_len, READSTAT_SEEK_CUR, io->io_ctx) == -1)\n            break;",
      "if (io->seek(ctx->value_label_table_padding_len, READSTAT_SEEK_CUR, io->io_ctx) == -1) { retval = READSTAT_ERROR_SEEK; goto cleanup; }",
    ),
    (
      "if (io->read(&table_header_len, sizeof(int16_t), io->io_ctx) < sizeof(int16_t))\n                break;",
      "ssize_t got = io->read(&table_header_len, sizeof(int16_t), io->io_ctx);\n            if (got == 0) break;\n            if (got != (ssize_t)sizeof(int16_t)) { retval = READSTAT_ERROR_READ; goto cleanup; }",
    ),
    (
      "if (io->read(&table_header_len, sizeof(int32_t), io->io_ctx) < sizeof(int32_t))\n                break;",
      "ssize_t got = io->read(&table_header_len, sizeof(int32_t), io->io_ctx);\n            if (got == 0) break;\n            if (got != (ssize_t)sizeof(int32_t)) { retval = READSTAT_ERROR_READ; goto cleanup; }",
    ),
    (
      "if (io->read(labname, ctx->value_label_table_labname_len, io->io_ctx) < ctx->value_label_table_labname_len)\n            break;",
      "ssize_t got_name = io->read(labname, ctx->value_label_table_labname_len, io->io_ctx);\n        if (got_name < 0 || (size_t)got_name != ctx->value_label_table_labname_len) { retval = READSTAT_ERROR_READ; goto cleanup; }",
    ),
    (
      "if (io->read(table_buffer, len, io->io_ctx) < len) {\n            break;\n        }",
      "ssize_t got_table = io->read(table_buffer, len, io->io_ctx);\n        if (got_table < 0 || (size_t)got_table != len) { retval = READSTAT_ERROR_READ; goto cleanup; }",
    ),
  ] {
    assert_eq!(
      text.matches(old).count(),
      1,
      "DTA read-error patch anchor changed"
    );
    text = text.replacen(old, new, 1);
  }
  std::fs::write(path, text).expect("write DTA read-error patch");
}

fn run_with_env(program: &str, args: &[&str], env: &[(&str, &str)], dir: &PathBuf) {
  let mut command = Command::new(program);
  command.args(args).current_dir(dir);
  for (key, value) in env {
    command.env(key, value);
  }
  let status = command
    .status()
    .unwrap_or_else(|error| panic!("failed to run {program} in {}: {error}", dir.display()));
  assert!(status.success(), "{program} {} failed", args.join(" "));
}

fn run(program: &str, args: &[&str], env: &[(&str, &str)], dir: &PathBuf) {
  if env.is_empty() {
    let status = Command::new(program)
      .args(args)
      .current_dir(dir)
      .status()
      .unwrap_or_else(|error| panic!("failed to run {program} in {}: {error}", dir.display()));
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
