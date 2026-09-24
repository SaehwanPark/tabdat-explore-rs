//! Canonical command catalogs, schemas, effects, and serialization envelopes.

use serde::{Deserialize, Serialize};

use crate::help::{available_help_topics, load_help_topic_text};

pub const COMMAND_NAMES: &[&str] = &[
  "append",
  "assert",
  "bar",
  "bayes",
  "bayesplot",
  "by",
  "cfregress",
  "codebook",
  "collapse",
  "count",
  "cvelasticnet",
  "cvlasso",
  "cvridge",
  "datasignature",
  "decode",
  "describe",
  "did",
  "dml",
  "doctor",
  "drdid",
  "drop",
  "duplicates",
  "elasticnet",
  "encode",
  "estat",
  "exit",
  "export",
  "generate",
  "gsort",
  "head",
  "heckman",
  "help",
  "histogram",
  "isid",
  "ivregress",
  "join",
  "keep",
  "label",
  "lasso",
  "lincom",
  "logit",
  "lowess",
  "missing",
  "nbreg",
  "nl",
  "panel",
  "poisson",
  "postlasso",
  "predict",
  "probit",
  "qreg",
  "quit",
  "recode",
  "regress",
  "rename",
  "replace",
  "reshape",
  "ridge",
  "run",
  "save",
  "scatter",
  "select",
  "set",
  "sort",
  "spregress",
  "sql",
  "status",
  "streg",
  "summarize",
  "tabulate",
  "tail",
  "test",
  "tobit",
  "ttest",
  "use",
  "xtabond",
  "xtdata",
  "xtlogit",
  "xtreg",
  "zinb",
  "zip",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

pub fn command_catalog_result() -> CommandCatalogResult {
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

pub fn command_effect_catalog_result() -> CommandEffectCatalogResult {
  let commands = vec![
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "append".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "assert".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "plot".to_string()],
      name: "bar".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "bayes".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "plot".to_string()],
      name: "bayesplot".to_string(),
    },
    CommandEffectEntry {
      effects: vec![
        "read".to_string(),
        "write".to_string(),
        "control".to_string(),
        "plot".to_string(),
      ],
      name: "by".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "cfregress".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "codebook".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "collapse".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "count".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "cvelasticnet".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "cvlasso".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "cvridge".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "datasignature".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "decode".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "describe".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "did".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "dml".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "doctor".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "drdid".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "drop".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "duplicates".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "elasticnet".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "encode".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "plot".to_string()],
      name: "estat".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["control".to_string()],
      name: "exit".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "export".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "generate".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "gsort".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "head".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "heckman".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["control".to_string()],
      name: "help".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "plot".to_string()],
      name: "histogram".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "isid".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "ivregress".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "join".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "keep".to_string(),
    },
    CommandEffectEntry {
      effects: vec![
        "read".to_string(),
        "write".to_string(),
        "control".to_string(),
      ],
      name: "label".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "lasso".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "lincom".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "logit".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "lowess".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "missing".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "nbreg".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "nl".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["control".to_string()],
      name: "panel".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "poisson".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "postlasso".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "predict".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "probit".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "qreg".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["control".to_string()],
      name: "quit".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "recode".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "regress".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "rename".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "replace".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "reshape".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "ridge".to_string(),
    },
    CommandEffectEntry {
      effects: vec![
        "read".to_string(),
        "write".to_string(),
        "control".to_string(),
        "plot".to_string(),
      ],
      name: "run".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "save".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "plot".to_string()],
      name: "scatter".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "select".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["control".to_string()],
      name: "set".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "sort".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "spregress".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "sql".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "status".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "streg".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "summarize".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "tabulate".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "tail".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "test".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "tobit".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "ttest".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "control".to_string()],
      name: "use".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "xtabond".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string(), "write".to_string()],
      name: "xtdata".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "xtlogit".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "xtreg".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "zinb".to_string(),
    },
    CommandEffectEntry {
      effects: vec!["read".to_string()],
      name: "zip".to_string(),
    },
  ];
  CommandEffectCatalogResult { commands }
}

