use serde_json::Value;
use std::{
  collections::BTreeMap,
  fmt::Write,
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let p = std::env::temp_dir().join(format!(
      "tabdat-combined-choice-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&p).unwrap();
    let mut csv = String::from(
      "source_row,mode,pbeach,ppier,pprivate,pcharter,qbeach,qpier,qprivate,qcharter,income,dbeach,dpier,dprivate,dcharter\n",
    );
    let mut id = 1;
    // Full-rank eight-column design. Every cell's exact odds relative to beach
    // are 2^(1+y+y²),3^(1+y+y²),4^(1+y); p=y² on pier, q=y² on private.
    for (y, counts) in [(-1, [2, 4, 6, 2]), (0, [2, 4, 6, 8]), (1, [2, 16, 54, 32])] {
      for (choice, count) in counts.iter().enumerate() {
        for _ in 0..*count {
          writeln!(
            csv,
            "{id},{},0,{},0,0,0,0,{},0,{},{},{},{},{}",
            choice + 1,
            y * y,
            y * y,
            y * 1000,
            u8::from(choice == 0),
            u8::from(choice == 1),
            u8::from(choice == 2),
            u8::from(choice == 3)
          )
          .unwrap();
          id += 1;
        }
      }
    }
    fs::write(p.join("Nldata.csv"), csv).unwrap();
    fs::write(
      p.join("model.td"),
      include_str!("../examples/mma15p1mnl/combined.td"),
    )
    .unwrap();
    Self(p)
  }
  fn run(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn json(&self) -> Output {
    self.run(&["--json", "-f", "model.td"])
  }
  fn records(output: &Output) -> Vec<Value> {
    String::from_utf8(output.stdout.clone())
      .unwrap()
      .lines()
      .map(|l| serde_json::from_str(l).unwrap())
      .collect()
  }
  fn csv(&self, name: &str) -> Vec<BTreeMap<String, f64>> {
    let content = fs::read_to_string(self.0.join(name)).unwrap();
    let mut lines = content.lines();
    let fields = lines.next().unwrap().split(',').collect::<Vec<_>>();
    lines
      .map(|l| {
        fields
          .iter()
          .zip(l.split(','))
          .map(|(k, v)| (k.to_string(), v.parse().unwrap()))
          .collect()
      })
      .collect()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

#[test]
fn actual_script_matches_exact_combined_odds_and_joint_income_update_without_refitting() {
  let f = Fixture::new();
  let input = fs::read(f.0.join("Nldata.csv")).unwrap();
  let out = f.json();
  assert!(out.status.success(), "{out:?}");
  let records = Fixture::records(&out);
  assert!(records.iter().all(|r| r["schema_version"] == 1));
  let conditional = records
    .iter()
    .filter(|r| r["result_type"] == "ConditionalLogitResult")
    .map(|r| &r["data"])
    .collect::<Vec<_>>();
  assert_eq!(conditional.len(), 2);
  let encoded = conditional[0];
  let combined = conditional[1];
  for fit in &conditional {
    assert_eq!(fit["choice_set_count"], 138);
    assert_eq!(fit["observation_count"], 552);
    assert_eq!(
      fit["sample"]["retained_indices"],
      serde_json::json!((0..552).collect::<Vec<_>>())
    );
    assert_eq!(fit["covariance_correction"], 1.0);
    assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
  }
  assert_eq!(
    combined["predictors"],
    serde_json::json!(["p", "q", "dummy2", "dummy3", "dummy4", "d2y", "d3y", "d4y"])
  );
  let expected = [
    2.0_f64.ln(),
    3.0_f64.ln(),
    2.0_f64.ln(),
    3.0_f64.ln(),
    4.0_f64.ln(),
    2.0_f64.ln(),
    3.0_f64.ln(),
    4.0_f64.ln(),
  ];
  for (c, &expected) in combined["coefficients"]
    .as_array()
    .unwrap()
    .iter()
    .zip(&expected)
  {
    close(c["value"].as_f64().unwrap(), expected);
  }
  let mnl = &records
    .iter()
    .find(|r| r["result_type"] == "MultinomialLogitResult")
    .unwrap()["data"];
  let permutation = [0, 2, 4, 1, 3, 5];
  for (i, &p) in permutation.iter().enumerate() {
    close(
      mnl["coefficients"][p]["value"].as_f64().unwrap(),
      encoded["coefficients"][i]["value"].as_f64().unwrap(),
    );
    for (j, &q) in permutation.iter().enumerate() {
      close(
        mnl["covariance_matrix"][p][q].as_f64().unwrap(),
        encoded["covariance_matrix"][i][j].as_f64().unwrap(),
      );
    }
  }
  close(
    mnl["log_likelihood"].as_f64().unwrap(),
    encoded["log_likelihood"].as_f64().unwrap(),
  );
  assert_eq!(mnl["likelihood_ratio_df"], 3);
  assert_eq!(encoded["likelihood_ratio_df"], 6);
  assert_eq!(combined["likelihood_ratio_df"], 8);
  assert_ne!(mnl["null_model"], encoded["null_model"]);
  let rows = f.csv("fishing_combined_predictions.csv");
  assert_eq!(rows.len(), 552);
  for (index, row) in rows.iter().enumerate() {
    let id = index / 4 + 1;
    let alternative = index % 4 + 1;
    let y = row["ybase"];
    close(row["id"], id as f64);
    close(row["alternative"], alternative as f64);
    close(row["ydiv1000"], y);
    close(row["p"], row["pbase"]);
    close(row["q"], row["qbase"]);
    let odds = [
      1.0,
      2.0_f64.powf(1.0 + y + y * y),
      3.0_f64.powf(1.0 + y + y * y),
      4.0_f64.powf(1.0 + y),
    ];
    let shifted = [1.0, odds[1] * 2.0, odds[2] * 3.0, odds[3] * 4.0];
    let price = [odds[0], odds[1], odds[2], odds[3] * 2.0];
    let catch = [odds[0], odds[1], odds[2], odds[3] * 3.0_f64.powf(0.001)];
    for (name, weights) in [
      ("baseline", odds),
      ("income", shifted),
      ("price", price),
      ("catch", catch),
    ] {
      close(
        row[name],
        weights[alternative - 1] / weights.iter().sum::<f64>(),
      );
    }
    close(row["encoded_mnl"], row["mnl_probability"]);
    for j in 2..=4 {
      let dummy = f64::from(alternative == j);
      close(row[&format!("dummy{j}")], dummy);
      close(row[&format!("d{j}y")], dummy * y);
    }
  }
  let contrast = f.csv("fishing_combined_contrasts.csv");
  assert_eq!(contrast.len(), 4);
  for (index, row) in contrast.iter().enumerate() {
    close(row["alternative"], (index + 1) as f64);
    for (field, target, factor) in [
      ("mean_probability", "baseline", 0.0),
      ("income1000", "income", 1.0),
      ("charter_price1", "price", 100.0),
      ("charter_catch001", "catch", 1000.0),
    ] {
      let mean = rows
        .iter()
        .skip(index)
        .step_by(4)
        .map(|r| {
          if factor == 0.0 {
            r[target]
          } else {
            factor * (r[target] - r["baseline"])
          }
        })
        .sum::<f64>()
        / 138.0;
      close(row[field], mean);
    }
  }
  assert_eq!(fs::read(f.0.join("Nldata.csv")).unwrap(), input);
  let original = fs::read(f.0.join("fishing_combined_predictions.csv")).unwrap();
  let contrast_bytes = fs::read(f.0.join("fishing_combined_contrasts.csv")).unwrap();
  let collision = f.json();
  assert!(!collision.status.success());
  assert_eq!(
    fs::read(f.0.join("fishing_combined_predictions.csv")).unwrap(),
    original
  );
  assert_eq!(
    fs::read(f.0.join("fishing_combined_contrasts.csv")).unwrap(),
    contrast_bytes
  );
}

#[test]
fn source_identity_and_choice_flags_fail_before_export_without_touching_input() {
  for duplicate in [true, false] {
    let f = Fixture::new();
    let mut content = fs::read_to_string(f.0.join("Nldata.csv"))
      .unwrap()
      .lines()
      .map(str::to_string)
      .collect::<Vec<_>>();
    let mut row = content[if duplicate { 2 } else { 1 }]
      .split(',')
      .map(str::to_string)
      .collect::<Vec<_>>();
    if duplicate {
      row[0] = "1".into();
    } else {
      row[11] = "0".into();
    }
    content[if duplicate { 2 } else { 1 }] = row.join(",");
    let input = content.join("\n") + "\n";
    fs::write(f.0.join("Nldata.csv"), &input).unwrap();
    let out = f.json();
    assert!(!out.status.success());
    let diagnostic = format!(
      "{}{}",
      String::from_utf8_lossy(&out.stdout),
      String::from_utf8_lossy(&out.stderr)
    );
    assert!(
      diagnostic.contains(if duplicate {
        "isid"
      } else {
        "source_choice_valid"
      }),
      "{diagnostic}"
    );
    assert!(!diagnostic.contains("MultinomialLogitResult"));
    assert!(!diagnostic.contains("ConditionalLogitResult"));
    assert!(!f.0.join("fishing_combined_predictions.csv").exists());
    assert!(!f.0.join("fishing_combined_contrasts.csv").exists());
    assert_eq!(fs::read_to_string(f.0.join("Nldata.csv")).unwrap(), input);
  }
}
