use std::{env, error::Error, fs, time::Instant};

use tabdat_duckdb_spike::{ActiveRelation, write_fixture};

fn main() -> Result<(), Box<dyn Error>> {
  let root = env::temp_dir().join(format!("tabdat-duckdb-measure-{}", std::process::id()));
  fs::create_dir_all(&root)?;
  let csv = root.join("input.csv");
  write_fixture(&csv)?;

  let start = Instant::now();
  let relation = ActiveRelation::from_csv(&csv)?;
  let cold_open = start.elapsed();

  let start = Instant::now();
  let rows = relation.row_count()?;
  let first_query = start.elapsed();

  let start = Instant::now();
  for _ in 0..100 {
    relation.row_count()?;
  }
  let repeated_query = start.elapsed();

  println!("duckdb_version={}", relation.version()?);
  println!("rows={rows}");
  println!("cold_open_us={}", cold_open.as_micros());
  println!("first_query_us={}", first_query.as_micros());
  println!("repeated_100_queries_us={}", repeated_query.as_micros());

  drop(relation);
  fs::remove_dir_all(root)?;
  Ok(())
}
