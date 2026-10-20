mod common;
use common::TestDir;
use std::{fs, path::PathBuf, process::Command};

fn example(name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma21p1panel")
    .join(name)
}
fn convert(dir: &TestDir, text: &str) -> std::process::Output {
  fs::write(dir.path().join("raw input.dat"), text).unwrap();
  Command::new("sh")
    .arg(example("prepare.sh"))
    .arg(dir.path().join("raw input.dat"))
    .arg(dir.path().join("MOM.csv"))
    .output()
    .unwrap()
}
#[test]
fn converter_preserves_token_records_and_refuses_invalid_keys_and_partial_publication() {
  let d = TestDir::new();
  let out = convert(
    &d,
    "7.580 1.91 2 27\r\n729 0 1.00 1979.00\r\n7.750 1.89 2 28 784 0 1.00 1980.00",
  );
  assert!(out.status.success(), "{out:?}");
  let expected = "source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year\n1,7.580,1.91,2,27,729,0,1.00,1979.00\n2,7.750,1.89,2,28,784,0,1.00,1980.00\n";
  assert_eq!(
    fs::read_to_string(d.path().join("MOM.csv")).unwrap(),
    expected
  );
  let out = convert(&d, "bad");
  assert!(!out.status.success());
  assert_eq!(
    fs::read_to_string(d.path().join("MOM.csv")).unwrap(),
    expected
  );
  for input in [
    "",
    "7.58 1.91 2 27 729 0 1",
    "bad 1.91 2 27 729 0 1 1979",
    "1e999 1.91 2 27 729 0 1 1979",
    "NaN 1.91 2 27 729 0 1 1979",
    "7.58 1.91 2 27 729 0 1.00000000000000001 1979",
    "7.58 1.91 2 27 729 0 533 1979",
    "7.58 1.91 2 27 729 0 1 1979.1",
    "7.58 1.91 2 27 729 0 1 1989",
  ] {
    let d = TestDir::new();
    let out = convert(&d, input);
    assert!(!out.status.success(), "{input}: {out:?}");
    assert!(!d.path().join("MOM.csv").exists());
    assert!(
      !fs::read_dir(d.path()).unwrap().any(|p| p
        .unwrap()
        .file_name()
        .to_string_lossy()
        .contains(".tmp."))
    );
  }
}
