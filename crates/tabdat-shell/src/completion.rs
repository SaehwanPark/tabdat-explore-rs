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
  "status",
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

fn is_bare_identifier(name: &str) -> bool {
  let mut characters = name.chars();
  characters
    .next()
    .is_some_and(|c| c.is_alphabetic() || c == '_')
    && characters.all(|c| c.is_alphanumeric() || c == '_')
}

fn is_bare_column(name: &str) -> bool {
  !name.eq_ignore_ascii_case("if") && is_bare_identifier(name)
}

#[derive(Default)]
pub(crate) struct ShellHelper {
  columns: Vec<String>,
  named_tables: Vec<String>,
}

impl ShellHelper {
  pub(crate) fn set_named_tables<'a>(&mut self, names: impl Iterator<Item = &'a str>) {
    self.named_tables = names
      .filter(|name| is_bare_identifier(name))
      .map(str::to_owned)
      .collect();
    self.named_tables.sort();
    self.named_tables.dedup();
  }

  pub(crate) fn set_columns(&mut self, columns: &[tabdat_domain::ColumnInfo]) {
    self.columns = columns
      .iter()
      .map(|column| column.name.as_str())
      .filter(|name| is_bare_column(name))
      .map(str::to_owned)
      .collect();
    self.columns.sort();
    self.columns.dedup();
  }

  fn complete_columns(&self, argument: &str, pos: usize, suffix: &str) -> (usize, Vec<String>) {
    if !argument.split_whitespace().all(is_bare_column)
      || suffix
        .chars()
        .next()
        .is_some_and(|c| !c.is_whitespace() && c != ',')
    {
      return (pos, Vec::new());
    }
    let prefix = argument
      .rsplit(char::is_whitespace)
      .next()
      .unwrap_or_default();
    (
      pos - prefix.len(),
      self
        .columns
        .iter()
        .filter(|name| name.starts_with(prefix))
        .cloned()
        .collect(),
    )
  }
}

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
        let argument = text[end..].trim_start();
        let use_command = command.eq_ignore_ascii_case("use");
        let (start, mut matches) = complete_path(argument, pos, &line[pos..], use_command);
        // Named activation has no options. Keep quotes, path navigation and run
        // on the existing filesystem-only path; explicit ./ disambiguates names.
        if use_command
          && line[pos..].trim().is_empty()
          && (argument.is_empty() || is_bare_identifier(argument))
        {
          matches.extend(
            self
              .named_tables
              .iter()
              .filter(|name| name.starts_with(argument))
              .cloned(),
          );
          matches.sort();
          matches.dedup();
        }
        return Ok((start, matches));
      }
      if [
        "summarize",
        "codebook",
        "missing",
        "tabulate",
        "regress",
        "keep",
        "drop",
        "select",
        "sort",
      ]
      .iter()
      .any(|name| command.eq_ignore_ascii_case(name))
      {
        return Ok(self.complete_columns(&text[end..], pos, &line[pos..]));
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
    ShellHelper::default()
      .complete(line, pos, &Context::new(&DefaultHistory::new()))
      .unwrap()
  }

  fn columns(names: &[&str]) -> Vec<tabdat_domain::ColumnInfo> {
    names
      .iter()
      .map(|name| tabdat_domain::ColumnInfo {
        name: (*name).into(),
        data_type: "DOUBLE".into(),
      })
      .collect()
  }

  fn complete_with(helper: &ShellHelper, line: &str, pos: usize) -> (usize, Vec<String>) {
    helper
      .complete(line, pos, &Context::new(&DefaultHistory::new()))
      .unwrap()
  }

  #[test]
  fn named_use_completion_is_literal_sorted_and_roundtrips_without_model_inference() {
    let mut helper = ShellHelper::default();
    helper.set_named_tables(
      [
        "cohort_b",
        "CoHort",
        "cohort_a",
        "cohort_a",
        "if",
        "变量",
        "_sample",
        "bad-name",
        "../cohort",
        "space table",
        "control\t",
        "2bad",
      ]
      .into_iter(),
    );
    assert_eq!(
      helper.named_tables,
      ["CoHort", "_sample", "cohort_a", "cohort_b", "if", "变量"]
    );
    for prefix in ["use cohort_", "  UsE cohort_"] {
      let (start, matches) = complete_with(&helper, prefix, prefix.len());
      assert_eq!(start, prefix.len() - "cohort_".len());
      let mut expected = complete(prefix, prefix.len()).1;
      expected.extend(["cohort_a".to_owned(), "cohort_b".to_owned()]);
      expected.sort();
      expected.dedup();
      assert_eq!(matches, expected);
    }
    for name in &helper.named_tables {
      let tabdat_language::Command::Use {
        source: tabdat_language::DataSource::LocalPath(source),
        ..
      } = tabdat_language::parse_command(&format!("use {name}")).unwrap()
      else {
        panic!("use")
      };
      assert_eq!(&source, name);
    }
    let line = "use 变";
    assert_eq!(complete_with(&helper, line, line.len()).1, ["变量"]);
    helper.set_named_tables(["fresh_cohort"].into_iter());
    assert_eq!(helper.named_tables, ["fresh_cohort"]);
    helper.set_named_tables(std::iter::empty());
    assert_eq!(complete_with(&helper, "use ", 4), complete("use ", 4));
    assert_eq!(complete_with(&helper, "summ", 4).1, ["summarize"]);
  }

  #[test]
  fn named_use_completion_leaves_paths_quotes_options_other_commands_and_suffixes_unchanged() {
    let mut helper = ShellHelper::default();
    helper.set_named_tables(["cohort_a", "data", "dataset", "nested"].into_iter());
    for line in [
      "run cohort_",
      "append cohort_",
      "join cohort_",
      "use './cohort_",
      "use \"cohort_",
      "use ./cohort_",
      "use ../cohort_",
      "use folder/cohort_",
      "use cohort_, delimiter(',')",
    ] {
      assert_eq!(
        complete_with(&helper, line, line.len()),
        complete(line, line.len()),
        "{line}"
      );
    }
    for suffix in [", delimiter(',')", " , lazy", " extra", "abc"] {
      let line = format!("use cohort_{suffix}");
      let pos = "use cohort_".len();
      assert_eq!(
        complete_with(&helper, &line, pos),
        complete(&line, pos),
        "{line}"
      );
    }
    let line = "  UsE cohort_   ";
    let pos = line.len() - 3;
    let (start, candidates) = complete_with(&helper, line, pos);
    let mut updated = line.to_owned();
    let candidate = candidates
      .iter()
      .find(|name| name.as_str() == "cohort_a")
      .unwrap();
    updated.replace_range(start..pos, candidate);
    assert_eq!(updated, "  UsE cohort_a   ");
    let files = Files::new();
    let line = format!("use {}", files.path("dat"));
    assert_eq!(
      complete_with(&helper, &line, line.len()),
      complete(&line, line.len())
    );
    let directory = std::fs::read_dir(".")
      .unwrap()
      .flatten()
      .find(|entry| {
        entry.file_type().unwrap().is_dir()
          && entry.file_name().to_str().is_some_and(is_bare_identifier)
      })
      .unwrap()
      .file_name()
      .into_string()
      .unwrap();
    helper.set_named_tables(std::iter::once(directory.as_str()));
    let line = format!("use {directory}");
    let mut expected = complete(&line, line.len()).1;
    expected.push(directory);
    expected.sort();
    expected.dedup();
    assert_eq!(complete_with(&helper, &line, line.len()).1, expected);
  }

  #[test]
  fn bare_column_lists_use_current_literal_names_without_type_or_model_inference() {
    let mut helper = ShellHelper::default();
    helper.set_columns(&columns(&[
      "z",
      "Age",
      "_id",
      "age",
      "age_sq",
      "变量",
      "has space",
      "if",
      "IF",
      "bad-name",
      "2number",
      "age",
    ]));
    let expected = ["Age", "_id", "age", "age_sq", "z", "变量"];
    for command in [
      "summarize",
      "codebook",
      "missing",
      "tabulate",
      "regress",
      "keep",
      "drop",
      "select",
      "sort",
    ] {
      let line = format!("{command} ");
      assert_eq!(
        complete_with(&helper, &line, line.len()),
        (line.len(), expected.map(str::to_owned).to_vec())
      );
    }
    for name in expected {
      assert_eq!(
        tabdat_language::parse_command(&format!("summarize {name}")).unwrap(),
        tabdat_language::Command::Summarize {
          variables: vec![name.into()]
        }
      );
    }
    assert_eq!(
      complete_with(&helper, "summarize a", 11),
      (10, vec!["age".into(), "age_sq".into()])
    );
    assert_eq!(
      complete_with(&helper, "SUMMARIZE A", 11),
      (10, vec!["Age".into()])
    );
    assert_eq!(
      complete_with(&helper, "regress age a", 13),
      (12, vec!["age".into(), "age_sq".into()])
    );
    let line = "regress age 变";
    assert_eq!(
      complete_with(&helper, line, line.len()),
      (12, vec!["变量".into()])
    );
  }

  #[test]
  fn column_completion_preserves_suffixes_but_declines_richer_contexts_and_inside_words() {
    let mut helper = ShellHelper::default();
    helper.set_columns(&columns(&["age", "age_sq"]));
    let line = "  SuMmArIzE a if age > 0";
    let pos = line.find("a if").unwrap() + 1;
    let (start, candidates) = complete_with(&helper, line, pos);
    assert_eq!(start, pos - 1);
    let mut updated = line.to_owned();
    updated.replace_range(start..pos, &candidates[1]);
    assert_eq!(updated, "  SuMmArIzE age_sq if age > 0");
    let tabdat_language::Command::SummarizeQuery {
      variables, options, ..
    } = tabdat_language::parse_command(&updated).unwrap()
    else {
      panic!("expected conditional summary")
    };
    assert_eq!(variables, ["age_sq"]);
    assert!(options.is_empty());
    let line = "regress age a, robust";
    let pos = line.find(',').unwrap();
    let (start, candidates) = complete_with(&helper, line, pos);
    let mut updated = line.to_owned();
    updated.replace_range(start..pos, &candidates[1]);
    let tabdat_language::Command::Regress { command } =
      tabdat_language::parse_command(&updated).unwrap()
    else {
      panic!("expected regress")
    };
    assert_eq!(command.predictors, ["age_sq"]);
    assert!(command.robust);
    for line in [
      "summarize age if a",
      "regress age a, robust",
      "summarize 'a",
      "by g: summarize a",
      "predict a",
      "generate new = a",
      "sql select a",
      "sort -a",
      "count a",
      "summarize a+1",
    ] {
      assert!(
        complete_with(&helper, line, line.len()).1.is_empty(),
        "{line}"
      );
    }
    assert!(complete_with(&helper, "summarize age", 12).1.is_empty());
    assert!(complete_with(&helper, "summarize 变量", 11).1.is_empty());
  }

  #[test]
  fn column_snapshot_replaces_old_names_and_can_be_cleared_without_losing_command_help() {
    let mut helper = ShellHelper::default();
    helper.set_columns(&columns(&["old", "x"]));
    assert_eq!(complete_with(&helper, "summarize ", 10).1, ["old", "x"]);
    helper.set_columns(&columns(&["new", "x"]));
    assert_eq!(complete_with(&helper, "summarize ", 10).1, ["new", "x"]);
    helper.set_columns(&[]);
    assert!(complete_with(&helper, "summarize ", 10).1.is_empty());
    assert_eq!(complete_with(&helper, "summ", 4).1, ["summarize"]);
    assert_eq!(complete_with(&helper, "help summ", 9).1, ["summarize"]);
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
