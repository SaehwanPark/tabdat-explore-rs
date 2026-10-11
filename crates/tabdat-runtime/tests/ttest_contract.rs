use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::parse_command;
use tabdat_runtime::{ExecutionResult, RuntimeError, Session, TtestResult};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
  root: PathBuf,
  parquet: PathBuf,
}

impl Fixture {
  fn new_with_sql(sql: &str) -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-ttest-{}-{}-{}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let parquet = root.join("ttest.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        &format!("COPY ({sql}) TO ? (FORMAT PARQUET)"),
        [&parquet_string],
      )
      .expect("fixture table should write to parquet");
    Self { root, parquet }
  }

  fn load(&self, session: &mut Session) {
    session
      .execute(parse_command(&format!("use {}", self.parquet.display())).unwrap())
      .expect("fixture dataset should load");
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

fn fixture() -> Fixture {
  Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (1.0, 10.0, 5.0, 'group1'), \
      (2.0, 12.0, 6.0, 'group1'), \
      (3.0, 15.0, 8.0, 'group2'), \
      (4.0, 18.0, 9.0, 'group2'), \
      (5.0, 20.0, 11.0, 'group1'), \
      (6.0, 25.0, 12.0, 'group2') \
    ) AS t(x1, x2, y, grp)",
  )
}

fn execute_ttest(session: &mut Session, source: &str) -> TtestResult {
  let result = session.execute(parse_command(source).expect("valid ttest syntax"));
  let ExecutionResult::Ttest(result) = result.expect("ttest should execute") else {
    panic!("expected ExecutionResult::Ttest");
  };
  *result
}

fn close(actual: f64, expected: f64) {
  let tolerance = 1e-9_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "expected {expected}, got {actual}"
  );
}

#[test]
fn ttest_requires_active_dataset() {
  let mut session = Session::new();
  let command = parse_command("ttest wage == 0").expect("valid syntax");
  let error = session.execute(command).unwrap_err();
  assert_eq!(error, RuntimeError::NoActiveDataset { command: "ttest" });
  assert_eq!(
    error.to_string(),
    "ttest requires an active dataset; run use <path> first"
  );
}

