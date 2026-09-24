import json

with open('/tmp/tabdat_catalogs.json') as f:
    catalogs = json.load(f)

sorted_names = sorted(catalogs.keys())

lines = []
lines.append('//! Canonical command catalogs, schemas, effects, and serialization envelopes.')
lines.append('')
lines.append('use serde::{Deserialize, Serialize};')
lines.append('')
lines.append('use crate::help::{available_help_topics, load_help_topic_text};')
lines.append('')

# COMMAND_NAMES
lines.append('pub const COMMAND_NAMES: &[&str] = &[')
for name in sorted_names:
    lines.append(f'  "{name}",')
lines.append('];')
lines.append('')

# Models
lines.append('''#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandCatalogEntry {
  pub help_topic: Option<String>,
  pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandCatalogResult {
  pub commands: Vec<CommandCatalogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandEffectEntry {
  pub effects: Vec<String>,
  pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandEffectCatalogResult {
  pub commands: Vec<CommandEffectEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgumentDescriptor {
  pub name: String,
  pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionDescriptor {
  pub name: String,
  pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandSchemaResult {
  pub arguments: Vec<ArgumentDescriptor>,
  pub help_topic: Option<String>,
  pub name: String,
  pub options: Vec<OptionDescriptor>,
  pub syntax: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpTopicResult {
  pub help_topic: String,
  pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandExplainResult {
  pub command_name: String,
  pub execution: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultEnvelope<T> {
  pub data: T,
  pub result_type: String,
  pub schema_version: u32,
}

impl<T> ResultEnvelope<T> {
  pub fn new(data: T, result_type: &str) -> Self {
    Self {
      data,
      result_type: result_type.to_string(),
      schema_version: 1,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDetail {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub line: Option<usize>,
  pub message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub path: Option<String>,
  #[serde(rename = "type")]
  pub error_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorEnvelope {
  pub error: ErrorDetail,
  pub schema_version: u32,
}

impl ErrorEnvelope {
  pub fn new(message: impl Into<String>, error_type: &str) -> Self {
    Self {
      error: ErrorDetail {
        line: None,
        message: message.into(),
        path: None,
        error_type: error_type.to_string(),
      },
      schema_version: 1,
    }
  }

  pub fn with_location(
    message: impl Into<String>,
    error_type: &str,
    path: impl Into<String>,
    line: usize,
  ) -> Self {
    Self {
      error: ErrorDetail {
        line: Some(line),
        message: message.into(),
        path: Some(path.into()),
        error_type: error_type.to_string(),
      },
      schema_version: 1,
    }
  }
}
''')

# command_catalog_result
lines.append('''pub fn command_catalog_result() -> CommandCatalogResult {
  let help_topics = available_help_topics();
  let commands = COMMAND_NAMES
    .iter()
    .map(|&name| CommandCatalogEntry {
      help_topic: if help_topics.contains(&name) {
        Some(name.to_string())
      } else {
        None
      },
      name: name.to_string(),
    })
    .collect();
  CommandCatalogResult { commands }
}
''')

# command_effect_catalog_result
lines.append('pub fn command_effect_catalog_result() -> CommandEffectCatalogResult {')
lines.append('  let commands = vec![')
for name in sorted_names:
    effs = catalogs[name]['effects']
    eff_items = ', '.join(f'"{e}".to_string()' for e in effs)
    lines.append(f'    CommandEffectEntry {{ effects: vec![{eff_items}], name: "{name}".to_string() }},')
lines.append('  ];')
lines.append('  CommandEffectCatalogResult { commands }')
lines.append('}')
lines.append('')

# describe_command_result
lines.append('''pub fn describe_command_result(name: &str) -> Result<CommandSchemaResult, String> {
  let normalized = name.trim().to_ascii_lowercase();
  if normalized.is_empty() {
    return Err("command name cannot be empty".to_string());
  }
  if !COMMAND_NAMES.contains(&normalized.as_str()) {
    return Err(format!("unknown command name: {normalized}"));
  }
  let help_topics = available_help_topics();
  let help_topic = if help_topics.contains(&normalized.as_str()) {
    Some(normalized.clone())
  } else {
    None
  };
  get_command_schema(&normalized, help_topic)
}
''')

lines.append('fn get_command_schema(name: &str, help_topic: Option<String>) -> Result<CommandSchemaResult, String> {')
lines.append('  match name {')
for name in sorted_names:
    c = catalogs[name]
    syntax = c['syntax']
    args = c['arguments']
    opts = c['options']
    arg_code = ', '.join(f'ArgumentDescriptor {{ name: "{a["name"]}".to_string(), required: {str(a["required"]).lower()} }}' for a in args)
    opt_code = ', '.join(f'OptionDescriptor {{ name: "{o["name"]}".to_string(), required: {str(o["required"]).lower()} }}' for o in opts)
    lines.append(f'    "{name}" => Ok(CommandSchemaResult {{')
    lines.append(f'      arguments: vec![{arg_code}],')
    lines.append('      help_topic,')
    lines.append(f'      name: "{name}".to_string(),')
    lines.append(f'      options: vec![{opt_code}],')
    lines.append(f'      syntax: "{syntax}".to_string(),')
    lines.append('    }),')
lines.append('    _ => Err(format!("unknown command name: {name}")),')
lines.append('  }')
lines.append('}')
lines.append('')

# help_topic_result
lines.append('''pub fn help_topic_result(topic: &str) -> Result<HelpTopicResult, String> {
  let normalized = topic.trim().to_ascii_lowercase();
  if normalized.is_empty() {
    return Err("help topic cannot be empty".to_string());
  }
  let help_topics = available_help_topics();
  if !help_topics.contains(&normalized.as_str()) {
    return Err(format!("unknown help topic: {normalized}"));
  }
  let text = match load_help_topic_text(&normalized) {
    Some(txt) => txt.to_string(),
    None => return Err(format!("unable to load help topic: {normalized}")),
  };
  Ok(HelpTopicResult {
    help_topic: normalized,
    text,
  })
}
''')

# preview_command_name & explain_result
lines.append('''pub fn preview_command_name(command_text: &str) -> String {
  let trimmed = command_text.trim();
  if trimmed.is_empty() {
    return String::new();
  }
  let first_token = trimmed
    .split_whitespace()
    .next()
    .unwrap_or("")
    .to_ascii_lowercase();
  if first_token == "?" {
    return "help".to_string();
  }
  first_token.trim_end_matches([',', ':']).to_string()
}

pub fn explain_result(command_text: &str) -> Result<CommandExplainResult, (String, &'static str)> {
  match tabdat_language::parse_command(command_text) {
    Ok(_) => Ok(CommandExplainResult {
      command_name: preview_command_name(command_text),
      execution: "not_run".to_string(),
    }),
    Err(err) => Err((err.to_string(), "ParseError")),
  }
}
''')

with open('src/catalog.rs', 'w') as f:
    f.write('\n'.join(lines) + '\n')

print('Wrote src/catalog.rs, size:', len('\n'.join(lines)))
