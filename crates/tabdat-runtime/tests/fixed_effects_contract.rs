use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, EstimationState, ExecutionResult, FixedEffectsState};
use tabdat_language::parse_command;
use tabdat_runtime::Session;
const DATA: &str = "id,year,y,x\n2,1982,5.3,2.0\n1,1980,3.1,1.1\n2,1979,4.0,0.5\n1,1979,2.1,0.7\n2,1980,4.8,1.0\n1,1981,,1.3\n3,1984,8.0,2.2\n2,1981,4.9,1.2\n1,1983,3.6,1.5\n4,1979,,1.0\n1,1984,4.2,\n";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-within-fe-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("data.csv"), DATA).unwrap();
    Self(path)
  }
  fn load(&self) -> Session {
    let mut s = Session::new();
    run(
      &mut s,
      &format!("use \"{}\"", self.0.join("data.csv").display()),
    );
    s
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn run(s: &mut Session, text: &str) -> ExecutionResult {
  s.execute(parse_command(text).unwrap())
    .unwrap_or_else(|error| panic!("{text}: {error}"))
}
fn fit(s: &mut Session, extra: &str) -> FixedEffectsState {
  let ExecutionResult::FixedEffects(state) =
    run(s, &format!("xtreg y x, fe id(id) time(year) {extra}"))
  else {
    panic!("wrong result");
  };
  *state
}
fn rows(s: &mut Session, query: &str) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Table(t) = run(s, &format!("sql \"\"\"{query}\"\"\"")) else {
    panic!("wrong result");
  };
  t.rows
}
fn number(cell: &CellValue) -> Option<f64> {
  match cell {
    CellValue::Float(x) => Some(*x),
    CellValue::SignedInteger(x) => Some(*x as f64),
    CellValue::UnsignedInteger(x) => Some(*x as f64),
    CellValue::Null => None,
    _ => panic!("not numeric: {cell:?}"),
  }
}

#[test]
fn raw_keys_joint_sample_unit_centers_and_inference_df_are_explicit() {
  let f = Fixture::new();
  let mut s = f.load();
  let classical = fit(&mut s, "");
  let result = &classical.result;
  assert_eq!(
    result.within.sample.as_ref().unwrap().retained_indices,
    [0, 1, 2, 3, 4, 6, 7, 8]
  );
  assert_eq!(
    result.within.sample.as_ref().unwrap().total_observations,
    11
  );
  assert_eq!(
    result.within.sample.as_ref().unwrap().dropped_observations,
    3
  );
  assert_eq!(result.effects.len(), 3);
  assert_eq!(result.singleton_groups, 1);
  assert_eq!(result.minimum_group_size, 1);
  assert_eq!(result.maximum_group_size, 4);
  assert_eq!(result.within.fit_statistics.degrees_of_freedom, 4);
  let unit = result.effects.iter().find(|unit| unit.key == "1").unwrap();
  assert!((unit.predictor_means[0] - 1.1).abs() < 1e-12);
  assert!((unit.response_mean - 8.8 / 3.0).abs() < 1e-12);
  assert!(!result.effects.iter().any(|unit| unit.key == "4"));
  let ExecutionResult::Lincom(ci) = run(&mut s, "lincom x") else {
    panic!();
  };
  assert_eq!(ci.df_residual, Some(4));
  let clustered = fit(&mut s, "cluster(id)");
  let alias = fit(&mut s, "robust");
  assert_eq!(clustered, alias);
  assert_eq!(clustered.result.within.fit_statistics.degrees_of_freedom, 4);
  let ExecutionResult::Lincom(ci) = run(&mut s, "lincom x") else {
    panic!();
  };
  assert_eq!(ci.df_residual, Some(2));
  let ExecutionResult::Test(test) = run(&mut s, "test x = 0") else {
    panic!();
  };
  assert_eq!(test.df_residual, Some(2));
  assert_eq!(test.df, 1);
  assert!(!test.is_chi2);
  assert!(
    s.execute(parse_command("lincom intercept").unwrap())
      .is_err()
  );
  assert_eq!(
    s.state().estimation.as_ref().unwrap().family(),
    tabdat_domain::EstimationFamily::PanelLinear
  );
}