#[test]
fn parsed_ttest_forms_match_python_and_scipy_references() {
  let fixture = fixture();
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x1").unwrap())
    .expect("regression state should be available for the read-only check");
  let before_dataset = session.active_dataset().cloned();
  let before_regression = session.last_regression().cloned();

  let one = execute_ttest(&mut session, "ttest x1 == 3");
  assert_eq!(one.varname1, "x1");
  assert_eq!(one.null_value, 3.0);
  assert!(!one.is_paired);
  assert!(!one.is_welch);
  assert_eq!(one.group1_stats.name, "x1");
  assert_eq!(one.group1_stats.observations, 6);
  close(one.group1_stats.mean, 3.5);
  close(one.group1_stats.standard_deviation, 1.870_828_693_386_970_7);
  close(one.group1_stats.standard_error, 0.763_762_615_825_973_4);
  close(one.group1_stats.ci_lower, 1.536_685_693_019_676);
  close(one.group1_stats.ci_upper, 5.463_314_306_980_324);
  assert_eq!(one.difference_stats.name, "mean");
  close(one.t_statistic, 0.654_653_670_707_977_1);
  close(one.degrees_of_freedom, 5.0);
  close(one.p_left, 0.729_197_719_603_439_8);
  close(one.p_two, 0.541_604_560_793_120_4);
  close(one.p_right, 0.270_802_280_396_560_2);
  assert!(one.group2_stats.is_none());
  assert!(one.combined_stats.is_none());

  let paired = execute_ttest(&mut session, "ttest x1 == x2");
  assert!(paired.is_paired);
  assert_eq!(paired.null_value, 0.0);
  assert_eq!(paired.group1_stats.name, "x1");
  assert_eq!(paired.group2_stats.as_ref().unwrap().name, "x2");
  close(paired.group1_stats.ci_lower, 1.536_685_693_019_676);
  close(paired.group1_stats.ci_upper, 5.463_314_306_980_324);
  close(
    paired.group2_stats.as_ref().unwrap().mean,
    16.666_666_666_666_668,
  );
  close(
    paired.group2_stats.as_ref().unwrap().ci_lower,
    10.893_180_812_194_3,
  );
  close(
    paired.group2_stats.as_ref().unwrap().ci_upper,
    22.440_152_521_139_034,
  );
  assert_eq!(paired.difference_stats.name, "diff");
  assert_eq!(paired.difference_stats.observations, 6);
  close(paired.difference_stats.mean, -13.166_666_666_666_666);
  close(
    paired.difference_stats.standard_error,
    1.492_574_211_815_874_2,
  );
  close(paired.difference_stats.ci_lower, -17.003_450_823_899_74);
  close(paired.difference_stats.ci_upper, -9.329_882_509_433_59);
  close(paired.t_statistic, -8.821_448_583_550_177);
  close(paired.degrees_of_freedom, 5.0);
  close(paired.p_left, 0.000_155_440_402_967_537_34);
  close(paired.p_two, 0.000_310_880_805_935_109_8);
  close(paired.p_right, 0.999_844_559_597_032_4);
  assert!(paired.combined_stats.is_none());

  let pooled = execute_ttest(&mut session, "ttest x1, by(grp)");
  assert_eq!(pooled.by_variable.as_deref(), Some("grp"));
  assert!(!pooled.is_paired);
  assert!(!pooled.is_welch);
  assert_eq!(pooled.group1_stats.name, "group1");
  assert_eq!(pooled.group1_stats.observations, 3);
  close(pooled.group1_stats.mean, 2.666_666_666_666_666_5);
  close(pooled.group1_stats.standard_error, 1.201_850_425_154_663_3);
  close(
    pooled.group1_stats.standard_deviation,
    2.081_665_999_466_133,
  );
  close(pooled.group1_stats.ci_lower, -2.504_478_345_875_597);
  close(pooled.group1_stats.ci_upper, 7.837_811_679_208_929);
  assert_eq!(pooled.group2_stats.as_ref().unwrap().name, "group2");
  assert_eq!(pooled.group2_stats.as_ref().unwrap().observations, 3);
  close(
    pooled.group2_stats.as_ref().unwrap().mean,
    4.333_333_333_333_333,
  );
  close(
    pooled.group2_stats.as_ref().unwrap().standard_error,
    0.881_917_103_688_196_9,
  );
  close(
    pooled.group2_stats.as_ref().unwrap().standard_deviation,
    1.527_525_231_651_946_5,
  );
  close(
    pooled.group2_stats.as_ref().unwrap().ci_lower,
    0.538_750_299_736_573_6,
  );
  close(
    pooled.group2_stats.as_ref().unwrap().ci_upper,
    8.127_916_366_930_092,
  );
  assert_eq!(pooled.combined_stats.as_ref().unwrap().observations, 6);
  close(pooled.combined_stats.as_ref().unwrap().mean, 3.5);
  close(
    pooled.combined_stats.as_ref().unwrap().standard_error,
    0.763_762_615_825_973_4,
  );
  close(
    pooled.combined_stats.as_ref().unwrap().standard_deviation,
    1.870_828_693_386_970_7,
  );
  close(
    pooled.combined_stats.as_ref().unwrap().ci_lower,
    1.536_685_693_019_676,
  );
  close(
    pooled.combined_stats.as_ref().unwrap().ci_upper,
    5.463_314_306_980_324,
  );
  assert_eq!(pooled.difference_stats.name, "diff");
  assert_eq!(pooled.difference_stats.observations, 4);
  close(pooled.difference_stats.mean, -1.666_666_666_666_666_5);
  close(
    pooled.difference_stats.standard_error,
    1.490_711_984_999_859_8,
  );
  close(pooled.difference_stats.ci_lower, -5.805_546_660_679_214);
  close(pooled.difference_stats.ci_upper, 2.472_213_327_345_880_6);
  assert!(pooled.difference_stats.standard_deviation.is_nan());
  close(pooled.t_statistic, -1.118_033_988_749_894_7);
  close(pooled.degrees_of_freedom, 4.0);
  close(pooled.p_left, 0.163_082_117_672_530_1);
  close(pooled.p_two, 0.326_164_235_345_060_2);
  close(pooled.p_right, 0.836_917_882_327_469_9);

  let welch = execute_ttest(&mut session, "ttest x1, by(grp) welch");
  assert!(welch.is_welch);
  assert_eq!(welch.difference_stats.observations, 0);
  close(welch.difference_stats.mean, pooled.difference_stats.mean);
  close(
    welch.difference_stats.standard_error,
    pooled.difference_stats.standard_error,
  );
  close(welch.difference_stats.ci_lower, -5.956_674_371_741_123);
  close(welch.difference_stats.ci_upper, 2.623_341_038_407_79);
  close(welch.t_statistic, -1.118_033_988_749_894_7);
  close(welch.degrees_of_freedom, 3.669_724_770_642_200_7);
  close(welch.p_left, 0.165_676_002_677_430_96);
  close(welch.p_two, 0.331_352_005_354_862);
  close(welch.p_right, 0.834_323_997_322_569);

  assert_eq!(session.active_dataset().cloned(), before_dataset);
  assert_eq!(session.last_regression().cloned(), before_regression);
}

