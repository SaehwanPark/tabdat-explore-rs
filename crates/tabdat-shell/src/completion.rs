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

fn complete_path(
  argument: &str,
  pos: usize,
  suffix: &str,
  use_options: bool,
) -> (usize, Vec<String>) {
  let quote = argument.chars().next().filter(|c| matches!(c, '\'' | '"'));
  let prefix = if quote.is_some() {
    &argument[1..]
  } else {
    argument
  };
  let start = pos - prefix.len();
  let closing_quote = quote.is_some_and(|quote| suffix.starts_with(quote));
  if prefix.contains("://")
    || prefix.chars().any(char::is_control)
    || match quote {
      Some(quote) => prefix.contains(quote) || (!suffix.is_empty() && !closing_quote),
      None => {
        prefix
          .chars()
          .any(|c| c.is_whitespace() || matches!(c, ',' | '\'' | '"'))
          || suffix
            .chars()
            .next()
            .is_some_and(|c| !c.is_whitespace() && !(use_options && c == ','))
      }
    }
  {
    return (start, Vec::new());
  }
  let split = prefix
    .rfind(std::path::is_separator)
    .map_or(0, |index| index + 1);
  let (directory, needle) = prefix.split_at(split);
  let Ok(entries) = std::fs::read_dir(if directory.is_empty() { "." } else { directory }) else {
    return (start, Vec::new());
  };
  let mut matches = Vec::new();
  for entry in entries.flatten() {
    let Ok(name) = entry.file_name().into_string() else {
      continue;
    };
    if !name.starts_with(needle) || (name.starts_with('.') && !needle.starts_with('.')) {
      continue;
    }
    let mut path = format!("{directory}{name}");
    // The parser treats quoted paths literally, so shell backslash escapes cannot
    // make whitespace or a matching quote safe. Require a compatible opening quote.
    if path.chars().any(char::is_control)
      || match quote {
        Some(quote) => path.contains(quote),
        None => path
          .chars()
          .any(|c| c.is_whitespace() || matches!(c, ',' | '\'' | '"')),
      }
    {
      continue;
    }
    let Ok(metadata) = entry.path().metadata() else {
      continue;
    };
    if metadata.is_dir() {
      path.push(std::path::MAIN_SEPARATOR);
    } else if metadata.is_file() {
      if let Some(quote) = quote.filter(|_| !closing_quote) {
        path.push(quote);
      }
    } else {
      continue;
    }
    matches.push(path);
  }
  matches.sort();
  (start, matches)
}

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
    let text = before.trim_start();
    if let Some(end) = text.find(char::is_whitespace) {
      let command = &text[..end];
      if command.eq_ignore_ascii_case("use") || command.eq_ignore_ascii_case("run") {
        return Ok(complete_path(
          text[end..].trim_start(),
          pos,
          &line[pos..],
          command.eq_ignore_ascii_case("use"),
        ));
      }
    }
    if line[pos..]
      .chars()
      .next()
      .is_some_and(|c| !c.is_whitespace())
    {
      return Ok((pos, Vec::new()));
    }
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

  struct Files(std::path::PathBuf);
  impl Files {
    fn new() -> Self {
      static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
      let root = std::env::temp_dir().join(format!(
        "tabdat-completion-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
      ));
      std::fs::create_dir(&root).unwrap();
      for name in ["data.csv", "dataset.csv", ".hidden.csv", "run me.td"] {
        std::fs::write(root.join(name), "").unwrap();
      }
      std::fs::create_dir(root.join("nested space")).unwrap();
      std::fs::write(root.join("nested space/input.csv"), "").unwrap();
      Self(root)
    }
    fn path(&self, suffix: &str) -> String {
      self.0.join(suffix).to_str().unwrap().to_owned()
    }
  }
  impl Drop for Files {
    fn drop(&mut self) {
      let _ = std::fs::remove_dir_all(&self.0);
    }
  }

  #[test]
  fn local_paths_are_sorted_literal_and_preserve_cursor_suffixes() {
    let files = Files::new();
    let line = format!("  UsE {}, has_header(false)", files.path("dat"));
    let pos = line.find(", has_header").unwrap();
    let (start, candidates) = complete(&line, pos);
    assert_eq!(start, 6);
    assert_eq!(
      candidates,
      [files.path("data.csv"), files.path("dataset.csv")]
    );
    let mut updated = line.clone();
    updated.replace_range(start..pos, &candidates[0]);
    let tabdat_language::Command::Use {
      source: tabdat_language::DataSource::LocalPath(path),
      has_header,
      ..
    } = tabdat_language::parse_command(&updated).unwrap()
    else {
      panic!("expected local use")
    };
    assert_eq!(path, files.path("data.csv"));
    assert_eq!(has_header, Some(false));

    let line = format!("use \"{}\", delimiter(\";\")", files.path("dat"));
    let pos = line.find("\", delimiter").unwrap();
    let (start, candidates) = complete(&line, pos);
    assert_eq!(start, 5);
    assert_eq!(
      candidates,
      [files.path("data.csv"), files.path("dataset.csv")]
    );
    let mut updated = line.clone();
    updated.replace_range(start..pos, &candidates[0]);
    let tabdat_language::Command::Use {
      source: tabdat_language::DataSource::LocalPath(path),
      delimiter,
      ..
    } = tabdat_language::parse_command(&updated).unwrap()
    else {
      panic!("expected quoted use")
    };
    assert_eq!(path, files.path("data.csv"));
    assert_eq!(delimiter.as_deref(), Some(";"));
    assert!(complete(&line, pos - 1).1.is_empty());
    for source in [
      format!("use {}", files.path("DAT")),
      format!("use {}", files.path("missing/child")),
      format!("use {}", files.path("data.csv/child")),
      format!("use {}, has_header=", files.path("data.csv")),
      format!("run {} extra", files.path("data.csv")),
      format!("save {}", files.path("dat")),
    ] {
      assert!(complete(&source, source.len()).1.is_empty(), "{source}");
    }
  }

  #[test]
  fn quoted_directory_navigation_and_file_closure_roundtrip_through_the_parser() {
    let files = Files::new();
    let line = format!("use \"{}", files.path("nested"));
    assert_eq!(
      complete(&line, line.len()),
      (
        5,
        vec![format!(
          "{}{}",
          files.path("nested space"),
          std::path::MAIN_SEPARATOR
        )]
      )
    );
    let line = format!("use \"{}", files.path("nested space/in"));
    let (start, candidates) = complete(&line, line.len());
    assert_eq!(
      candidates,
      [format!("{}\"", files.path("nested space/input.csv"))]
    );
    let completed = format!("{}{}", &line[..start], candidates[0]);
    let tabdat_language::Command::Use {
      source: tabdat_language::DataSource::LocalPath(path),
      ..
    } = tabdat_language::parse_command(&completed).unwrap()
    else {
      panic!("expected use")
    };
    assert_eq!(path, files.path("nested space/input.csv"));
    let line = format!("RUN '{}", files.path("run"));
    let (start, candidates) = complete(&line, line.len());
    assert_eq!(candidates, [format!("{}'", files.path("run me.td"))]);
    assert_eq!(
      tabdat_language::parse_command(&format!("{}{}", &line[..start], candidates[0])).unwrap(),
      tabdat_language::Command::Run {
        path: files.path("run me.td")
      }
    );
    let line = format!("use {}", files.path(""));
    let candidates = complete(&line, line.len()).1;
    assert!(!candidates.iter().any(|path| path.contains("hidden")
      || path.contains("nested space")
      || path.contains("run me")));
    let line = format!("use {}", files.path(".h"));
    assert_eq!(complete(&line, line.len()).1, [files.path(".hidden.csv")]);
  }

  #[cfg(unix)]
  #[test]
  fn incompatible_names_are_skipped_without_shell_escaping_or_symlink_rewriting() {
    use std::os::unix::fs::symlink;
    let files = Files::new();
    for name in [
      "odd name,雪.csv",
      "double\"quote.csv",
      "single'quote.csv",
      "both'\".csv",
      "literal\\x.csv",
      "control\n.csv",
      "escape\u{1b}[31m.csv",
    ] {
      std::fs::write(files.0.join(name), "").unwrap();
    }
    symlink(files.0.join("data.csv"), files.0.join("link.csv")).unwrap();
    symlink(files.0.join("nested space"), files.0.join("alias")).unwrap();
    symlink(files.0.join("absent"), files.0.join("broken")).unwrap();
    let line = format!("use '{}", files.path(""));
    let candidates = complete(&line, line.len()).1;
    assert!(candidates.contains(&format!("{}'", files.path("odd name,雪.csv"))));
    assert!(candidates.contains(&format!("{}'", files.path("double\"quote.csv"))));
    assert!(candidates.contains(&format!("{}'", files.path("literal\\x.csv"))));
    assert!(candidates.contains(&format!("{}'", files.path("link.csv"))));
    assert!(candidates.contains(&format!("{}/", files.path("alias"))));
    assert!(!candidates.iter().any(|path| path.contains("single")
      || path.contains("both")
      || path.contains("control")
      || path.contains("escape")
      || path.contains("broken")));
    let line = format!("use \"{}", files.path("single"));
    assert_eq!(
      complete(&line, line.len()).1,
      [format!("{}\"", files.path("single'quote.csv"))]
    );
  }

  #[test]
  fn only_word_ends_complete_without_replacing_arguments_or_suffixes() {
    assert_eq!(complete("summ y", 4), (0, vec!["summarize".into()]));
    assert_eq!(complete("help summ  ", 9), (5, vec!["summarize".into()]));
    for line in [
      "regress summ",
      "summarize ",
      "sql select summ",
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
