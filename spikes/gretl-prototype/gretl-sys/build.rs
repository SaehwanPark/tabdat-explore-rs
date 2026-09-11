//! Builds the pinned gretl 2026b `libgretl` static library and the C
//! ownership shim, then emits the link flags for the Rust crate.
//!
//! Two paths:
//! - If `GRETL_PREBUILT_DIR` is set and contains `libgretl-1.0.a`, use it
//!   directly (fast local iteration; the library must have been built with the
//!   documented recipe).
//! - Otherwise, download the pinned release archive (SHA-256 verified), apply
//!   the documented `monte_carlo.c` compatibility patch, configure, and build
//!   the static library from source.
//!
//! The from-source path is what CI exercises; it is slow (minutes) but
//! reproducible.

use std::path::{Path, PathBuf};
use std::process::Command;

const GRETL_VERSION: &str = "2026b";
const GRETL_URL: &str =
  "https://downloads.sourceforge.net/project/gretl/gretl/2026b/gretl-2026b.tar.xz";
const GRETL_SHA256: &str = "fb57f4922da546067c8be542aafc5a26be77faf2668b40e651ee8c90b702563d";

fn main() {
  let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
  let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
  let is_macos = cfg!(target_os = "macos");

  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=shim.c");

  let libgretl_a = build_libgretl(&out_dir, is_macos);
  let shim_a = build_shim(&manifest_dir, &out_dir, is_macos);

  // Emit link flags. Order matters for static symbol resolution: the shim
  // references libgretl symbols, so the shim archive must precede libgretl.
  let shim_dir = shim_a.parent().unwrap();
  println!("cargo:rustc-link-search=native={}", shim_dir.display());
  println!("cargo:rustc-link-lib=static=shim");

  let lib_dir = libgretl_a.parent().unwrap();
  println!("cargo:rustc-link-search=native={}", lib_dir.display());
  println!("cargo:rustc-link-lib=static=gretl-1.0");

  emit_dependency_flags(is_macos);
}

/// Return the path to `libgretl-1.0.a`, building it if necessary.
///
/// The source is always downloaded and extracted (the C shim needs the gretl
/// headers to compile). If `GRETL_PREBUILT_DIR` provides a prebuilt
/// `libgretl-1.0.a`, the configure/make build is skipped.
fn build_libgretl(out_dir: &Path, is_macos: bool) -> PathBuf {
  let src = out_dir.join("gretl-src");
  let tarball = out_dir.join(format!("gretl-{GRETL_VERSION}.tar.xz"));

  if !tarball.exists() {
    download_and_verify(&tarball);
  }
  extract(&tarball, &src);
  patch_monte_carlo(&src);

  // Fast path: prebuilt library (skip configure/make).
  if let Ok(prebuilt) = std::env::var("GRETL_PREBUILT_DIR") {
    let lib = Path::new(&prebuilt).join("libgretl-1.0.a");
    if lib.exists() {
      eprintln!("build.rs: using prebuilt libgretl at {}", lib.display());
      return lib;
    }
    panic!("GRETL_PREBUILT_DIR set but {} not found", lib.display());
  }

  configure(&src, is_macos);
  make_buildstamp(&src);
  make_lib(&src, is_macos);

  let lib = src.join("lib/.libs/libgretl-1.0.a");
  assert!(
    lib.exists(),
    "libgretl-1.0.a missing after build: {}",
    lib.display()
  );
  lib
}

fn download_and_verify(tarball: &Path) {
  eprintln!("build.rs: downloading gretl {GRETL_VERSION} ...");
  let out = tarball.to_str().unwrap();
  let dir = tarball.parent().unwrap();
  // SourceForge mirrors can stall; bound the attempt and retry.
  run(
    "curl",
    &[
      "--fail",
      "--silent",
      "--show-error",
      "--location",
      "--connect-timeout",
      "30",
      "--max-time",
      "600",
      "--retry",
      "5",
      "--retry-delay",
      "5",
      GRETL_URL,
      "--output",
      out,
    ],
    dir,
  );
  verify_sha256(tarball, GRETL_SHA256);
}