#[test]
fn failed_ttest_preserves_active_and_estimation_state() {
  let fixture = fixture();
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x1").unwrap())
    .expect("regression should fit");
  let before_dataset = session.active_dataset().cloned();
  let before_regression = session.last_regression().cloned();

  let error = session
    .execute(parse_command("ttest x1, by(missing)").unwrap())
    .unwrap_err();

  assert_eq!(
    error,
    RuntimeError::TtestUnknownVariable {
      variables: vec!["missing".into()]
    }
  );
  assert_eq!(session.active_dataset().cloned(), before_dataset);
  assert_eq!(session.last_regression().cloned(), before_regression);
}

#[test]
fn ttest_filters_null_and_nonfinite_observations_and_sorts_group_labels() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (1.0, 10.0, 'B'), \
      (2.0, CAST(NULL AS DOUBLE), 'A'), \
      (3.0, 30.0, 'A'), \
      (CAST(NULL AS DOUBLE), 40.0, 'C'), \
      ('NaN'::DOUBLE, 50.0, 'D'), \
      ('Infinity'::DOUBLE, 60.0, 'E'), \
      (4.0, 40.0, CAST(NULL AS VARCHAR)) \
    ) AS t(x, y, grp)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);

  let one = execute_ttest(&mut session, "ttest x == 0");
  assert_eq!(one.group1_stats.observations, 4);
  close(one.group1_stats.mean, 2.5);

  let paired = execute_ttest(&mut session, "ttest x == y");
  assert_eq!(paired.difference_stats.observations, 3);
  close(paired.difference_stats.mean, -24.0);

  let by_group = execute_ttest(&mut session, "ttest x, by(grp) welch");
  assert_eq!(by_group.group1_stats.name, "A");
  assert_eq!(by_group.group1_stats.observations, 2);
  assert_eq!(by_group.group2_stats.as_ref().unwrap().name, "B");
  assert_eq!(by_group.group2_stats.as_ref().unwrap().observations, 1);
  close(by_group.degrees_of_freedom, 1.0);

  let pooled = execute_ttest(&mut session, "ttest x, by(grp)");
  assert_eq!(pooled.degrees_of_freedom, 0.0);
  assert!(pooled.difference_stats.standard_error.is_nan());
  assert!(pooled.difference_stats.ci_lower.is_nan());
  assert!(pooled.p_two.is_nan());
}

#[test]
fn ttest_preserves_python_scalar_group_labels() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES (1.0, true), (2.0, false), (3.0, true), (4.0, false)) AS t(x, grp)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);

  let result = execute_ttest(&mut session, "ttest x, by(grp)");
  assert_eq!(result.group1_stats.name, "False");
  assert_eq!(result.group2_stats.as_ref().unwrap().name, "True");

  let numeric_fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES (1.0, 1e-7), (2.0, 1e-7), (3.0, 1.0), (4.0, 1.0)) AS t(x, grp)",
  );
  numeric_fixture.load(&mut session);
  let numeric = execute_ttest(&mut session, "ttest x, by(grp)");
  assert_eq!(numeric.group1_stats.name, "1.0");
  assert_eq!(numeric.group2_stats.as_ref().unwrap().name, "1e-07");

  let decimal_fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (1.0, 1.20::DECIMAL(5, 2)), \
      (2.0, 1.20::DECIMAL(5, 2)), \
      (3.0, 2.30::DECIMAL(5, 2)), \
      (4.0, 2.30::DECIMAL(5, 2)) \
    ) AS t(x, grp)",
  );
  decimal_fixture.load(&mut session);
  let decimal = execute_ttest(&mut session, "ttest x, by(grp)");
  assert_eq!(decimal.group1_stats.name, "1.20");
  assert_eq!(decimal.group2_stats.as_ref().unwrap().name, "2.30");

  let blob_fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (1.0, from_hex('4127')), \
      (2.0, from_hex('4127')), \
      (3.0, from_hex('2242')), \
      (4.0, from_hex('2242')) \
    ) AS t(x, grp)",
  );
  blob_fixture.load(&mut session);
  let blob = execute_ttest(&mut session, "ttest x, by(grp)");
  assert_eq!(blob.group1_stats.name, "b\"A'\"");
  assert_eq!(blob.group2_stats.as_ref().unwrap().name, "b'\"B'");
}

