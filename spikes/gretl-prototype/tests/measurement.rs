#![forbid(unsafe_code)]

#[test]
fn metadata_mode_does_not_initialize_native_capability() {
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_measure"))
    .arg("--metadata")
    .output()
    .unwrap();
  assert!(output.status.success());
  assert_eq!(
    output.stdout,
    b"tabdat-gretl-spike gretl=2026b measurement-only\n"
  );
  // Native first-use currently emits an otool diagnostic on this platform;
  // metadata must not enter that path (nor run any estimator/fixture work).
  assert!(output.stderr.is_empty());
}
