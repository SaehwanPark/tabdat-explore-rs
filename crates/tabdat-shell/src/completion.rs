use rustyline::{
  Context, Helper, completion::Completer, highlight::Highlighter, hint::Hinter, validate::Validator,
};
use tabdat_reporting::help::available_help_topics;

const CORE_COMMANDS: &[&str] = &[
  "append",
  "assert",
  "by",
  "codebook",
  "collapse",
  "count",
  "datasignature",
  "decode",
  "describe",
  "drop",
  "duplicates",
  "encode",
  "estat",
  "exit",
  "export",
  "generate",
  "gsort",
  "head",
  "help",
  "isid",
  "join",
  "keep",
  "label",
  "lincom",
  "missing",
  "predict",
  "quit",
  "recode",
  "regress",
  "rename",
  "replace",
  "reshape",
  "run",
  "save",
  "select",
  "sort",
  "sql",
  "summarize",
  "tabulate",
  "tail",
  "test",
  "ttest",
  "use",
];

pub(crate) struct ShellHelper;

impl Completer for ShellHelper {
  type Candidate = String;

  fn complete(
    &self,
    line: &str,
    pos: usize,
    _: &Context<'_>,
  ) -> rustyline::Result<(usize, Vec<String>)> {
    let Some(before) = line.get(..pos) else {
      return Ok((pos, Vec::new()));
    };
    if line[pos..]
      .chars()
      .next()
      .is_some_and(|c| !c.is_whitespace())
    {
      return Ok((pos, Vec::new()));
    }
    let text = before.trim_start();
    let start = before.len() - text.len();
    let (start, prefix, candidates) = if let Some(topic) = text.strip_prefix('?') {
      let prefix = topic.trim_start();
      (pos - prefix.len(), prefix, available_help_topics())
    } else if let Some(end) = text.find(char::is_whitespace) {
      if !text[..end].eq_ignore_ascii_case("help") {
        return Ok((pos, Vec::new()));
      }
      let prefix = text[end..].trim_start();
      (pos - prefix.len(), prefix, available_help_topics())
    } else {
      (start, text, CORE_COMMANDS)
    };
    let prefix = prefix.to_ascii_lowercase();
    let matches = candidates
      .iter()
      .filter(|candidate| candidate.starts_with(&prefix))
      .map(|candidate| (*candidate).to_owned())
      .collect();
    Ok((start, matches))
  }
}

impl Hinter for ShellHelper {
  type Hint = String;
}
impl Highlighter for ShellHelper {}
impl Validator for ShellHelper {}
impl Helper for ShellHelper {}

#[cfg(test)]
mod tests {
  use super::*;
  use rustyline::history::DefaultHistory;

  fn complete(line: &str, pos: usize) -> (usize, Vec<String>) {
    ShellHelper
      .complete(line, pos, &Context::new(&DefaultHistory::new()))
      .unwrap()
  }

  #[test]
  fn core_prefixes_are_bounded_ordered_and_case_insensitive() {
    assert_eq!(complete("summ", 4), (0, vec!["summarize".into()]));
    assert_eq!(
      complete("RE", 2),
      (
        0,
        vec![
          "recode".into(),
          "regress".into(),
          "rename".into(),
          "replace".into(),
          "reshape".into()
        ]
      )
    );
    assert_eq!(complete("  cou", 5), (2, vec!["count".into()]));
    assert_eq!(complete("\u{3000}cou", 6), (3, vec!["count".into()]));
    assert!(complete("log", 3).1.is_empty());
    assert!(complete("set", 3).1.is_empty());
    assert!(complete("# summ", 6).1.is_empty());
    let (_, all) = complete("", 0);
    assert_eq!(all.len(), CORE_COMMANDS.len());
    assert!(all.windows(2).all(|pair| pair[0] < pair[1]));
    for command in all {
      assert!(tabdat_reporting::catalog::COMMAND_NAMES.contains(&command.as_str()));
    }
  }

  #[test]
  fn help_topics_reuse_the_packaged_reference_including_aliases() {
    for line in ["help summ", "HELP SuMm", "?summ", "? summ", "  ?summ"] {
      assert_eq!(
        complete(line, line.len()),
        (line.len() - 4, vec!["summarize".into()])
      );
    }
    for line in ["help ", "?", "? "] {
      let (start, topics) = complete(line, line.len());
      assert_eq!(start, line.len());
      assert_eq!(topics, available_help_topics());
    }
    assert!(complete("help log", 8).1.contains(&"logit".into()));
    assert_eq!(complete("help tt", 7), (5, vec!["ttest".into()]));
    assert_eq!(complete("?TT", 3), (1, vec!["ttest".into()]));
    assert!(complete("help summarize extra", 20).1.is_empty());
  }

  #[test]
  fn only_word_ends_complete_without_replacing_arguments_or_suffixes() {
    assert_eq!(complete("summ y", 4), (0, vec!["summarize".into()]));
    assert_eq!(complete("help summ  ", 9), (5, vec!["summarize".into()]));
    for line in [
      "use summ",
      "regress summ",
      "summarize ",
      "sql select summ",
      "run summ",
      "generate x = summ",
      "help summarize ",
    ] {
      assert!(complete(line, line.len()).1.is_empty(), "{line}");
    }
    assert!(complete("summarize", 4).1.is_empty());
    assert!(complete("help summarize", 9).1.is_empty());
    assert!(complete("summ", 10).1.is_empty());
    assert!(complete("é", 1).1.is_empty());
    assert!(complete("summé", 4).1.is_empty());
    assert!(complete("变量", 6).1.is_empty());
  }
}