#[test]
fn conditional_predictions_use_current_values_and_retained_effects_without_row_position_caches() {
  let f = Fixture::new();
  let mut s = f.load();
  let state = fit(&mut s, "cluster(id)");
  let cached = s.state().estimation;
  let beta = state.result.within.coefficients[0].value;
  run(&mut s, "predict original_effect, u");
  run(&mut s, "replace x = x + 0.4");
  run(&mut s, "replace y = y + 0.2");
  run(&mut s, "sort id year");
  for (target, kind) in [
    ("common", "xb"),
    ("conditional", "xbu"),
    ("unit", "u"),
    ("error", "residuals"),
  ] {
    run(&mut s, &format!("predict {target}, {kind}"));
  }
  assert_eq!(s.state().estimation, cached);
  let data = rows(
    &mut s,
    "SELECT CAST(id AS VARCHAR), year, x, y, common, conditional, unit, error, original_effect FROM active",
  );
  assert_eq!(data.len(), 11);
  for row in data {
    let CellValue::Text(key) = &row[0] else {
      panic!();
    };
    let effect = state
      .result
      .effects
      .iter()
      .find(|effect| effect.key == *key);
    assert_eq!(number(&row[6]), number(&row[8]));
    let x = number(&row[2]);
    let y = number(&row[3]);
    if let Some(x) = x {
      assert!(
        (number(&row[4]).unwrap() - (state.result.common_intercept + x * beta)).abs() < 1e-12
      );
    } else {
      assert_eq!(row[4], CellValue::Null);
    }
    if let Some(effect) = effect {
      assert!((number(&row[6]).unwrap() - effect.effect).abs() < 1e-12);
      if let Some(x) = x {
        let mu = effect.response_mean + (x - effect.predictor_means[0]) * beta;
        assert!((number(&row[5]).unwrap() - mu).abs() < 1e-12);
        if let Some(y) = y {
          assert!((number(&row[7]).unwrap() - (y - mu)).abs() < 1e-12);
        } else {
          assert_eq!(row[7], CellValue::Null);
        }
      } else {
        assert_eq!(row[5], CellValue::Null);
        assert_eq!(row[7], CellValue::Null);
      }
    } else {
      for index in [5, 6, 7, 8] {
        assert_eq!(row[index], CellValue::Null);
      }
    }
  }
}

#[test]
fn wide_entity_and_time_keys_are_never_rounded_and_conditional_type_mismatch_is_atomic() {
  let f = Fixture::new();
  let mut s = f.load();
  let base = fit(&mut s, "");
  run(
    &mut s,
    "sql \"\"\"SELECT CAST(CASE WHEN id=2 THEN 18446744073709551614 WHEN id=1 THEN 18446744073709551613 ELSE id END AS UBIGINT) AS id, CAST(year+18446744073709540000 AS UBIGINT) AS year, y, x FROM active\"\"\" into wide",
  );
  let state = fit(&mut s, "cluster(id)");
  assert_eq!(state.entity_data_type, "UBIGINT");
  assert!(
    state
      .result
      .effects
      .iter()
      .any(|effect| effect.key == "18446744073709551614")
  );
  assert!(
    state
      .result
      .effects
      .iter()
      .any(|effect| effect.key == "18446744073709551613")
  );
  assert!(
    (state.result.within.coefficients[0].value - base.result.within.coefficients[0].value).abs()
      < 1e-12
  );
  run(&mut s, "predict mu, xbu");
  run(&mut s, "predict effect, u");
  let keys = rows(&mut s, "SELECT CAST(id AS VARCHAR), mu, effect FROM active");
  assert!(keys.iter().filter(|row| number(&row[1]).is_some()).count() > 0);
  let cached = s.state().estimation;
  run(
    &mut s,
    "sql \"\"\"SELECT CAST(id AS VARCHAR) AS id, year, y, x FROM active\"\"\" into text_ids",
  );
  let before = s.state();
  let data = rows(&mut s, "SELECT * FROM active");
  assert!(
    s.execute(parse_command("predict bad, xbu").unwrap())
      .is_err()
  );
  assert_eq!(s.state(), before);
  assert_eq!(rows(&mut s, "SELECT * FROM active"), data);
  run(&mut s, "predict common, xb");
  assert_eq!(s.state().estimation, cached);
}

