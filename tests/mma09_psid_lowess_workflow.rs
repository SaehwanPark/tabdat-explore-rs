mod common;
use common::{TestDir, tabdat_bin};
use std::{
  collections::BTreeMap,
  fs,
  path::PathBuf,
  process::{Command, Output},
};

const INPUT: &str = include_str!("fixtures/psid_density/input.csv");
const EXPECTED: &str = include_str!("fixtures/lowess/source_control_expected.csv");
const OUTPUTS: &[&str] = &["psid_lowess.csv", "psid_lowess.svg"];
fn example(name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma09p1np")
    .join(name)
}
fn setup() -> TestDir {
  let d = TestDir::new();
  fs::write(d.path().join("psid.csv"), INPUT).unwrap();
  fs::create_dir(d.path().join("reference")).unwrap();
  for (name, text) in [
    (
      "sample.csv",
      include_str!("fixtures/lowess/source_control_sample.csv"),
    ),
    ("fitted.csv", EXPECTED),
    (
      "metadata.csv",
      include_str!("fixtures/lowess/source_control_metadata.csv"),
    ),
  ] {
    fs::write(d.path().join("reference").join(name), text).unwrap();
  }
  d
}
fn run(d: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(example("lowess.td"))
    .output()
    .unwrap()
}
fn audit(d: &TestDir) -> Output {
  Command::new("python3")
    .arg(example("check_lowess.py"))
    .arg(d.path())
    .arg(d.path().join("psid.csv"))
    .arg(d.path().join("reference"))
    .output()
    .unwrap()
}
fn values_only(d: &TestDir) -> Output {
  Command::new("python3").args(["-c","import sys;sys.path.insert(0,sys.argv[1]);from check_lowess import check_values;check_values(*sys.argv[2:])"])
    .arg(example("check_lowess.py").parent().unwrap()).arg(d.path()).arg(d.path().join("psid.csv")).arg(d.path().join("reference")).output().unwrap()
}
fn success(out: &Output) {
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
  let at = header.split(',').position(|v| v == name).unwrap();
  let mut out = format!("{header}\n");
  for (i, line) in lines.enumerate() {
    let mut cells: Vec<_> = line.split(',').map(str::to_owned).collect();
    cells[at] = change(i, &cells[at]);
    out.push_str(&cells.join(","));
    out.push('\n');
  }
  out
}
fn close(a: f64, b: f64) {
  assert!(
    (a - b).abs() <= 1e-11 + 1e-10 * b.abs(),
    "{a:.17e} != {b:.17e}"
  );
}

