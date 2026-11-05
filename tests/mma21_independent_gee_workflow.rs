use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const SCRIPT: &str = include_str!("../examples/mma21p1panel/independent_gee.td");
const FILES: [&str; 4] = ["fit", "coefficients", "predictions", "unit_scores"];
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Dir(PathBuf);
impl Dir {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-independent-gee-{}-{}",
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
fn fixture(kind: &str, name: &str) -> String {
  let folder = if name == "inputs" {
    "mma21_gaussian_pa"
  } else {
    "mma21_independent_gee"
  };
  fs::read_to_string(
    Path::new(env!("CARGO_MANIFEST_DIR"))
      .join(format!("tests/fixtures/{folder}/{kind}_{name}.csv")),
  )
  .unwrap()
}
fn output_path(dir: &Path, name: &str) -> PathBuf {
  dir.join(format!("panel_independent_gee_{name}.csv"))
}
fn run(dir: &Path, script: &str, json: bool) -> Output {
  let path = dir.join("workflow.td");
  fs::write(&path, script).unwrap();
  Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(dir)
    .arg(if json { "--json" } else { "--text" })
    .arg("-f")
    .arg(path)
    .output()
    .unwrap()
}
type Row = BTreeMap<String, String>;
fn rows(text: &str) -> Vec<Row> {
  let mut lines = text.lines();
  let headers: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let values: Vec<_> = line.split(',').collect();
      assert_eq!(headers.len(), values.len());
      headers
        .iter()
        .zip(values)
        .map(|(k, v)| (k.to_string(), v.to_string()))
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
        "{key}: {x} != {y}, tol {tol}"
      );
    }
  }
}
fn success(kind: &str) {
  let dir = Dir::new();
  let input = fixture(kind, "inputs");
  fs::write(dir.0.join("MOM.csv"), &input).unwrap();
  let output = run(&dir.0, SCRIPT, true);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let mut original = Vec::new();
  for name in FILES {
    let bytes = fs::read_to_string(output_path(&dir.0, name)).unwrap();
    let actual = rows(&bytes);
    let expected = rows(&fixture(kind, name));
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(&expected) {
      compare(a, b);
    }
    if name == "fit" {
      assert_eq!(actual[0]["convention"], "rss_over_n_independent");
      assert_eq!(
        actual[0]["unit_covariance"],
        "g_over_g_minus_1_unit_sandwich"
      );
      assert_eq!(actual[0]["inference"], "normal_z_chi2_1");
    }
    if name == "predictions" {
      for (i, (a, b)) in actual.iter().zip(rows(&input)).enumerate() {
        assert_eq!(a["source_row"].parse::<usize>().unwrap(), i + 1);
        for (key, value) in b {
          assert_eq!(
            a[&key].parse::<f64>().unwrap(),
            value.parse::<f64>().unwrap()
          );
        }
      }
    }
    if name == "coefficients" {
      let table = events
        .iter()
        .find(|e| e["result_type"] == "TableResult" && e["data"]["headers"][0] == "covariance")
        .unwrap();
      let headers = table["data"]["headers"].as_array().unwrap();
      let values = table["data"]["rows"].as_array().unwrap();
      assert_eq!(headers.len(), 9);
      assert_eq!(values.len(), 4);
      for (row, expected) in values.iter().zip(&expected) {
        for (header, value) in headers.iter().zip(row.as_array().unwrap()) {
          let key = header.as_str().unwrap();
          if expected[key].is_empty() {
            assert!(value.is_null());
          } else if matches!(key, "covariance" | "parameter") {
            assert_eq!(value.as_str().unwrap(), expected[key]);
          } else {
            assert!(value.is_number());
            let one = BTreeMap::from([(key.to_string(), value.to_string())]);
            let exp = BTreeMap::from([(key.to_string(), expected[key].clone())]);
            compare(&one, &exp);
          }
        }
      }
    }
    original.push(bytes);
  }
  let status = events
    .iter()
    .find(|e| e["result_type"] == "StatusResult")
    .unwrap();
  assert_eq!(status["data"]["estimation_family"], "linear");
  assert_eq!(
    status["data"]["active_table_name"],
    "independent_gee_predictions"
  );
  assert_eq!(status["data"]["linear_model"]["observation_count"], 320);
  for name in FILES {
    fs::remove_file(output_path(&dir.0, name)).unwrap();
  }
  let text = run(
    &dir.0,
    &format!("{SCRIPT}\npredict retained_pols, xb\nexport retained_check.csv\n"),
    false,
  );
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let stdout = String::from_utf8(text.stdout).unwrap();
  assert!(stdout.contains("unit_sandwich_g_adjusted") && stdout.contains("slope_wald_chi2_1"));
  for (name, bytes) in FILES.iter().zip(original) {
    assert_eq!(
      fs::read_to_string(output_path(&dir.0, name)).unwrap(),
      bytes
    );
  }
  for row in rows(&fs::read_to_string(dir.0.join("retained_check.csv")).unwrap()) {
    let model: f64 = row["retained_pols"].parse().unwrap();
    let existing: f64 = row["pols_mean"].parse().unwrap();
    assert!((model - existing).abs() < 1e-12);
  }
}
#[test]
fn independent_gee_matches_r_positive_control_and_preserves_pols() {
  success("positive");
}
#[test]
fn independent_gee_matches_r_negative_control() {
  success("negative");
}

