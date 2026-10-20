mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::collections::BTreeMap;
use std::{fs, path::PathBuf, process::Command};

#[derive(Clone)]
struct Row {
  source: usize,
  id: i32,
  year: i32,
  y: Option<f64>,
  x: f64,
}

// Deliberately interleaved, reverse-time input with one gap and one missing y.
fn panel_fixture() -> (String, Vec<Row>) {
  let mut rows = Vec::new();
  let mut csv = String::from("source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year\n");
  for t in (0..4).rev() {
    for id in (1..=6).rev() {
      if id == 2 && t == 1 {
        continue;
      }
      let y = (id != 3 || t != 1).then_some(
        7.0 + 0.2 * f64::from(id) + 0.04 * f64::from(t) + 0.005 * f64::from((id * t + 1) % 7),
      );
      let x = 2.0 + 0.15 * f64::from(id) + 0.1 * f64::from(t) + 0.03 * f64::from((id * t + 2) % 5);
      let source = rows.len() + 1;
      let year = 1979 + t;
      csv.push_str(&format!(
        "{source},{},{x},0,30,900,0,{id},{year}\n",
        y.map(|v| v.to_string()).unwrap_or_default()
      ));
      rows.push(Row {
        source,
        id,
        year,
        y,
        x,
      });
    }
  }
  (csv, rows)
}

fn run(dir: &TestDir, script: &str, json: bool) -> std::process::Output {
  let mut command = tabdat_bin();
  command.current_dir(dir.path());
  command.arg(if json { "--json" } else { "--text" });
  command.arg("-f").arg(example(script)).output().unwrap()
}

fn numeric_csv(text: &str) -> Vec<BTreeMap<&str, Option<f64>>> {
  let mut lines = text.lines();
  let headers: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      headers
        .iter()
        .copied()
        .zip(line.split(',').map(|v| {
          if v.is_empty() {
            None
          } else {
            Some(v.parse().unwrap())
          }
        }))
        .collect()
    })
    .collect()
}

fn example(name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma21p1panel")
    .join(name)
}
fn convert(dir: &TestDir, text: &str) -> std::process::Output {
  fs::write(dir.path().join("raw input.dat"), text).unwrap();
  Command::new("sh")
    .arg(example("prepare.sh"))
    .arg(dir.path().join("raw input.dat"))
    .arg(dir.path().join("MOM.csv"))
    .output()
    .unwrap()
}
#[test]
fn converter_preserves_token_records_and_refuses_invalid_keys_and_partial_publication() {
  let d = TestDir::new();
  let out = convert(
    &d,
    "7.580 1.91 2 27\r\n729 0 1.00 1979.00\r\n7.750 1.89 2 28 784 0 1.00 1980.00",
  );
  assert!(out.status.success(), "{out:?}");
  let expected = "source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year\n1,7.580,1.91,2,27,729,0,1.00,1979.00\n2,7.750,1.89,2,28,784,0,1.00,1980.00\n";
  assert_eq!(
    fs::read_to_string(d.path().join("MOM.csv")).unwrap(),
    expected
  );
  let out = convert(&d, "bad");
  assert!(!out.status.success());
  assert_eq!(
    fs::read_to_string(d.path().join("MOM.csv")).unwrap(),
    expected
  );
  for input in [
    "",
    "7.58 1.91 2 27 729 0 1",
    "7.58 1.91 2 27 729 0 1 1979\n7.75 1.89 2",
    "bad 1.91 2 27 729 0 1 1979",
    "1e999 1.91 2 27 729 0 1 1979",
    "NaN 1.91 2 27 729 0 1 1979",
    "7.58 1.91 2 27 729 0 1.00000000000000001 1979",
    "7.58 1.91 2 27 729 0 533 1979",
    "7.58 1.91 2 27 729 0 1 1979.1",
    "7.58 1.91 2 27 729 0 1 1989",
  ] {
    let d = TestDir::new();
    let out = convert(&d, input);
    assert!(!out.status.success(), "{input}: {out:?}");
    assert!(!d.path().join("MOM.csv").exists());
    assert!(
      !fs::read_dir(d.path()).unwrap().any(|p| p
        .unwrap()
        .file_name()
        .to_string_lossy()
        .contains(".tmp."))
    );
  }
}