#[test]
fn ttest_normalizes_nanosecond_timestamp_labels_and_rejects_time_zone_keys() {
  let timestamp_fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (1.0, '2020-01-02 03:04:05.123456789'::TIMESTAMP_NS), \
      (2.0, '2020-01-02 03:04:05.123456789'::TIMESTAMP_NS), \
      (3.0, '2020-01-02 03:04:05.000000001'::TIMESTAMP_NS), \
      (4.0, '2020-01-02 03:04:05.000000001'::TIMESTAMP_NS) \
    ) AS t(x, grp)",
  );
  let mut session = Session::new();
  timestamp_fixture.load(&mut session);
  let timestamp = execute_ttest(&mut session, "ttest x, by(grp)");
  assert_eq!(timestamp.group1_stats.name, "2020-01-02 03:04:05");
  assert_eq!(
    timestamp.group2_stats.as_ref().unwrap().name,
    "2020-01-02 03:04:05.123456"
  );

  let time_fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (1.0, '03:04:05+02:00'::TIME WITH TIME ZONE), \
      (2.0, '03:04:05+02:00'::TIME WITH TIME ZONE), \
      (3.0, '04:05:06+03:00'::TIME WITH TIME ZONE), \
      (4.0, '04:05:06+03:00'::TIME WITH TIME ZONE) \
    ) AS t(x, grp)",
  );
  time_fixture.load(&mut session);
  let error = session
    .execute(parse_command("ttest x, by(grp)").unwrap())
    .unwrap_err();
  assert!(matches!(
    error,
    RuntimeError::TtestUnsupportedGroupType { variable, data_type }
      if variable == "grp" && data_type.contains("TIME")
  ));
}

#[test]
fn ttest_preserves_single_observation_and_zero_variance_results() {
  let constant_fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (2.0, 2.0, 'a'), \
      (2.0, 2.0, 'a'), \
      (2.0, 2.0, 'b'), \
      (2.0, 2.0, 'b') \
    ) AS t(x, y, grp)",
  );
  let mut session = Session::new();
  constant_fixture.load(&mut session);

  let one = execute_ttest(&mut session, "ttest x == 2");
  assert_eq!(one.group1_stats.mean, 2.0);
  assert_eq!(one.group1_stats.ci_lower, 2.0);
  assert_eq!(one.group1_stats.ci_upper, 2.0);
  assert_eq!(one.degrees_of_freedom, 3.0);
  assert!(one.t_statistic.is_nan());
  assert!(one.p_left.is_nan());
  assert!(one.p_two.is_nan());
  assert!(one.p_right.is_nan());

  let paired = execute_ttest(&mut session, "ttest x == y");
  assert_eq!(paired.difference_stats.mean, 0.0);
  assert_eq!(paired.difference_stats.ci_lower, 0.0);
  assert_eq!(paired.difference_stats.ci_upper, 0.0);
  assert_eq!(paired.degrees_of_freedom, 3.0);
  assert!(paired.t_statistic.is_nan());
  assert!(paired.p_two.is_nan());

  let pooled = execute_ttest(&mut session, "ttest x, by(grp)");
  assert_eq!(pooled.difference_stats.mean, 0.0);
  assert_eq!(pooled.difference_stats.ci_lower, 0.0);
  assert_eq!(pooled.difference_stats.ci_upper, 0.0);
  assert_eq!(pooled.degrees_of_freedom, 2.0);
  assert!(pooled.t_statistic.is_nan());
  assert!(pooled.p_two.is_nan());

  let one_row_fixture = Fixture::new_with_sql("SELECT * FROM (VALUES (3.5)) AS t(x)");
  one_row_fixture.load(&mut session);
  let single = execute_ttest(&mut session, "ttest x == 0");
  assert_eq!(single.group1_stats.observations, 1);
  assert_eq!(single.group1_stats.mean, 3.5);
  assert!(single.group1_stats.standard_deviation.is_nan());
  assert!(single.group1_stats.standard_error.is_nan());
  assert!(single.group1_stats.ci_lower.is_nan());
  assert!(single.group1_stats.ci_upper.is_nan());
  assert_eq!(single.degrees_of_freedom, 0.0);
  assert!(single.t_statistic.is_nan());
  assert!(single.p_two.is_nan());
}

