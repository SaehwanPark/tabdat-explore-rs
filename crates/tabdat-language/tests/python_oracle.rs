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
    // Keep the pinned corpus unchanged while checking the intentional local
    // extensions and clearer IV syntax diagnostics below.
    if input == "count if age > 18" {
      assert!(matches!(
        result,
        Ok(tabdat_domain::Command::CountQuery { .. })
      ));
      continue;
    }
    // Selected-cohort linear fitting intentionally enables this old rejection too.
    if input == "regress cost age if age > 18" {
      assert!(matches!(
        result,
        Ok(tabdat_domain::Command::RegressQuery { .. })
      ));
      continue;
    }
    // IV estimation needs to accept multiple endogenous regressors; the pinned
    // Python parser rejects this additive, ordered variable-list form.
    if input == "ivregress 2sls y x, endog(z w) iv(q)" {
      let Ok(tabdat_domain::Command::IvRegress { command }) = result else {
        panic!("multi-endogenous ivregress syntax should be accepted");
      };
      assert_eq!(command.endogenous, ["z", "w"]);
      continue;
    }
    if input == "ivregress" {
      assert_eq!(
        result.unwrap_err().message(),
        "ivregress expects syntax: ivregress 2sls|gmm <y> [exog_vars], endog(<vars>) iv(<vars>)"
      );
      continue;
    }
    if input == "ivregress 2sls y x" || input == "ivregress 2sls y x, iv(w)" {
      assert_eq!(
        result.unwrap_err().message(),
        "ivregress option endog expects at least one variable"
      );
      continue;
    }
    // MIG-0013: the installed AR extension adds only `ar` to these diagnostic
    // lists. Preserve the historical corpus and assert the exact new wording.
    if matches!(
      input,
      "estat"
        | "estat bayes detail"
        | "estat detail"
        | "estat endog"
        | "estat first"
        | "estat margin"
        | "estat vif extra"
        | "estat, vif"
    ) {
      let expected = case["error"]
        .as_str()
        .unwrap()
        .replace("vif|firststage", "vif|ar|firststage")
        .replace("vif, firststage", "vif, ar, firststage");
      assert_eq!(result.unwrap_err().message(), expected);
      continue;
    }
    if input == "count age" {
      assert_eq!(
        result.unwrap_err().message(),
        "count does not accept arguments, options, or assignment syntax"
      );
      continue;
    }
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
