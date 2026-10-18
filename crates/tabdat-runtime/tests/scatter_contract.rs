use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::ExecutionResult;
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-scatter-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("data.csv"),"source,y,x,p,q,r,category\n10,4,2,3,0.4,1.3,A\n11,0,0,1,0.2,-0.2,B\n12,1,1,2,0.3,0.5,A\n13,2,1,3,0.5,0.6,B\n14,3,3,4,0.6,1.4,A\n15,1,4,3,0.7,2,A\n").unwrap();
    Self(path)
  }
  fn load(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use \"{}\"", self.0.join("data.csv").display()),
    );
    session
  }
  fn plot(&self, options: &str) -> String {
    format!(
      "scatter y x, saving(\"{}\") noopen {options}",
      self.0.join("plots with spaces/figure, v1.svg").display()
    )
  }
  fn target(&self) -> PathBuf {
    self.0.join("plots with spaces/figure, v1.svg")
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn execute(session: &mut Session, text: &str) -> ExecutionResult {
  session.execute(parse_command(text).unwrap()).unwrap()
}
fn fail(session: &mut Session, text: &str) -> RuntimeError {
  session.execute(parse_command(text).unwrap()).unwrap_err()
}

#[test]
fn configuration_errors_are_actionable_and_valid_form_requires_data_lazily() {
  let mut session = Session::new();
  assert!(
    fail(&mut session, "scatter y x")
      .to_string()
      .contains("saving")
  );
  assert!(
    fail(&mut session, "scatter y x, saving(figure.svg)")
      .to_string()
      .contains("noopen")
  );
  assert!(
    fail(&mut session, "scatter y x, saving(out.png) noopen")
      .to_string()
      .contains(".svg")
  );
  assert_eq!(
    fail(&mut session, "scatter y x, saving(out.svg) noopen"),
    RuntimeError::NoActiveDataset { command: "scatter" }
  );
  assert!(session.initialized_capabilities().is_empty());
  assert!(session.state().estimation.is_none());
}

#[test]
fn complete_artifact_counts_labels_and_three_overlays_preserve_fit_data_and_order() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  execute(&mut session, "regress y x");
  execute(&mut session, "predict fitted");
  execute(&mut session, "label variable y \"Observed <outcome>\"");
  execute(&mut session, "label variable fitted \"OLS fitted values\"");
  let state = session.state();
  let signature = execute(&mut session, "datasignature");
  let rows = execute(&mut session, "head 100");
  let ExecutionResult::Plot(result) = execute(
    &mut session,
    &fixture.plot("lines(fitted p r) jitter(0.03) title(\"Native comparison\")"),
  ) else {
    panic!("expected plot")
  };
  assert_eq!(result.path, fixture.target());
  assert_eq!(result.format, "svg");
  assert_eq!((result.width, result.height), (900, 660));
  assert_eq!(result.candidate_rows, 6);
  assert_eq!(result.scatter.variable, "y");
  assert_eq!(result.scatter.point_count, 6);
  assert_eq!(result.scatter.omitted_rows, 0);
  assert_eq!(
    result
      .lines
      .iter()
      .map(|s| s.variable.as_str())
      .collect::<Vec<_>>(),
    ["fitted", "p", "r"]
  );
  assert!(
    result
      .lines
      .iter()
      .all(|s| s.point_count == 6 && s.omitted_rows == 0)
  );
  assert_eq!(result.jitter_y, 0.03);
  assert_eq!(result.coordinate_precision, "float64");
  let svg = fs::read_to_string(fixture.target()).unwrap();
  assert_eq!(svg.matches("class=\"scatter-point\"").count(), 6);
  assert_eq!(svg.matches("class=\"line-series\"").count(), 3);
  assert!(svg.contains("Observed &lt;outcome&gt;"));
  assert!(svg.contains("OLS fitted values"));
  assert!(svg.contains("data-row-indices=\"1,2,3,0,4,5\""));
  assert!(svg.contains("-2e-1"));
  assert!(svg.contains("2e0"));
  assert_eq!(session.state(), state);
  assert_eq!(execute(&mut session, "datasignature"), signature);
  assert_eq!(execute(&mut session, "head 100"), rows);
  assert!(
    fs::read_dir(fixture.target().parent().unwrap())
      .unwrap()
      .all(|e| e.unwrap().file_name() == "figure, v1.svg")
  );
}

