mod common;
use common::{TestDir, tabdat_bin};
use std::{fs, path::PathBuf, process::Command};

fn example(name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma17p2kmextra")
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
fn audit(d: &TestDir) -> std::process::Output {
  Command::new("python3")
    .arg(example("check_censored.py"))
    .arg(d.path())
    .arg(d.path().join("censored.csv"))
    .arg(d.path().join("reference.csv"))
    .output()
    .unwrap()
}
const INPUT: &str = include_str!("fixtures/censored_survival/input.csv");
const REFERENCE: &str = include_str!("fixtures/censored_survival/reference.csv");
const OUTPUTS: &[&str] = &[
  "censored_source.csv",
  "censored_survival.csv",
  "censored_steps.csv",
  "censored_survival.svg",
  "censored_cumhaz.svg",
];

#[test]
fn censored_ties_initial_censor_and_nonzero_tail_match_r_and_native_step_graphs() {
  let d = TestDir::new();
  fs::write(d.path().join("censored.csv"), INPUT).unwrap();
  fs::write(d.path().join("reference.csv"), REFERENCE).unwrap();
  let output = run(&d, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<serde_json::Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  assert!(events.iter().all(|e| e.get("error").is_none()));
  let checked = audit(&d);
  assert!(
    checked.status.success(),
    "{}",
    String::from_utf8_lossy(&checked.stderr)
  );
  assert!(String::from_utf8_lossy(&checked.stdout).contains("60 R table cells"));
  assert!(String::from_utf8_lossy(&checked.stdout).contains("66 SVG vertices"));
  let saved: Vec<_> = OUTPUTS
    .iter()
    .map(|file| fs::read(d.path().join(file)).unwrap())
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
  for (file, bytes) in OUTPUTS.iter().zip(&saved) {
    assert_eq!(&fs::read(d.path().join(file)).unwrap(), bytes);
  }
  for (file, forged) in [
    ("censored_source.csv", INPUT.replace("5,2,0", "5,2,1")),
    ("censored_steps.csv", "time,phase,survival,survival_lower,survival_upper,cumhaz,cumhaz_lower,cumhaz_upper\n0,0,1,1,1,0,0,0\n".into()),
    ("censored_survival.csv", REFERENCE.replace(",0.8,", ",0.7,")),
  ] {
    let path = d.path().join(file);
    let before = fs::read(&path).unwrap();
    fs::write(&path, forged).unwrap();
    assert!(!audit(&d).status.success(), "corrupt {file} accepted");
    fs::write(&path, before).unwrap();
  }
  // Alter rendered geometry while retaining every serialized data coordinate.
  let path = d.path().join("censored_cumhaz.svg");
  let svg = fs::read_to_string(&path).unwrap();
  let start = svg.find("class=\"line-series\"").unwrap();
  let coordinate = start + svg[start..].find("d=\"M").unwrap() + 4;
  fs::write(
    &path,
    format!("{}0 0L{}", &svg[..coordinate], &svg[coordinate..]),
  )
  .unwrap();
  assert!(
    !audit(&d).status.success(),
    "wrong actual SVG pixels accepted"
  );
  fs::write(&path, svg).unwrap();
  // Shared wrong R/native survival must fail independent sample reconstruction.
  let forged = REFERENCE.replace(",0.8,", ",0.7,");
  assert_ne!(forged, REFERENCE);
  fs::write(d.path().join("reference.csv"), &forged).unwrap();
  fs::write(d.path().join("censored_survival.csv"), forged).unwrap();
  assert!(!audit(&d).status.success());
}

#[test]
fn terminal_all_failure_risk_set_has_undefined_km_uncertainty_but_finite_na() {
  let d = TestDir::new();
  fs::write(
    d.path().join("censored.csv"),
    "source_row,duration,failed\n1,5,1\n2,5,1\n3,5,1\n",
  )
  .unwrap();
  let output = run(&d, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let table = fs::read_to_string(d.path().join("censored_survival.csv")).unwrap();
  let values: Vec<_> = table.lines().nth(1).unwrap().split(',').collect();
  assert_eq!(&values[..4], &["5", "3", "3", "0"]);
  assert_eq!(values[4].parse::<f64>().unwrap(), 0.0);
  assert_eq!(&values[5..8], &["", "", ""]);
  assert_eq!(values[8].parse::<f64>().unwrap(), 1.0);
  assert!((values[9].parse::<f64>().unwrap() - (1.0_f64 / 3.0).sqrt()).abs() < 1e-14);
  let z = 1.959963984540054 / 3.0_f64.sqrt();
  assert!((values[10].parse::<f64>().unwrap() - (-z).exp()).abs() < 1e-14);
  assert!((values[11].parse::<f64>().unwrap() - z.exp()).abs() < 1e-14);
}

#[test]
fn invalid_samples_and_file_collisions_preserve_inputs_and_stop_later_effects() {
  let mut cases = vec![
    "source_row,duration,failed\n".into(),
    INPUT.replace("1,5,0", "1,0,0"),
    INPUT.replace("1,5,0", "1,-5,0"),
    INPUT.replace("1,5,0", "1,,0"),
    INPUT.replace("1,5,0", "1,inf,0"),
    INPUT.replace("1,5,0", "1,NaN,0"),
    INPUT.replace("1,5,0", "1,5,"),
    INPUT.replace("1,5,0", "1,5,2"),
    INPUT.replace("1,5,0", "1,5,0.5"),
    INPUT.replace("1,5,0", "1,5,inf"),
    INPUT.replace("1,5,0", "1.5,5,0"),
    INPUT.replace("2,2,1", "1,2,1"),
    INPUT.replace(",1\n", ",0\n"),
  ];
  let too_many_times = format!(
    "source_row,duration,failed\n{}",
    (1..=5000)
      .map(|i| format!("{i},{i},1\n"))
      .collect::<String>()
  );
  cases.push(too_many_times);
  for input in cases {
    let d = TestDir::new();
    fs::write(d.path().join("censored.csv"), &input).unwrap();
    let output = run(&d, "--json");
    assert!(!output.status.success(), "invalid sample accepted");
    for file in OUTPUTS {
      assert!(!d.path().join(file).exists(), "published {file}");
    }
    assert_eq!(
      fs::read_to_string(d.path().join("censored.csv")).unwrap(),
      input
    );
  }
  for (i, file) in OUTPUTS.iter().enumerate() {
    let d = TestDir::new();
    fs::write(d.path().join("censored.csv"), INPUT).unwrap();
    fs::write(d.path().join(file), "sentinel").unwrap();
    assert!(!run(&d, "--json").status.success());
    assert_eq!(fs::read_to_string(d.path().join(file)).unwrap(), "sentinel");
    for later in &OUTPUTS[i + 1..] {
      assert!(!d.path().join(later).exists());
    }
  }
}

#[test]
fn executable_input_conversion_handles_crlf_and_failure_without_partial_publication() {
  let d = TestDir::new();
  let source = d.path().join("author.do");
  let target = d.path().join("censored.csv");
  let body = [
    (10, 1, 6),
    (15, 0, 4),
    (20, 1, 5),
    (25, 0, 3),
    (30, 1, 2),
    (35, 0, 1),
    (40, 1, 1),
    (45, 0, 32),
    (50, 1, 26),
  ]
  .iter()
  .map(|(t, f, n)| format!("{t} {f}\r\n").repeat(*n))
  .collect::<String>();
  let valid = format!(
    "* synthetic extraction control, not redistributed author program\r\ninput duration failed\r\n{body}end\r\n"
  );
  let prepare = || {
    Command::new("sh")
      .arg(example("prepare.sh"))
      .arg(&source)
      .arg(&target)
      .output()
      .unwrap()
  };
  for malformed in [
    String::new(),
    valid.replace("end\r\n", ""),
    valid.replace("10 1", "10 2"),
    valid.replace("50 1\r\n", ""),
    valid.replacen("20 1\r\n", "end\r\ninput duration failed\r\n20 1\r\n", 1),
  ] {
    fs::write(&source, malformed).unwrap();
    assert!(!prepare().status.success());
    assert!(!target.exists());
  }
  fs::write(&source, valid).unwrap();
  let result = prepare();
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
  let bytes = fs::read(&target).unwrap();
  let text = String::from_utf8(bytes.clone()).unwrap();
  assert_eq!(text.lines().count(), 81);
  assert!(text.starts_with("source_row,duration,failed\n1,10,1\n"));
  assert!(text.ends_with("80,50,1\n"));
  assert!(!prepare().status.success());
  assert_eq!(fs::read(&target).unwrap(), bytes);
  assert_eq!(fs::read_dir(d.path()).unwrap().count(), 2);
}