fn verify_sha256(path: &Path, expected: &str) {
  let is_macos = cfg!(target_os = "macos");
  let (tool, args) = if is_macos {
    ("shasum", vec!["-a", "256", path.to_str().unwrap()])
  } else {
    ("sha256sum", vec![path.to_str().unwrap()])
  };
  let out = Command::new(tool)
    .args(&args)
    .output()
    .expect("failed to run hash tool");
  let stdout = String::from_utf8_lossy(&out.stdout);
  let actual = stdout.split_whitespace().next().unwrap_or("");
  assert_eq!(
    actual, expected,
    "gretl archive SHA-256 mismatch: got {actual}, expected {expected}"
  );
  eprintln!("build.rs: SHA-256 verified: {actual}");
}

fn extract(tarball: &Path, dest: &Path) {
  if dest.exists() {
    remove_recursive(dest);
  }
  std::fs::create_dir_all(dest).expect("create gretl source dir");
  let file = tarball.to_str().unwrap();
  let directory = dest.to_str().unwrap();
  let dir = dest.parent().unwrap();
  run(
    "tar",
    &[
      "--extract",
      "--xz",
      "--file",
      file,
      "--strip-components=1",
      "--directory",
      directory,
    ],
    dir,
  );
}

/// Apply the documented compatibility patch for gretl 2026b: `monte_carlo.c`
/// references `prog_cmd_started` (only a macro in `prog_loop.c`), which breaks
/// a clean static-library link. Add the missing macro definition.
fn patch_monte_carlo(src: &Path) {
  let path = src.join("lib/src/monte_carlo.c");
  let s = std::fs::read_to_string(&path).expect("read monte_carlo.c");
  if s.contains("#define prog_cmd_started") {
    return; // already patched
  }
  let anchor = "#define loop_line_quiet(ll)  (ll->flags & LOOP_LINE_QUIET)";
  assert!(s.contains(anchor), "monte_carlo.c patch anchor not found");
  let patched = s.replacen(
    anchor,
    &format!("{anchor}\n#define prog_cmd_started(l,j) (l->lines[j].flags & LOOP_LINE_PDONE)"),
    1,
  );
  std::fs::write(&path, patched).expect("write patched monte_carlo.c");
  eprintln!("build.rs: applied monte_carlo.c compatibility patch");
}

fn configure(src: &Path, is_macos: bool) {
  let args = vec![
    "--disable-gui",
    "--disable-json",
    "--disable-nls",
    "--disable-build-addons",
    "--disable-xdg-utils",
    "--disable-sse2",
    "--disable-avx",
    "--disable-shared",
    "--enable-static",
  ];
  let mut env: Vec<(&str, &str)> = vec![];
  if is_macos {
    env.push(("OMP_LIB", "-L/opt/homebrew/opt/libomp/lib -lomp"));
    env.push((
      "LDFLAGS",
      "-L/opt/homebrew/lib -L/opt/homebrew/opt/libomp/lib",
    ));
    env.push((
      "PKG_CONFIG_PATH",
      "/opt/homebrew/opt/libxml2/lib/pkgconfig:/opt/homebrew/opt/glib/lib/pkgconfig:/opt/homebrew/lib/pkgconfig",
    ));
  }
  run_with_env("./configure", &args, &env, src);
}

fn make_buildstamp(src: &Path) {
  run("make", &["buildstamp"], src);
}

fn make_lib(src: &Path, is_macos: bool) {
  let jobs = std::thread::available_parallelism()
    .map(|n| n.to_string())
    .unwrap_or_else(|_| "2".to_string());
  let cflags = if is_macos {
    "-g -O2 -I/opt/homebrew/include -I/opt/homebrew/opt/libomp/include -Xclang -fopenmp"
  } else {
    "-g -O2 -fopenmp"
  };
  let jobs_arg = format!("-j{jobs}");
  let cflags_arg = format!("CFLAGS={cflags}");
  let args = vec!["-C", "lib", &jobs_arg, &cflags_arg];
  run("make", &args, src);
}

