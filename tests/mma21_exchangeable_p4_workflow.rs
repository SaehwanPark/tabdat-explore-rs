use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASE: &str = include_str!("../examples/mma21p1panel/gaussian_pa.td");
const P4: &str = include_str!("../examples/mma21p1panel/exchangeable_p4.td");
const BASE_FILES: [&str; 4] = ["fit", "coefficients", "predictions", "unit_scores"];
const P4_FILES: [&str; 2] = ["fit", "coefficients"];
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Dir(PathBuf);
impl Dir {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-p4-exchangeable-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    Self(path)
  }
}
impl Drop for Dir {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn fixture(kind: &str, name: &str, p4: bool) -> String {
  let folder = if p4 {
    "mma21_exchangeable_p4"
  } else {
    "mma21_gaussian_pa"
  };
  fs::read_to_string(
    Path::new(env!("CARGO_MANIFEST_DIR"))
      .join(format!("tests/fixtures/{folder}/{kind}_{name}.csv")),
  )
  .unwrap()
}
fn path(dir: &Path, p4: bool, name: &str) -> PathBuf {
  dir.join(format!(
    "panel_{}_{name}.csv",
    if p4 { "exchangeable_p4" } else { "gaussian_pa" }
  ))
}
fn run(dir: &Path, base: &str, p4: &str, script: &str, json: bool) -> Output {
  fs::write(dir.join("pa.td"), base).unwrap();
  fs::write(dir.join("p4.td"), p4).unwrap();
  fs::write(dir.join("workflow.td"), script).unwrap();
  Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(dir)
    .arg(if json { "--json" } else { "--text" })
    .arg("-f")
    .arg("workflow.td")
    .output()
    .unwrap()
}
const BOTH: &str = "run \"pa.td\"\nrun \"p4.td\"\n";
type Row = BTreeMap<String, String>;
fn rows(text: &str) -> Vec<Row> {
  let mut lines = text.lines();
  let header: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let values: Vec<_> = line.split(',').collect();
      assert_eq!(header.len(), values.len());
      header
        .iter()
        .zip(values)
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
    })
    .collect()
}
fn compare(actual: &Row, expected: &Row) {
  for (key, value) in expected {
    let got = &actual[key];
    if matches!(key.as_str(), "covariance" | "parameter") || value.is_empty() {
      assert_eq!(got, value, "{key}");
    } else {
      let x: f64 = got.parse().unwrap();
      let y: f64 = value.parse().unwrap();
      let tol = if key == "p_value" {
        y.abs() * 1e-8
      } else {
        1e-9
      };
      assert!(
        x.is_finite() && y.is_finite() && (x - y).abs() <= tol,
        "{key}: {x} != {y}, tol={tol}"
      );
    }
  }
}
fn success(kind: &str) {
  let original = Dir::new();
  let dir = Dir::new();
  let input = fixture(kind, "inputs", false);
  for d in [&original, &dir] {
    fs::write(d.0.join("MOM.csv"), &input).unwrap();
  }
  let base = run(&original.0, BASE, P4, "run \"pa.td\"\n", true);
  assert!(
    base.status.success(),
    "{}",
    String::from_utf8_lossy(&base.stderr)
  );
  let out = run(&dir.0, BASE, P4, BOTH, true);
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let events: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  let mut bytes = Vec::new();
  for name in BASE_FILES {
    let actual = fs::read(path(&dir.0, false, name)).unwrap();
    assert_eq!(
      actual,
      fs::read(path(&original.0, false, name)).unwrap(),
      "original export changed: {name}"
    );
    bytes.push(actual);
  }
  for name in P4_FILES {
    let text = fs::read_to_string(path(&dir.0, true, name)).unwrap();
    let actual = rows(&text);
    let expected = rows(&fixture(kind, name, true));
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(&expected) {
      compare(a, b);
    }
    if name == "fit" {
      let a = &actual[0];
      assert_eq!(a["convention"], "legacy_pearson_exchangeable_rss_over_n");
      assert_eq!(a["unit_covariance"], "g_over_g_minus_1_unit_sandwich");
      assert_eq!(a["inference"], "normal_z_chi2_1");
      let g: f64 = a["people"].parse().unwrap();
      let m: f64 = a["unit_multiplier"].parse().unwrap();
      assert!((m - g / (g - 1.0)).abs() < 1e-14);
      let rho: f64 = a["working_correlation"].parse().unwrap();
      assert_eq!(rho > 0.0, kind == "positive");
      for suffix in ["aa", "ab", "bb"] {
        let raw: f64 = a[&format!("raw_unit_covariance_{suffix}")].parse().unwrap();
        let adjusted: f64 = a[&format!("adjusted_unit_covariance_{suffix}")]
          .parse()
          .unwrap();
        assert!((adjusted - raw * m).abs() < 1e-14);
      }
    } else {
      let table = events
        .iter()
        .find(|e| {
          e["result_type"] == "TableResult"
            && e["data"]["headers"][0] == "covariance"
            && e["data"]["headers"].as_array().unwrap().len() == 9
        })
        .unwrap();
      let h = table["data"]["headers"].as_array().unwrap();
      let values = table["data"]["rows"].as_array().unwrap();
      assert_eq!(values.len(), 4);
      for (row, expected) in values.iter().zip(&expected) {
        for (key, value) in h.iter().zip(row.as_array().unwrap()) {
          let key = key.as_str().unwrap();
          if expected[key].is_empty() {
            assert!(value.is_null());
          } else if matches!(key, "covariance" | "parameter") {
            assert_eq!(value.as_str().unwrap(), expected[key]);
          } else {
            assert!(value.is_number());
            let a = BTreeMap::from([(key.to_owned(), value.to_string())]);
            let b = BTreeMap::from([(key.to_owned(), expected[key].clone())]);
            compare(&a, &b);
          }
        }
      }
    }
    bytes.push(text.into_bytes());
  }
  let statuses: Vec<_> = events
    .iter()
    .filter(|e| e["result_type"] == "StatusResult")
    .collect();
  let status = statuses.last().unwrap();
  assert_eq!(status["data"]["estimation_family"], "linear");
  assert_eq!(
    status["data"]["active_table_name"],
    "gaussian_pa_predictions"
  );
  assert_eq!(status["data"]["linear_model"]["observation_count"], 320);
  for name in BASE_FILES {
    fs::remove_file(path(&dir.0, false, name)).unwrap();
  }
  for name in P4_FILES {
    fs::remove_file(path(&dir.0, true, name)).unwrap();
  }
  let text = run(
    &dir.0,
    BASE,
    P4,
    &format!("{BOTH}predict retained_pols, xb\nexport retained.csv\n"),
    false,
  );
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let stdout = String::from_utf8(text.stdout).unwrap();
  assert!(stdout.contains("unit_sandwich_g_adjusted") && stdout.contains("slope_wald_chi2_1"));
  for (i, name) in BASE_FILES.iter().enumerate() {
    assert_eq!(fs::read(path(&dir.0, false, name)).unwrap(), bytes[i]);
  }
  for (i, name) in P4_FILES.iter().enumerate() {
    assert_eq!(fs::read(path(&dir.0, true, name)).unwrap(), bytes[4 + i]);
  }
  let mut distinct = false;
  for r in rows(&fs::read_to_string(dir.0.join("retained.csv")).unwrap()) {
    let x: f64 = r["retained_pols"].parse().unwrap();
    let pols: f64 = r["pols_mean"].parse().unwrap();
    let gee: f64 = r["gee_mean"].parse().unwrap();
    assert!((x - pols).abs() < 1e-12);
    distinct |= (x - gee).abs() > 1e-8;
  }
  assert!(
    distinct,
    "retained state is pooled OLS, not exchangeable GEE"
  );
}
#[test]
fn positive_source_script_matches_r_and_preserves_baseline() {
  success("positive");
}
#[test]
fn negative_exchangeable_source_script_matches_r() {
  success("negative");
}

