//! Deterministic generated properties: no runtime/backend or random dependency.
use std::path::Path;
use tabdat_language::{Command, parse_command, parse_script, tokenize};

#[test]
fn generated_unicode_inputs_are_deterministic_and_tokens_stay_in_bounds() {
  let alphabet = [
    'a', 'é', '界', '😀', '_', '0', '9', '.', '-', '+', '*', '/', '=', '(', ')', ',', ':', '`',
    '\'', '"', '$', ' ', '\t', '\n', '\u{1c}',
  ];
  let mut state = 0x5eed_u64;
  for length in 0..128 {
    for _ in 0..32 {
      let input: String = (0..length)
        .map(|_| {
          state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
          alphabet[(state >> 32) as usize % alphabet.len()]
        })
        .collect();
      assert_eq!(parse_command(&input), parse_command(&input), "{input:?}");
      assert_eq!(
        parse_script(&input, Path::new("property.td")),
        parse_script(&input, Path::new("property.td"))
      );
      let tokens = tokenize(&input);
      assert_eq!(tokens, tokenize(&input));
      if let Ok(tokens) = tokens {
        let length = input.chars().count();
        let mut end = 0;
        for token in tokens {
          assert!(
            end <= token.start && token.start <= token.end && token.end <= length,
            "{input:?}: {token:?}"
          );
          end = token.end;
        }
      }
    }
  }
}

#[test]
fn recursive_and_deep_ast_inputs_fail_with_resource_diagnostics() {
  for expression in [
    format!("{}1{}", "(".repeat(20_000), ")".repeat(20_000)),
    format!("{}1", "-".repeat(20_000)),
    format!("{}1{}", "f(".repeat(10_000), ")".repeat(10_000)),
    std::iter::repeat_n("1", 1_000)
      .collect::<Vec<_>>()
      .join("+"),
  ] {
    assert_eq!(
      parse_command(&format!("generate x = {expression}"))
        .unwrap_err()
        .message(),
      "expression exceeds limit of 256 tokens"
    );
  }
  assert_eq!(
    parse_command(&format!("{}summarize x", "by x: ".repeat(100)))
      .unwrap_err()
      .message(),
    "command nesting exceeds limit of 32"
  );
  for prefix in ["assert ", "keep if ", "replace x = "] {
    assert_eq!(
      parse_command(&format!("{prefix}{}1", "-".repeat(256)))
        .unwrap_err()
        .message(),
      "expression exceeds limit of 256 tokens"
    );
    // Exactly 256 tokens remains accepted; equality and drop visit the AST.
    let input = format!("{prefix}{}1", "-".repeat(255));
    assert_eq!(
      parse_command(&input).unwrap(),
      parse_command(&input).unwrap()
    );
  }
  assert_eq!(
    parse_command(&format!("{}regress y x", "bayes: ".repeat(100)))
      .unwrap_err()
      .message(),
    "command nesting exceeds limit of 32"
  );
  assert_eq!(
    parse_command(&format!("{}count", "by x: ".repeat(31)))
      .unwrap_err()
      .message(),
    "nested by commands are not supported"
  );
  let accepted = std::iter::repeat_n("1", 128).collect::<Vec<_>>().join("+");
  assert!(parse_command(&format!("generate x = {accepted}")).is_ok());
}

#[test]
fn quoted_identifier_roundtrip_and_case_whitespace_invariance() {
  for name in [
    "x",
    "age years",
    "if",
    "界😀",
    "x`y",
    "a,b",
    "a(b)",
    "x/y",
    "é",
  ] {
    let quoted = format!("`{}`", name.replace('`', "``"));
    for command in ["keep", "KEEP", "KeEp"] {
      for whitespace in [" ", "\t", "\u{1c}", "\n"] {
        assert_eq!(
          parse_command(&format!(
            "{whitespace}{command}{whitespace}{quoted}{whitespace}"
          ))
          .unwrap(),
          Command::Keep {
            variables: vec![name.into()]
          }
        );
      }
    }
  }
}