pub fn describe_command_result(name: &str) -> Result<CommandSchemaResult, String> {
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

fn get_command_schema(
  name: &str,
  help_topic: Option<String>,
) -> Result<CommandSchemaResult, String> {
  match name {
    "append" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "table".to_string(),
        required: true,
      }],
      help_topic,
      name: "append".to_string(),
      options: vec![],
      syntax: "append tablename".to_string(),
    }),
    "assert" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "expression".to_string(),
        required: true,
      }],
      help_topic,
      name: "assert".to_string(),
      options: vec![],
      syntax: "assert <boolean-expression>".to_string(),
    }),
    "bar" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variable".to_string(),
        required: true,
      }],
      help_topic,
      name: "bar".to_string(),
      options: vec![],
      syntax: "bar varname".to_string(),
    }),
    "bayes" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "model".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "bayes".to_string(),
      options: vec![
        OptionDescriptor {
          name: "n_iter".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "tol".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "bayes model y x".to_string(),
    }),
    "bayesplot" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "subcommand".to_string(),
        required: true,
      }],
      help_topic,
      name: "bayesplot".to_string(),
      options: vec![],
      syntax: "bayesplot trace".to_string(),
    }),
    "by" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "variables".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "command".to_string(),
          required: true,
        },
      ],
      help_topic,
      name: "by".to_string(),
      options: vec![],
      syntax: "by varlist: command".to_string(),
    }),
    "cfregress" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "cfregress".to_string(),
      options: vec![
        OptionDescriptor {
          name: "endog".to_string(),
          required: true,
        },
        OptionDescriptor {
          name: "iv".to_string(),
          required: true,
        },
      ],
      syntax: "cfregress y x, endog(vars) iv(vars)".to_string(),
    }),
    "codebook" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variables".to_string(),
        required: false,
      }],
      help_topic,
      name: "codebook".to_string(),
      options: vec![],
      syntax: "codebook [varlist]".to_string(),
    }),
    "collapse" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "targets".to_string(),
        required: true,
      }],
      help_topic,
      name: "collapse".to_string(),
      options: vec![OptionDescriptor {
        name: "by".to_string(),
        required: false,
      }],
      syntax: "collapse (stat) vars [, by(vars)]".to_string(),
    }),
    "count" => Ok(CommandSchemaResult {
      arguments: vec![],
      help_topic,
      name: "count".to_string(),
      options: vec![],
      syntax: "count".to_string(),
    }),
    "cvelasticnet" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "model".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "cvelasticnet".to_string(),
      options: vec![
        OptionDescriptor {
          name: "cv".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "l1_ratio".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "cvelasticnet model y x".to_string(),
    }),
    "cvlasso" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "model".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "cvlasso".to_string(),
      options: vec![
        OptionDescriptor {
          name: "cv".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "cvlasso model y x".to_string(),
    }),
    "cvridge" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "model".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "cvridge".to_string(),
      options: vec![
        OptionDescriptor {
          name: "cv".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "cvridge model y x".to_string(),
    }),
    "datasignature" => Ok(CommandSchemaResult {
      arguments: vec![],
      help_topic,
      name: "datasignature".to_string(),
      options: vec![],
      syntax: "datasignature".to_string(),
    }),
    "decode" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "source".to_string(),
        required: true,
      }],
      help_topic,
      name: "decode".to_string(),
      options: vec![OptionDescriptor {
        name: "generate".to_string(),
        required: true,
      }],
      syntax: "decode numvar, generate(newvar)".to_string(),
    }),
    "describe" => Ok(CommandSchemaResult {
      arguments: vec![],
      help_topic,
      name: "describe".to_string(),
      options: vec![],
      syntax: "describe".to_string(),
    }),
    "did" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "depvar".to_string(),
        required: true,
      }],
      help_topic,
      name: "did".to_string(),
      options: vec![
        OptionDescriptor {
          name: "treat".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "post".to_string(),
          required: false,
        },
      ],
      syntax: "did y [, treat(var) post(var)]".to_string(),
    }),
    "dml" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "estimator".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "controls".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "dml".to_string(),
      options: vec![OptionDescriptor {
        name: "treat".to_string(),
        required: true,
      }],
      syntax: "dml estimator y controls, treat(var)".to_string(),
    }),
    "doctor" => Ok(CommandSchemaResult {
      arguments: vec![],
      help_topic,
      name: "doctor".to_string(),
      options: vec![],
      syntax: "doctor".to_string(),
    }),
    "drdid" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "depvar".to_string(),
        required: true,
      }],
      help_topic,
      name: "drdid".to_string(),
      options: vec![
        OptionDescriptor {
          name: "treat".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "post".to_string(),
          required: false,
        },
      ],
      syntax: "drdid y [, treat(var) post(var)]".to_string(),
    }),
    "drop" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variables".to_string(),
        required: true,
      }],
      help_topic,
      name: "drop".to_string(),
      options: vec![],
      syntax: "drop varlist".to_string(),
    }),
    "duplicates" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variables".to_string(),
        required: false,
      }],
      help_topic,
      name: "duplicates".to_string(),
      options: vec![],
      syntax: "duplicates [report] [varlist]".to_string(),
    }),
    "elasticnet" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "model".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "elasticnet".to_string(),
      options: vec![
        OptionDescriptor {
          name: "alpha".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "l1_ratio".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "elasticnet model y x".to_string(),
    }),
    "encode" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "source".to_string(),
        required: true,
      }],
      help_topic,
      name: "encode".to_string(),
      options: vec![
        OptionDescriptor {
          name: "generate".to_string(),
          required: true,
        },
        OptionDescriptor {
          name: "label".to_string(),
          required: false,
        },
      ],
      syntax: "encode strvar, generate(newvar)".to_string(),
    }),
    "estat" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "subcommand".to_string(),
        required: true,
      }],
      help_topic,
      name: "estat".to_string(),
      options: vec![
        OptionDescriptor {
          name: "saving".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noopen".to_string(),
          required: false,
        },
      ],
      syntax: "estat subcommand".to_string(),
    }),
    "exit" => Ok(CommandSchemaResult {
      arguments: vec![],
      help_topic,
      name: "exit".to_string(),
      options: vec![],
      syntax: "exit".to_string(),
    }),
    "export" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "filename".to_string(),
        required: true,
      }],
      help_topic,
      name: "export".to_string(),
      options: vec![],
      syntax: "export filename".to_string(),
    }),
    "generate" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "variable".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "expression".to_string(),
          required: true,
        },
      ],
      help_topic,
      name: "generate".to_string(),
      options: vec![],
      syntax: "generate newvar = exp".to_string(),
    }),
    "gsort" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "keys".to_string(),
        required: true,
      }],
      help_topic,
      name: "gsort".to_string(),
      options: vec![],
      syntax: "gsort [+|-]varlist".to_string(),
    }),
    "head" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "n".to_string(),
        required: false,
      }],
      help_topic,
      name: "head".to_string(),
      options: vec![],
      syntax: "head [N]".to_string(),
    }),
    "heckman" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "heckman".to_string(),
      options: vec![
        OptionDescriptor {
          name: "selectdep".to_string(),
          required: true,
        },
        OptionDescriptor {
          name: "select".to_string(),
          required: true,
        },
      ],
      syntax: "heckman y x1 x2, selectdep(var) select(vars)".to_string(),
    }),
    "help" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "topic".to_string(),
        required: false,
      }],
      help_topic,
      name: "help".to_string(),
      options: vec![],
      syntax: "help [topic]".to_string(),
    }),
    "histogram" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variable".to_string(),
        required: true,
      }],
      help_topic,
      name: "histogram".to_string(),
      options: vec![],
      syntax: "histogram varname".to_string(),
    }),
    "isid" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variables".to_string(),
        required: true,
      }],
      help_topic,
      name: "isid".to_string(),
      options: vec![OptionDescriptor {
        name: "missok".to_string(),
        required: false,
      }],
      syntax: "isid varlist [, missok]".to_string(),
    }),
    "ivregress" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "estimator".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "ivregress".to_string(),
      options: vec![
        OptionDescriptor {
          name: "endog".to_string(),
          required: true,
        },
        OptionDescriptor {
          name: "iv".to_string(),
          required: true,
        },
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "cluster".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "ivregress estimator y x, endog(vars) iv(vars)".to_string(),
    }),
    "join" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "table".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "keys".to_string(),
          required: true,
        },
      ],
      help_topic,
      name: "join".to_string(),
      options: vec![
        OptionDescriptor {
          name: "how".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "suffix".to_string(),
          required: false,
        },
      ],
      syntax: "join tablename on keys".to_string(),
    }),
    "keep" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variables".to_string(),
        required: true,
      }],
      help_topic,
      name: "keep".to_string(),
      options: vec![],
      syntax: "keep varlist".to_string(),
    }),
    "label" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "subcommand".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "arguments".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "label".to_string(),
      options: vec![
        OptionDescriptor {
          name: "clear".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "replace".to_string(),
          required: false,
        },
      ],
      syntax: "label variable|define|values|list|drop|save|use ...".to_string(),
    }),
    "lasso" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "model".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "lasso".to_string(),
      options: vec![
        OptionDescriptor {
          name: "alpha".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "lasso model y x".to_string(),
    }),
    "lincom" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "specification".to_string(),
        required: true,
      }],
      help_topic,
      name: "lincom".to_string(),
      options: vec![],
      syntax: "lincom spec".to_string(),
    }),
    "logit" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "logit".to_string(),
      options: vec![
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "cluster".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "logit y x1 x2".to_string(),
    }),
    "lowess" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "y_variable".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "x_variable".to_string(),
          required: true,
        },
      ],
      help_topic,
      name: "lowess".to_string(),
      options: vec![OptionDescriptor {
        name: "gen".to_string(),
        required: true,
      }],
      syntax: "lowess y x, gen(newvar)".to_string(),
    }),
    "missing" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variables".to_string(),
        required: false,
      }],
      help_topic,
      name: "missing".to_string(),
      options: vec![],
      syntax: "missing [varlist]".to_string(),
    }),
    "nbreg" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "nbreg".to_string(),
      options: vec![
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "cluster".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "nbreg y x1 x2".to_string(),
    }),
    "nl" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "formula".to_string(),
        required: true,
      }],
      help_topic,
      name: "nl".to_string(),
      options: vec![
        OptionDescriptor {
          name: "params".to_string(),
          required: true,
        },
        OptionDescriptor {
          name: "start".to_string(),
          required: true,
        },
      ],
      syntax: "nl formula, params(vars) start(vals)".to_string(),
    }),
    "panel" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "id_variable".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "time_variable".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "panel".to_string(),
      options: vec![],
      syntax: "panel id time".to_string(),
    }),
    "poisson" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "poisson".to_string(),
      options: vec![
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "cluster".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "poisson y x1 x2".to_string(),
    }),
    "postlasso" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "model".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "postlasso".to_string(),
      options: vec![
        OptionDescriptor {
          name: "alpha".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "postlasso model y x".to_string(),
    }),
    "predict" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variable".to_string(),
        required: true,
      }],
      help_topic,
      name: "predict".to_string(),
      options: vec![
        OptionDescriptor {
          name: "xb".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "residuals".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "probabilities".to_string(),
          required: false,
        },
      ],
      syntax: "predict newvar [, xb residuals probabilities]".to_string(),
    }),
    "probit" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "probit".to_string(),
      options: vec![
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "cluster".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "probit y x1 x2".to_string(),
    }),
    "qreg" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "qreg".to_string(),
      options: vec![
        OptionDescriptor {
          name: "quantile".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "cluster".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "qreg y x [, quantile(val)]".to_string(),
    }),
    "quit" => Ok(CommandSchemaResult {
      arguments: vec![],
      help_topic,
      name: "quit".to_string(),
      options: vec![],
      syntax: "quit".to_string(),
    }),
    "recode" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "variable".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "rules".to_string(),
          required: true,
        },
      ],
      help_topic,
      name: "recode".to_string(),
      options: vec![OptionDescriptor {
        name: "generate".to_string(),
        required: false,
      }],
      syntax: "recode varname (rules) [, generate(newvar)]".to_string(),
    }),
    "regress" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "regress".to_string(),
      options: vec![
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "cluster".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "wls".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "gls".to_string(),
          required: false,
        },
      ],
      syntax: "regress y x1 x2 [, robust cluster(var) noconstant wls(var) gls(var)]".to_string(),
    }),
    "rename" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "old_name".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "new_name".to_string(),
          required: true,
        },
      ],
      help_topic,
      name: "rename".to_string(),
      options: vec![],
      syntax: "rename oldvar newvar".to_string(),
    }),
    "replace" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "variable".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "expression".to_string(),
          required: true,
        },
      ],
      help_topic,
      name: "replace".to_string(),
      options: vec![OptionDescriptor {
        name: "if".to_string(),
        required: false,
      }],
      syntax: "replace oldvar = exp [if exp]".to_string(),
    }),
    "reshape" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "direction".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "stub".to_string(),
          required: true,
        },
      ],
      help_topic,
      name: "reshape".to_string(),
      options: vec![
        OptionDescriptor {
          name: "i".to_string(),
          required: true,
        },
        OptionDescriptor {
          name: "j".to_string(),
          required: true,
        },
      ],
      syntax: "reshape long|wide stub, i(vars) j(var)".to_string(),
    }),
    "ridge" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "model".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "ridge".to_string(),
      options: vec![
        OptionDescriptor {
          name: "alpha".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "ridge model y x".to_string(),
    }),
    "run" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "filename".to_string(),
        required: true,
      }],
      help_topic,
      name: "run".to_string(),
      options: vec![],
      syntax: "run filename".to_string(),
    }),
    "save" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "filename".to_string(),
        required: true,
      }],
      help_topic,
      name: "save".to_string(),
      options: vec![],
      syntax: "save filename".to_string(),
    }),
    "scatter" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "y_variable".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "x_variable".to_string(),
          required: true,
        },
      ],
      help_topic,
      name: "scatter".to_string(),
      options: vec![],
      syntax: "scatter yvar xvar".to_string(),
    }),
    "select" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variables".to_string(),
        required: true,
      }],
      help_topic,
      name: "select".to_string(),
      options: vec![],
      syntax: "select varlist".to_string(),
    }),
    "set" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "name".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "value".to_string(),
          required: true,
        },
      ],
      help_topic,
      name: "set".to_string(),
      options: vec![],
      syntax: "set name value".to_string(),
    }),
    "sort" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variables".to_string(),
        required: true,
      }],
      help_topic,
      name: "sort".to_string(),
      options: vec![],
      syntax: "sort varlist".to_string(),
    }),
    "spregress" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "spregress".to_string(),
      options: vec![
        OptionDescriptor {
          name: "coord".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "weights".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "id".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "contiguity".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "knn".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "model".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
      ],
      syntax: "spregress y x, coord(lat lon)".to_string(),
    }),
    "sql" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "query".to_string(),
        required: true,
      }],
      help_topic,
      name: "sql".to_string(),
      options: vec![],
      syntax: "sql query".to_string(),
    }),
    "status" => Ok(CommandSchemaResult {
      arguments: vec![],
      help_topic,
      name: "status".to_string(),
      options: vec![],
      syntax: "status".to_string(),
    }),
    "streg" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "time_variable".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "variables".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "streg".to_string(),
      options: vec![
        OptionDescriptor {
          name: "failure".to_string(),
          required: true,
        },
        OptionDescriptor {
          name: "dist".to_string(),
          required: true,
        },
      ],
      syntax: "streg time vars, failure(var) dist(name)".to_string(),
    }),
    "summarize" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variables".to_string(),
        required: false,
      }],
      help_topic,
      name: "summarize".to_string(),
      options: vec![],
      syntax: "summarize [varlist]".to_string(),
    }),
    "tabulate" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variable".to_string(),
        required: true,
      }],
      help_topic,
      name: "tabulate".to_string(),
      options: vec![
        OptionDescriptor {
          name: "missing".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "values".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "stat".to_string(),
          required: false,
        },
      ],
      syntax: "tabulate varname".to_string(),
    }),
    "tail" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "n".to_string(),
        required: false,
      }],
      help_topic,
      name: "tail".to_string(),
      options: vec![],
      syntax: "tail [N]".to_string(),
    }),
    "test" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "specification".to_string(),
        required: true,
      }],
      help_topic,
      name: "test".to_string(),
      options: vec![],
      syntax: "test spec".to_string(),
    }),
    "tobit" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "tobit".to_string(),
      options: vec![
        OptionDescriptor {
          name: "ll".to_string(),
          required: true,
        },
        OptionDescriptor {
          name: "ul".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "cluster".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "tobit y x1 x2 [, ll(val) ul(val)]".to_string(),
    }),
    "ttest" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variable".to_string(),
        required: true,
      }],
      help_topic,
      name: "ttest".to_string(),
      options: vec![OptionDescriptor {
        name: "by".to_string(),
        required: false,
      }],
      syntax: "ttest varname [, by(var)]".to_string(),
    }),
    "use" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "path".to_string(),
        required: true,
      }],
      help_topic,
      name: "use".to_string(),
      options: vec![
        OptionDescriptor {
          name: "lazy".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "engine".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "delimiter".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "has_header".to_string(),
          required: false,
        },
      ],
      syntax: "use <path>".to_string(),
    }),
    "xtabond" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "xtabond".to_string(),
      options: vec![OptionDescriptor {
        name: "lags".to_string(),
        required: false,
      }],
      syntax: "xtabond y x [, lags(n)]".to_string(),
    }),
    "xtdata" => Ok(CommandSchemaResult {
      arguments: vec![ArgumentDescriptor {
        name: "variables".to_string(),
        required: true,
      }],
      help_topic,
      name: "xtdata".to_string(),
      options: vec![
        OptionDescriptor {
          name: "within".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "between".to_string(),
          required: false,
        },
      ],
      syntax: "xtdata varlist [, within between]".to_string(),
    }),
    "xtlogit" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "xtlogit".to_string(),
      options: vec![
        OptionDescriptor {
          name: "fe".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "cluster".to_string(),
          required: false,
        },
      ],
      syntax: "xtlogit y x [, fe]".to_string(),
    }),
    "xtreg" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "xtreg".to_string(),
      options: vec![
        OptionDescriptor {
          name: "fe".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "re".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "cluster".to_string(),
          required: false,
        },
      ],
      syntax: "xtreg y x [, fe re]".to_string(),
    }),
    "zinb" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "zinb".to_string(),
      options: vec![
        OptionDescriptor {
          name: "inflate".to_string(),
          required: true,
        },
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "cluster".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "zinb y x1 x2, inflate(vars)".to_string(),
    }),
    "zip" => Ok(CommandSchemaResult {
      arguments: vec![
        ArgumentDescriptor {
          name: "depvar".to_string(),
          required: true,
        },
        ArgumentDescriptor {
          name: "indepvars".to_string(),
          required: false,
        },
      ],
      help_topic,
      name: "zip".to_string(),
      options: vec![
        OptionDescriptor {
          name: "inflate".to_string(),
          required: true,
        },
        OptionDescriptor {
          name: "robust".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "cluster".to_string(),
          required: false,
        },
        OptionDescriptor {
          name: "noconstant".to_string(),
          required: false,
        },
      ],
      syntax: "zip y x1 x2, inflate(vars)".to_string(),
    }),
    _ => Err(format!("unknown command name: {name}")),
  }
}