#[test]
fn native_panel_scripts_preserve_keys_differences_samples_and_text_json_exports() {
  let (csv, rows) = panel_fixture();
  for (script, case, total, n, df) in [
    ("pooled.td", "pols", 23, 22, 20),
    ("first_difference.td", "fd", 16, 14, 12),
  ] {
    let json_dir = TestDir::new();
    let text_dir = TestDir::new();
    for dir in [&json_dir, &text_dir] {
      fs::write(dir.path().join("MOM.csv"), &csv).unwrap();
    }
    let out = run(&json_dir, script, true);
    assert!(out.status.success(), "{out:?}");
    let events: Vec<Value> = String::from_utf8(out.stdout)
      .unwrap()
      .lines()
      .map(|line| serde_json::from_str(line).unwrap())
      .collect();
    assert!(
      events
        .iter()
        .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
    );
    let fits: Vec<_> = events
      .iter()
      .filter(|e| e["result_type"] == "RegressionResult")
      .map(|e| &e["data"])
      .collect();
    assert_eq!(fits.len(), 3);
    for (fit, covariance) in fits.iter().zip(["nonrobust", "robust", "cluster(id)"]) {
      assert_eq!(fit["covariance"], covariance);
      assert_eq!(fit["observation_count"], n);
      assert_eq!(fit["df_residual"], df);
      assert_eq!(fit["sample"]["total_observations"], total);
      assert_eq!(fit["sample"]["dropped_observations"], total - n);
      assert_eq!(fit["include_intercept"], true);
    }
    for kind in ["TestResult", "LincomResult"] {
      let values: Vec<_> = events.iter().filter(|e| e["result_type"] == kind).collect();
      assert_eq!(values.len(), 3);
      for (event, reference_df) in values.iter().zip([df, df, 5]) {
        assert_eq!(event["data"]["df_residual"], reference_df);
      }
    }
    let out = run(&text_dir, script, false);
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains(&format!("{case}_cluster")), "{text}");
    let filename = if case == "pols" {
      "panel_pooled_predictions.csv"
    } else {
      "panel_fd_predictions.csv"
    };
    let exported = fs::read_to_string(json_dir.path().join(filename)).unwrap();
    assert_eq!(
      exported,
      fs::read_to_string(text_dir.path().join(filename)).unwrap()
    );
    let data = numeric_csv(&exported);
    assert_eq!(data.len(), total as usize);
    let beta: Vec<_> = fits[2]["coefficients"]
      .as_array()
      .unwrap()
      .iter()
      .map(|c| c["value"].as_f64().unwrap())
      .collect();
    let mut last_key = (0, 0);
    for (position, row) in data.into_iter().enumerate() {
      let source = row["source_row"].unwrap() as usize;
      let raw = &rows[source - 1];
      assert_eq!(row["id"], Some(f64::from(raw.id)));
      assert_eq!(row["year"], Some(f64::from(raw.year)));
      let (x, y) = if case == "fd" {
        assert!((raw.id, raw.year) > last_key);
        last_key = (raw.id, raw.year);
        let previous = rows
          .iter()
          .find(|p| p.id == raw.id && p.year == raw.year - 1)
          .unwrap();
        assert_eq!(row["previous_source_row"], Some(previous.source as f64));
        assert_eq!(row["previous_year"], Some(f64::from(previous.year)));
        let dx = raw.x - previous.x;
        let dy = raw.y.zip(previous.y).map(|(a, b)| a - b);
        assert!((row["dlnwg"].unwrap() - dx).abs() < 1e-12);
        if let Some(dy) = dy {
          assert!((row["dlnhr"].unwrap() - dy).abs() < 1e-12);
        } else {
          assert_eq!(row["dlnhr"], None);
        }
        (dx, dy)
      } else {
        assert_eq!(source, position + 1);
        (raw.x, raw.y)
      };
      let fitted = beta[0] + beta[1] * x;
      assert!((row[format!("{case}_fitted").as_str()].unwrap() - fitted).abs() < 1e-12);
      let residual = row[format!("{case}_residual").as_str()];
      if let Some(y) = y {
        assert!((residual.unwrap() - (y - fitted)).abs() < 1e-12);
      } else {
        assert_eq!(residual, None);
      }
    }
    assert_eq!(
      fs::read_to_string(json_dir.path().join("MOM.csv")).unwrap(),
      csv
    );
  }
}

