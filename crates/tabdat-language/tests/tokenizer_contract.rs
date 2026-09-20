use tabdat_language::{Token, TokenKind, tokenize};

fn compact(tokens: Vec<Token>) -> Vec<(TokenKind, String, usize, usize)> {
  tokens
    .into_iter()
    .map(|token| (token.kind, token.text, token.start, token.end))
    .collect()
}

#[test]
fn tokenizes_identifiers_numbers_strings_and_operators_with_offsets() {
  assert_eq!(
    compact(tokenize("  wage exposure, robust lags(2) >= .5").unwrap()),
    vec![
      (
        TokenKind::Identifier { quoted: false },
        "wage".to_owned(),
        2,
        6,
      ),
      (
        TokenKind::Identifier { quoted: false },
        "exposure".to_owned(),
        7,
        15,
      ),
      (TokenKind::Symbol, ",".to_owned(), 15, 16),
      (
        TokenKind::Identifier { quoted: false },
        "robust".to_owned(),
        17,
        23,
      ),
      (
        TokenKind::Identifier { quoted: false },
        "lags".to_owned(),
        24,
        28,
      ),
      (TokenKind::Symbol, "(".to_owned(), 28, 29),
      (TokenKind::Number, "2".to_owned(), 29, 30),
      (TokenKind::Symbol, ")".to_owned(), 30, 31),
      (TokenKind::Symbol, ">=".to_owned(), 32, 34),
      (TokenKind::Number, ".5".to_owned(), 35, 37),
    ]
  );
}

#[test]
fn decodes_quoted_tokens_and_unicode_character_offsets() {
  assert_eq!(
    compact(tokenize("\u{1c}\u{1d}α_1 ١٢ `a``b` \"hello world\" ''").unwrap()),
    vec![
      (
        TokenKind::Identifier { quoted: false },
        "α_1".to_owned(),
        2,
        5,
      ),
      (TokenKind::Number, "١٢".to_owned(), 6, 8),
      (
        TokenKind::Identifier { quoted: true },
        "a`b".to_owned(),
        9,
        15,
      ),
      (TokenKind::String, "hello world".to_owned(), 17, 29,),
      (TokenKind::String, String::new(), 31, 32),
    ]
  );
}

#[test]
fn supports_empty_input_and_the_complete_bounded_symbol_set() {
  assert!(tokenize(" \u{1c}\u{1f}\n\t").unwrap().is_empty());
  assert_eq!(
    compact(tokenize(", = < > + - * / ( ) : . == != <= >=").unwrap()),
    vec![
      (TokenKind::Symbol, ",".to_owned(), 0, 1),
      (TokenKind::Symbol, "=".to_owned(), 2, 3),
      (TokenKind::Symbol, "<".to_owned(), 4, 5),
      (TokenKind::Symbol, ">".to_owned(), 6, 7),
      (TokenKind::Symbol, "+".to_owned(), 8, 9),
      (TokenKind::Symbol, "-".to_owned(), 10, 11),
      (TokenKind::Symbol, "*".to_owned(), 12, 13),
      (TokenKind::Symbol, "/".to_owned(), 14, 15),
      (TokenKind::Symbol, "(".to_owned(), 16, 17),
      (TokenKind::Symbol, ")".to_owned(), 18, 19),
      (TokenKind::Symbol, ":".to_owned(), 20, 21),
      (TokenKind::Symbol, ".".to_owned(), 22, 23),
      (TokenKind::Symbol, "==".to_owned(), 24, 26),
      (TokenKind::Symbol, "!=".to_owned(), 27, 29),
      (TokenKind::Symbol, "<=".to_owned(), 30, 32),
      (TokenKind::Symbol, ">=".to_owned(), 33, 35),
    ]
  );
}

#[test]
fn preserves_recovered_tokenizer_diagnostics() {
  let cases = [
    ("1..2", "malformed number: 1..2"),
    ("`", "unterminated quoted identifier"),
    ("``", "quoted identifier cannot be empty"),
    ("'unterminated", "unterminated quoted string"),
    (";", "unsupported token in command: ;"),
    ("@", "unsupported token in command: @"),
  ];

  for (input, expected) in cases {
    assert_eq!(
      tokenize(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}
