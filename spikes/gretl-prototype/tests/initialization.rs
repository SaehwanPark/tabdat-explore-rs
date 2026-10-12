use std::process::Command;
fn probe(parallel: bool) -> std::process::Output {
  let mut cmd = Command::new(env!("CARGO_BIN_EXE_init-probe"));
  cmd.env_remove("GRETL_LIB_NAME");
  if parallel {
    cmd.arg("--parallel");
  }
  cmd.output().unwrap()
}
#[test]
fn fresh_normal_and_concurrent_init_have_truthful_clean_diagnostics() {
  for parallel in [false, true] {
    let out = probe(parallel);
    assert!(
      out.status.success(),
      "{}",
      String::from_utf8_lossy(&out.stderr)
    );
    assert!(
      out.stderr.is_empty(),
      "{}",
      String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(text.lines().count(), 1);
    assert!(text.contains("ols=ok"));
    #[cfg(target_os = "macos")]
    assert!(text.starts_with("blas=veclib "), "{text}");
    #[cfg(target_os = "linux")]
    assert!(
      text.starts_with("blas=netlib ")
        || text.starts_with("blas=openblas ")
        || text.starts_with("blas=mkl ")
        || text.starts_with("blas=blis ")
        || text.starts_with("blas=atlas "),
      "{text}"
    );
  }
}
#[cfg(target_os = "linux")]
#[test]
fn fallback_ldd_target_is_resolved_in_parent_not_child() {
  use std::{
    fs,
    os::unix::fs::PermissionsExt,
    time::{SystemTime, UNIX_EPOCH},
  };
  let dir = std::env::temp_dir().join(format!(
    "tabdat-blas-probe-{}-{}",
    std::process::id(),
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos()
  ));
  fs::create_dir(&dir).unwrap();
  let tool = dir.join("ldd");
  let trace = dir.join("args");
  fs::write(
    &tool,
    "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$TABDAT_BLAS_TRACE\"\nexec /usr/bin/ldd \"$@\"\n",
  )
  .unwrap();
  fs::set_permissions(&tool, fs::Permissions::from_mode(0o700)).unwrap();
  let exe = std::path::Path::new(env!("CARGO_BIN_EXE_init-probe"))
    .canonicalize()
    .unwrap();
  let out = Command::new(&exe)
    .env_remove("GRETL_LIB_NAME")
    .env("TABDAT_BLAS_TRACE", &trace)
    .env(
      "PATH",
      format!("{}:{}", dir.display(), std::env::var("PATH").unwrap()),
    )
    .output()
    .unwrap();
  let args = fs::read_to_string(&trace).ok();
  fs::remove_dir_all(dir).unwrap();
  assert!(out.status.success());
  assert!(
    out.stderr.is_empty(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  if let Some(args) = args {
    assert_eq!(args.trim(), exe.to_str().unwrap());
  } else {
    assert!(
      !String::from_utf8(out.stdout)
        .unwrap()
        .starts_with("blas=netlib "),
      "netlib discovery must exercise fallback"
    );
  }
}
