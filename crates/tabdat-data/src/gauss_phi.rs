//! Deterministic Gaussian probability UDF for SQL-side binary predictions.
//!
//! DuckDB ships its own built-in normal CDF, but the capability contract for
//! binary results requires probabilities to come from the same validated
//! function the estimators use — `tabdat_stats::normal_cdf`, checked against
//! R to machine precision, mapping extreme tails exactly onto 0/1, and
//! keeping `p + q` exactly one through complementary half-tails. The
//! `tabdat_gauss_phi(double) -> double` scalar function delegates every call
//! to that function, so a `predict` probability column agrees bit-for-bit
//! with the fitted probabilities under the same linear index.
//!
//! The arrow-scalar path is chosen over the data-chunk path because pointer
//! access to DuckDB vectors needs `unsafe`, while arrow's typed arrays are
//! safe; `duckdb`'s re-exported arrow keeps the version aligned.

use std::error::Error;
use std::sync::Arc;

use duckdb::arrow::array::{Array, Float64Array, RecordBatch};
use duckdb::arrow::datatypes::DataType;
use duckdb::vscalar::arrow::{ArrowFunctionSignature, VArrowScalar};

/// The standard normal CDF exposed to DuckDB as `tabdat_gauss_phi`.
pub struct GaussPhi;

impl VArrowScalar for GaussPhi {
  type State = ();

  /// Per chunk: read the double-precision values (validity rides along;
  /// null maps to null) and emit `normal_cdf` for the rest.
  fn invoke(_: &Self::State, input: RecordBatch) -> Result<Arc<dyn Array>, Box<dyn Error>> {
    let Some(values) = input.column(0).as_any().downcast_ref::<Float64Array>() else {
      return Err("tabdat_gauss_phi requires a double precision argument".into());
    };
    let phi = values
      .iter()
      .map(|value| value.map(tabdat_stats::normal_cdf));
    Ok(Arc::new(Float64Array::from_iter(phi)) as Arc<dyn Array>)
  }

  fn signatures() -> Vec<ArrowFunctionSignature> {
    vec![ArrowFunctionSignature::exact(
      vec![DataType::Float64],
      DataType::Float64,
    )]
  }
}

#[cfg(test)]
mod tests {
  use duckdb::Connection;

  use super::GaussPhi;

  fn registered() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    connection
      .register_scalar_function::<GaussPhi>("tabdat_gauss_phi")
      .unwrap();
    connection
  }

  #[test]
  fn delegates_bit_for_bit_to_the_validated_engine_function() {
    let connection = registered();
    // 6001 rows cross several default-sized chunks, forcing multi-chunk
    // invocation; every value must match the engine function exactly, and
    // complementary tails must sum to exactly one (the probit convention).
    let mut statement = connection
      .prepare(
        "SELECT tabdat_gauss_phi(x), tabdat_gauss_phi(-x)
         FROM range(-3000, 3001) t(x)",
      )
      .unwrap();
    let mut index = -3000_i64;
    statement
      .query_map([], |row| {
        let p: f64 = row.get(0)?;
        let q: f64 = row.get(1)?;
        let x = index as f64;
        index += 1;
        assert_eq!(p, tabdat_stats::normal_cdf(x));
        assert_eq!(p + q, 1.0);
        Ok(())
      })
      .unwrap()
      .for_each(|row| row.unwrap());
    assert_eq!(index, 3001);
  }

  #[test]
  fn saturation_null_and_integer_coercion_follow_the_contracts() {
    let connection = registered();
    let mut statement = connection
      .prepare(
        "SELECT tabdat_gauss_phi(1e6), tabdat_gauss_phi(-1e6),
              tabdat_gauss_phi(NULL::DOUBLE) IS NULL,
              tabdat_gauss_phi(2::INTEGER) = tabdat_gauss_phi(2.0::DOUBLE)",
      )
      .unwrap();
    let row = statement
      .query_row([], |row| {
        Ok((
          row.get::<_, f64>(0)?,
          row.get::<_, f64>(1)?,
          row.get::<_, bool>(2)?,
          row.get::<_, bool>(3)?,
        ))
      })
      .unwrap();
    assert_eq!(row.0, 1.0);
    assert_eq!(row.1, 0.0);
    assert!(row.2);
    assert!(row.3);
  }
}