/// Compile the C ownership shim into a static archive (libshim.a).
///
/// The shim includes `libgretl.h`, which pulls in glib, libxml2, and other
/// dependency headers. Those live in platform-specific subdirectories, so the
/// include flags are obtained from `pkg-config` (the same mechanism gretl's
/// configure uses) rather than hard-coded paths.
fn build_shim(manifest_dir: &Path, out_dir: &Path, is_macos: bool) -> PathBuf {
  let shim_c = manifest_dir.join("shim.c");
  let shim_obj = out_dir.join("shim.o");
  let src_include = out_dir.join("gretl-src/lib/src");
  let gretl_root = out_dir.join("gretl-src");

  let shim_c_s = shim_c.to_str().unwrap();
  let shim_obj_s = shim_obj.to_str().unwrap();
  let src_include_s = src_include.to_str().unwrap();
  let gretl_root_s = gretl_root.to_str().unwrap();

  let mut args = vec![
    "-c",
    shim_c_s,
    "-o",
    shim_obj_s,
    "-I",
    src_include_s,
    "-I",
    gretl_root_s,
  ];

  // Dependency include flags via pkg-config (glib, libxml2, fftw, gmp, mpfr).
  let dep_cflags = pkg_config_cflags(
    &["glib-2.0", "libxml-2.0", "fftw3", "gmp", "mpfr"],
    is_macos,
  );
  for flag in dep_cflags.split_whitespace() {
    args.push(flag);
  }

  if is_macos {
    args.extend([
      "-I/opt/homebrew/include",
      "-I/opt/homebrew/opt/libomp/include",
    ]);
  } else {
    args.push("-fopenmp");
  }
  run("cc", &args, manifest_dir);

  // Archive the shim object so it links as a standard static library.
  let shim_a = out_dir.join("libshim.a");
  let shim_a_s = shim_a.to_str().unwrap();
  let shim_obj_s2 = shim_obj.to_str().unwrap();
  run("ar", &["cru", shim_a_s, shim_obj_s2], out_dir);
  shim_a
}

/// Run `pkg-config --cflags` for the given packages, returning the flags.
///
/// On macOS, Homebrew's pkgconfig files are not on the default search path, so
/// `PKG_CONFIG_PATH` is set to the Homebrew opt directories.
fn pkg_config_cflags(packages: &[&str], is_macos: bool) -> String {
  let mut command = Command::new("pkg-config");
  command.arg("--cflags");
  for p in packages {
    command.arg(p);
  }
  if is_macos {
    command.env(
      "PKG_CONFIG_PATH",
      "/opt/homebrew/opt/glib/lib/pkgconfig:/opt/homebrew/opt/libxml2/lib/pkgconfig:/opt/homebrew/opt/fftw/lib/pkgconfig:/opt/homebrew/opt/gmp/lib/pkgconfig:/opt/homebrew/opt/mpfr/lib/pkgconfig:/opt/homebrew/lib/pkgconfig",
    );
  }
  let out = command
    .output()
    .unwrap_or_else(|e| panic!("failed to run pkg-config: {e}"));
  let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
  if !out.status.success() {
    let stderr = String::from_utf8_lossy(&out.stderr);
    panic!(
      "pkg-config --cflags {} failed: {stderr}",
      packages.join(" ")
    );
  }
  stdout
}

fn emit_dependency_flags(is_macos: bool) {
  if is_macos {
    // Homebrew dependency libraries + Apple frameworks.
    for lib in [
      "m", "z", "xml2", "glib-2.0", "intl", "gmp", "mpfr", "fftw3", "curl", "omp",
    ] {
      println!("cargo:rustc-link-lib={lib}");
    }
    println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
    println!("cargo:rustc-link-search=native=/opt/homebrew/opt/libomp/lib");
    println!("cargo:rustc-link-search=native=/opt/homebrew/opt/libxml2/lib");
    println!("cargo:rustc-link-search=native=/opt/homebrew/opt/glib/lib");
    println!("cargo:rustc-link-search=native=/opt/homebrew/opt/gettext/lib");
    println!("cargo:rustc-link-search=native=/opt/homebrew/opt/fftw/lib");
    println!("cargo:rustc-link-lib=framework=Accelerate");
    println!("cargo:rustc-link-lib=framework=CoreServices");
  } else {
    for lib in [
      "m", "z", "xml2", "glib-2.0", "gmp", "mpfr", "fftw3", "curl", "gomp", "lapack", "blas",
    ] {
      println!("cargo:rustc-link-lib={lib}");
    }
  }
}

fn run_with_env(program: &str, args: &[&str], env: &[(&str, &str)], dir: &Path) {
  let mut command = Command::new(program);
  command.args(args).current_dir(dir);
  for (key, value) in env {
    command.env(key, value);
  }
  let status = command
    .status()
    .unwrap_or_else(|e| panic!("failed to run {program} in {}: {e}", dir.display()));
  assert!(status.success(), "{program} {} failed", args.join(" "));
}

fn run(program: &str, args: &[&str], dir: &Path) {
  run_with_env(program, args, &[], dir);
}

fn remove_recursive(path: &Path) {
  let _ = std::fs::remove_dir_all(path);
}
