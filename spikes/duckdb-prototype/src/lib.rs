#![forbid(unsafe_code)]

use std::{fs, path::Path};

use duckdb::{Connection, Result};

/// The smallest relation owner needed to test the proposed DuckDB boundary.
///
/// This type deliberately stays inside the feasibility spike. A production
/// session must translate this ownership model into domain-owned state after
/// the relation lifecycle and threading contract are accepted.
pub struct ActiveRelation {
  connection: Connection,
}

impl ActiveRelation {
  pub fn from_csv(path: &Path) -> Result<Self> {
    let connection = Connection::open_in_memory()?;
    let path = sql_string_literal(path)?;
    connection.execute_batch(&format!(
      "CREATE VIEW active AS SELECT * FROM read_csv_auto({path}, header = true)"
    ))?;

    Ok(Self { connection })
  }

  pub fn from_parquet(path: &Path) -> Result<Self> {
    let connection = Connection::open_in_memory()?;
    let path = sql_string_literal(path)?;
    connection.execute_batch(&format!(
      "CREATE VIEW active AS SELECT * FROM read_parquet({path})"
    ))?;

    Ok(Self { connection })
  }

  pub fn row_count(&self) -> Result<i64> {
    self
      .connection
      .query_row("SELECT COUNT(*) FROM active", [], |row| row.get(0))
  }

  pub fn version(&self) -> Result<String> {
    self
      .connection
      .query_row("SELECT version()", [], |row| row.get(0))
  }

  pub fn query_arrow(&mut self, query: &str) -> Result<usize> {
    let mut statement = self.connection.prepare(query)?;
    Ok(
      statement
        .query_arrow([])?
        .map(|batch| batch.num_rows())
        .sum(),
    )
  }
}

fn sql_string_literal(path: &Path) -> Result<String> {
  let path = path
    .to_str()
    .ok_or_else(|| duckdb::Error::InvalidPath(path.to_path_buf()))?;
  Ok(format!("'{}'", path.replace('\'', "''")))
}

pub fn write_fixture(path: &Path) -> std::io::Result<()> {
  fs::write(path, "id,label\n1,alpha\n2,beta\n")
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{env, time::SystemTime};

  struct Fixture {
    csv: std::path::PathBuf,
    parquet: std::path::PathBuf,
  }

  impl Fixture {
    fn new() -> Self {
      let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
      let root = env::temp_dir().join(format!("tabdat-duckdb-spike-{nonce}"));
      fs::create_dir(&root).expect("fixture directory should be created");
      Self {
        csv: root.join("input.csv"),
        parquet: root.join("input.parquet"),
      }
    }
  }

  impl Drop for Fixture {
    fn drop(&mut self) {
      if let Some(root) = self.csv.parent() {
        let _ = fs::remove_dir_all(root);
      }
    }
  }

  #[test]
  fn csv_relation_supports_repeated_count_and_arrow_results() -> Result<()> {
    let fixture = Fixture::new();
    write_fixture(&fixture.csv).expect("CSV fixture should be written");

    let mut relation = ActiveRelation::from_csv(&fixture.csv)?;
    assert_eq!(relation.row_count()?, 2);
    assert_eq!(relation.row_count()?, 2);
    assert_eq!(
      relation.query_arrow("SELECT id, label FROM active ORDER BY id")?,
      2
    );
    Ok(())
  }

  #[test]
  fn parquet_relation_supports_scan_and_arrow_results() -> Result<()> {
    let fixture = Fixture::new();
    write_fixture(&fixture.csv).expect("CSV fixture should be written");

    let csv_relation = ActiveRelation::from_csv(&fixture.csv)?;
    csv_relation.connection.execute(
      &format!(
        "COPY active TO {} (FORMAT PARQUET)",
        sql_string_literal(&fixture.parquet)?
      ),
      [],
    )?;
    drop(csv_relation);

    let mut relation = ActiveRelation::from_parquet(&fixture.parquet)?;
    assert_eq!(relation.row_count()?, 2);
    assert_eq!(
      relation.query_arrow("SELECT label FROM active ORDER BY id")?,
      2
    );
    Ok(())
  }
}
