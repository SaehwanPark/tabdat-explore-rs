#![forbid(unsafe_code)]

use std::process::Command;

#[test]
fn scaffold_exits_successfully_with_only_the_placeholder_greeting() {
  let output = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .output()
    .expect("scaffold binary should launch");

  assert!(output.status.success());
  assert_eq!(output.stdout, b"Hello, world!\n");
  assert!(output.stderr.is_empty());
}
