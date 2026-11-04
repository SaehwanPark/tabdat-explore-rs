mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

fn run(dir: &TestDir) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args(["--json", "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma22p1panel/base_2sls.td"))
    .output()
    .unwrap()
}

fn fixture() -> String {
  fixture_n(40)
}

fn fixture_n(people: usize) -> String {
  let mut state = 0x22f1_9a34_u64;
  let mut draw = || {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    (state % 10000) as f64 / 10000.0
  };
  let mut data = String::from("lnhr,lnwg,kids,ageh,agesq,disab,id,year\n");
  for id in 1..=people {
    for year in 1979..=1988 {
      let lnhr = draw();
      let lnwg = draw();
      let kids = draw();
      let ageh = draw();
      let agesq = draw();
      let disab = draw();
      data.push_str(&format!(
        "{lnhr},{lnwg},{kids},{ageh},{agesq},{disab},{id},{year}\n"
      ));
    }
  }
  data
}

#[test]
fn shea_diagnostic_script_reports_and_guards_panel_and_output() {
  let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma22p1panel/shea.td");
  let run = |dir: &TestDir, mode: &str| {
    tabdat_bin()
      .current_dir(dir.path())
      .args([mode, "-f"])
      .arg(&script)
      .output()
      .unwrap()
  };
  let input = fixture();
  let dir = TestDir::new();
  fs::write(dir.path().join("MOMprecise.csv"), &input).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let values: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  assert_eq!(
    values
      .iter()
      .filter(|v| v["result_type"] == "RegressionResult")
      .count(),
    7
  );
  let table = values
    .iter()
    .find(|v| {
      v["result_type"] == "TableResult"
        && v["data"]["headers"]
          == serde_json::json!([
            "observations",
            "residual_correlation",
            "shea_partial_r_squared",
            "interpretation"
          ])
    })
    .unwrap();
  let row = &table["data"]["rows"][0];
  assert_eq!(row[0], 320);
  let rho = row[1].as_f64().unwrap();
  // Independent R QR/correlation reference: shea_synthetic_reference.R.
  assert!((rho - 0.15265692612205664).abs() < 1e-12);
  assert!((row[2].as_f64().unwrap() - 0.023304137093035058).abs() < 1e-12);
  assert_eq!(
    row[3],
    "Descriptive relevance only; not a joint weak-IV test or evidence of instrument validity"
  );
  let path = dir.path().join("shea_design.csv");
  let original = fs::read(&path).unwrap();
  assert_eq!(String::from_utf8_lossy(&original).lines().count(), 321);
  assert!(!run(&dir, "--json").status.success());
  assert_eq!(fs::read(&path).unwrap(), original);
  let text_dir = TestDir::new();
  let mut rows: Vec<_> = input.lines().collect();
  let header = rows.remove(0);
  rows.reverse();
  fs::write(
    text_dir.path().join("MOMprecise.csv"),
    format!("{header}\n{}\n", rows.join("\n")),
  )
  .unwrap();
  let text = run(&text_dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(String::from_utf8_lossy(&text.stdout).contains("shea_partial_r_squared"));
  assert_eq!(
    fs::read(text_dir.path().join("shea_design.csv")).unwrap(),
    original
  );
  let duplicate = format!("{}{}\n", input, input.lines().nth(1).unwrap());
  let missing = input
    .lines()
    .enumerate()
    .filter(|(i, _)| *i != 3)
    .map(|(_, s)| s)
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  let zero_wage = input
    .lines()
    .enumerate()
    .map(|(i, line)| {
      if i == 0 {
        format!("{line}\n")
      } else {
        let mut row = line.split(',').collect::<Vec<_>>();
        row[1] = "0";
        format!("{}\n", row.join(","))
      }
    })
    .collect::<String>();
  for bad in [duplicate, missing, zero_wage] {
    let bad_dir = TestDir::new();
    fs::write(bad_dir.path().join("MOMprecise.csv"), bad).unwrap();
    assert!(!run(&bad_dir, "--json").status.success());
    assert!(!bad_dir.path().join("shea_design.csv").exists());
  }
  // Vary wage across people but not across years: instruments remain nonconstant,
  // while both wage residual vectors have exactly zero variance.
  let zero_variation = input
    .lines()
    .enumerate()
    .map(|(i, line)| {
      if i == 0 {
        format!("{line}\n")
      } else {
        let mut row = line.split(',').collect::<Vec<_>>();
        row[1] = row[6];
        format!("{}\n", row.join(","))
      }
    })
    .collect::<String>();
  let bad_dir = TestDir::new();
  fs::write(bad_dir.path().join("MOMprecise.csv"), zero_variation).unwrap();
  let failure = run(&bad_dir, "--json");
  assert!(!failure.status.success());
  let leaves: Vec<Value> = String::from_utf8(failure.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  assert_eq!(
    leaves
      .iter()
      .filter(|v| v["result_type"] == "RegressionResult")
      .count(),
    7
  );
  assert_eq!(leaves.last().unwrap()["error"]["message"], "sql failed");
  assert!(!bad_dir.path().join("shea_design.csv").exists());
}

#[test]
fn stacked_gmm_panel_script_runs_and_guards_input_and_export() {
  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma22p1panel/stacked_gmm.td");
  let run = |dir: &TestDir, mode: &str| {
    tabdat_bin()
      .current_dir(dir.path())
      .args([mode, "-f"])
      .arg(&script)
      .output()
      .unwrap()
  };
  let input = fixture_n(100);
  let dir = TestDir::new();
  fs::write(dir.path().join("MOMprecise.csv"), &input).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let values: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fit = values
    .iter()
    .find(|v| v["result_type"] == "ClusterGmmResult")
    .unwrap();
  assert_eq!(fit["data"]["observation_count"], 800);
  assert_eq!(fit["data"]["cluster_count"], 100);
  assert_eq!(fit["data"]["overidentification_df"], 67);
  assert_eq!(fit["data"]["instruments"].as_array().unwrap().len(), 72);
  let path = dir.path().join("stacked_gmm_design.csv");
  let original = fs::read(&path).unwrap();
  assert_eq!(String::from_utf8_lossy(&original).lines().count(), 801);
  assert!(!run(&dir, "--json").status.success());
  assert_eq!(fs::read(&path).unwrap(), original);
  let text_dir = TestDir::new();
  let mut rows: Vec<_> = input.lines().collect();
  let header = rows.remove(0);
  rows.reverse();
  fs::write(
    text_dir.path().join("MOMprecise.csv"),
    format!("{header}\n{}\n", rows.join("\n")),
  )
  .unwrap();
  let text = run(&text_dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(String::from_utf8_lossy(&text.stdout).contains("Clustered two-step linear IV GMM"));
  assert_eq!(
    fs::read(text_dir.path().join("stacked_gmm_design.csv")).unwrap(),
    original
  );
  let duplicate = format!("{}{}\n", input, input.lines().nth(1).unwrap());
  let missing = input
    .lines()
    .enumerate()
    .filter(|(i, _)| *i != 3)
    .map(|(_, s)| s)
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  for bad in [duplicate, missing, fixture_n(40)] {
    let bad_dir = TestDir::new();
    fs::write(bad_dir.path().join("MOMprecise.csv"), bad).unwrap();
    assert!(!run(&bad_dir, "--json").status.success());
    assert!(!bad_dir.path().join("stacked_gmm_design.csv").exists());
  }
}

#[test]
fn base_gmm_panel_script_runs_and_guards_input_and_export() {
  let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma22p1panel/base_gmm.td");
  let run = |dir: &TestDir, mode: &str| {
    tabdat_bin()
      .current_dir(dir.path())
      .args([mode, "-f"])
      .arg(&script)
      .output()
      .unwrap()
  };
  let dir = TestDir::new();
  let input = fixture();
  fs::write(dir.path().join("MOMprecise.csv"), &input).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let values: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fits: Vec<_> = values
    .iter()
    .filter(|v| v["result_type"] == "ClusterGmmResult")
    .collect();
  assert_eq!(fits.len(), 1);
  assert_eq!(fits[0]["data"]["observation_count"], 320);
  assert_eq!(fits[0]["data"]["cluster_count"], 40);
  assert_eq!(fits[0]["data"]["overidentification_df"], 4);
  assert_eq!(fits[0]["data"]["coefficients"].as_array().unwrap().len(), 5);
  let path = dir.path().join("panel_gmm_design.csv");
  let original = fs::read(&path).unwrap();
  assert_eq!(String::from_utf8_lossy(&original).lines().count(), 321);
  assert!(!run(&dir, "--json").status.success());
  assert_eq!(fs::read(&path).unwrap(), original);
  let text_dir = TestDir::new();
  let mut rows: Vec<_> = input.lines().collect();
  let header = rows.remove(0);
  rows.reverse();
  fs::write(
    text_dir.path().join("MOMprecise.csv"),
    format!("{header}\n{}\n", rows.join("\n")),
  )
  .unwrap();
  let text = run(&text_dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(String::from_utf8_lossy(&text.stdout).contains("Clustered two-step linear IV GMM"));
  assert_eq!(
    fs::read(text_dir.path().join("panel_gmm_design.csv")).unwrap(),
    original
  );
  let duplicate = format!("{}{}\n", input, input.lines().nth(1).unwrap());
  let missing = input
    .lines()
    .enumerate()
    .filter(|(i, _)| *i != 3)
    .map(|(_, s)| s)
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  let singular = input
    .lines()
    .enumerate()
    .map(|(i, line)| {
      if i == 0 {
        format!("{line}\n")
      } else {
        let mut row = line.split(',').collect::<Vec<_>>();
        row[1] = "0";
        format!("{}\n", row.join(","))
      }
    })
    .collect::<String>();
  for bad in [duplicate, missing, singular] {
    let bad_dir = TestDir::new();
    fs::write(bad_dir.path().join("MOMprecise.csv"), bad).unwrap();
    assert!(!run(&bad_dir, "--json").status.success());
    assert!(!bad_dir.path().join("panel_gmm_design.csv").exists());
  }
}

#[test]
fn complete_panels_fit_both_covariances_and_export_design() {
  let dir = TestDir::new();
  fs::write(dir.path().join("MOMprecise.csv"), fixture()).unwrap();
  let output = run(&dir);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let values: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fits: Vec<_> = values
    .iter()
    .filter(|v| v["result_type"] == "RegressionResult")
    .collect();
  assert_eq!(fits.len(), 2);
  for fit in fits {
    assert_eq!(fit["data"]["observation_count"], 320);
    assert_eq!(fit["data"]["sample"]["dropped_observations"], 0);
    assert_eq!(fit["data"]["include_intercept"], false);
    assert_eq!(fit["data"]["coefficients"].as_array().unwrap().len(), 5);
  }
  let path = dir.path().join("panel_iv_design.csv");
  let original = fs::read(&path).unwrap();
  assert_eq!(String::from_utf8_lossy(&original).lines().count(), 321);
  assert!(!run(&dir).status.success());
  assert_eq!(fs::read(&path).unwrap(), original);

  let shuffled_dir = TestDir::new();
  let fixture = fixture();
  let mut rows: Vec<_> = fixture.lines().collect();
  let header = rows.remove(0);
  rows.reverse();
  let shuffled = format!("{header}\n{}\n", rows.join("\n"));
  fs::write(shuffled_dir.path().join("MOMprecise.csv"), shuffled).unwrap();
  let shuffled_output = run(&shuffled_dir);
  assert!(
    shuffled_output.status.success(),
    "{}",
    String::from_utf8_lossy(&shuffled_output.stderr)
  );
  assert_eq!(
    fs::read(shuffled_dir.path().join("panel_iv_design.csv")).unwrap(),
    original
  );
}

#[test]
fn duplicate_or_missing_years_and_rank_failure_do_not_export() {
  let original = fixture();
  let lines: Vec<_> = original.lines().collect();
  let duplicate = format!("{}\n{}\n", original, lines[1]);
  let missing = format!(
    "{}\n",
    lines
      .iter()
      .enumerate()
      .filter(|(i, _)| *i != 3)
      .map(|(_, line)| *line)
      .collect::<Vec<_>>()
      .join("\n")
  );
  let constant = original
    .lines()
    .enumerate()
    .map(|(i, line)| {
      if i == 0 {
        format!("{line}\n")
      } else {
        let mut row: Vec<_> = line.split(',').collect();
        row[1] = "0";
        format!("{}\n", row.join(","))
      }
    })
    .collect::<String>();
  for data in [duplicate, missing, constant] {
    let dir = TestDir::new();
    fs::write(dir.path().join("MOMprecise.csv"), data).unwrap();
    let output = run(&dir);
    assert!(!output.status.success());
    assert!(!dir.path().join("panel_iv_design.csv").exists());
  }
}

#[test]
fn first_difference_ols_reports_three_covariances_and_protects_export() {
  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma22p1panel/first_difference_ols.td");
  let run_ols = |dir: &TestDir, mode: &str| {
    tabdat_bin()
      .current_dir(dir.path())
      .args([mode, "-f"])
      .arg(&script)
      .output()
      .unwrap()
  };
  let dir = TestDir::new();
  fs::write(dir.path().join("MOMprecise.csv"), fixture()).unwrap();
  let output = run_ols(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let values: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fits: Vec<_> = values
    .iter()
    .filter(|v| v["result_type"] == "RegressionResult")
    .collect();
  assert_eq!(fits.len(), 3);
  for fit in fits {
    assert_eq!(fit["data"]["observation_count"], 320);
    assert_eq!(fit["data"]["sample"]["dropped_observations"], 0);
    assert_eq!(fit["data"]["include_intercept"], false);
    assert_eq!(fit["data"]["coefficients"].as_array().unwrap().len(), 5);
  }
  let path = dir.path().join("fd_ols_design.csv");
  let original = fs::read(&path).unwrap();
  assert_eq!(String::from_utf8_lossy(&original).lines().count(), 321);
  assert!(!run_ols(&dir, "--json").status.success());
  assert_eq!(fs::read(&path).unwrap(), original);

  let text_dir = TestDir::new();
  let input = fixture();
  let mut rows: Vec<_> = input.lines().collect();
  let header = rows.remove(0);
  rows.reverse();
  let reversed = format!("{header}\n{}\n", rows.join("\n"));
  fs::write(text_dir.path().join("MOMprecise.csv"), reversed).unwrap();
  let text = run_ols(&text_dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert_eq!(
    fs::read(text_dir.path().join("fd_ols_design.csv")).unwrap(),
    original
  );

  let data = fixture();
  let missing = data
    .lines()
    .enumerate()
    .filter(|(i, _)| *i != 3)
    .map(|(_, line)| line)
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  let duplicate = format!("{}{}\n", data, data.lines().nth(1).unwrap());
  let singular = data
    .lines()
    .enumerate()
    .map(|(i, line)| {
      if i == 0 {
        format!("{line}\n")
      } else {
        let mut row: Vec<_> = line.split(',').collect();
        row[1] = "0";
        format!("{}\n", row.join(","))
      }
    })
    .collect::<String>();
  for bad in [missing, duplicate, singular] {
    let bad_dir = TestDir::new();
    fs::write(bad_dir.path().join("MOMprecise.csv"), bad).unwrap();
    assert!(!run_ols(&bad_dir, "--json").status.success());
    assert!(!bad_dir.path().join("fd_ols_design.csv").exists());
  }
}

#[test]
fn base_first_stage_tests_joint_instruments_and_guards_output() {
  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma22p1panel/base_first_stage.td");
  let run = |dir: &TestDir, mode: &str| {
    tabdat_bin()
      .current_dir(dir.path())
      .args([mode, "-f"])
      .arg(&script)
      .output()
      .unwrap()
  };
  let dir = TestDir::new();
  fs::write(dir.path().join("MOMprecise.csv"), fixture()).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let values: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fits: Vec<_> = values
    .iter()
    .filter(|v| v["result_type"] == "RegressionResult")
    .collect();
  let tests: Vec<_> = values
    .iter()
    .filter(|v| v["result_type"] == "TestResult")
    .collect();
  assert_eq!(fits.len(), 2);
  assert_eq!(tests.len(), 2);
  for (fit, covariance) in fits.iter().zip(["cluster(id)", "nonrobust"]) {
    assert_eq!(fit["data"]["observation_count"], 320);
    assert_eq!(fit["data"]["include_intercept"], true);
    assert_eq!(fit["data"]["covariance"], covariance);
    assert_eq!(fit["data"]["coefficients"].as_array().unwrap().len(), 10);
  }
  for (test, df) in tests.iter().zip([39, 310]) {
    assert_eq!(test["data"]["df"], 9);
    assert_eq!(test["data"]["df_residual"], df);
    assert_eq!(test["data"]["constraints"].as_array().unwrap().len(), 9);
  }
  let path = dir.path().join("first_stage_design.csv");
  let original = fs::read(&path).unwrap();
  assert_eq!(String::from_utf8_lossy(&original).lines().count(), 321);
  assert!(!run(&dir, "--json").status.success());
  assert_eq!(fs::read(&path).unwrap(), original);

  let text_dir = TestDir::new();
  let input = fixture();
  let mut rows: Vec<_> = input.lines().collect();
  let header = rows.remove(0);
  rows.reverse();
  fs::write(
    text_dir.path().join("MOMprecise.csv"),
    format!("{header}\n{}\n", rows.join("\n")),
  )
  .unwrap();
  let text = run(&text_dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert_eq!(
    fs::read(text_dir.path().join("first_stage_design.csv")).unwrap(),
    original
  );

  let missing = input
    .lines()
    .enumerate()
    .filter(|(i, _)| *i != 3)
    .map(|(_, line)| line)
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  let singular = input
    .lines()
    .enumerate()
    .map(|(i, line)| {
      if i == 0 {
        format!("{line}\n")
      } else {
        let mut row: Vec<_> = line.split(',').collect();
        row[4] = row[3];
        format!("{}\n", row.join(","))
      }
    })
    .collect::<String>();
  for bad in [missing, singular] {
    let bad_dir = TestDir::new();
    fs::write(bad_dir.path().join("MOMprecise.csv"), bad).unwrap();
    assert!(!run(&bad_dir, "--json").status.success());
    assert!(!bad_dir.path().join("first_stage_design.csv").exists());
  }
}

#[test]
fn stacked_wage_relevance_tests_large_joint_restriction() {
  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma22p1panel/stacked_first_stage.td");
  let run = |dir: &TestDir, mode: &str| {
    tabdat_bin()
      .current_dir(dir.path())
      .args([mode, "-f"])
      .arg(&script)
      .output()
      .unwrap()
  };
  let dir = TestDir::new();
  let input = fixture_n(100);
  fs::write(dir.path().join("MOMprecise.csv"), &input).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let values: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fits: Vec<_> = values
    .iter()
    .filter(|v| v["result_type"] == "RegressionResult")
    .collect();
  let tests: Vec<_> = values
    .iter()
    .filter(|v| v["result_type"] == "TestResult")
    .collect();
  assert_eq!(fits.len(), 2);
  assert_eq!(tests.len(), 2);
  for (fit, covariance) in fits.iter().zip(["cluster(id)", "nonrobust"]) {
    assert_eq!(fit["data"]["observation_count"], 800);
    assert_eq!(fit["data"]["include_intercept"], true);
    assert_eq!(fit["data"]["covariance"], covariance);
    assert_eq!(fit["data"]["coefficients"].as_array().unwrap().len(), 73);
  }
  for (test, df) in tests.iter().zip([99, 727]) {
    assert_eq!(test["data"]["df"], 72);
    assert_eq!(test["data"]["df_residual"], df);
    assert_eq!(test["data"]["constraints"].as_array().unwrap().len(), 72);
  }
  let path = dir.path().join("stacked_stage_design.csv");
  let original = fs::read(&path).unwrap();
  assert_eq!(String::from_utf8_lossy(&original).lines().count(), 801);
  assert!(!run(&dir, "--json").status.success());
  assert_eq!(fs::read(&path).unwrap(), original);

  let text_dir = TestDir::new();
  let mut rows: Vec<_> = input.lines().collect();
  let header = rows.remove(0);
  rows.reverse();
  fs::write(
    text_dir.path().join("MOMprecise.csv"),
    format!("{header}\n{}\n", rows.join("\n")),
  )
  .unwrap();
  let text = run(&text_dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert_eq!(
    fs::read(text_dir.path().join("stacked_stage_design.csv")).unwrap(),
    original
  );

  let missing = input
    .lines()
    .enumerate()
    .filter(|(i, _)| *i != 3)
    .map(|(_, line)| line)
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  for (bad, expected_error) in [
    (missing, "sql failed"),
    (fixture(), "constraints are collinear or singular"),
  ] {
    let bad_dir = TestDir::new();
    fs::write(bad_dir.path().join("MOMprecise.csv"), bad).unwrap();
    let output = run(&bad_dir, "--json");
    assert!(!output.status.success());
    assert!(
      String::from_utf8_lossy(&output.stderr).contains(expected_error),
      "expected {expected_error}: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    assert!(!bad_dir.path().join("stacked_stage_design.csv").exists());
  }
}

#[test]
fn remaining_stacked_first_stages_report_and_guard_joint_tests() {
  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma22p1panel/stacked_other_stages.td");
  let run = |dir: &TestDir, mode: &str| {
    tabdat_bin()
      .current_dir(dir.path())
      .args([mode, "-f"])
      .arg(&script)
      .output()
      .unwrap()
  };
  let dir = TestDir::new();
  let input = fixture_n(100);
  fs::write(dir.path().join("MOMprecise.csv"), &input).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let values: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fits: Vec<_> = values
    .iter()
    .filter(|v| v["result_type"] == "RegressionResult")
    .collect();
  let tests: Vec<_> = values
    .iter()
    .filter(|v| v["result_type"] == "TestResult")
    .collect();
  assert_eq!(fits.len(), 4);
  assert_eq!(tests.len(), 4);
  for fit in fits {
    assert_eq!(fit["data"]["observation_count"], 800);
    assert_eq!(fit["data"]["include_intercept"], true);
    assert_eq!(fit["data"]["covariance"], "cluster(id)");
    assert_eq!(fit["data"]["coefficients"].as_array().unwrap().len(), 73);
  }
  for test in tests {
    assert_eq!(test["data"]["df"], 72);
    assert_eq!(test["data"]["df_residual"], 99);
    assert_eq!(test["data"]["constraints"].as_array().unwrap().len(), 72);
  }
  let path = dir.path().join("stacked_other_stages.csv");
  let original = fs::read(&path).unwrap();
  assert_eq!(String::from_utf8_lossy(&original).lines().count(), 801);
  assert!(!run(&dir, "--json").status.success());
  assert_eq!(fs::read(&path).unwrap(), original);

  let text_dir = TestDir::new();
  let mut rows: Vec<_> = input.lines().collect();
  let header = rows.remove(0);
  rows.reverse();
  fs::write(
    text_dir.path().join("MOMprecise.csv"),
    format!("{header}\n{}\n", rows.join("\n")),
  )
  .unwrap();
  let text = run(&text_dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert_eq!(
    fs::read(text_dir.path().join("stacked_other_stages.csv")).unwrap(),
    original
  );

  let missing = input
    .lines()
    .enumerate()
    .filter(|(i, _)| *i != 3)
    .map(|(_, line)| line)
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  for (bad, expected_error) in [
    (missing, "sql failed"),
    (fixture(), "constraints are collinear or singular"),
  ] {
    let bad_dir = TestDir::new();
    fs::write(bad_dir.path().join("MOMprecise.csv"), bad).unwrap();
    let output = run(&bad_dir, "--json");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(expected_error));
    assert!(!bad_dir.path().join("stacked_other_stages.csv").exists());
  }
}

#[test]
fn stacked_instruments_fit_and_preserve_panel_keys() {
  let dir = TestDir::new();
  fs::write(dir.path().join("MOMprecise.csv"), fixture()).unwrap();
  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma22p1panel/stacked_2sls.td");
  let run_stacked = || {
    tabdat_bin()
      .current_dir(dir.path())
      .args(["--json", "-f"])
      .arg(&script)
      .output()
      .unwrap()
  };
  let output = run_stacked();
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let values: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fits: Vec<_> = values
    .iter()
    .filter(|v| v["result_type"] == "RegressionResult")
    .collect();
  assert_eq!(fits.len(), 2);
  for fit in fits {
    assert_eq!(fit["data"]["observation_count"], 320);
    assert_eq!(
      fit["data"]["first_stage"]["rows"][0]["excluded_instruments"]
        .as_array()
        .unwrap()
        .len(),
      72
    );
  }
  let path = dir.path().join("stacked_iv_design.csv");
  let data = fs::read_to_string(&path).unwrap();
  let mut lines = data.lines();
  let header: Vec<_> = lines.next().unwrap().split(',').collect();
  let first: Vec<_> = lines.next().unwrap().split(',').collect();
  let column = |name: &str| header.iter().position(|c| *c == name).unwrap();
  assert_eq!(first[column("year")].parse::<i32>().unwrap(), 1981);
  assert_eq!(first[column("z1y1")], first[column("agehl1")]);
  assert_eq!(first[column("z1y2")].parse::<f64>().unwrap(), 0.0);
  assert!(!run_stacked().status.success());
  assert_eq!(fs::read_to_string(path).unwrap(), data);

  let bad_dir = TestDir::new();
  let fixture = fixture();
  let missing = fixture
    .lines()
    .enumerate()
    .filter(|(i, _)| *i != 3)
    .map(|(_, line)| line)
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  fs::write(bad_dir.path().join("MOMprecise.csv"), missing).unwrap();
  let bad = tabdat_bin()
    .current_dir(bad_dir.path())
    .args(["--json", "-f"])
    .arg(&script)
    .output()
    .unwrap();
  assert!(!bad.status.success());
  assert!(!bad_dir.path().join("stacked_iv_design.csv").exists());
}
