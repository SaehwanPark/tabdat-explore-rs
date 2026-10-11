//! Agent-owned HTTP fixture; never downloads a third-party dataset.
use super::*;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
  Arc,
  atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

struct Server {
  address: std::net::SocketAddr,
  stop: Arc<AtomicBool>,
  ranges: Arc<AtomicUsize>,
  worker: Option<JoinHandle<()>>,
}

impl Server {
  fn new(bytes: Vec<u8>) -> Self {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let ranges = Arc::new(AtomicUsize::new(0));
    let worker = {
      let stop = Arc::clone(&stop);
      let ranges = Arc::clone(&ranges);
      thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
          match listener.accept() {
            Ok((stream, _)) => serve(stream, &bytes, &ranges),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
              thread::sleep(Duration::from_millis(1))
            }
            Err(error) => panic!("HTTP fixture accept: {error}"),
          }
        }
      })
    };
    Self {
      address,
      stop,
      ranges,
      worker: Some(worker),
    }
  }

  fn url(&self, path: &str) -> String {
    format!("http://{}{path}", self.address)
  }
}

impl Drop for Server {
  fn drop(&mut self) {
    self.stop.store(true, Ordering::Relaxed);
    self.worker.take().unwrap().join().unwrap();
  }
}

fn serve(mut stream: TcpStream, bytes: &[u8], ranges: &AtomicUsize) {
  stream
    .set_read_timeout(Some(Duration::from_secs(5)))
    .unwrap();
  stream
    .set_write_timeout(Some(Duration::from_secs(5)))
    .unwrap();
  let mut reader = BufReader::new(&stream);
  let mut first = String::new();
  if reader.read_line(&mut first).is_err() {
    return;
  }
  let mut range = None;
  loop {
    let mut line = String::new();
    if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
      break;
    }
    if let Some(value) = line.to_ascii_lowercase().strip_prefix("range: bytes=") {
      let (start, end) = value.trim().split_once('-').unwrap();
      let start: usize = start.parse().unwrap();
      let end: usize = if end.is_empty() {
        bytes.len() - 1
      } else {
        end.parse().unwrap()
      };
      range = Some((start, end));
    }
  }
  drop(reader);
  if !first.contains(" /fixture.parquet ") && !first.contains(" /bucket/fixture.parquet ") {
    let _ = write!(
      stream,
      "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    return;
  }
  let (status, body, content_range) = if let Some((start, end)) = range {
    if start > end || end >= bytes.len() {
      let _ = write!(
        stream,
        "HTTP/1.1 416 Range Not Satisfiable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
      );
      return;
    }
    ranges.fetch_add(1, Ordering::Relaxed);
    (
      "206 Partial Content",
      &bytes[start..=end],
      format!("Content-Range: bytes {start}-{end}/{}\r\n", bytes.len()),
    )
  } else {
    ("200 OK", bytes, String::new())
  };
  let _ = write!(
    stream,
    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\n{content_range}Connection: close\r\n\r\n",
    body.len()
  );
  if !first.starts_with("HEAD ") {
    let _ = stream.write_all(body);
  }
}

#[test]
#[ignore = "opt-in: installs signed httpfs extension, possibly using network"]
fn http_parquet_values_ranges_arrow_and_missing_object() -> Result<()> {
  let path = std::env::temp_dir().join(format!("tabdat-http-{}.parquet", std::process::id()));
  let connection = Connection::open_in_memory()?;
  connection.execute_batch(&format!(
    "COPY (SELECT i AS id, CASE WHEN i = 2 THEN NULL ELSE 'row-' || i END AS label FROM range(1, 4097) t(i)) TO {} (FORMAT PARQUET)",
    sql_string_literal(&path)?
  ))?;
  let bytes = fs::read(&path).unwrap();
  fs::remove_file(path).unwrap();
  let server = Server::new(bytes);
  let mut relation = ActiveRelation::from_http_parquet(&server.url("/fixture.parquet"))?;
  assert_eq!(relation.row_count()?, 4096);
  assert_eq!(relation.row_count()?, 4096);
  let values: (i64, Option<String>, String) = relation.connection.query_row(
    "SELECT id, label, (SELECT label FROM active WHERE id=4096) FROM active WHERE id=2",
    [],
    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
  )?;
  assert_eq!(values, (2, None, "row-4096".into()));
  assert_eq!(
    relation.query_arrow("SELECT * FROM active ORDER BY id")?,
    4096
  );
  assert!(
    server.ranges.load(Ordering::Relaxed) > 0,
    "must exercise HTTP byte ranges"
  );
  let missing = server.url("/missing.parquet").replace('\'', "''");
  assert!(
    relation
      .connection
      .execute_batch(&format!("SELECT * FROM read_parquet('{missing}')"))
      .is_err()
  );
  assert_eq!(
    relation.row_count()?,
    4096,
    "connection/relation survive failed HTTP query"
  );
  // Exercise the S3 protocol route without credentials or a third-party bucket.
  // This checks unsigned path-style reads, not signing/IAM or AWS deployment.
  let s3 = Connection::open_in_memory()?;
  s3.execute_batch(&format!(
    "LOAD httpfs; SET s3_endpoint='{}'; SET s3_url_style='path'; SET s3_use_ssl=false; SET s3_region='us-east-1';",
    server.address
  ))?;
  let s3_values: (i64, i64, i64) = s3.query_row(
    "SELECT COUNT(*), SUM(id), COUNT(label) FROM read_parquet('s3://bucket/fixture.parquet')",
    [],
    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
  )?;
  assert_eq!(s3_values, (4096, 4096 * 4097 / 2, 4095));
  println!(
    "duckdb={} rows=4096 ranges={} missing_object=recovered s3_unsigned=passed",
    relation.version()?,
    server.ranges.load(Ordering::Relaxed)
  );
  Ok(())
}