#[test]
fn invalid_panel_keys_stop_before_fits_or_exports_and_existing_exports_are_not_replaced() {
  let (csv, _) = panel_fixture();
  let lines: Vec<_> = csv.lines().collect();
  for (kind, line) in [("duplicate", 4), ("missing", 4), ("fractional", 5)] {
    let dir = TestDir::new();
    let mut fields: Vec<_> = lines[1].split(',').collect();
    let bad = match kind {
      "duplicate" => format!("{csv}{}\n", lines[1]),
      "missing" => {
        fields[7] = "";
        format!(
          "{}\n{}\n{}\n",
          lines[0],
          fields.join(","),
          lines[2..].join("\n")
        )
      }
      _ => {
        fields[8] = "1982.5";
        format!(
          "{}\n{}\n{}\n",
          lines[0],
          fields.join(","),
          lines[2..].join("\n")
        )
      }
    };
    fs::write(dir.path().join("MOM.csv"), &bad).unwrap();
    let out = run(&dir, "first_difference.td", true);
    assert!(!out.status.success(), "{kind}: {out:?}");
    let events: Vec<Value> = String::from_utf8(out.stdout)
      .unwrap()
      .lines()
      .map(|s| serde_json::from_str(s).unwrap())
      .collect();
    assert_eq!(events.last().unwrap()["error"]["line"], line);
    assert!(
      !events
        .iter()
        .any(|e| e["result_type"] == "RegressionResult")
    );
    assert!(!dir.path().join("panel_fd_predictions.csv").exists());
    assert_eq!(fs::read_to_string(dir.path().join("MOM.csv")).unwrap(), bad);
  }
  let dir = TestDir::new();
  fs::write(dir.path().join("MOM.csv"), &csv).unwrap();
  let target = dir.path().join("panel_fd_predictions.csv");
  fs::write(&target, "do not replace\n").unwrap();
  let out = run(&dir, "first_difference.td", true);
  assert!(!out.status.success());
  assert_eq!(fs::read_to_string(target).unwrap(), "do not replace\n");
  assert_eq!(fs::read_to_string(dir.path().join("MOM.csv")).unwrap(), csv);
}