pub fn help_topic_result(topic: &str) -> Result<HelpTopicResult, String> {
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

pub fn preview_command_name(command_text: &str) -> String {
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_catalog_counts_and_sorting() {
    let catalog = command_catalog_result();
    assert_eq!(catalog.commands.len(), 81);
    for window in catalog.commands.windows(2) {
      assert!(window[0].name < window[1].name);
    }
    // Verify lincom, test, ttest are in catalog but have no help topic
    let lincom_entry = catalog
      .commands
      .iter()
      .find(|e| e.name == "lincom")
      .unwrap();
    assert_eq!(lincom_entry.help_topic, None);
    let summarize_entry = catalog
      .commands
      .iter()
      .find(|e| e.name == "summarize")
      .unwrap();
    assert_eq!(summarize_entry.help_topic.as_deref(), Some("summarize"));
  }

  #[test]
  fn test_command_effects_coverage() {
    let effects = command_effect_catalog_result();
    assert_eq!(effects.commands.len(), 81);
    let valid_categories = ["read", "write", "control", "plot", "unknown"];
    for entry in &effects.commands {
      assert!(!entry.effects.is_empty());
      for eff in &entry.effects {
        assert!(valid_categories.contains(&eff.as_str()));
      }
    }
  }

  #[test]
  fn test_describe_command() {
    let summarize = describe_command_result("summarize").unwrap();
    assert_eq!(summarize.name, "summarize");
    assert_eq!(summarize.help_topic.as_deref(), Some("summarize"));
    assert_eq!(summarize.arguments.len(), 1);
    assert_eq!(summarize.arguments[0].name, "variables");
    assert!(!summarize.arguments[0].required);
    assert!(summarize.options.is_empty());

    assert_eq!(
      describe_command_result("   ").unwrap_err(),
      "command name cannot be empty"
    );
    assert_eq!(
      describe_command_result("does_not_exist").unwrap_err(),
      "unknown command name: does_not_exist"
    );
  }

  #[test]
  fn test_explain_result() {
    let res = explain_result("summarize age").unwrap();
    assert_eq!(res.command_name, "summarize");
    assert_eq!(res.execution, "not_run");

    let err = explain_result("not_a_valid_command").unwrap_err();
    assert_eq!(err.1, "ParseError");
    assert!(err.0.contains("unknown command"));
  }
}
