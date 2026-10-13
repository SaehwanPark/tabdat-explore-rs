//! Replay of pinned script APIs, with the five explicit MIG-0005 seed deviations.
use serde_json::{Value, json};
use std::{collections::HashMap, path::Path};
use tabdat_language::script::{
  ControlFlowDirective, ScriptContext, ScriptDirective, ScriptError, evaluate_script_condition,
  expand_script_macros, parse_control_flow_directive, parse_script, parse_script_directive,
  read_script,
};

fn tagged(name: &str, fields: Value) -> Value {
  json!({"type": name, "fields": fields})
}

fn replay(case: &Value) -> Result<Value, ScriptError> {
  let inputs = &case["inputs"];
  let path = Path::new(inputs["path"].as_str().unwrap());
  let line = inputs["line"].as_u64().unwrap_or(1) as usize;
  let text = inputs["text"].as_str().unwrap_or("");
  let context_fields = &inputs["context"]["fields"];
  let macros: HashMap<String, String> = context_fields["macros"]
    .as_object()
    .map(|values| {
      values
        .iter()
        .map(|(key, value)| (key.clone(), value.as_str().unwrap().to_owned()))
        .collect()
    })
    .unwrap_or_default();
  let seed = context_fields["seed"]
    .as_str()
    .map(|value| value.parse().unwrap());
  let context = ScriptContext::new(macros, seed);
  match case["operation"].as_str().unwrap() {
    "parse_script" => parse_script(text, path).map(|commands| {
      Value::Array(
        commands
          .into_iter()
          .map(|command| {
            tagged(
              "ScriptCommand",
              json!({"text": command.text, "start_line": command.start_line}),
            )
          })
          .collect(),
      )
    }),
    "expand_script_macros" => expand_script_macros(text, &context, path, line).map(Value::String),
    "evaluate_script_condition" => {
      evaluate_script_condition(inputs["condition"].as_str().unwrap(), path, line).map(Value::Bool)
    }
    "parse_script_directive" => {
      parse_script_directive(text, &context, path, line).map(|directive| match directive {
        None => Value::Null,
        Some(ScriptDirective::Seed(seed)) => {
          tagged("SeedDirective", json!({"value": seed.value.to_string()}))
        }
        Some(ScriptDirective::Let(value)) => tagged(
          "LetDirective",
          json!({"name": value.name, "value": value.value}),
        ),
      })
    }
    "parse_control_flow_directive" => {
      parse_control_flow_directive(text, path, line).map(|directive| match directive {
        None => Value::Null,
        Some(ControlFlowDirective::If(value)) => {
          tagged("IfDirective", json!({"active": value.active}))
        }
        Some(ControlFlowDirective::Else(_)) => tagged("ElseDirective", json!({})),
        Some(ControlFlowDirective::End(_)) => tagged("EndDirective", json!({})),
      })
    }
    "read_script" => {
      // Recreate the synthetic missing-file fixture in an isolated directory.
      assert!(case["file_text"].is_null());
      let directory =
        std::env::temp_dir().join(format!("tabdat-script-oracle-{}", std::process::id()));
      std::fs::create_dir_all(&directory).unwrap();
      let result = read_script(&directory.join(path));
      std::fs::remove_dir(&directory).unwrap();
      result.map(|_| unreachable!("captured fixture is missing"))
    }
    other => panic!("uncaptured operation: {other}"),
  }
}

#[test]
fn pinned_python_script_fields_outputs_and_diagnostics() {
  let corpus: Value = serde_json::from_str(include_str!("fixtures/python-script.json")).unwrap();
  assert_eq!(
    corpus["revision"],
    "16b45d9b66b0d80f32d4d220e84d81bc5180bdbe"
  );
  let cases = corpus["cases"].as_array().unwrap();
  assert_eq!(cases.len(), 91);
  // Keep Python outputs intact. Only these exact inputs may use the documented
  // Rust rejection; new differences still fail rather than being normalized away.
  let seed_deviations = [
    "seed -9223372036854775809",
    "seed 1_000",
    "seed 9223372036854775808",
    "seed 99999999999999999999999999999999999999999999999999999999999999999999999999999999",
    "seed ٠١٢",
  ];
  let mut deviations_seen = Vec::new();
  let mut differences = Vec::new();
  for case in cases {
    let actual = match replay(case) {
      Ok(result) => json!({"result": result}),
      Err(error) => json!({"error": {
        "path": error.path().file_name().unwrap().to_str().unwrap(),
        "line": error.line(), "message": error.message()
      }}),
    };
    let expected = if case.get("error").is_some() {
      json!({"error": case["error"]})
    } else {
      json!({"result": case["result"]})
    };
    let text = case["inputs"]["text"].as_str().unwrap_or("");
    if case["operation"] == "parse_script_directive" && seed_deviations.contains(&text) {
      assert_eq!(case["result"]["type"], "SeedDirective");
      assert_ne!(actual, expected, "MIG-0005 changed: {text}");
      assert_eq!(
        actual,
        json!({"error": {
          "path": "boundary.td", "line": 4, "message": "seed expects an integer"
        }}),
        "MIG-0005 rejection changed: {text}"
      );
      deviations_seen.push(text);
    } else if actual != expected {
      differences.push(format!(
        "{} {}: expected {expected}, got {actual}",
        case["operation"], case["inputs"]
      ));
    }
  }
  assert_eq!(deviations_seen, seed_deviations);
  assert!(
    differences.is_empty(),
    "script differences:\n{}",
    differences.join("\n")
  );
}
