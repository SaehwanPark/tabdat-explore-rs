use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{ExecutionResult, RuntimeError};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-binary-interaction-discrete-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let source =
      include_str!("../../../examples/binary-discrete-change/binary_discrete_change.csv");
    let data = source
      .lines()
      .enumerate()
      .map(|(row_index, line)| {
        if row_index == 0 {
          line.to_owned()
        } else {
          let mut fields = line.split(',').map(str::to_owned).collect::<Vec<_>>();
          if fields[3] == "1" {
            fields[1].clear();
          }
          fields.join(",")
        }
      })
      .collect::<Vec<_>>()
      .join("\n");
    fs::write(root.join("data.csv"), format!("{data}\n")).unwrap();
    Self(root)
  }

  fn load(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", self.0.join("data.csv").display()),
    );
    session
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn execute(session: &mut Session, command: &str) -> ExecutionResult {
  session.execute(parse_command(command).unwrap()).unwrap()
}

fn fail(session: &mut Session, command: &str) -> RuntimeError {
  session
    .execute(parse_command(command).unwrap())
    .unwrap_err()
}

fn reference(model: &str, design: &str, covariance: &str) -> Vec<f64> {
  let row = include_str!("fixtures/binary_interaction_discrete_change_reference.csv")
    .lines()
    .skip(1)
    .find(|line| {
      let fields = line.split(',').collect::<Vec<_>>();
      fields[0] == model && fields[1] == design && fields[2] == covariance
    })
    .unwrap_or_else(|| panic!("missing reference {model}/{design}/{covariance}"));
  row
    .split(',')
    .skip(5)
    .map(|value| value.parse::<f64>().unwrap())
    .collect()
}

fn close(actual: f64, expected: f64, label: &str) {
  assert!(
    (actual - expected).abs() <= 3e-10,
    "{label}: {actual:.17e} != {expected:.17e}"
  );
}

#[test]
fn interaction_discrete_changes_match_independent_base_r_for_both_links_designs_and_covariances() {
  let fixture = Fixture::new();
  for model in ["logit", "probit"] {
    for (design, terms) in [
      ("full", "x z treated c.treated#c.x c.treated#c.z"),
      (
        "full_factor",
        "x z treated c.treated#c.x c.treated#c.z i.region",
      ),
      ("product_only", "c.treated#c.x c.treated#c.z"),
    ] {
      for covariance in ["model", "robust", "cluster"] {
        let option = match covariance {
          "model" => "",
          "robust" => ", robust",
          "cluster" => ", cluster(group)",
          _ => unreachable!(),
        };
        let mut session = fixture.load();
        execute(&mut session, "generate z = x * x");
        if design == "full_factor" {
          execute(&mut session, "generate region = 1");
          execute(&mut session, "replace region = 2 if group > 7");
          execute(&mut session, "replace region = 3 if group > 14");
        }
        execute(&mut session, &format!("{model} y {terms}{option}"));
        let before = session.state().estimation;
        let ExecutionResult::BinaryDiscreteChange(actual) =
          execute(&mut session, "estat margins, discrete(treated)")
        else {
          panic!("expected a discrete probability change for {model}/{design}/{covariance}")
        };
        let expected = reference(model, design, covariance);
        assert_eq!(actual.predictor, "treated");
        assert_eq!(actual.observation_count, 114);
        assert_eq!((actual.from_value, actual.to_value), (0, 1));
        assert_eq!(actual.robust, covariance != "model");
        assert_eq!(
          actual.cluster.as_ref().map(|cluster| cluster.group_count),
          (covariance == "cluster").then_some(19)
        );
        for (label, value, expected_value) in [
          ("estimate", actual.effect.value, expected[0]),
          ("standard error", actual.effect.standard_error, expected[1]),
          ("statistic", actual.effect.statistic, expected[2]),
          ("p-value", actual.effect.p_value, expected[3]),
          (
            "lower interval",
            actual.effect.confidence_lower,
            expected[4],
          ),
          (
            "upper interval",
            actual.effect.confidence_upper,
            expected[5],
          ),
        ] {
          close(
            value,
            expected_value,
            &format!("{model}/{design}/{covariance} {label}"),
          );
        }
        assert_eq!(session.state().estimation, before);
      }
    }
  }
}

#[test]
fn interaction_discrete_changes_use_retained_sources_and_preserve_fit_on_failures() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  execute(&mut session, "probit y c.treated#c.x, robust");
  let before = session.state().estimation;
  let ExecutionResult::BinaryDiscreteChange(expected) =
    execute(&mut session, "estat margins, discrete(treated)")
  else {
    panic!("expected product-only discrete probability change")
  };

  assert!(matches!(
    fail(&mut session, "estat margins, discrete(missing)"),
    RuntimeError::EstatMarginsFailed { .. }
  ));
  assert!(
    fail(&mut session, "estat margins, discrete(x)")
      .to_string()
      .contains("only 0 and 1")
  );
  assert_eq!(session.state().estimation, before);

  execute(&mut session, "replace treated = 1 - treated");
  execute(&mut session, "replace x = x + 100 if group == 1");
  let ExecutionResult::BinaryDiscreteChange(after_edit) =
    execute(&mut session, "estat margins, discrete(treated)")
  else {
    panic!("expected retained-source discrete probability change after edits")
  };
  assert_eq!(*after_edit, *expected);
}

#[test]
fn interaction_discrete_changes_require_both_source_levels_in_the_retained_sample() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  execute(&mut session, "replace treated = 1");
  execute(&mut session, "probit y c.treated#c.x");
  let before = session.state().estimation;
  assert!(
    fail(&mut session, "estat margins, discrete(treated)")
      .to_string()
      .contains("requires both 0 and 1")
  );
  assert_eq!(session.state().estimation, before);
}
