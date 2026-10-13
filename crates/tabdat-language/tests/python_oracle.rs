//! Replay actual calls captured from the verified Python parser tests.
use serde_json::Value;
use tabdat_language::parse_command;

#[test]
fn pinned_python_parser_acceptance_variants_and_diagnostics() {
  let corpus: Value = serde_json::from_str(include_str!("fixtures/python-parser.json")).unwrap();
  assert_eq!(
    corpus["revision"],
    "16b45d9b66b0d80f32d4d220e84d81bc5180bdbe"
  );
  let cases = corpus["cases"].as_array().unwrap();
  assert_eq!(cases.len(), 672);
  let mut failures = Vec::new();
  for case in cases {
    let input = case["input"].as_str().unwrap();
    let result = parse_command(input);
    let mismatch = if let Some(expected) = case["error"].as_str() {
      match result {
        Err(error) if error.message() == expected => None,
        other => Some(format!("expected error {expected:?}, got {other:?}")),
      }
    } else {
      let expected = case["command"]["type"]
        .as_str()
        .unwrap()
        .trim_end_matches("Command")
        .to_lowercase();
      match result {
        Ok(command) => {
          // Rust uses specific typed query/filter variants where Python uses a
          // generic ParsedCommand or a condition field on column operations.
          let name = command.name();
          let value = serde_json::to_value(command).unwrap();
          let mut kind = value["kind"].as_str().unwrap().replace('_', "");
          if kind == "summarizequery" && expected == "parsed" {
            kind = "parsed".into();
          } else if kind == "rowfilter" || kind == "tabulatequery" {
            kind = name.into();
          }
          (kind != expected).then(|| format!("expected variant {expected}, got {kind}"))
        }
        Err(error) => Some(format!("expected {expected}, got error {error}")),
      }
    };
    if let Some(mismatch) = mismatch {
      failures.push(format!("{input:?}: {mismatch}"));
    }
  }
  // Known differences remain visible debt, not blessed oracle outputs. Removing
  // one requires its own grammar regression and updating this explicit ledger.
  let gaps: Value = serde_json::from_str(include_str!("fixtures/python-parser-gaps.json")).unwrap();
  let expected: Vec<String> = gaps
    .as_array()
    .unwrap()
    .iter()
    .map(|gap| {
      format!(
        "{:?}: {}",
        gap["input"].as_str().unwrap(),
        gap["difference"].as_str().unwrap()
      )
    })
    .collect();
  assert_eq!(
    failures, expected,
    "parser differences changed; reconcile the ledger"
  );
}
