//! Local development measurements, not a benchmark threshold or platform certification.
#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  io::{BufWriter, Write},
  path::{Path, PathBuf},
  process::{Command as Process, ExitCode, Output},
  sync::atomic::{AtomicU64, Ordering},
  time::{Instant, SystemTime, UNIX_EPOCH},
};
use tabdat_language::{Command, DataSource, ExecutionMode, parse_command};
use tabdat_runtime::Session;

const HELP: &str = "usage: measure_core_workflow BINARY [--rows N] [--repetitions N] [--label TEXT]\n\
  rows: 4..=1000000 (default 10000); repetitions: 1..=50 (default 7)\n\
  Example: cargo run --locked --release --example measure_core_workflow -- \
  target/release/tabdat-explore-rs --label release\n\
  One excluded warmup per case. Fresh-process timings are NOT OS-cold-cache timings.\n\
  Session command timings exclude parsing, setup, reporting, validation and teardown.";
type Result<T> = std::result::Result<T, String>;

#[derive(Debug)]
struct Options {
  binary: PathBuf,
  rows: usize,
  repetitions: usize,
  label: String,
}
fn options(args: impl IntoIterator<Item = String>) -> Result<Option<Options>> {
  let mut args = args.into_iter();
  let Some(binary) = args.next() else {
    return Err(HELP.to_owned());
  };
  if binary == "--help" {
    return Ok(None);
  }
  let mut options = Options {
    binary: binary.into(),
    rows: 10000,
    repetitions: 7,
    label: "unspecified".to_owned(),
  };
  while let Some(flag) = args.next() {
    let value = args
      .next()
      .ok_or_else(|| format!("missing value for {flag}"))?;
    match flag.as_str() {
      "--rows" => options.rows = value.parse().map_err(|_| "--rows expects an integer")?,
      "--repetitions" => {
        options.repetitions = value
          .parse()
          .map_err(|_| "--repetitions expects an integer")?
      }
      "--label" => options.label = value,
      _ => return Err(format!("unknown option: {flag}")),
    }
  }
  if !(4..=1_000_000).contains(&options.rows) {
    return Err("--rows must be 4..=1000000".to_owned());
  }
  if !(1..=50).contains(&options.repetitions) {
    return Err("--repetitions must be 1..=50".to_owned());
  }
  Ok(Some(options))
}

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(rows: usize) -> Result<Self> {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_err(|error| error.to_string())?
      .as_nanos();
    let root = std::env::temp_dir().join(format!(
      "tabdat-core-measure-{}-{nonce}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).map_err(|error| format!("create fixture: {error}"))?;
    let fixture = Self(root);
    let mut file = BufWriter::new(
      fs::File::create(fixture.0.join("data.csv")).map_err(|error| error.to_string())?,
    );
    writeln!(file, "x,y,group").map_err(|error| error.to_string())?;
    for index in 0..rows {
      let x = index as f64 / rows as f64;
      let noise = if index.is_multiple_of(2) { -1.0 } else { 1.0 };
      writeln!(file, "{x},{},{}", 3.0 + 2.0 * x + noise, index % 10)
        .map_err(|error| error.to_string())?;
    }
    file.flush().map_err(|error| error.to_string())?;
    Ok(fixture)
  }
  fn load(&self) -> Result<Command> {
    Ok(Command::Use {
      source: DataSource::LocalPath(
        self
          .0
          .join("data.csv")
          .to_str()
          .ok_or("fixture path must be UTF-8")?
          .to_owned(),
      ),
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: None,
      has_header: None,
    })
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn statistics(mut samples: Vec<f64>) -> Result<Value> {
  if samples.is_empty()
    || samples
      .iter()
      .any(|value| !value.is_finite() || *value < 0.0)
  {
    return Err("timings must be a nonempty set of finite nonnegative values".to_owned());
  }
  let raw = samples.clone();
  samples.sort_by(f64::total_cmp);
  let middle = samples.len() / 2;
  let median = if samples.len().is_multiple_of(2) {
    samples[middle - 1] + (samples[middle] - samples[middle - 1]) / 2.0
  } else {
    samples[middle]
  };
  Ok(
    json!({"samples_ms": raw, "min_ms": samples[0], "median_ms": median, "max_ms": samples[samples.len()-1]}),
  )
}
fn measure(repetitions: usize, mut run: impl FnMut() -> Result<f64>) -> Result<Value> {
  run()?;
  let samples = (0..repetitions)
    .map(|_| run())
    .collect::<Result<Vec<_>>>()?;
  statistics(samples)
}
fn checked_output(output: &Output) -> Result<&str> {
  if !output.status.success() || !output.stderr.is_empty() {
    return Err(format!(
      "binary failed ({}) or emitted diagnostics: {}",
      output.status,
      String::from_utf8_lossy(&output.stderr)
    ));
  }
  std::str::from_utf8(&output.stdout).map_err(|error| format!("non-UTF-8 binary output: {error}"))
}
fn require(condition: bool, message: &str) -> Result<()> {
  if condition {
    Ok(())
  } else {
    Err(format!("result validation failed: {message}"))
  }
}
fn close(actual: &Value, expected: f64) -> Result<()> {
  // A fixed health-check tolerance for the normalized x=i/N fixture, not an
  // estimator certification or a timing threshold. Failed values never count.
  require(
    actual
      .as_f64()
      .is_some_and(|value| value.is_finite() && (value - expected).abs() <= 1e-8),
    &format!(
      "numeric fixture value: actual={actual}, expected={expected:.17}; absolute tolerance 1e-8"
    ),
  )
}
fn coefficients(rows: usize) -> (f64, f64) {
  let n = rows as f64;
  // x=i/N, y=3+2x+alternating(-1,+1). Centered noise covariance is 1/2
  // for even N, zero for odd N; Sxx=(N^2-1)/(12N). No fit is an oracle.
  if rows.is_multiple_of(2) {
    (3.0 - 3.0 / (n + 1.0), 2.0 + 6.0 * n / (n * n - 1.0))
  } else {
    (3.0 - 1.0 / n, 2.0)
  }
}
fn validate(value: &Value, kind: &str, rows: usize) -> Result<()> {
  require(
    value["schema_version"] == 1 && value["result_type"] == kind,
    "CLI-v1 result type/version",
  )?;
  let data = &value["data"];
  match kind {
    "LoadResult" | "PredictionResult" => {
      require(data["dataset"]["row_count"] == rows, "dataset row count")
    }
    "CountResult" => require(data["row_count"] == rows, "count"),
    "SummarizeResult" => {
      let summaries = data["rows"].as_array().ok_or("summary rows absent")?;
      require(
        summaries.len() == 2 && summaries[0]["variable"] == "x" && summaries[1]["variable"] == "y",
        "summary order",
      )?;
      for row in summaries {
        require(row["count"] == rows, "finite summary sample")?;
      }
      close(
        &summaries[0]["mean"],
        (rows - 1) as f64 / (2.0 * rows as f64),
      )?;
      close(
        &summaries[1]["mean"],
        4.0 - if rows.is_multiple_of(2) { 1.0 } else { 2.0 } / rows as f64,
      )
    }
    "RegressionResult" => {
      require(
        data["observation_count"] == rows
          && data["df_residual"] == rows - 2
          && data["include_intercept"] == true,
        "OLS sample/design",
      )?;
      require(
        data["sample"]["dropped_observations"] == 0
          && data["sample"]["total_observations"] == rows
          && data["predictors"] == json!(["x"])
          && data["outcome"] == "y"
          && data["estimator"] == "ols"
          && data["covariance"] == "nonrobust",
        "OLS exclusions/predictor",
      )?;
      let estimates = data["coefficients"]
        .as_array()
        .ok_or("coefficients absent")?;
      require(estimates.len() == 2, "coefficient count")?;
      let (intercept, slope) = coefficients(rows);
      close(&estimates[0]["value"], intercept)?;
      close(&estimates[1]["value"], slope)
    }
    "PreviewResult" => {
      require(
        data["columns"] == json!(["x", "y", "group", "fitted"]),
        "prediction schema",
      )?;
      let preview = data["rows"].as_array().ok_or("preview rows absent")?;
      require(preview.len() == 3, "preview row count")?;
      let (intercept, slope) = coefficients(rows);
      for (index, row) in preview.iter().enumerate() {
        let x = index as f64 / rows as f64;
        let noise = if index.is_multiple_of(2) { -1.0 } else { 1.0 };
        require(
          row.as_array().is_some_and(|cells| cells.len() == 4) && row[2] == index,
          "preview cells/order",
        )?;
        close(&row[0], x)?;
        close(&row[1], 3.0 + 2.0 * x + noise)?;
        close(&row[3], intercept + slope * x)?;
      }
      Ok(())
    }
    _ => Err(format!("unknown validation case: {kind}")),
  }
}
fn envelope(session: &mut Session, command: Command) -> Result<Value> {
  let result = session
    .execute(command)
    .map_err(|error| error.to_string())?;
  let json = tabdat_reporting::json_results::serialize_execution_result(&result)
    .ok_or("missing result envelope")?;
  serde_json::from_str(&json).map_err(|error| error.to_string())
}
fn sample_session(fixture: &Fixture, rows: usize, source: &str, kind: &str) -> Result<f64> {
  let mut session = Session::new();
  let command = if source == "use" {
    fixture.load()?
  } else {
    parse_command(source).map_err(|error| error.to_string())?
  };
  if source != "use" {
    envelope(&mut session, fixture.load()?)?;
  }
  if kind == "PredictionResult" {
    envelope(
      &mut session,
      parse_command("regress y x").map_err(|error| error.to_string())?,
    )?;
  }
  let start = Instant::now();
  let result = session
    .execute(command)
    .map_err(|error| error.to_string())?;
  let elapsed = start.elapsed().as_secs_f64() * 1000.0;
  let serialized = tabdat_reporting::json_results::serialize_execution_result(&result)
    .ok_or("missing timed envelope")?;
  let value = serde_json::from_str(&serialized).map_err(|error| error.to_string())?;
  validate(&value, kind, rows)?;
  if matches!(kind, "RegressionResult" | "PredictionResult") {
    let sample = session
      .last_regression()
      .and_then(|fit| fit.sample.as_ref())
      .ok_or("retained fit sample absent")?;
    require(
      sample.retained_indices.iter().copied().eq(0..rows),
      "retained sample identity",
    )?;
  }
  if kind == "PredictionResult" {
    let preview = envelope(
      &mut session,
      parse_command("head 3").map_err(|error| error.to_string())?,
    )?;
    validate(&preview, "PreviewResult", rows)?;
  }
  Ok(elapsed)
}
fn sample_cli(binary: &Path, fixture: &Fixture, rows: usize) -> Result<f64> {
  let start = Instant::now();
  let output = Process::new(binary)
    .current_dir(&fixture.0)
    .args([
      "--json",
      "-c",
      "use data.csv",
      "-c",
      "count",
      "-c",
      "summarize x y",
      "-c",
      "regress y x",
      "-c",
      "predict fitted",
      "-c",
      "head 3",
    ])
    .output()
    .map_err(|error| format!("launch binary: {error}"))?;
  let elapsed = start.elapsed().as_secs_f64() * 1000.0;
  let text = checked_output(&output)?;
  let values = text
    .lines()
    .map(serde_json::from_str::<Value>)
    .collect::<std::result::Result<Vec<_>, _>>()
    .map_err(|error| format!("invalid CLI JSON: {error}"))?;
  require(values.len() == 6, "workflow result count")?;
  for (value, kind) in values.iter().zip([
    "LoadResult",
    "CountResult",
    "SummarizeResult",
    "RegressionResult",
    "PredictionResult",
    "PreviewResult",
  ]) {
    validate(value, kind, rows)?;
  }
  Ok(elapsed)
}
fn run(options: Options) -> Result<Value> {
  let binary = fs::canonicalize(&options.binary)
    .map_err(|error| format!("binary {}: {error}", options.binary.display()))?;
  require(binary.is_file(), "binary path must be a file")?;
  let fixture = Fixture::new(options.rows)?;
  let mut version = String::new();
  let startup = measure(options.repetitions, || {
    let start = Instant::now();
    let output = Process::new(&binary)
      .arg("--version")
      .output()
      .map_err(|error| format!("launch binary: {error}"))?;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let text = checked_output(&output)?.trim();
    require(
      text.starts_with("tabdat ") && text.lines().count() == 1,
      "TabDat version output",
    )?;
    version = text.to_owned();
    Ok(elapsed)
  })?;
  let workflow = measure(options.repetitions, || {
    sample_cli(&binary, &fixture, options.rows)
  })?;
  let mut cases =
    json!({"fresh_process_version": startup, "fresh_process_json_workflow": workflow});
  for (name, source, kind) in [
    ("session_load_csv", "use", "LoadResult"),
    ("session_summarize", "summarize x y", "SummarizeResult"),
    ("session_regress", "regress y x", "RegressionResult"),
    ("session_predict", "predict fitted", "PredictionResult"),
  ] {
    cases[name] = measure(options.repetitions, || {
      sample_session(&fixture, options.rows, source, kind)
    })?;
  }
  Ok(json!({
    "measurement_schema_version": 1,
    "binary": binary.to_string_lossy(), "binary_version": version,
    "binary_build_label": options.label,
    "harness_profile": if cfg!(debug_assertions) { "debug_assertions" } else { "no_debug_assertions" },
    "harness_package_version": env!("CARGO_PKG_VERSION"),
    "platform": {"os": std::env::consts::OS, "arch": std::env::consts::ARCH},
    "rows": options.rows,
    "csv_bytes": fs::metadata(fixture.0.join("data.csv")).map_err(|error| error.to_string())?.len(),
    "repetitions": options.repetitions, "warmups_per_case": 1,
    "fixture": "i=0..N-1; x=i/N; y=3+2x+alternating(-1,+1); group=i%10; all finite",
    "numeric_validation_absolute_tolerance": 1e-8,
    "timing_scope": {
      "process": "spawn through wait/output capture; includes CLI/reporting/teardown; excludes parent validation",
      "session": "execute only; fresh session per sample; excludes parsing/setup/reporting/validation/teardown; CSV load includes lazy backend initialization",
      "cache": "one excluded warmup per case; no OS cache flush; fresh process is not cold cache",
      "comparison": "session cases use linked harness runtime; compare with CLI only when built from matching source/profile"
    },
    "cases": cases
  }))
}
fn emit(value: &str) -> std::io::Result<()> {
  let mut output = std::io::stdout().lock();
  writeln!(output, "{value}")?;
  output.flush()
}
fn diagnostic(error: &str) {
  let mut errors = std::io::stderr().lock();
  let _ = writeln!(errors, "Error: {error}");
  let _ = errors.flush();
}
fn main() -> ExitCode {
  let options = match options(std::env::args().skip(1)) {
    Ok(Some(options)) => options,
    Ok(None) => {
      return if emit(HELP).is_ok() {
        ExitCode::SUCCESS
      } else {
        ExitCode::FAILURE
      };
    }
    Err(error) => {
      diagnostic(&error);
      return ExitCode::from(2);
    }
  };
  match run(options) {
    Ok(value) => match emit(&value.to_string()) {
      Ok(()) => ExitCode::SUCCESS,
      Err(error) => {
        diagnostic(&format!("measurement output: {error}"));
        ExitCode::FAILURE
      }
    },
    Err(error) => {
      diagnostic(&error);
      ExitCode::FAILURE
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  fn args(values: &[&str]) -> Result<Option<Options>> {
    options(values.iter().map(|value| (*value).to_owned()))
  }
  #[test]
  fn bounded_arguments_and_help() {
    let defaults = args(&["binary"]).unwrap().unwrap();
    assert_eq!((defaults.rows, defaults.repetitions), (10000, 7));
    assert!(args(&["--help"]).unwrap().is_none());
    for values in [
      vec![],
      vec!["binary", "--rows"],
      vec!["binary", "--rows", "3"],
      vec!["binary", "--rows", "1000001"],
      vec!["binary", "--rows", "no"],
      vec!["binary", "--repetitions", "0"],
      vec!["binary", "--repetitions", "51"],
      vec!["binary", "--unknown", "x"],
    ] {
      assert!(args(&values).is_err());
    }
  }
  #[test]
  fn timing_statistics_preserve_samples_and_even_odd_medians() {
    let value = statistics(vec![9.0, 1.0, 5.0]).unwrap();
    assert_eq!(value["samples_ms"], json!([9.0, 1.0, 5.0]));
    assert_eq!(
      (
        value["min_ms"].as_f64(),
        value["median_ms"].as_f64(),
        value["max_ms"].as_f64()
      ),
      (Some(1.0), Some(5.0), Some(9.0))
    );
    assert_eq!(statistics(vec![4.0, 2.0]).unwrap()["median_ms"], 3.0);
    assert_eq!(
      statistics(vec![f64::MAX, f64::MAX]).unwrap()["median_ms"].as_f64(),
      Some(f64::MAX)
    );
    for values in [vec![], vec![f64::NAN], vec![f64::INFINITY], vec![-1.0]] {
      assert!(statistics(values).is_err());
    }
    let mut calls = 0;
    let value = measure(2, || {
      calls += 1;
      Ok(calls as f64)
    })
    .unwrap();
    assert_eq!(calls, 3);
    assert_eq!(value["samples_ms"], json!([2.0, 3.0]));
    assert!(measure(2, || Err("failed command".to_owned())).is_err());
  }
  #[test]
  fn fixture_is_deterministic_and_removed_after_drop() {
    let fixture = Fixture::new(4).unwrap();
    let root = fixture.0.clone();
    assert_eq!(
      fs::read_to_string(root.join("data.csv")).unwrap(),
      "x,y,group\n0,2,0\n0.25,4.5,1\n0.5,3,2\n0.75,5.5,3\n"
    );
    drop(fixture);
    assert!(!root.exists());
  }
  #[test]
  fn real_session_samples_match_independent_even_and_odd_fixture_algebra() {
    for rows in [4, 5, 10000] {
      let fixture = Fixture::new(rows).unwrap();
      for (source, kind) in [
        ("use", "LoadResult"),
        ("summarize x y", "SummarizeResult"),
        ("regress y x", "RegressionResult"),
        ("predict fitted", "PredictionResult"),
      ] {
        assert!(sample_session(&fixture, rows, source, kind).unwrap() >= 0.0);
      }
    }
  }
  #[test]
  fn malformed_or_wrong_results_cannot_be_accepted_as_timings() {
    assert!(validate(&json!({}), "LoadResult", 4).is_err());
    let wrong = json!({"schema_version":1,"result_type":"CountResult","data":{"row_count":3}});
    assert!(validate(&wrong, "CountResult", 4).is_err());
    assert!(close(&Value::Null, 1.0).is_err());
    assert!(close(&json!(1.1), 1.0).is_err());
    assert!(run(args(&["missing-binary-for-measurement"]).unwrap().unwrap()).is_err());
  }
}
