mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::fs;

#[test]
fn xtdes_reports_stable_json_and_readable_text_from_a_script() {
  let dir = TestDir::new();
  fs::write(
    dir.path().join("panel.csv"),
    "firm,year\nB,2000\nA,2004\nC,2010\nA,2000\nA,2000\nB,2010\n",
  )
  .unwrap();
  fs::write(
    dir.path().join("inspect.td"),
    "use panel.csv\nxtdes, i(firm) t(year) patterns(2)\n",
  )
  .unwrap();

  let json_output = tabdat_bin()
    .current_dir(dir.path())
    .args(["--json", "-f", "inspect.td"])
    .output()
    .unwrap();
  assert!(
    json_output.status.success(),
    "{}\n{}",
    String::from_utf8_lossy(&json_output.stdout),
    String::from_utf8_lossy(&json_output.stderr)
  );
  let events = String::from_utf8(json_output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str::<Value>(line).unwrap())
    .collect::<Vec<_>>();
  let result = events
    .iter()
    .find(|event| event["result_type"] == "XtDesResult")
    .unwrap();
  assert_eq!(result["data"]["observation_count"], 6);
  assert_eq!(result["data"]["duplicate_key_pairs"], 1);
  assert_eq!(result["data"]["omitted_pattern_count"], 1);
  assert_eq!(
    result["data"]["patterns"][0]["counts_by_period"],
    serde_json::json!([0, 0, 1])
  );

  let text_output = tabdat_bin()
    .current_dir(dir.path())
    .args(["--text", "-f", "inspect.td"])
    .output()
    .unwrap();
  assert!(
    text_output.status.success(),
    "{}\n{}",
    String::from_utf8_lossy(&text_output.stdout),
    String::from_utf8_lossy(&text_output.stderr)
  );
  let text = String::from_utf8(text_output.stdout).unwrap();
  assert!(text.contains("Panel identifier: firm"));
  assert!(text.contains("Duplicate panel-time pairs=1"));
  assert!(text.contains("Participation patterns: 3 distinct; returned 2; omitted 1"));
  assert!(text.contains("[0,0,1]"));
}