#[test]
fn within_script_reports_absorbed_and_inference_df_and_exports_conditional_predictions() {
  let (csv, source) = panel_fixture();
  let json_dir = TestDir::new();
  let text_dir = TestDir::new();
  for dir in [&json_dir, &text_dir] {
    fs::write(dir.path().join("MOM.csv"), &csv).unwrap();
  }
  let out = run(&json_dir, "within.td", true);
  assert!(out.status.success(), "{out:?}");
  assert!(out.stderr.is_empty());
  let events: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  assert!(
    events
      .iter()
      .all(|event| event["schema_version"] == 1 && event.get("error").is_none())
  );
  let fits: Vec<_> = events
    .iter()
    .filter(|e| e["result_type"] == "FixedEffectsResult")
    .map(|e| &e["data"])
    .collect();
  assert_eq!(fits.len(), 2);
  for (fit, covariance, inference_df) in [(fits[0], "nonrobust", 15), (fits[1], "cluster(id)", 5)] {
    assert_eq!(fit["estimator"], "within_fixed_effects");
    assert_eq!(fit["covariance"], covariance);
    assert_eq!(fit["observation_count"], 22);
    assert_eq!(fit["group_count"], 6);
    assert_eq!(fit["slope_count"], 1);
    assert_eq!(fit["absorbed_intercepts"], 6);
    assert_eq!(fit["df_residual"], 15);
    assert_eq!(fit["inference_df"], inference_df);
    assert_eq!(
      fit["sample"],
      serde_json::json!({"total_observations":23,"dropped_observations":1})
    );
    assert_eq!(
      fit["covariance_parameter_names"],
      serde_json::json!(["lnwg"])
    );
    assert_eq!(fit["coefficients"].as_array().unwrap().len(), 1);
    assert_eq!(
      fit["common_intercept_inference"],
      "normalization_point_only"
    );
    assert!(fit.get("log_likelihood").is_none());
  }
  for kind in ["TestResult", "LincomResult"] {
    let inference: Vec<_> = events.iter().filter(|e| e["result_type"] == kind).collect();
    assert_eq!(inference.len(), 2);
    assert_eq!(inference[0]["data"]["df_residual"], 15);
    assert_eq!(inference[1]["data"]["df_residual"], 5);
  }
  let out = run(&text_dir, "within.td", false);
  assert!(out.status.success(), "{out:?}");
  let text = String::from_utf8(out.stdout).unwrap();
  assert!(text.contains("Within fixed-effects regression"));
  assert!(text.contains("residual DF (N-G-p) = 15"));
  assert!(text.contains("normalization point only"));
  let filename = "panel_within_fe_predictions.csv";
  let exported = fs::read_to_string(json_dir.path().join(filename)).unwrap();
  assert_eq!(
    exported,
    fs::read_to_string(text_dir.path().join(filename)).unwrap()
  );
  let beta = fits[0]["coefficients"][0]["value"].as_f64().unwrap();
  let c = fits[0]["common_intercept"].as_f64().unwrap();
  assert_eq!(
    fits[0]["coefficients"][0]["value"],
    fits[1]["coefficients"][0]["value"]
  );
  let data = numeric_csv(&exported);
  assert_eq!(data.len(), source.len());
  for (position, row) in data.iter().enumerate() {
    let original = &source[position];
    assert_eq!(row["source_row"], Some(original.source as f64));
    assert_eq!(row["id"], Some(f64::from(original.id)));
    assert_eq!(row["year"], Some(f64::from(original.year)));
    let group: Vec<_> = source
      .iter()
      .filter(|r| r.id == original.id && r.y.is_some())
      .collect();
    let xmean = group.iter().map(|r| r.x).sum::<f64>() / group.len() as f64;
    let ymean = group.iter().map(|r| r.y.unwrap()).sum::<f64>() / group.len() as f64;
    let alpha = ymean - xmean * beta;
    let mu = alpha + original.x * beta;
    for prefix in ["fe_classical", "fe_cluster"] {
      assert!(
        (row[format!("{prefix}_xb").as_str()].unwrap() - (c + original.x * beta)).abs() < 1e-12
      );
      assert!((row[format!("{prefix}_xbu").as_str()].unwrap() - mu).abs() < 1e-12);
      assert!((row[format!("{prefix}_u").as_str()].unwrap() - (alpha - c)).abs() < 1e-12);
      let residual = row[format!("{prefix}_residual").as_str()];
      if let Some(y) = original.y {
        assert!((residual.unwrap() - (y - mu)).abs() < 1e-12);
      } else {
        assert_eq!(residual, None);
      }
    }
  }
  assert_eq!(
    fs::read_to_string(json_dir.path().join("MOM.csv")).unwrap(),
    csv
  );
  let out = run(&json_dir, "within.td", true);
  assert!(!out.status.success());
  assert_eq!(
    fs::read_to_string(json_dir.path().join(filename)).unwrap(),
    exported
  );
}

