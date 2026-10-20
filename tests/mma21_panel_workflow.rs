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
