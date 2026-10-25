mod common;
use common::{TestDir, tabdat_bin};
use std::{fs, path::PathBuf, process::Command};

fn example(name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma17p1km")
    .join(name)
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(example("survival.td"))
    .output()
    .unwrap()
}
const INPUT: &str = include_str!("fixtures/strike_survival/input.csv");
const REFERENCE: &str = include_str!("fixtures/strike_survival/reference.csv");
const OUTPUTS: &[&str] = &[
  "strikes_source.csv",
  "strikes_survival.csv",
  "strikes_steps.csv",
  "strikes_survival.svg",
  "strikes_cumhaz.svg",
];
fn audit(d: &TestDir) -> std::process::Output {
  Command::new("python3")
    .arg(example("check_reference.py"))
    .arg(d.path())
    .arg(d.path().join("strikes.csv"))
    .arg(d.path().join("reference.csv"))
    .output()
    .unwrap()
}

#[test]
fn actual_tied_strike_workflow_matches_r_and_svg_geometry_in_both_cli_modes() {
  let d = TestDir::new();
  fs::write(d.path().join("strikes.csv"), INPUT).unwrap();
  fs::write(d.path().join("reference.csv"), REFERENCE).unwrap();
  let json = run(&d, "--json");
  assert!(
    json.status.success(),
    "{}",
    String::from_utf8_lossy(&json.stderr)
  );
  let events: Vec<serde_json::Value> = String::from_utf8(json.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  assert!(events.iter().all(|e| e.get("error").is_none()));
  let checked = audit(&d);
  assert!(
    checked.status.success(),
    "{}",
    String::from_utf8_lossy(&checked.stderr)
  );
  assert!(String::from_utf8_lossy(&checked.stdout).contains("45 R table cells"));
  assert!(String::from_utf8_lossy(&checked.stdout).contains("42 SVG vertices"));
  let saved: Vec<_> = OUTPUTS
    .iter()
    .map(|f| fs::read(d.path().join(f)).unwrap())
    .collect();
  for file in OUTPUTS {
    fs::remove_file(d.path().join(file)).unwrap();
  }
  let text = run(&d, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(String::from_utf8_lossy(&text.stdout).contains("survival_lower"));
  for (file, bytes) in OUTPUTS.iter().zip(&saved) {
    assert_eq!(&fs::read(d.path().join(file)).unwrap(), bytes);
  }
  // Comparison against R alone is insufficient: audit also binds the sample,
  // independently reconstructs risk sets, and checks rendered geometry.
  for (file, forged) in [
    ("strikes_source.csv", INPUT.replace("1,4,0.1", "1,5,0.1")),
    (
      "strikes_steps.csv",
      "time,phase,survival,survival_lower,survival_upper,cumhaz\n0,0,1,1,1,0\n".into(),
    ),
    ("strikes_survival.svg", "<svg/>".into()),
    ("strikes_survival.csv", REFERENCE.replace("0.75", "0.74")),
  ] {
    let path = d.path().join(file);
    let before = fs::read(&path).unwrap();
    fs::write(&path, forged).unwrap();
    assert!(!audit(&d).status.success(), "corrupted {file} accepted");
    fs::write(&path, before).unwrap();
  }
}

#[test]
fn invalid_durations_keys_and_existing_exports_stop_before_later_effects() {
  for input in [
    "source_row,dur,gdp\n".to_owned(),
    INPUT.replace("1,4,0.1", "1,0,0.1"),
    INPUT.replace("1,4,0.1", "1,-4,0.1"),
    INPUT.replace("1,4,0.1", "1,,0.1"),
    INPUT.replace("1,4,0.1", "1,inf,0.1"),
    INPUT.replace("1,4,0.1", "1,NaN,0.1"),
    INPUT.replace("1,4,0.1", "1.5,4,0.1"),
    INPUT.replace("2,1,-0.2", "1,1,-0.2"),
  ] {
    let d = TestDir::new();
    fs::write(d.path().join("strikes.csv"), &input).unwrap();
    let o = run(&d, "--json");
    assert!(
      !o.status.success(),
      "{input}: {}",
      String::from_utf8_lossy(&o.stdout)
    );
    for file in OUTPUTS {
      assert!(
        !d.path().join(file).exists(),
        "published {file} after invalid input"
      );
    }
    assert_eq!(
      fs::read_to_string(d.path().join("strikes.csv")).unwrap(),
      input
    );
  }
  // Ordered script effects are not a multi-file transaction. A collision stops
  // subsequent writes, but preserves any earlier successful exports.
  for (index, file) in OUTPUTS.iter().enumerate() {
    let d = TestDir::new();
    fs::write(d.path().join("strikes.csv"), INPUT).unwrap();
    fs::write(d.path().join(file), "sentinel").unwrap();
    let o = run(&d, "--json");
    assert!(!o.status.success());
    assert_eq!(fs::read_to_string(d.path().join(file)).unwrap(), "sentinel");
    for later in &OUTPUTS[index + 1..] {
      assert!(
        !d.path().join(later).exists(),
        "published {later} after collision at {file}"
      );
    }
  }
}

#[test]
fn converter_publishes_only_complete_validated_input_and_never_replaces_files() {
  let d = TestDir::new();
  let input = d.path().join("raw.asc");
  let target = d.path().join("strikes.csv");
  let prepare = || {
    Command::new("sh")
      .arg(example("prepare.sh"))
      .arg(&input)
      .arg(&target)
      .output()
      .unwrap()
  };
  // Synthetic shape control, not replacement textbook data: exact source totals.
  let valid = format!("{}219 .1\r\n220 -.1\r\n", "43 .01\r\n".repeat(564));
  for malformed in [
    "".into(),
    "1 .2\n".into(),
    valid.replace("219 .1", "219 .1 junk"),
  ] {
    fs::write(&input, malformed).unwrap();
    assert!(!prepare().status.success());
    assert!(!target.exists());
  }
  fs::write(&input, valid).unwrap();
  let result = prepare();
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
  let bytes = fs::read(&target).unwrap();
  let text = String::from_utf8(bytes.clone()).unwrap();
  assert_eq!(text.lines().count(), 567);
  assert!(text.starts_with("source_row,dur,gdp\n1,43,.01\n"));
  assert!(text.ends_with("566,220,-.1\n"));
  assert!(!prepare().status.success());
  assert_eq!(fs::read(&target).unwrap(), bytes);
  assert_eq!(fs::read_dir(d.path()).unwrap().count(), 2);
}
