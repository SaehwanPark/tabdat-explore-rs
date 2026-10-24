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
      "tabdat-normalized-common-workflow-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let mut csv = String::from(
      "source_row,mode,price,crate,dbeach,dpier,dprivate,dcharter,pbeach,ppier,pprivate,pcharter,qbeach,qpier,qprivate,qcharter,income\n",
    );
    let mut id = 1;
    for (p, q, k) in [
      ([0, 2, 1, 3], [1, 0, 1, 0], 2),
      ([0, 2, 0, 2], [1, 0, 2, 1], 3),
      ([0, 2, 0, 2], [1, 0, 1, 0], 1),
    ] {
      for income in 0..=1 {
        let e = 3_usize.pow(income);
        for (chosen, count) in [18 * e, 32 * e, 9 * k, 16 * k].into_iter().enumerate() {
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
    assert_eq!(id, 901);
    fs::write(path.join("Nldata.csv"), csv).unwrap();
    fs::write(
      path.join("normalized_common.td"),
      include_str!("../examples/mma15p1mnl/normalized_common.td"),
    )
    .unwrap();
    Self(path)
  }
  fn run(&self) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(["--json", "-f", "normalized_common.td"])
      .output()
      .unwrap()
  }
  fn csv(&self, name: &str) -> Vec<Vec<f64>> {
    fs::read_to_string(self.0.join(name))
      .unwrap()
      .lines()
      .skip(1)
      .map(|r| r.split(',').map(|s| s.parse().unwrap()).collect())
      .collect()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn records(output: &std::process::Output) -> Vec<Value> {
  String::from_utf8(output.stdout.clone())
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}
fn close(a: f64, b: f64) {
  assert!((a - b).abs() <= 1e-9 + 1e-8 * b.abs(), "{a} != {b}");
}
fn probabilities(rows: &[Vec<f64>; 4], scenario: usize) -> [f64; 4] {
  let u = std::array::from_fn::<_, 4, _>(|j| {
    let p = rows[j][3] + f64::from(scenario == 2 && j == 3);
    let q = rows[j][4] + if scenario == 3 && j == 3 { 0.001 } else { 0.0 };
    // Natural beta=ln2/ln3, lambda=.5: conditional weights4^p*9^q.
    4_f64.powf(p) * 9_f64.powf(q)
  });
  let shore = u[0] + u[1];
  let boat = u[2] + u[3];
  let income = rows[0][5] + f64::from(scenario == 1);
  let a = 2.0 * 3_f64.powf(income) * shore.sqrt();
  let b = boat.sqrt();
  std::array::from_fn(|j| {
    if j < 2 {
      u[j] / shore * a / (a + b)
    } else {
      u[j] / boat * b / (a + b)
    }
  })
}

#[test]
fn actual_normalized_common_script_matches_r_natural_covariance_predictions_and_restores() {
  let f = Fixture::new();
  let input = fs::read(f.0.join("Nldata.csv")).unwrap();
  let output = f.run();
  assert!(output.status.success(), "{output:?}");
  let records = records(&output);
  assert!(
    records
      .iter()
      .all(|r| r["schema_version"] == 1 && r.get("error").is_none())
  );
  let data = |name: &str| &records.iter().find(|r| r["result_type"] == name).unwrap()["data"];
  let fit = data("NormalizedCommonNestedLogitResult");
  let restricted = data("ConditionalLogitResult");
  assert_eq!(fit["choice_set_count"], 900);
  assert_eq!(fit["observation_count"], 3600);
  assert_eq!(fit["parameter_count"], 5);
  assert_eq!(fit["lower_predictor_count"], 2);
  assert_eq!(fit["preferred_rum_normalization"], true);
  assert_eq!(fit["shared_nest_scale"], true);
  assert_eq!(fit["random_utility_iv_bounds_satisfied"], true);
  assert_eq!(fit["scale_test_null"], 1.0);
  assert_eq!(fit["independence_df"], 1);
  assert_eq!(
    fit["covariance_parameter_names"],
    serde_json::json!(["p", "q", "dshore", "dshorey", "iv_common"])
  );
  assert!(fit.get("fitted_probabilities").is_none());
  assert!(fit.get("null_log_likelihood").is_none());
  let mut checked = 0;
  for line in
    include_str!("../crates/tabdat-stats/tests/fixtures/normalized_common_choice_reference.csv")
      .lines()
      .skip(1)
  {
    let c = line.split(',').collect::<Vec<_>>();
    let i = c[2].parse::<usize>().unwrap();
    let j = c[3].parse::<usize>().unwrap();
    let b = c[4].parse::<f64>().unwrap();
    let model = if c[0] == "normalized_common" {
      fit
    } else {
      restricted
    };
    let value = match c[1] {
      "covariance" => &model["covariance_matrix"][i - 1][j - 1],
      s if s.starts_with("parameter_") => {
        let parameter = if i <= 4 {
          &model["coefficients"][i - 1]
        } else {
          &model["dissimilarity"]
        };
        &parameter[s.strip_prefix("parameter_").unwrap()]
      }
      key => &model[key],
    };
    let a = value.as_f64().unwrap();
    close(a, b);
    if c[1].ends_with("p_value") && b > 0.0 {
      assert!((a / b - 1.0).abs() < 1e-8);
    }
    checked += 1;
  }
  assert_eq!(checked, 99);
  let prediction = f.csv("fishing_normalized_common_predictions.csv");
  let summary = f.csv("fishing_normalized_common_contrasts.csv");
  assert_eq!(prediction.len(), 3600);
  assert_eq!(summary.len(), 4);
  let mut means = [[0.0; 4]; 4];
  let (choice_sets, remainder) = prediction.as_chunks::<4>();
  assert!(remainder.is_empty());
  for (g, chunk) in choice_sets.iter().enumerate() {
    let rows: [Vec<f64>; 4] = std::array::from_fn(|j| chunk[j].clone());
    for (j, row) in rows.iter().enumerate() {
      assert_eq!(row[0], (g + 1) as f64);
      assert_eq!(row[1], (j + 1) as f64);
      assert_eq!(row[3], row[10]);
      assert_eq!(row[4], row[11]);
      assert_eq!(row[5], row[9]);
      assert_eq!(row[8], row[7] * row[5]);
    }
    for scenario in 0..4 {
      let expected = probabilities(&rows, scenario);
      let column = 13 + scenario;
      close(rows.iter().map(|r| r[column]).sum(), 1.0);
      for j in 0..4 {
        assert!((rows[j][column] - expected[j]).abs() < 1e-10);
      }
    }
    for j in 0..4 {
      for (s, v) in [
        rows[j][13],
        rows[j][14] - rows[j][13],
        100.0 * (rows[j][15] - rows[j][13]),
        1000.0 * (rows[j][16] - rows[j][13]),
      ]
      .into_iter()
      .enumerate()
      {
        means[j][s] += v / 900.0;
      }
    }
  }
  let preview = data("PreviewResult");
  assert_eq!(preview["rows"].as_array().unwrap().len(), 4);
  for j in 0..4 {
    assert_eq!(summary[j][0], (j + 1) as f64);
    for s in 0..4 {
      close(summary[j][s + 1], means[j][s]);
      close(
        preview["rows"][j][s + 1].as_f64().unwrap(),
        summary[j][s + 1],
      );
    }
  }
  assert_eq!(fs::read(f.0.join("Nldata.csv")).unwrap(), input);
  let exported = [
    "fishing_normalized_common_predictions.csv",
    "fishing_normalized_common_contrasts.csv",
  ]
  .map(|name| fs::read(f.0.join(name)).unwrap());
  assert!(!f.run().status.success());
  for (name, before) in [
    "fishing_normalized_common_predictions.csv",
    "fishing_normalized_common_contrasts.csv",
  ]
  .into_iter()
  .zip(exported)
  {
    assert_eq!(fs::read(f.0.join(name)).unwrap(), before);
  }
  assert_eq!(fs::read(f.0.join("Nldata.csv")).unwrap(), input);
}

#[test]
fn actual_normalized_common_source_guards_stop_before_fitting_or_export() {
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
    let rec = records(&output);
    assert!(rec.iter().all(|r| !matches!(
      r["result_type"].as_str(),
      Some("ConditionalLogitResult" | "NormalizedCommonNestedLogitResult")
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
    assert!(
      !f.0
        .join("fishing_normalized_common_predictions.csv")
        .exists()
    );
    assert!(!f.0.join("fishing_normalized_common_contrasts.csv").exists());
    assert_eq!(fs::read_to_string(f.0.join("Nldata.csv")).unwrap(), bad);
  }
}
