#![forbid(unsafe_code)]
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-advertised-options-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "x\n1\n").unwrap();
    fs::write(root.join("child.td"), "export forbidden.csv\n").unwrap();
    fs::write(root.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
    fs::write(
      root.join("settings.toml"),
      "[unknown]\nnever_applied = true\n",
    )
    .unwrap();
    Self(root)
  }
  fn run(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
#[test]
fn help_advertises_installed_options_and_distinguishes_the_typed_mcp_boundary() {
  let fixture = Fixture::new();
  let output = fixture.run(&["--help"]);
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let help = String::from_utf8(output.stdout).unwrap();
  assert!(!help.contains("--config"));
  assert!(!help.contains("--mcp"));
  assert!(
    help.contains("Configuration-file loading and an MCP transport/server are not installed.")
  );
  assert!(help.contains("typed transport-independent MCP boundary does not start a server."));
  for option in [
    "--command",
    "--file",
    "--json",
    "--list-commands",
    "--list-command-effects",
    "--help-topic",
    "--explain",
    "--describe-command",
  ] {
    assert!(help.contains(option), "{option}");
  }
  let error = fixture.run(&["--unknown"]);
  assert_eq!(error.status.code(), Some(2));
  let usage = String::from_utf8(error.stderr).unwrap();
  assert!(usage.contains("usage: tabdat"));
  assert!(!usage.contains("--config"));
  assert!(!usage.contains("--mcp"));
}
#[test]
fn uninstalled_flags_remain_errors_before_batch_or_nested_file_effects() {
  let fixture = Fixture::new();
  for json in [false, true] {
    for script in [false, true] {
      for options in [
        vec!["--mcp"],
        vec!["--mcp=stdio"],
        vec!["--config", "settings.toml"],
        vec!["--config=missing.toml"],
      ] {
        let mut args = Vec::new();
        if json {
          args.push("--json");
        }
        if script {
          args.extend(["-f", "parent.td"]);
        } else {
          args.extend(["-c", "use data.csv", "-c", "export forbidden.csv"]);
        }
        args.extend(options.iter().copied());
        let output = fixture.run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("unrecognized arguments:"), "{error}");
        assert!(error.contains(options[0]));
        // The reported bad flag may contain its own spelling; usage must not advertise it.
        let usage = error.split_once("\ntabdat: error:").unwrap().0;
        assert!(!usage.contains("--config"));
        assert!(!usage.contains("--mcp"));
        assert!(!fixture.0.join("forbidden.csv").exists());
      }
    }
  }
}
#[test]
fn help_and_version_precedence_remains_effect_free_with_uninstalled_flags() {
  let fixture = Fixture::new();
  for options in [vec!["--mcp"], vec!["--config", "settings.toml"]] {
    for request in ["--help", "--version"] {
      let mut args = vec!["--json", "-f", "parent.td"];
      args.extend(options.iter().copied());
      args.push(request);
      let output = fixture.run(&args);
      assert!(output.status.success(), "{args:?}");
      assert!(output.stderr.is_empty());
      let text = String::from_utf8(output.stdout).unwrap();
      if request == "--help" {
        assert!(text.contains("usage: tabdat"));
        assert!(!text.contains("--config"));
        assert!(!text.contains("--mcp"));
      } else {
        assert!(text.contains(env!("CARGO_PKG_VERSION")));
      }
      assert!(!fixture.0.join("forbidden.csv").exists());
    }
  }
}
