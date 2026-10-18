use tabdat_language::{Command, parse_command};

#[test]
fn old_scatter_wire_defaults_stay_unchanged_and_new_options_roundtrip() {
  let old = parse_command("scatter y x").unwrap();
  let serialized = serde_json::to_string(&old).unwrap();
  assert!(!serialized.contains("line_variables"));
  assert!(!serialized.contains("jitter_y"));
  assert!(!serialized.contains("title"));
  assert_eq!(serde_json::from_str::<Command>(&serialized).unwrap(), old);
  let new = parse_command("scatter y x, lines(p) jitter(0.03) title(Model)").unwrap();
  assert_eq!(
    serde_json::from_str::<Command>(&serde_json::to_string(&new).unwrap()).unwrap(),
    new
  );
}

#[test]
fn parses_bounded_overlays_title_and_display_jitter() {
  let Command::Scatter { command } = parse_command(
    "scatter charter lnrelp, lines(logit probit `OLS fit`) jitter(0.03) title(\"Predicted Probabilities Across Models\") saving(\"Figure 14.1.svg\") noopen",
  ).unwrap() else { panic!("not scatter") };
  assert_eq!(command.y_variable, "charter");
  assert_eq!(command.x_variable, "lnrelp");
  assert_eq!(command.line_variables, ["logit", "probit", "OLS fit"]);
  assert_eq!(command.jitter_y.as_deref(), Some("0.03"));
  assert_eq!(
    command.title.as_deref(),
    Some("Predicted Probabilities Across Models")
  );
  assert_eq!(command.saving.as_deref(), Some("Figure 14.1.svg"));
  assert!(!command.open_artifact);
  assert!(parse_command("scatter y x, lines(p) jitter(0) title(Model) noopen").is_ok());
}

#[test]
fn rejects_invalid_duplicate_and_unbounded_options() {
  for (options, message) in [
    ("lines", "one to three unique"),
    ("lines(a b c d)", "one to three unique"),
    ("lines(a a)", "one to three unique"),
    ("lines(a) lines(b)", "only be supplied once"),
    ("jitter(-0.1)", "finite nonnegative"),
    ("jitter(NaN)", "finite nonnegative"),
    ("jitter(1e999)", "finite nonnegative"),
    ("jitter(0.1) jitter(0.2)", "only be supplied once"),
    ("title", "nonempty text"),
    ("title(\"\")", "nonempty text"),
    ("title(a b)", "single text value"),
    ("title(a) title(b)", "only be supplied once"),
    ("replace", "unsupported option"),
  ] {
    let error = parse_command(&format!("scatter y x, {options}")).unwrap_err();
    assert!(error.to_string().contains(message), "{options}: {error}");
  }
}
