#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-svg-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(
      path.join("data.csv"),
      "x,y\n0,1\n1,2\n2,1.5\n3,4\n4,3\n5,5\n6,\n,8\n",
    )
    .unwrap();
    Self(path)
  }
  fn run(&self, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn script(&self, name: &str) {
    fs::write(self.0.join(format!("{name}.td")),format!("use data.csv\nregress y x\npredict fitted\ngenerate upper = fitted + 0.1\ngenerate lower = fitted - 0.1\nscatter y x, lines(fitted upper lower) jitter(0.03) title(\"Native comparison\") saving(\"{name}.svg\") noopen\nstatus\n")).unwrap();
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn envelopes(bytes: &[u8]) -> Vec<Value> {
  std::str::from_utf8(bytes)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

#[test]
fn discovery_and_help_advertise_only_the_bounded_native_workflow() {
  let fixture = Fixture::new();
  let output = fixture.run(&["--json", "--describe-command", "scatter"]);
  assert!(output.status.success());
  let schema: Value = serde_json::from_slice(&output.stdout).unwrap();
  assert_eq!(schema["schema_version"], 1);
  assert_eq!(
    schema["data"]["syntax"],
    "scatter yvar xvar, saving(\"file.svg\") noopen [lines(varlist) jitter(number) title(\"text\")]"
  );
  assert_eq!(
    schema["data"]["options"],
    json!([
      {"name":"saving","required":true},{"name":"noopen","required":true},
      {"name":"lines","required":false},{"name":"jitter","required":false},{"name":"title","required":false},
    ])
  );
  let output = fixture.run(&["--json", "--help-topic", "scatter"]);
  assert!(output.status.success());
  let help: Value = serde_json::from_slice(&output.stdout).unwrap();
  let text = help["data"]["text"].as_str().unwrap();
  for phrase in [
    "Installed syntax:",
    "no replace",
    "not lossless",
    "not Stata",
    "does NOT hide",
    "10,000",
    "hard-link-capable",
  ] {
    assert!(text.contains(phrase), "missing {phrase}");
  }
}

#[test]
fn json_metadata_has_explicit_mark_samples_and_keeps_the_retained_model() {
  let fixture = Fixture::new();
  let output=fixture.run(&["--json","-c","use data.csv","-c","regress y x","-c","predict fitted","-c","generate upper = fitted + 0.1","-c","generate lower = fitted - 0.1","-c","scatter y x, lines(fitted upper lower) jitter(0.03) title(\"Native comparison\") saving(\"graphs/figure, v1.SVG\") noopen","-c","status"]);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let rows = envelopes(&output.stdout);
  assert_eq!(
    rows
      .iter()
      .map(|v| v["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "RegressionResult",
      "PredictionResult",
      "GenerateResult",
      "GenerateResult",
      "PlotArtifactResult",
      "StatusResult"
    ]
  );
  assert!(rows.iter().all(|v| v["schema_version"] == 1));
  assert_eq!(
    rows[5]["data"],
    json!({
      "path":"graphs/figure, v1.SVG","format":"svg","width":900,"height":660,"candidate_rows":8,"x_variable":"x",
      "scatter":{"variable":"y","point_count":6,"omitted_rows":2},
      "lines":[{"variable":"fitted","point_count":7,"omitted_rows":1},{"variable":"upper","point_count":7,"omitted_rows":1},{"variable":"lower","point_count":7,"omitted_rows":1}],
      "jitter_y":0.03,"jitter_convention":"deterministic_row_index_pattern_outcome_units","coordinate_precision":"float64",
    })
  );
  assert_eq!(rows[6]["data"]["estimation_family"], "linear");
  let svg = fs::read_to_string(fixture.0.join("graphs/figure, v1.SVG")).unwrap();
  assert_eq!(svg.matches("class=\"scatter-point\"").count(), 6);
  assert_eq!(svg.matches("class=\"line-series\"").count(), 3);
  assert!(svg.contains("data-row-indices=\"0,1,2,3,4,5,6\""));
}

#[test]
fn json_text_and_piped_script_surfaces_publish_identical_svg_content() {
  let fixture = Fixture::new();
  for (name, flag) in [("json", "--json"), ("text", "--text")] {
    fixture.script(name);
    let output = fixture.run(&[flag, "-f", &format!("{name}.td")]);
    assert!(
      output.status.success(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
    );
    if name == "json" {
      assert!(
        envelopes(&output.stdout)
          .iter()
          .any(|v| v["result_type"] == "PlotArtifactResult")
      );
    } else {
      let text = String::from_utf8(output.stdout).unwrap();
      assert!(text.contains("SVG scatter saved to text.svg"));
      assert!(text.contains("finite points = 6; omitted missing/nonfinite = 2"));
      assert!(text.contains("Display-only y jitter amplitude = 0.03 outcome units"));
      assert!(text.contains("not RNG state"));
    }
  }
  fixture.script("piped");
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&fixture.0)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child
    .stdin
    .take()
    .unwrap()
    .write_all(b"run piped.td\nexit\n")
    .unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(String::from_utf8_lossy(&output.stdout).contains("SVG scatter saved to piped.svg"));
  let bytes = fs::read(fixture.0.join("json.svg")).unwrap();
  assert_eq!(fs::read(fixture.0.join("text.svg")).unwrap(), bytes);
  assert_eq!(fs::read(fixture.0.join("piped.svg")).unwrap(), bytes);
}

#[test]
fn file_collision_and_located_script_failure_stop_later_output_safely() {
  let fixture = Fixture::new();
  fs::write(fixture.0.join("existing.svg"), "original bytes").unwrap();
  let output = fixture.run(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "scatter y x, saving(existing.svg) noopen",
    "-c",
    "export after.csv",
  ]);
  assert_eq!(output.status.code(), Some(1));
  assert!(String::from_utf8_lossy(&output.stderr).contains("already exists"));
  assert_eq!(
    fs::read_to_string(fixture.0.join("existing.svg")).unwrap(),
    "original bytes"
  );
  assert!(!fixture.0.join("after.csv").exists());
  let records = envelopes(&output.stdout);
  assert_eq!(records.len(), 2);
  assert_eq!(records[1]["error"]["type"], "ExecutionError");
  fs::write(fixture.0.join("bad.td"),"use data.csv\nregress y x\nscatter y x, lines(absent) saving(bad.svg) noopen\nexport after.csv\n").unwrap();
  let output = fixture.run(&["--json", "-f", "bad.td"]);
  assert_eq!(output.status.code(), Some(1));
  let error = String::from_utf8_lossy(&output.stderr);
  assert!(
    error.contains("bad.td:3:") && error.contains("unknown variable"),
    "{error}"
  );
  assert!(!fixture.0.join("bad.svg").exists());
  assert!(!fixture.0.join("after.csv").exists());
  let records = envelopes(&output.stdout);
  assert_eq!(records.len(), 3);
  assert_eq!(records[2]["error"]["type"], "ScriptError");
  assert_eq!(records[2]["error"]["line"], 3);
  assert!(fs::read_dir(&fixture.0).unwrap().all(|entry| {
    !entry
      .unwrap()
      .file_name()
      .to_string_lossy()
      .starts_with(".tabdat-svg-")
  }));
}