#[test]
fn actual_psid_lowess_script_matches_qr_sample_diagnostic_and_geometry_in_both_modes() {
  let d = setup();
  let json = run(&d, "--json");
  success(&json);
  let events: Vec<serde_json::Value> = String::from_utf8(json.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let fits: Vec<_> = events
    .iter()
    .filter(|e| e["result_type"] == "LowessResult")
    .map(|e| &e["data"])
    .collect();
  assert_eq!(fits.len(), 3);
  for (fit, (span, k, constant)) in fits.iter().zip([(0.8, 2, 0), (0.4, 1, 1), (0.1, 0, 8)]) {
    assert_eq!(fit["bandwidth"], span);
    assert_eq!(fit["half_window"], k);
    assert_eq!(fit["local_constant_windows"], constant);
    assert_eq!(
      fit["sample"]["retained_indices"],
      serde_json::json!([0, 1, 2, 3, 4, 5, 6, 7])
    );
    assert_eq!(fit["sample"]["total_observations"], 8);
    assert_eq!(fit["sample"]["dropped_observations"], 0);
    assert_eq!(fit["algorithm"], "centered_rank_local_linear_tricube");
    assert_eq!(fit["robust_iterations"], 0);
    assert_eq!(fit["out_of_sample_prediction"], false);
    assert_eq!(fit["estimation_state"], "unchanged");
  }
  let plot = &events
    .iter()
    .find(|e| e["result_type"] == "PlotArtifactResult")
    .unwrap()["data"];
  assert_eq!(plot["candidate_rows"], 8);
  assert_eq!(plot["scatter"]["point_count"], 8);
  let lines = plot["lines"].as_array().unwrap();
  assert_eq!(lines.len(), 3);
  for line in lines {
    assert_eq!(line["point_count"], 8);
    assert_eq!(line["omitted_rows"], 0);
  }
  let mut groups: BTreeMap<i32, Vec<Vec<f64>>> = BTreeMap::new();
  for line in EXPECTED.lines().skip(1) {
    let r: Vec<f64> = line.split(',').map(|v| v.parse().unwrap()).collect();
    groups.entry(r[1] as i32).or_default().push(r);
  }
  let headers = serde_json::json!(["educatn", "n", "observed_mean", "span01_min", "span01_max"]);
  let diagnostic = &events
    .iter()
    .find(|e| e["result_type"] == "TableResult" && e["data"]["headers"] == headers)
    .unwrap()["data"]["rows"];
  assert_eq!(diagnostic.as_array().unwrap().len(), groups.len());
  for (row, (x, values)) in diagnostic.as_array().unwrap().iter().zip(groups) {
    let want = [
      x as f64,
      values.len() as f64,
      values.iter().map(|r| r[2]).sum::<f64>() / values.len() as f64,
      values.iter().map(|r| r[5]).fold(f64::INFINITY, f64::min),
      values
        .iter()
        .map(|r| r[5])
        .fold(f64::NEG_INFINITY, f64::max),
    ];
    for (a, b) in row.as_array().unwrap().iter().zip(want) {
      close(a.as_f64().unwrap(), b);
    }
  }
  let checked = audit(&d);
  success(&checked);
  assert!(String::from_utf8_lossy(&checked.stdout).contains("128 distinct exported R cells"));
  let saved: Vec<_> = OUTPUTS
    .iter()
    .map(|name| fs::read(d.path().join(name)).unwrap())
    .collect();
  for name in OUTPUTS {
    fs::remove_file(d.path().join(name)).unwrap();
  }
  let text = run(&d, "--text");
  success(&text);
  assert!(
    String::from_utf8_lossy(&text.stdout)
      .contains("Tied-X fits may differ and depend on input order")
  );
  success(&audit(&d));
  for (name, bytes) in OUTPUTS.iter().zip(saved) {
    assert_eq!(fs::read(d.path().join(name)).unwrap(), bytes, "{name}");
  }
  assert_eq!(
    fs::read_to_string(d.path().join("psid.csv")).unwrap(),
    INPUT
  );
}

#[test]
fn independent_numeric_audit_rejects_shared_wrong_native_and_r_fits_cohort_and_windows() {
  let d = setup();
  success(&run(&d, "--json"));
  success(&values_only(&d));
  // Test numeric reconstruction WITHOUT SVG: an inconsistent old path must not
  // accidentally make a shared-wrong-reference test pass for the wrong reason.
  for field in ["lowess_08", "lowess_01"] {
    let a = d.path().join("psid_lowess.csv");
    let b = d.path().join("reference/fitted.csv");
    let aa = fs::read_to_string(&a).unwrap();
    let bb = fs::read_to_string(&b).unwrap();
    let forge = |_: usize, v: &str| (v.parse::<f64>().unwrap() * 1.1 + 0.125).to_string();
    fs::write(&a, column(&aa, field, forge)).unwrap();
    fs::write(&b, column(&bb, field, forge)).unwrap();
    assert!(
      !values_only(&d).status.success(),
      "accepted shared wrong {field}"
    );
    fs::write(a, aa).unwrap();
    fs::write(b, bb).unwrap();
  }
  let paths = [
    d.path().join("psid_lowess.csv"),
    d.path().join("reference/sample.csv"),
    d.path().join("reference/fitted.csv"),
  ];
  let originals: Vec<_> = paths
    .iter()
    .map(|p| fs::read_to_string(p).unwrap())
    .collect();
  for (p, text) in paths.iter().zip(&originals) {
    fs::write(
      p,
      column(text, "source_row", |i, v| {
        if i == 0 { "7".into() } else { v.into() }
      }),
    )
    .unwrap();
  }
  assert!(
    !values_only(&d).status.success(),
    "accepted shared wrong source membership"
  );
  for (p, text) in paths.iter().zip(originals) {
    fs::write(p, text).unwrap();
  }
  let meta = d.path().join("reference/metadata.csv");
  let original = fs::read_to_string(&meta).unwrap();
  for field in ["half_window", "local_constant_windows"] {
    fs::write(&meta, column(&original, field, |_, _| "99".into())).unwrap();
    assert!(!values_only(&d).status.success(), "accepted wrong {field}");
  }
  fs::write(meta, original).unwrap();
  success(&audit(&d));
}

#[test]
fn actual_svg_path_and_marker_and_axis_forgeries_fail_with_correct_raw_metadata() {
  let d = setup();
  success(&run(&d, "--json"));
  success(&audit(&d));
  let path = d.path().join("psid_lowess.svg");
  let original = fs::read_to_string(&path).unwrap();
  let class = original.find("class=\"line-series\"").unwrap();
  let start = class + original[class..].find(" d=\"M").unwrap() + " d=\"".len();
  let stop = start + original[start..].find('"').unwrap();
  for wrong in [
    original[start..stop].replace('L', "M"),
    format!("{}Z", &original[start..stop]),
  ] {
    let mut forged = original.clone();
    forged.replace_range(start..stop, &wrong);
    fs::write(&path, forged).unwrap();
    assert!(
      !audit(&d).status.success(),
      "accepted altered path connectivity"
    );
  }
  let mut forged = original.clone();
  let class = forged.find("class=\"line-series\"").unwrap();
  let start = class + forged[class..].find(" d=\"M").unwrap() + " d=\"M".len();
  let stop = start + forged[start..].find('L').unwrap();
  forged.replace_range(start..stop, "0 0");
  fs::write(&path, forged).unwrap();
  assert!(
    !audit(&d).status.success(),
    "accepted wrong actual ML point with correct metadata"
  );
  let mut forged = original.clone();
  let class = forged.find("class=\"scatter-point\"").unwrap();
  let start = class + forged[class..].find(" cy=\"").unwrap() + " cy=\"".len();
  let stop = start + forged[start..].find('"').unwrap();
  forged.replace_range(start..stop, "0");
  fs::write(&path, forged).unwrap();
  assert!(!audit(&d).status.success(), "accepted wrong actual mark");
  let mut forged = original;
  let start = forged.find("data-x-min=\"").unwrap() + "data-x-min=\"".len();
  let stop = start + forged[start..].find('"').unwrap();
  forged.replace_range(start..stop, "-999");
  fs::write(path, forged).unwrap();
  assert!(!audit(&d).status.success(), "accepted wrong padded domain");
}

#[test]
fn source_guards_no_clobber_and_supported_constant_responses_or_predictors() {
  for bad in [
    column(
      INPUT,
      "hours",
      |i, v| if i == 0 { "0".into() } else { v.into() },
    ),
    column(INPUT, "age", |_, _| "35".into()),
    column(
      INPUT,
      "age",
      |i, _| if i < 2 { "36".into() } else { "35".into() },
    ),
    column(INPUT, "source_row", |_, _| "1".into()),
    column(
      INPUT,
      "sex",
      |i, v| if i == 0 { "1".into() } else { v.into() },
    ),
  ] {
    let d = setup();
    fs::write(d.path().join("psid.csv"), &bad).unwrap();
    assert!(!run(&d, "--json").status.success());
    assert_eq!(fs::read_to_string(d.path().join("psid.csv")).unwrap(), bad);
    assert!(OUTPUTS.iter().all(|name| !d.path().join(name).exists()));
  }
  for name in OUTPUTS {
    let d = setup();
    fs::write(d.path().join(name), "existing artifact\n").unwrap();
    assert!(!run(&d, "--json").status.success());
    assert_eq!(
      fs::read_to_string(d.path().join(name)).unwrap(),
      "existing artifact\n"
    );
    assert_eq!(
      fs::read_to_string(d.path().join("psid.csv")).unwrap(),
      INPUT
    );
  }
  let zero = column(
    &column(INPUT, "earnings", |_, _| "1000".into()),
    "hours",
    |_, _| "1000".into(),
  );
  for input in [&zero, &column(INPUT, "educatn", |_, _| "12".into())] {
    let d = setup();
    fs::write(d.path().join("psid.csv"), input).unwrap();
    let out = run(&d, "--json");
    success(&out);
    let csv = fs::read_to_string(d.path().join("psid_lowess.csv")).unwrap();
    let header: Vec<_> = csv.lines().next().unwrap().split(',').collect();
    if input == &zero {
      for row in csv.lines().skip(1) {
        let fields: Vec<_> = row.split(',').collect();
        for key in ["lnhwage", "lowess_08", "lowess_04", "lowess_01"] {
          assert_eq!(
            fields[header.iter().position(|v| *v == key).unwrap()]
              .parse::<f64>()
              .unwrap(),
            0.
          );
        }
      }
    } else {
      let events: Vec<serde_json::Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
      for e in events.iter().filter(|e| e["result_type"] == "LowessResult") {
        assert_eq!(
          e["data"]["local_constant_windows"],
          e["data"]["observation_count"]
        );
      }
    }
    let checked=Command::new("python3").args(["-c","import sys;sys.path.insert(0,sys.argv[1]);from check_lowess import check_svg,rows;check_svg(sys.argv[2],rows(sys.argv[3]))"])
      .arg(example("check_lowess.py").parent().unwrap()).arg(d.path().join("psid_lowess.svg")).arg(d.path().join("psid_lowess.csv")).output().unwrap();
    success(&checked);
    assert_eq!(
      fs::read_to_string(d.path().join("psid.csv")).unwrap(),
      *input
    );
  }
}