#[test]
fn requested_series_use_independent_finite_pairs_and_missing_line_values_break_paths() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  execute(&mut session, "replace y = null if source == 11");
  execute(&mut session, "replace p = null if source == 12");
  execute(&mut session, "replace y = 0 / 0 if source == 14");
  execute(&mut session, "replace x = 1 / 0 if source == 15");
  let state = session.state();
  let signature = execute(&mut session, "datasignature");
  let ExecutionResult::Plot(result) = execute(&mut session, &fixture.plot("lines(p)")) else {
    panic!("expected plot")
  };
  assert_eq!(result.scatter.point_count, 3);
  assert_eq!(result.scatter.omitted_rows, 3);
  assert_eq!(result.lines[0].point_count, 4);
  assert_eq!(result.lines[0].omitted_rows, 2);
  let svg = fs::read_to_string(fixture.target()).unwrap();
  assert!(svg.contains("data-row-indices=\"1,3,0,4\""));
  let line = svg
    .lines()
    .find(|s| s.contains("class=\"line-series\""))
    .unwrap();
  let path = line
    .split_once(" d=\"")
    .unwrap()
    .1
    .split_once('"')
    .unwrap()
    .0;
  assert_eq!(path.matches('M').count(), 2);
  assert_eq!(session.state(), state);
  assert_eq!(execute(&mut session, "datasignature"), signature);
}

#[test]
fn schema_render_and_publication_failures_preserve_state_and_existing_files() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  execute(&mut session, "regress y x");
  execute(&mut session, "label variable x \"X coordinate\"");
  let state = session.state();
  let signature = execute(&mut session, "datasignature");
  for options in ["lines(unknown)", "lines(category)"] {
    let error = fail(&mut session, &fixture.plot(options)).to_string();
    assert!(
      error.contains("unknown variable") || error.contains("numeric coordinates"),
      "{error}"
    );
    assert!(!fixture.target().exists());
    assert!(!fixture.target().parent().unwrap().exists());
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "datasignature"), signature);
  }
  execute(&mut session, &fixture.plot(""));
  let original = fs::read(fixture.target()).unwrap();
  assert!(
    fail(&mut session, &fixture.plot("jitter(0.1)"))
      .to_string()
      .contains("already exists")
  );
  assert_eq!(fs::read(fixture.target()).unwrap(), original);
  assert_eq!(session.state(), state);
  assert_eq!(execute(&mut session, "datasignature"), signature);
  execute(&mut session, "label variable y \"Bad\"");
  let mut command = match parse_command(&fixture.plot("")).unwrap() {
    tabdat_domain::Command::Scatter { command } => command,
    _ => unreachable!(),
  };
  command.title = Some("bad\0XML".into());
  let before = session.state();
  assert!(
    session
      .execute(tabdat_domain::Command::Scatter { command })
      .unwrap_err()
      .to_string()
      .contains("invalid XML")
  );
  assert_eq!(session.state(), before);
  assert_eq!(fs::read(fixture.target()).unwrap(), original);
  fs::remove_file(fixture.target()).unwrap();
  execute(&mut session, "replace y = null");
  let before = session.state();
  let signature = execute(&mut session, "datasignature");
  assert!(
    fail(&mut session, &fixture.plot(""))
      .to_string()
      .contains("no complete finite")
  );
  assert!(!fixture.target().exists());
  assert_eq!(session.state(), before);
  assert_eq!(execute(&mut session, "datasignature"), signature);
}

#[test]
fn no_drawable_line_and_overflowing_finite_domains_leave_no_artifact() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  execute(&mut session, "replace p = null if source != 10");
  let state = session.state();
  let signature = execute(&mut session, "datasignature");
  assert!(
    fail(&mut session, &fixture.plot("lines(p)"))
      .to_string()
      .contains("adjacent finite pair")
  );
  assert!(!fixture.target().exists());
  assert_eq!(session.state(), state);
  assert_eq!(execute(&mut session, "datasignature"), signature);
  let extreme = fixture.0.join("extreme.csv");
  fs::write(&extreme, "x,y\n1e308,1\n-1e308,2\n").unwrap();
  execute(&mut session, &format!("use \"{}\"", extreme.display()));
  let state = session.state();
  let signature = execute(&mut session, "datasignature");
  assert!(
    fail(&mut session, &fixture.plot(""))
      .to_string()
      .contains("range")
  );
  assert!(!fixture.target().exists());
  assert_eq!(session.state(), state);
  assert_eq!(execute(&mut session, "datasignature"), signature);
}

#[test]
fn large_candidate_cohort_is_rejected_before_artifact_or_parent_creation() {
  let fixture = Fixture::new();
  let mut csv = String::from("x,y\n");
  for i in 0..10_001 {
    csv.push_str(&format!("{i},{}\n", i % 5));
  }
  fs::write(fixture.0.join("data.csv"), csv).unwrap();
  let mut session = fixture.load();
  let state = session.state();
  let signature = execute(&mut session, "datasignature");
  assert!(
    fail(&mut session, &fixture.plot(""))
      .to_string()
      .contains("10000 candidate")
  );
  assert!(!fixture.target().parent().unwrap().exists());
  assert_eq!(session.state(), state);
  assert_eq!(execute(&mut session, "datasignature"), signature);
}