#[test]
fn between_script_uses_equal_weight_joint_case_means_and_exports_identically() {
  let (_, mut source) = panel_fixture();
  // One entirely excluded person, one singleton, and missing x in another
  // person distinguish joint-case means from separate-column SQL AVG samples.
  source.retain(|r| r.id != 6 || r.year == 1982);
  let mut csv = String::from("source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year\n");
  for row in &mut source {
    if row.id == 5 {
      row.y = None;
    }
    if row.id == 4 && row.year == 1980 {
      row.x = f64::NAN;
    }
    csv.push_str(&format!(
      "{},{},{},0,30,900,0,{},{}\n",
      row.source,
      row.y.map(|v| v.to_string()).unwrap_or_default(),
      if row.x.is_finite() {
        row.x.to_string()
      } else {
        String::new()
      },
      row.id,
      row.year
    ));
  }
  let json_dir = TestDir::new();
  let text_dir = TestDir::new();
  for dir in [&json_dir, &text_dir] {
    fs::write(dir.path().join("MOM.csv"), &csv).unwrap();
  }
  let out = run(&json_dir, "between.td", true);
  assert!(out.status.success(), "{out:?}");
  let events: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fits: Vec<_> = events
    .iter()
    .filter(|e| e["result_type"] == "RegressionResult")
    .map(|e| &e["data"])
    .collect();
  assert_eq!(fits.len(), 2);
  for (fit, covariance) in fits.iter().zip(["nonrobust", "robust"]) {
    assert_eq!(fit["estimator"], "ols");
    assert_eq!(fit["covariance"], covariance);
    assert_eq!(fit["observation_count"], 5);
    assert_eq!(fit["df_residual"], 3);
    assert_eq!(
      fit["sample"],
      serde_json::json!({"total_observations":5,"dropped_observations":0})
    );
  }
  assert_eq!(
    fits[0]["coefficients"][1]["value"],
    fits[1]["coefficients"][1]["value"]
  );
  for kind in ["TestResult", "LincomResult"] {
    let inference: Vec<_> = events.iter().filter(|e| e["result_type"] == kind).collect();
    assert_eq!(inference.len(), 2);
    assert!(inference.iter().all(|e| e["data"]["df_residual"] == 3));
  }
  let out = run(&text_dir, "between.td", false);
  assert!(out.status.success(), "{out:?}");
  assert!(
    String::from_utf8(out.stdout)
      .unwrap()
      .contains("between_hc1")
  );
  let filename = "panel_between_predictions.csv";
  let exported = fs::read_to_string(json_dir.path().join(filename)).unwrap();
  assert_eq!(
    exported,
    fs::read_to_string(text_dir.path().join(filename)).unwrap()
  );
  let means = numeric_csv(&exported);
  assert_eq!(means.len(), 5);
  let mut xy = Vec::new();
  for (mean, id) in means.iter().zip([1, 2, 3, 4, 6]) {
    assert_eq!(mean["id"], Some(f64::from(id)));
    let selected: Vec<_> = source
      .iter()
      .filter(|r| r.id == id && r.y.is_some() && r.x.is_finite())
      .collect();
    let n = selected.len() as f64;
    let x = selected.iter().map(|r| r.x).sum::<f64>() / n;
    let y = selected.iter().map(|r| r.y.unwrap()).sum::<f64>() / n;
    assert_eq!(mean["model_rows"], Some(n));
    assert_eq!(
      mean["first_year"],
      Some(f64::from(selected.iter().map(|r| r.year).min().unwrap()))
    );
    assert_eq!(
      mean["last_year"],
      Some(f64::from(selected.iter().map(|r| r.year).max().unwrap()))
    );
    assert!((mean["mean_lnwg"].unwrap() - x).abs() < 1e-12);
    assert!((mean["mean_lnhr"].unwrap() - y).abs() < 1e-12);
    xy.push((x, y));
  }
  // Independent centered scalar formula also rules out weighting by T_i.
  let xbar = xy.iter().map(|r| r.0).sum::<f64>() / 5.0;
  let ybar = xy.iter().map(|r| r.1).sum::<f64>() / 5.0;
  let slope = xy.iter().map(|(x, y)| (x - xbar) * (y - ybar)).sum::<f64>()
    / xy.iter().map(|(x, _)| (x - xbar).powi(2)).sum::<f64>();
  assert!((fits[0]["coefficients"][1]["value"].as_f64().unwrap() - slope).abs() < 1e-12);
  for (mean, (x, y)) in means.iter().zip(xy) {
    let fitted = ybar + slope * (x - xbar);
    assert!((mean["between_fitted"].unwrap() - fitted).abs() < 1e-12);
    assert!((mean["between_residual"].unwrap() - (y - fitted)).abs() < 1e-12);
  }
  assert_eq!(
    fs::read_to_string(json_dir.path().join("MOM.csv")).unwrap(),
    csv
  );
  let out = run(&json_dir, "between.td", true);
  assert!(!out.status.success());
  assert_eq!(
    fs::read_to_string(json_dir.path().join(filename)).unwrap(),
    exported
  );
}

#[test]
fn between_script_rejects_bad_keys_or_unusable_samples_before_export() {
  let (csv, _) = panel_fixture();
  for kind in [
    "duplicate",
    "missing_key",
    "fractional_key",
    "no_model_cases",
    "singular",
  ] {
    let dir = TestDir::new();
    let mut lines: Vec<String> = csv.lines().map(str::to_owned).collect();
    if kind == "duplicate" {
      lines.push(lines[1].clone());
    } else {
      for line in &mut lines[1..] {
        let mut fields: Vec<_> = line.split(',').map(str::to_owned).collect();
        match kind {
          "missing_key" => fields[7] = String::new(),
          "fractional_key" => fields[8] = "1982.5".into(),
          "no_model_cases" => fields[1] = String::new(),
          "singular" => fields[2] = "2".into(),
          _ => unreachable!(),
        }
        *line = fields.join(",");
      }
    }
    let bad = lines.join("\n") + "\n";
    fs::write(dir.path().join("MOM.csv"), &bad).unwrap();
    let out = run(&dir, "between.td", true);
    assert!(!out.status.success(), "{kind}: {out:?}");
    let events: Vec<Value> = String::from_utf8(out.stdout)
      .unwrap()
      .lines()
      .map(|line| serde_json::from_str(line).unwrap())
      .collect();
    assert!(
      events.last().unwrap().get("error").is_some(),
      "{kind}: {events:?}"
    );
    assert!(
      !events
        .iter()
        .any(|e| e["result_type"] == "RegressionResult")
    );
    assert!(!dir.path().join("panel_between_predictions.csv").exists());
    assert_eq!(fs::read_to_string(dir.path().join("MOM.csv")).unwrap(), bad);
  }
}