#[test]
fn prediction_failures_preserve_fit_and_data_but_runtime_fit_failures_clear_only_model() {
  let f = Fixture::new();
  let mut s = f.load();
  fit(&mut s, "");
  for text in [
    "predict y, xbu",
    "predict bad, pr",
    "predict bad, posterior_predictive interval",
  ] {
    let before = s.state();
    let data = rows(&mut s, "SELECT * FROM active");
    let command = parse_command(text).unwrap();
    assert!(s.execute(command).is_err(), "{text}");
    assert_eq!(s.state(), before);
    assert_eq!(rows(&mut s, "SELECT * FROM active"), data);
  }
  let before = s.state();
  assert!(parse_command("xtreg y x, fe id(id)").is_err());
  assert_eq!(s.state(), before);
  assert!(s.execute(parse_command("xtreg y x, re").unwrap()).is_err());
  assert_eq!(s.state(), before);
  run(&mut s, "drop x");
  let before = s.state();
  assert!(
    s.execute(parse_command("predict bad, xbu").unwrap())
      .is_err()
  );
  assert_eq!(s.state(), before);
  run(&mut s, "predict unit, u");
  for query in [
    "SELECT id, CASE WHEN id=4 THEN NULL ELSE year END AS year, y,x FROM active",
    "SELECT id, CAST(year AS DOUBLE)+CASE WHEN id=4 THEN 0.5 ELSE 0.0 END AS year,y,x FROM active",
    "SELECT id, CASE WHEN id=4 THEN 'NaN'::DOUBLE ELSE year END AS year,y,x FROM active",
    "SELECT CASE WHEN id=4 THEN 'NaN'::DOUBLE ELSE id END AS id,year,y,x FROM active",
    "SELECT id,CASE WHEN id=2 THEN 1979 ELSE year END AS year,y,x FROM active",
    "SELECT id,year,y,CAST(id AS DOUBLE) AS x FROM active",
  ] {
    let mut s = f.load();
    fit(&mut s, "");
    run(&mut s, &format!("sql \"\"\"{query}\"\"\" into bad_keys"));
    let mut expected = s.state();
    let data = run(&mut s, "datasignature");
    expected.estimation = None;
    assert!(
      s.execute(parse_command("xtreg y x, fe id(id) time(year)").unwrap())
        .is_err(),
      "{query}"
    );
    assert_eq!(s.state(), expected);
    assert_eq!(run(&mut s, "datasignature"), data);
  }
  for text in [
    "xtreg y x, fe",
    "xtreg y x, fe id(id) time(year) cluster(year)",
  ] {
    let mut s = f.load();
    fit(&mut s, "");
    let mut expected = s.state();
    expected.estimation = None;
    assert!(s.execute(parse_command(text).unwrap()).is_err());
    assert_eq!(s.state(), expected);
  }
  let mut empty = Session::new();
  assert!(
    empty
      .execute(parse_command("xtreg y x, fe id(id) time(year)").unwrap())
      .is_err()
  );
  assert_eq!(empty.state().estimation, None);
  assert!(matches!(
    s.state().estimation,
    Some(EstimationState::FixedEffects(_))
  ));
}
