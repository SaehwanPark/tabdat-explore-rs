//! Bounded SQL Student-t inference for workflows with fractional degrees of freedom.
//!
//! The safe Arrow bridge delegates to the same numerical functions as native
//! linear inference. Invalid or NULL inputs yield NULL, rather than an invented
//! probability; callers must check the result. The finite DF cap is the audited
//! boundary of this SQL interface, not a general restriction on the stats API.

use std::{error::Error, sync::Arc};

use duckdb::arrow::array::{Array, Float64Array, RecordBatch};
use duckdb::arrow::datatypes::DataType;
use duckdb::vscalar::arrow::{ArrowFunctionSignature, VArrowScalar};

pub struct StudentTPvalue;
pub struct StudentTQuantile;

fn inputs(input: &RecordBatch) -> Result<(&Float64Array, &Float64Array), Box<dyn Error>> {
  let Some(first) = input.column(0).as_any().downcast_ref::<Float64Array>() else {
    return Err("Student-t function requires double arguments".into());
  };
  let Some(second) = input.column(1).as_any().downcast_ref::<Float64Array>() else {
    return Err("Student-t function requires double arguments".into());
  };
  Ok((first, second))
}

fn signature() -> Vec<ArrowFunctionSignature> {
  vec![ArrowFunctionSignature::exact(
    vec![DataType::Float64, DataType::Float64],
    DataType::Float64,
  )]
}

fn valid_df(df: f64) -> bool {
  df.is_finite() && (1.0..=1_000_000.0).contains(&df)
}

impl VArrowScalar for StudentTPvalue {
  type State = ();

  fn invoke(_: &Self::State, input: RecordBatch) -> Result<Arc<dyn Array>, Box<dyn Error>> {
    let (values, dfs) = inputs(&input)?;
    let out = values.iter().zip(dfs.iter()).map(|(value, df)| {
      let (value, df) = (value?, df?);
      if !value.is_finite() || value.abs() > 1_000_000.0 || !valid_df(df) {
        return None;
      }
      let result = tabdat_stats::student_t_pvalue(value, df);
      result.is_finite().then_some(result)
    });
    Ok(Arc::new(Float64Array::from_iter(out)) as Arc<dyn Array>)
  }

  fn signatures() -> Vec<ArrowFunctionSignature> {
    signature()
  }
}

impl VArrowScalar for StudentTQuantile {
  type State = ();

  fn invoke(_: &Self::State, input: RecordBatch) -> Result<Arc<dyn Array>, Box<dyn Error>> {
    let (probabilities, dfs) = inputs(&input)?;
    let out = probabilities
      .iter()
      .zip(dfs.iter())
      .map(|(probability, df)| {
        let (probability, df) = (probability?, df?);
        if !probability.is_finite() || !(0.025..=0.975).contains(&probability) || !valid_df(df) {
          return None;
        }
        let result = tabdat_stats::student_t_quantile(probability, df);
        result.is_finite().then_some(result)
      });
    Ok(Arc::new(Float64Array::from_iter(out)) as Arc<dyn Array>)
  }

  fn signatures() -> Vec<ArrowFunctionSignature> {
    signature()
  }
}

#[cfg(test)]
mod tests {
  use duckdb::Connection;

  use super::{StudentTPvalue, StudentTQuantile};

  fn registered() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    connection
      .register_scalar_function::<StudentTPvalue>("tabdat_t_pvalue")
      .unwrap();
    connection
      .register_scalar_function::<StudentTQuantile>("tabdat_t_quantile")
      .unwrap();
    connection
  }

  #[test]
  fn fractional_df_and_tails_match_r() {
    // R 4.6.2 qt(.975, df), 2*pt(-8.41, df); relative tolerance on tails.
    let cases = [
      (1.0, 12.706204736174692, 0.07534419205759384),
      (3.0, 3.1824463052837078, 0.003527018840424672),
      (
        172.3775481377,
        1.9738215403315624,
        1.499_950_917_096_732e-14,
      ),
      (33662.0010164, 1.9600344602993407, 4.259_518_459_029_911e-17),
      (1_000_000.0, 1.9599663568141061, 4.1053821014395405e-17),
    ];
    let connection = registered();
    for (df, quantile, tail) in cases {
      let (q, p, neg): (f64, f64, f64) = connection
        .query_row(
          "SELECT tabdat_t_quantile(.975, ?), tabdat_t_pvalue(8.41, ?), tabdat_t_pvalue(-8.41, ?)",
          [df, df, df],
          |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
      assert!((q - quantile).abs() <= 1e-9, "df={df}: q {q} vs {quantile}");
      assert!((p - tail).abs() <= 1e-8 * tail, "df={df}: p {p} vs {tail}");
      assert_eq!(p, neg);
    }
    let (q, tail, zero): (f64, f64, f64) = connection.query_row(
      "SELECT tabdat_t_quantile(.025, 1.0), tabdat_t_pvalue(1000000.0,1.0), tabdat_t_pvalue(1000000.0,100.0)",
      [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).unwrap();
    assert!((q - (-12.706204736174703)).abs() <= 1e-9);
    assert!((tail - 6.366197723673691e-7).abs() <= 1e-8 * 6.366197723673691e-7);
    assert_eq!(zero, 0.0);
  }

  #[test]
  fn invalid_and_null_inputs_are_null_and_chunks_preserve_rows() {
    let connection = registered();
    let count: i64 = connection.query_row(
      "SELECT count(*) FROM range(0, 6001) t(i) WHERE tabdat_t_pvalue(i::DOUBLE, 3::DOUBLE) IS NOT NULL AND tabdat_t_quantile(.975, 3::DOUBLE) IS NOT NULL", [], |r|r.get(0)
    ).unwrap();
    assert_eq!(count, 6001);
    let all_null: bool = connection.query_row(
      "SELECT tabdat_t_pvalue(NULL::DOUBLE, 3.0) IS NULL AND tabdat_t_pvalue(1.0, NULL::DOUBLE) IS NULL AND tabdat_t_pvalue('NaN'::DOUBLE,3.0) IS NULL AND tabdat_t_pvalue(1.0,0.999) IS NULL AND tabdat_t_pvalue(1000001.0,3.0) IS NULL AND tabdat_t_pvalue(1.0,1000001.0) IS NULL AND tabdat_t_quantile(.024,3.0) IS NULL AND tabdat_t_quantile(.976,3.0) IS NULL AND tabdat_t_quantile(.975,'Infinity'::DOUBLE) IS NULL",[],|r|r.get(0)
    ).unwrap();
    assert!(all_null);
  }
}