#[test]
fn ttest_reports_oracle_precondition_and_sample_errors() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (CAST(NULL AS DOUBLE), CAST(NULL AS DOUBLE), 'only'), \
      ('NaN'::DOUBLE, 'Infinity'::DOUBLE, 'other') \
    ) AS t(x, y, grp)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);

  let unknown = session
    .execute(parse_command("ttest missing == 0").unwrap())
    .unwrap_err();
  assert_eq!(
    unknown,
    RuntimeError::TtestUnknownVariable {
      variables: vec!["missing".into()]
    }
  );
  assert_eq!(unknown.to_string(), "ttest unknown variable: missing");
  assert_eq!(
    session
      .execute(parse_command("ttest x, by(missing)").unwrap())
      .unwrap_err(),
    RuntimeError::TtestUnknownVariable {
      variables: vec!["missing".into()]
    }
  );
  let no_pairs = session
    .execute(parse_command("ttest x == y").unwrap())
    .unwrap_err();
  assert_eq!(
    no_pairs,
    RuntimeError::TtestNoValidPairedNumericObservations
  );
  assert_eq!(
    no_pairs.to_string(),
    "ttest: no valid paired numeric observations found"
  );
  let no_values = session
    .execute(parse_command("ttest x == 0").unwrap())
    .unwrap_err();
  assert_eq!(no_values, RuntimeError::TtestNoValidNumericObservations);
  assert_eq!(
    no_values.to_string(),
    "ttest: no valid numeric observations found"
  );
  let no_groups = session
    .execute(parse_command("ttest x, by(grp)").unwrap())
    .unwrap_err();
  assert_eq!(no_groups, RuntimeError::TtestInvalidGroupCount { found: 0 });
  assert_eq!(
    no_groups.to_string(),
    "ttest: by() variable must define exactly two groups (found 0)"
  );
}

#[test]
fn ttest_checks_tested_numeric_types_and_filtered_group_count() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES (1.0, 'a', 'kept'), (2.0, 'b', 'kept'), (3.0, 'c', 'kept'), \
      (CAST(NULL AS DOUBLE), 'd', 'ignored')) AS t(x, text_col, grp)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);

  let non_numeric = session
    .execute(parse_command("ttest text_col == 0").unwrap())
    .unwrap_err();
  assert_eq!(
    non_numeric,
    RuntimeError::TtestRequiresNumeric {
      variables: vec!["text_col".into()]
    }
  );
  assert_eq!(
    non_numeric.to_string(),
    "ttest requires numeric variables: text_col"
  );
  let one_group = session
    .execute(parse_command("ttest x, by(grp)").unwrap())
    .unwrap_err();
  assert_eq!(one_group, RuntimeError::TtestInvalidGroupCount { found: 1 });
  assert_eq!(
    one_group.to_string(),
    "ttest: by() variable must define exactly two groups (found 1)"
  );
}

#[test]
fn ttest_rejects_nested_group_keys_with_a_typed_limitation() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES (1.0, [1, 2]), (2.0, [1, 2]), (3.0, [3, 4]), (4.0, [3, 4])) AS t(x, grp)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);

  assert_eq!(
    session
      .execute(parse_command("ttest x, by(grp)").unwrap())
      .unwrap_err(),
    RuntimeError::TtestUnsupportedGroupType {
      variable: "grp".into(),
      data_type: "INTEGER[]".into(),
    }
  );
}