#[test]
fn missing_prerequisite_and_bad_panels_fail_without_new_exports() {
  let original = fixture("positive", "inputs", false);
  let lines: Vec<_> = original.lines().collect();
  let cases = [
    (original.clone(), BASE.to_owned(), "run \"p4.td\"\n"),
    (format!("{original}{}\n", lines[1]), BASE.to_owned(), BOTH),
    (
      format!("{}\n", lines[..lines.len() - 1].join("\n")),
      BASE.to_owned(),
      BOTH,
    ),
    (fixture("rank", "inputs", false), BASE.to_owned(), BOTH),
    (fixture("singular", "inputs", false), BASE.to_owned(), BOTH),
    (
      original.clone(),
      BASE.replacen("iteration<100", "iteration<0", 1),
      BOTH,
    ),
  ];
  assert!(BASE.contains("iteration<100"));
  for (input, base, script) in cases {
    let dir = Dir::new();
    fs::write(dir.0.join("MOM.csv"), &input).unwrap();
    let out = run(&dir.0, &base, P4, script, true);
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(1));
    for name in P4_FILES {
      assert!(!path(&dir.0, true, name).exists());
    }
    assert_eq!(fs::read_to_string(dir.0.join("MOM.csv")).unwrap(), input);
  }
}
#[test]
fn injected_unresolved_unit_variance_fails_before_new_exports() {
  for column in ["unit_covariance_aa", "unit_covariance_bb"] {
    let dir = Dir::new();
    let input = fixture("positive", "inputs", false);
    fs::write(dir.0.join("MOM.csv"), &input).unwrap();
    let contaminated = P4.replacen("use gaussian_pa_fit", "use contaminated_fit", 1);
    let script = format!(
      "run \"pa.td\"\nuse gaussian_pa_fit\nsql \"\"\"SELECT * REPLACE (1e-99 AS {column}) FROM active\"\"\" into contaminated_fit\nrun \"p4.td\"\n"
    );
    let out = run(&dir.0, BASE, &contaminated, &script, true);
    assert!(!out.status.success(), "accepted {column}");
    assert_eq!(out.status.code(), Some(1));
    for name in BASE_FILES {
      assert!(path(&dir.0, false, name).exists());
    }
    for name in P4_FILES {
      assert!(!path(&dir.0, true, name).exists());
    }
    assert_eq!(fs::read_to_string(dir.0.join("MOM.csv")).unwrap(), input);
  }
}
#[test]
fn all_six_export_collisions_preserve_owners_and_prior_effects() {
  let targets: Vec<_> = BASE_FILES
    .iter()
    .map(|n| (false, *n))
    .chain(P4_FILES.iter().map(|n| (true, *n)))
    .collect();
  for (stage, (p4, name)) in targets.iter().enumerate() {
    let dir = Dir::new();
    let input = fixture("positive", "inputs", false);
    fs::write(dir.0.join("MOM.csv"), &input).unwrap();
    fs::write(path(&dir.0, *p4, name), "existing owner\n").unwrap();
    let out = run(&dir.0, BASE, P4, BOTH, true);
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
      fs::read_to_string(path(&dir.0, *p4, name)).unwrap(),
      "existing owner\n"
    );
    for (i, (p, n)) in targets.iter().enumerate() {
      assert_eq!(path(&dir.0, *p, n).exists(), i <= stage);
    }
    assert_eq!(fs::read_to_string(dir.0.join("MOM.csv")).unwrap(), input);
  }
}

