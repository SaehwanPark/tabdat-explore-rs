#![forbid(unsafe_code)]

use std::process::Command;

#[test]
fn no_argument_session_exits_cleanly_on_eof_without_scaffold_output() {
  let output = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .output()
    .expect("binary should launch");

  assert!(output.status.success());
  assert!(output.stdout.is_empty());
  assert!(output.stderr.is_empty());
}
