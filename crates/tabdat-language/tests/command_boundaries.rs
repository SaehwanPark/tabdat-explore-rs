use tabdat_language::parse_command;

#[test]
fn attached_colons_keep_command_specific_diagnostics() {
  // These commands reject an attached colon before tokenizing their arguments.
  // Prefix commands and gsort deliberately follow different parsing paths.
  let names = [
    "use",
    "rename",
    "sort",
    "select",
    "run",
    "sql",
    "regress",
    "logit",
    "probit",
    "poisson",
    "nbreg",
    "zip",
    "zinb",
    "qreg",
    "tobit",
    "heckman",
    "nl",
    "streg",
    "spregress",
    "lasso",
    "postlasso",
    "ridge",
    "elasticnet",
    "cvlasso",
    "cvridge",
    "cvelasticnet",
    "predict",
    "xtlogit",
    "lowess",
    "did",
    "drdid",
    "dml",
    "cfregress",
    "lincom",
    "test",
    "histogram",
    "scatter",
    "bar",
    "bayesplot",
  ];
  for name in names {
    for spelling in [name.to_owned(), name.to_ascii_uppercase()] {
      for suffix in [":", ":x", ": 'unterminated", ":λ"] {
        let input = format!("{spelling}{suffix}");
        assert_eq!(
          parse_command(&input).unwrap_err().message(),
          "unsupported token in command: :",
          "{input}"
        );
      }
    }
  }
  assert_eq!(
    parse_command("by: count").unwrap_err().message(),
    "unknown command: by"
  );
  assert!(parse_command("gsort:age").is_ok());
  assert!(parse_command("bayes: regress y x").is_ok());
  assert_eq!(
    parse_command("regression:x").unwrap_err().message(),
    "unknown command: regression:x"
  );
  assert_eq!(
    parse_command("λ:x").unwrap_err().message(),
    "unknown command: λ:x"
  );
}

#[test]
fn attached_assignments_keep_command_specific_diagnostics() {
  let names = [
    "use",
    "run",
    "sql",
    "regress",
    "logit",
    "probit",
    "bayes",
    "poisson",
    "nbreg",
    "zip",
    "zinb",
    "qreg",
    "tobit",
    "heckman",
    "nl",
    "streg",
    "spregress",
    "lasso",
    "postlasso",
    "ridge",
    "elasticnet",
    "cvlasso",
    "cvridge",
    "cvelasticnet",
    "predict",
    "xtlogit",
    "lowess",
    "did",
    "drdid",
    "dml",
    "cfregress",
    "lincom",
    "test",
    "histogram",
    "scatter",
    "bar",
    "bayesplot",
  ];
  for name in names {
    for spelling in [name.to_owned(), name.to_ascii_uppercase()] {
      for suffix in ["=", "=x", "= 'unterminated"] {
        let input = format!("{spelling}{suffix}");
        assert_eq!(
          parse_command(&input).unwrap_err().message(),
          format!("{name} assignment requires a target before ="),
          "{input}"
        );
      }
      let input = format!("{spelling}==x");
      assert_eq!(
        parse_command(&input).unwrap_err().message(),
        "unsupported token in command: ==",
        "{input}"
      );
    }
  }
}

#[test]
fn attached_commas_preserve_empty_option_precedence() {
  for name in ["run", "lincom", "test"] {
    for spelling in [name.to_owned(), name.to_ascii_uppercase()] {
      for suffix in [",", ", ", ",\u{1c}"] {
        let input = format!("{spelling}{suffix}");
        assert_eq!(
          parse_command(&input).unwrap_err().message(),
          "comma must be followed by at least one option",
          "{input}"
        );
      }
      let input = format!("{spelling}, invalid");
      assert_eq!(
        parse_command(&input).unwrap_err().message(),
        format!("unknown command: {name}"),
        "{input}"
      );
    }
  }
  assert_eq!(
    parse_command("sql,").unwrap_err().message(),
    "unknown command: sql"
  );
  assert_eq!(
    parse_command("help=x").unwrap_err().message(),
    "unknown command: help"
  );
}
