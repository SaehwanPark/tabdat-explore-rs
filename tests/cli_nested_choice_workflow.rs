use serde_json::Value;
use std::{
  fmt::Write,
  fs,
  path::PathBuf,
  process::Command,
  sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-nested-workflow-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let mut csv = String::from(
      "source_row,mode,price,crate,dbeach,dpier,dprivate,dcharter,pbeach,ppier,pprivate,pcharter,qbeach,qpier,qprivate,qcharter,income\n",
    );
    let mut id = 1;
    for (p, q, counts) in [
      ([0, 1, 0, 0], [0, 0, 0, 1], [6, 12, 16, 48]),
      ([0, 0, 0, 1], [0, 1, 0, 0], [8, 24, 9, 18]),
      ([1, 0, 0, 1], [0, 0, 1, 0], [12, 6, 75, 50]),
    ] {
      for income in 0..=1 {
        for (chosen, count) in counts.into_iter().enumerate() {
          let count = count * if chosen < 2 { 3_usize.pow(income) } else { 1 };
          for _ in 0..count {
            writeln!(
              csv,
              "{id},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
              chosen + 1,
              p[chosen],
              q[chosen],
              u8::from(chosen == 0),
              u8::from(chosen == 1),
              u8::from(chosen == 2),
              u8::from(chosen == 3),
              p[0],
              p[1],
              p[2],
              p[3],
              q[0],
              q[1],
              q[2],
              q[3],
              income * 1000
            )
            .unwrap();
            id += 1;
          }
        }
      }
    }
    assert_eq!(id, 705);
    fs::write(path.join("Nldata.csv"), csv).unwrap();
    fs::write(
      path.join("nested.td"),
      include_str!("../examples/mma15p1mnl/nested.td"),
    )
    .unwrap();
    Self(path)
  }
  fn run(&self) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(["--json", "-f", "nested.td"])
      .output()
      .unwrap()
  }
  fn csv(&self, name: &str) -> Vec<Vec<f64>> {
    fs::read_to_string(self.0.join(name))
      .unwrap()
      .lines()
      .skip(1)
      .map(|line| {
        line
          .split(',')
          .map(|value| value.parse().unwrap())
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
fn records(output: &std::process::Output) -> Vec<Value> {
  String::from_utf8(output.stdout.clone())
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn probabilities(rows: &[Vec<f64>; 4], scenario: usize) -> [f64; 4] {
  let utility = std::array::from_fn::<_, 4, _>(|i| {
    let price = rows[i][3] + f64::from(scenario == 2 && i == 3);
    let catch = rows[i][4] + if scenario == 3 && i == 3 { 0.001 } else { 0. };
    2_f64.powf(price) * 3_f64.powf(catch)
  });
  let shore = utility[0] + utility[1];
  let boat = utility[2] + utility[3];
  let income = rows[0][5] + f64::from(scenario == 1);
  let wshore = 2. * 3_f64.powf(income) * shore.powi(2);
  let wboat = boat.powi(3);
  std::array::from_fn(|i| {
    if i < 2 {
      utility[i] / shore * wshore / (wshore + wboat)
    } else {
      utility[i] / boat * wboat / (wshore + wboat)
    }
  })
}

#[test]
fn actual_nested_book_script_fits_restores_displays_and_preserves_exports_and_input() {
  let f = Fixture::new();
  let input = fs::read(f.0.join("Nldata.csv")).unwrap();
  let output = f.run();
  assert!(output.status.success(), "{output:?}");
  let records = records(&output);
  let fit = &records
    .iter()
    .find(|r| r["result_type"] == "NonNormalizedNestedLogitResult")
    .unwrap()["data"];
  assert_eq!(fit["choice_set_count"], 704);
  assert_eq!(fit["observation_count"], 2816);
  assert_eq!(fit["lower_predictor_count"], 2);
  assert_eq!(fit["parameter_count"], 6);
  for (estimate, expected) in fit["coefficients"].as_array().unwrap().iter().zip([
    2_f64.ln(),
    3_f64.ln(),
    2_f64.ln(),
    3_f64.ln(),
  ]) {
    close(estimate["value"].as_f64().unwrap(), expected);
  }
  for (estimate, expected) in fit["dissimilarities"]
    .as_array()
    .unwrap()
    .iter()
    .zip([2., 3.])
  {
    close(estimate["estimate"]["value"].as_f64().unwrap(), expected);
    assert_eq!(estimate["test_null"], 1.);
  }
  let restricted = &records
    .iter()
    .find(|r| r["result_type"] == "ConditionalLogitResult")
    .unwrap()["data"];
  close(
    restricted["log_likelihood"].as_f64().unwrap(),
    fit["restricted_log_likelihood"].as_f64().unwrap(),
  );
  assert_eq!(
    records
      .iter()
      .filter(|r| r["result_type"] == "PredictionResult")
      .count(),
    5
  );
  let rows = f.csv("fishing_nested_predictions.csv");
  assert_eq!(rows.len(), 2816);
  let mut means = [[0.; 4]; 4];
  for set in rows.as_chunks::<4>().0 {
    for (i, row) in set.iter().enumerate() {
      close(row[1], (i + 1) as f64);
      close(row[6], if i < 2 { 1. } else { 2. });
      close(row[7], f64::from(i < 2));
      close(row[8], f64::from(i < 2) * row[5]);
      close(row[9], row[5]);
      close(row[10], row[3]);
      close(row[11], row[4]);
    }
    let scenarios = std::array::from_fn::<_, 4, _>(|scenario| probabilities(set, scenario));
    for i in 0..4 {
      for scenario in 0..4 {
        close(set[i][13 + scenario], scenarios[scenario][i]);
      }
      for (column, value) in [
        scenarios[0][i],
        scenarios[1][i] - scenarios[0][i],
        100. * (scenarios[2][i] - scenarios[0][i]),
        1000. * (scenarios[3][i] - scenarios[0][i]),
      ]
      .into_iter()
      .enumerate()
      {
        means[i][column] += value / 704.;
      }
    }
  }
  let contrasts = f.csv("fishing_nested_contrasts.csv");
  assert_eq!(contrasts.len(), 4);
  let displayed = &records
    .iter()
    .find(|r| r["result_type"] == "PreviewResult")
    .unwrap()["data"];
  assert_eq!(
    displayed["columns"],
    serde_json::json!([
      "alternative",
      "mean_probability",
      "income1000",
      "charter_price1",
      "charter_catch001"
    ])
  );
  for i in 0..4 {
    close(contrasts[i][0], (i + 1) as f64);
    for column in 0..4 {
      close(contrasts[i][column + 1], means[i][column]);
      close(
        displayed["rows"][i][column + 1].as_f64().unwrap(),
        means[i][column],
      );
    }
  }
  assert_eq!(fs::read(f.0.join("Nldata.csv")).unwrap(), input);
  let predictions = fs::read(f.0.join("fishing_nested_predictions.csv")).unwrap();
  let contrast_bytes = fs::read(f.0.join("fishing_nested_contrasts.csv")).unwrap();
  let collision = f.run();
  assert!(!collision.status.success());
  assert_eq!(
    fs::read(f.0.join("fishing_nested_predictions.csv")).unwrap(),
    predictions
  );
  assert_eq!(
    fs::read(f.0.join("fishing_nested_contrasts.csv")).unwrap(),
    contrast_bytes
  );
  assert_eq!(fs::read(f.0.join("Nldata.csv")).unwrap(), input);
}

#[test]
fn actual_nested_book_script_rejects_duplicate_source_ids_or_bad_flags_before_fitting() {
  for duplicate in [true, false] {
    let f = Fixture::new();
    let input = fs::read_to_string(f.0.join("Nldata.csv")).unwrap();
    let mut lines = input.lines().map(str::to_owned).collect::<Vec<_>>();
    if duplicate {
      lines.push(lines[1].clone());
    } else {
      let mut fields = lines[1].split(',').map(str::to_owned).collect::<Vec<_>>();
      fields[4] = "0".into();
      lines[1] = fields.join(",");
    }
    let bad = lines.join("\n") + "\n";
    fs::write(f.0.join("Nldata.csv"), &bad).unwrap();
    let output = f.run();
    assert!(!output.status.success());
    let records = records(&output);
    assert!(records.iter().all(|r| !matches!(
      r["result_type"].as_str(),
      Some("ConditionalLogitResult" | "NonNormalizedNestedLogitResult")
    )));
    assert!(
      String::from_utf8(output.stdout)
        .unwrap()
        .contains(if duplicate {
          "isid"
        } else {
          "source_choice_valid"
        })
    );
    assert!(!f.0.join("fishing_nested_predictions.csv").exists());
    assert!(!f.0.join("fishing_nested_contrasts.csv").exists());
    assert_eq!(fs::read_to_string(f.0.join("Nldata.csv")).unwrap(), bad);
  }
}