fn change_first_row(file: &Path, column: &str) -> String {
  let original = fs::read_to_string(file).unwrap();
  let mut lines: Vec<_> = original.lines().map(str::to_owned).collect();
  let index = lines[0].split(',').position(|v| v == column).unwrap();
  let mut fields: Vec<_> = lines[1].split(',').map(str::to_owned).collect();
  fields[index] = "0.123456789".to_owned();
  lines[1] = fields.join(",");
  fs::write(file, format!("{}\n", lines.join("\n"))).unwrap();
  original
}

#[test]
fn source_bound_checker_rejects_forged_reports_and_shared_wrong_sources() {
  let dir = Dir::new();
  let input = fixture("positive", "inputs", false);
  fs::write(dir.0.join("MOM.csv"), &input).unwrap();
  let source = input
    .lines()
    .skip(1)
    .map(|line| line.split_once(',').unwrap().1.replace(',', " "))
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  fs::write(dir.0.join("MOM.dat"), source).unwrap();
  let output = run(&dir.0, BASE, P4, BOTH, true);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let base_ref = dir.0.join("base_ref");
  let p4_ref = dir.0.join("p4_ref");
  fs::create_dir(&base_ref).unwrap();
  fs::create_dir(&p4_ref).unwrap();
  for name in BASE_FILES {
    fs::write(
      base_ref.join(format!("{name}.csv")),
      fixture("positive", name, false),
    )
    .unwrap();
  }
  for name in P4_FILES {
    fs::write(
      p4_ref.join(format!("{name}.csv")),
      fixture("positive", name, true),
    )
    .unwrap();
  }
  let check = || {
    Command::new("python3")
      .arg(
        Path::new(env!("CARGO_MANIFEST_DIR"))
          .join("examples/mma21p1panel/check_exchangeable_p4.py"),
      )
      .arg(&dir.0)
      .arg(&base_ref)
      .arg(&p4_ref)
      .arg(dir.0.join("MOM.dat"))
      .arg("--synthetic")
      .output()
      .unwrap()
  };
  let good = check();
  assert!(
    good.status.success(),
    "{}",
    String::from_utf8_lossy(&good.stderr)
  );
  for (p4, name, column) in [
    (true, "fit", "unit_multiplier"),
    (true, "fit", "adjusted_unit_covariance_ab"),
    (true, "coefficients", "p_value"),
    (false, "fit", "working_correlation"),
    (false, "predictions", "gee_mean"),
    (false, "unit_scores", "influence_slope"),
  ] {
    let file = path(&dir.0, p4, name);
    let original = change_first_row(&file, column);
    assert!(
      !check().status.success(),
      "checker accepted forged {name}.{column}"
    );
    fs::write(file, original).unwrap();
  }
  let native = path(&dir.0, false, "predictions");
  let reference = base_ref.join("predictions.csv");
  let n = change_first_row(&native, "lnwg");
  let r = change_first_row(&reference, "lnwg");
  assert!(
    !check().status.success(),
    "source binding accepted a shared wrong input"
  );
  fs::write(native, n).unwrap();
  fs::write(reference, r).unwrap();
}