#[test]
fn rejects_bad_keys_missing_design_and_perfect_fit_without_exports() {
  let input = fixture("positive", "inputs");
  let lines: Vec<_> = input.lines().collect();
  let mut missing: Vec<_> = lines[1].split(',').collect();
  missing[1] = "";
  let mut nonfinite: Vec<_> = lines[1].split(',').collect();
  nonfinite[2] = "NaN";
  let mut oversized: Vec<_> = lines[1].split(',').collect();
  oversized[2] = "21";
  let mut duplicated_row: Vec<_> = lines[2].split(',').collect();
  duplicated_row[0] = lines[1].split(',').next().unwrap();
  let mut perfect = format!("{}\n", lines[0]);
  let mut rank = format!("{}\n", lines[0]);
  for line in &lines[1..] {
    let mut fields: Vec<_> = line.split(',').map(str::to_owned).collect();
    fields[2] = "2".to_owned();
    rank.push_str(&fields.join(","));
    rank.push('\n');
    fields[2] = line.split(',').nth(2).unwrap().to_owned();
    fields[1] = (2.0 + 0.5 * fields[2].parse::<f64>().unwrap()).to_string();
    perfect.push_str(&fields.join(","));
    perfect.push('\n');
  }
  let cases = [
    format!("{input}{}\n", lines[1]),
    format!("{}\n", lines[..lines.len() - 1].join("\n")),
    format!(
      "{}\n{}\n{}\n",
      lines[0],
      missing.join(","),
      lines[2..].join("\n")
    ),
    format!(
      "{}\n{}\n{}\n",
      lines[0],
      nonfinite.join(","),
      lines[2..].join("\n")
    ),
    format!(
      "{}\n{}\n{}\n",
      lines[0],
      oversized.join(","),
      lines[2..].join("\n")
    ),
    format!(
      "{}\n{}\n{}\n{}\n",
      lines[0],
      lines[1],
      duplicated_row.join(","),
      lines[3..].join("\n")
    ),
    format!("{}\n", lines[0]),
    rank,
    perfect,
  ];
  for (index, input) in cases.into_iter().enumerate() {
    let dir = Dir::new();
    fs::write(dir.0.join("MOM.csv"), &input).unwrap();
    let output = run(&dir.0, SCRIPT, true);
    assert!(
      !output.status.success(),
      "case {index}: {}",
      String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(output.status.code(), Some(1));
    for file in FILES {
      assert!(!output_path(&dir.0, file).exists());
    }
    assert_eq!(fs::read_to_string(dir.0.join("MOM.csv")).unwrap(), input);
  }
}
#[test]
fn unresolved_zero_unit_scores_fail_before_export() {
  let dir = Dir::new();
  let mut input = String::from("source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year\n");
  for id in 1..=32 {
    for time in 0..10 {
      let x = 1.0 + f64::from(id) * 0.01 + f64::from(time / 2) * 0.02;
      let y = 7.0 + 0.2 * x + if time % 2 == 0 { 0.1 } else { -0.1 };
      input.push_str(&format!(
        "{},{y:.17},{x:.17},0,20,400,0,{id},{}\n",
        (id - 1) * 10 + time + 1,
        1979 + time
      ));
    }
  }
  fs::write(dir.0.join("MOM.csv"), &input).unwrap();
  let output = run(&dir.0, SCRIPT, true);
  assert!(!output.status.success());
  assert_eq!(output.status.code(), Some(1));
  assert!(
    String::from_utf8(output.stdout)
      .unwrap()
      .contains("RegressionResult")
  );
  for file in FILES {
    assert!(!output_path(&dir.0, file).exists());
  }
  assert_eq!(fs::read_to_string(dir.0.join("MOM.csv")).unwrap(), input);
}

#[test]
fn export_collisions_preserve_owners_and_prior_ordered_effects() {
  for (stage, file) in FILES.iter().enumerate() {
    let dir = Dir::new();
    let input = fixture("positive", "inputs");
    fs::write(dir.0.join("MOM.csv"), &input).unwrap();
    fs::write(output_path(&dir.0, file), "existing owner\n").unwrap();
    let output = run(&dir.0, SCRIPT, true);
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
      fs::read_to_string(output_path(&dir.0, file)).unwrap(),
      "existing owner\n"
    );
    for (i, name) in FILES.iter().enumerate() {
      assert_eq!(output_path(&dir.0, name).exists(), i <= stage);
    }
    assert_eq!(fs::read_to_string(dir.0.join("MOM.csv")).unwrap(), input);
  }
}

