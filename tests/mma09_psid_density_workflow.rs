mod common;
use common::{TestDir, tabdat_bin};
use std::{fs, path::PathBuf, process::Command};

const INPUT: &str = include_str!("fixtures/psid_density/input.csv");
const OUTPUTS: &[&str] = &[
  "psid_bandwidth.csv",
  "psid_density.csv",
  "psid_density.svg",
  "psid_selected.csv",
];
fn example(name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma09p1np")
    .join(name)
}
fn setup() -> TestDir {
  let d = TestDir::new();
  fs::write(d.path().join("psid.csv"), INPUT).unwrap();
  fs::create_dir(d.path().join("reference")).unwrap();
  for (name, contents) in [
    (
      "sample.csv",
      include_str!("fixtures/psid_density/sample.csv"),
    ),
    (
      "bandwidth.csv",
      include_str!("fixtures/psid_density/bandwidth.csv"),
    ),
    (
      "density.csv",
      include_str!("fixtures/psid_density/density.csv"),
    ),
  ] {
    fs::write(d.path().join("reference").join(name), contents).unwrap();
  }
  d
}
fn run(d: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(example("density.td"))
    .output()
    .unwrap()
}
fn audit(d: &TestDir) -> std::process::Output {
  Command::new("python3")
    .arg(example("check_reference.py"))
    .arg(d.path())
    .arg(d.path().join("psid.csv"))
    .arg(d.path().join("reference"))
    .output()
    .unwrap()
}
fn success(out: &std::process::Output) {
  assert!(
    out.status.success(),
    "stdout={} stderr={}",
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  );
}
fn column(text: &str, name: &str, mut change: impl FnMut(usize, &str) -> String) -> String {
  let mut lines = text.lines();
  let header = lines.next().unwrap();
  let at = header.split(',').position(|s| s == name).unwrap();
  let mut output = format!("{header}\n");
  for (i, line) in lines.enumerate() {
    let mut cells: Vec<_> = line.split(',').map(str::to_owned).collect();
    cells[at] = change(i, &cells[at]);
    output.push_str(&cells.join(","));
    output.push('\n');
  }
  output
}