#[test]
fn random_effects_script_reports_normal_gls_and_preserves_order_and_exports() {
  let (_, source) = panel_fixture();
  let source: Vec<_> = source.into_iter().filter(|r| r.id != 2).collect();
  let mut csv = String::from("source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year\n");
  for row in &source {
    csv.push_str(&format!(
      "{},{},{},0,30,900,0,{},{}\n",
      row.source,
      row.y.unwrap_or(7.8),
      row.x,
      row.id,
      row.year
    ));
  }
  let json_dir = TestDir::new();
  let text_dir = TestDir::new();
  for dir in [&json_dir, &text_dir] {
    fs::write(dir.path().join("MOM.csv"), &csv).unwrap();
  }
  let out = run(&json_dir, "random_effects.td", true);
  assert!(out.status.success(), "{out:?}");
  let events: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let fits: Vec<_> = events
    .iter()
    .filter(|e| e["result_type"] == "RandomEffectsResult")
    .map(|e| &e["data"])
    .collect();
  assert_eq!(fits.len(), 1);
  let fit = fits[0];
  assert_eq!(fit["observation_count"], 20);
  assert_eq!(fit["group_count"], 5);
  assert_eq!(fit["group_size"], 4);
  assert_eq!(fit["df_residual"], 18);
  assert_eq!(fit["reference_distribution"], "normal");
  assert_eq!(fit["wald_df"], 1);
  assert_eq!(fit["wald_reference"], "chi_square");
  assert_eq!(
    fit["sample"],
    serde_json::json!({"total_observations":20,"dropped_observations":0})
  );
  assert_eq!(fit["coefficients"][0]["ci_level"], 95);
  assert!(fit.get("log_likelihood").is_none());
  assert!(fit.get("r_squared").is_none());
  let out = run(&text_dir, "random_effects.td", false);
  assert!(out.status.success(), "{out:?}");
  let text = String::from_utf8(out.stdout).unwrap();
  assert!(text.contains("Balanced one-way random-effects GLS"));
  assert!(text.contains("normal inference"));
  let filename = "panel_re_gls_predictions.csv";
  let exported = fs::read_to_string(json_dir.path().join(filename)).unwrap();
  assert_eq!(
    exported,
    fs::read_to_string(text_dir.path().join(filename)).unwrap()
  );
  let data = numeric_csv(&exported);
  assert_eq!(data.len(), 20);
  let intercept = fit["coefficients"][0]["value"].as_f64().unwrap();
  let slope = fit["coefficients"][1]["value"].as_f64().unwrap();
  for (row, original) in data.iter().zip(&source) {
    assert_eq!(row["source_row"], Some(original.source as f64));
    assert_eq!(row["id"], Some(f64::from(original.id)));
    assert_eq!(row["year"], Some(f64::from(original.year)));
    let fitted = intercept + slope * original.x;
    assert!((row["re_xb"].unwrap() - fitted).abs() < 1e-12);
    assert!((row["re_residual"].unwrap() - (original.y.unwrap_or(7.8) - fitted)).abs() < 1e-12);
  }
  assert_eq!(
    fs::read_to_string(json_dir.path().join("MOM.csv")).unwrap(),
    csv
  );
  let out = run(&json_dir, "random_effects.td", true);
  assert!(!out.status.success());
  assert_eq!(
    fs::read_to_string(json_dir.path().join(filename)).unwrap(),
    exported
  );
  // The original incomplete/gapped fixture has unequal joint-case unit counts.
  let bad_dir = TestDir::new();
  fs::write(bad_dir.path().join("MOM.csv"), panel_fixture().0).unwrap();
  let out = run(&bad_dir, "random_effects.td", true);
  assert!(!out.status.success());
  assert!(!bad_dir.path().join(filename).exists());
}
