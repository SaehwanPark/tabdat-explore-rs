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