#[test]
fn actual_psid_kde_script_matches_r_cohort_density_and_svg_in_both_modes() {
  let d = setup();
  let json = run(&d, "--json");
  success(&json);
  let events: Vec<serde_json::Value> = String::from_utf8(json.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let plot = &events
    .iter()
    .find(|e| e["result_type"] == "PlotArtifactResult")
    .unwrap()["data"];
  assert_eq!(plot["candidate_rows"], 512);
  assert_eq!(plot["scatter"]["point_count"], 512);
  assert_eq!(plot["lines"].as_array().unwrap().len(), 3);
  for line in plot["lines"].as_array().unwrap() {
    assert_eq!(line["point_count"], 512);
    assert_eq!(line["omitted_rows"], 0);
  }
  let checked = audit(&d);
  success(&checked);
  assert!(String::from_utf8_lossy(&checked.stdout).contains("2673 direct-R cells"));
  let selected = fs::read_to_string(d.path().join("psid_selected.csv")).unwrap();
  assert_eq!(
    selected
      .lines()
      .skip(1)
      .map(|s| s.split(',').next().unwrap())
      .collect::<Vec<_>>(),
    ["1", "2", "3", "4", "5", "6", "13", "14"]
  );
  let saved: Vec<_> = OUTPUTS
    .iter()
    .map(|f| fs::read(d.path().join(f)).unwrap())
    .collect();
  for file in OUTPUTS {
    fs::remove_file(d.path().join(file)).unwrap();
  }
  success(&run(&d, "--text"));
  success(&audit(&d));
  for (file, bytes) in OUTPUTS.iter().zip(saved) {
    assert_eq!(fs::read(d.path().join(file)).unwrap(), bytes, "{file}");
  }
  assert_eq!(
    fs::read_to_string(d.path().join("psid.csv")).unwrap(),
    INPUT
  );
}

#[test]
fn shared_native_r_kde_normalization_bandwidth_cohort_and_svg_forgeries_fail() {
  let d = setup();
  success(&run(&d, "--json"));
  success(&audit(&d));
  for (native, reference, field) in [
    ("psid_density.csv", "density.csv", "density_plugin"),
    ("psid_bandwidth.csv", "bandwidth.csv", "bandwidth"),
    ("psid_selected.csv", "sample.csv", "source_row"),
  ] {
    let a = d.path().join(native);
    let b = d.path().join("reference").join(reference);
    let original_a = fs::read_to_string(&a).unwrap();
    let original_b = fs::read_to_string(&b).unwrap();
    let forge = |i, value: &str| {
      if field == "source_row" {
        if i == 0 { "7".into() } else { value.into() }
      } else {
        (value.parse::<f64>().unwrap() * 1.1).to_string()
      }
    };
    fs::write(&a, column(&original_a, field, forge)).unwrap();
    fs::write(&b, column(&original_b, field, forge)).unwrap();
    assert!(!audit(&d).status.success(), "accepted shared wrong {field}");
    fs::write(a, original_a).unwrap();
    fs::write(b, original_b).unwrap();
  }
  let path = d.path().join("psid_density.svg");
  let mut svg = fs::read_to_string(&path).unwrap();
  let start = svg.find(" d=\"M").unwrap() + " d=\"M".len();
  let stop = start + svg[start..].find('L').unwrap();
  svg.replace_range(start..stop, "0 0");
  fs::write(path, svg).unwrap();
  assert!(
    !audit(&d).status.success(),
    "accepted wrong actual path with unchanged raw metadata"
  );
}

#[test]
fn psid_density_guards_and_no_clobber_outputs_preserve_source_files() {
  for bad in [
    column(
      INPUT,
      "hours",
      |i, x| if i == 0 { "0".into() } else { x.into() },
    ),
    column(INPUT, "age", |_, _| "35".into()),
    column(
      &column(INPUT, "earnings", |_, _| "1000".into()),
      "hours",
      |_, _| "1000".into(),
    ),
    column(INPUT, "source_row", |_, _| "1".into()),
    column(
      INPUT,
      "sex",
      |i, x| if i == 0 { "1".into() } else { x.into() },
    ),
  ] {
    let d = setup();
    fs::write(d.path().join("psid.csv"), &bad).unwrap();
    let output = run(&d, "--json");
    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(d.path().join("psid.csv")).unwrap(), bad);
    assert!(OUTPUTS.iter().all(|f| !d.path().join(f).exists()));
  }
  for file in OUTPUTS {
    let d = setup();
    fs::write(d.path().join(file), "existing artifact\n").unwrap();
    assert!(!run(&d, "--json").status.success(), "replaced {file}");
    assert_eq!(
      fs::read_to_string(d.path().join(file)).unwrap(),
      "existing artifact\n"
    );
    assert_eq!(
      fs::read_to_string(d.path().join("psid.csv")).unwrap(),
      INPUT
    );
  }
}

#[test]
fn fixed_width_psid_conversion_validates_before_publication_and_refuses_replacement() {
  let d = TestDir::new();
  let mut fixed = String::new();
  for line in INPUT.lines().skip(1) {
    let fields: Vec<_> = line.split(',').collect();
    for (i, width) in [4, 3, 2, 2, 6, 4, 1, 2, 1].iter().enumerate() {
      fixed.push_str(&format!("{:>width$}", fields[i + 1], width = *width));
    }
    fixed.push('\n');
  }
  let source = d.path().join("source.dat");
  let target = d.path().join("converted.csv");
  fs::write(&source, &fixed).unwrap();
  let convert = || {
    Command::new("python3")
      .arg(example("prepare.py"))
      .arg(&source)
      .arg(&target)
      .output()
      .unwrap()
  };
  success(&convert());
  let converted = fs::read_to_string(&target).unwrap();
  assert_eq!(converted.replace("\r\n", "\n"), INPUT.replace("\r\n", "\n"));
  assert!(!convert().status.success());
  assert_eq!(fs::read_to_string(&target).unwrap(), converted);
  fs::remove_file(&target).unwrap();
  for bad in [format!("{fixed}short\n"), fixed.replacen(" 101", "x101", 1)] {
    fs::write(&source, &bad).unwrap();
    assert!(!convert().status.success());
    assert!(!target.exists());
    assert_eq!(fs::read_to_string(&source).unwrap(), bad);
  }
}