#[test]
fn audit_rejects_forged_fit_inference_predictions_and_scores() {
  let dir = Dir::new();
  fs::write(dir.0.join("MOM.csv"), fixture("positive", "inputs")).unwrap();
  let output = run(&dir.0, SCRIPT, true);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let reference = dir.0.join("reference");
  fs::create_dir(&reference).unwrap();
  for name in FILES {
    fs::write(
      reference.join(format!("{name}.csv")),
      fixture("positive", name),
    )
    .unwrap();
  }
  let check = || {
    Command::new("python3")
      .arg(
        Path::new(env!("CARGO_MANIFEST_DIR"))
          .join("examples/mma21p1panel/check_independent_gee.py"),
      )
      .arg(&dir.0)
      .arg(&reference)
      .output()
      .unwrap()
  };
  let initial = check();
  assert!(
    initial.status.success(),
    "{}",
    String::from_utf8_lossy(&initial.stderr)
  );
  for (name, column) in [
    ("fit", "unit_covariance_ab"),
    ("fit", "scale"),
    ("coefficients", "p_value"),
    ("predictions", "gee_mean"),
    ("predictions", "lnwg"),
    ("unit_scores", "influence_slope"),
  ] {
    let path = output_path(&dir.0, name);
    let original = fs::read_to_string(&path).unwrap();
    let mut lines: Vec<_> = original.lines().map(str::to_owned).collect();
    let column_index = lines[0]
      .split(',')
      .position(|field| field == column)
      .unwrap();
    let mut values: Vec<_> = lines[1].split(',').map(str::to_owned).collect();
    values[column_index] = "0.123456789".to_owned();
    lines[1] = values.join(",");
    fs::write(&path, format!("{}\n", lines.join("\n"))).unwrap();
    assert!(
      !check().status.success(),
      "checker accepted forged {name}.{column}"
    );
    fs::write(&path, original).unwrap();
  }
}
