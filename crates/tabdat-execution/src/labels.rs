use crate::*;

impl Session {
  pub(crate) fn execute_encode(
    &mut self,
    source: String,
    generate: String,
    label: Option<String>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "encode" })?
      .clone();
    let source_column = dataset
      .columns
      .iter()
      .find(|column| column.name == source)
      .ok_or_else(|| RuntimeError::EncodeUnknownVariable {
        variable: source.clone(),
      })?;
    if dataset.columns.iter().any(|column| column.name == generate) {
      return Err(RuntimeError::EncodeTargetExists { variable: generate });
    }
    if data_type_expression_domain(&source_column.data_type) != ExpressionDomain::String {
      return Err(RuntimeError::EncodeRequiresString { variable: source });
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::EncodeFailed)?;
    let values = backend
      .distinct_nonmissing_string_values(&source)
      .map_err(|_| RuntimeError::EncodeFailed)?;
    let mapping = values
      .into_iter()
      .enumerate()
      .map(|(index, value)| {
        let code = index
          .checked_add(1)
          .and_then(|value| i64::try_from(value).ok())
          .ok_or(RuntimeError::EncodeFailed)?;
        Ok((value, code))
      })
      .collect::<Result<Vec<_>, RuntimeError>>()?;
    let next_dataset = backend
      .encode_column(&dataset, &source, &generate, &mapping)
      .map_err(|_| RuntimeError::EncodeFailed)?;
    let set_name = label.unwrap_or_else(|| generate.clone());
    let value_mappings = mapping
      .into_iter()
      .map(|(value, code)| (code, value))
      .collect::<Vec<_>>();
    let mut metadata = self.label_metadata.clone();
    upsert_value_label_set(
      &mut metadata,
      ValueLabelSet {
        name: set_name.clone(),
        mappings: value_mappings
          .into_iter()
          .map(|(code, value)| (LabelValue::Integer(code), value))
          .collect(),
      },
    );
    upsert_attachment(&mut metadata, generate.clone(), set_name);
    if let Some(source_label) = variable_label(&metadata, &source) {
      upsert_variable_label(&mut metadata, generate.clone(), source_label);
    }
    self.label_metadata = normalize_label_metadata(metadata);
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Encode(EncodeResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn execute_decode(
    &mut self,
    source: String,
    generate: String,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "decode" })?
      .clone();
    let set_name = self
      .label_metadata
      .attachments
      .iter()
      .find(|(variable, _)| variable == &source)
      .map(|(_, set_name)| set_name.clone())
      .ok_or_else(|| RuntimeError::DecodeRequiresAttachedLabels {
        variable: source.clone(),
      })?;
    let value_set = self
      .label_metadata
      .value_sets
      .iter()
      .find(|value_set| value_set.name == set_name)
      .ok_or_else(|| RuntimeError::DecodeRequiresAttachedLabels {
        variable: source.clone(),
      })?;
    let mapping = value_set
      .mappings
      .iter()
      .map(|(value, text)| match value {
        LabelValue::Integer(code) => Ok((*code, text.clone())),
        LabelValue::Number(_) | LabelValue::Text(_) => {
          Err(RuntimeError::DecodeRequiresIntegerLabels {
            variable: source.clone(),
          })
        }
      })
      .collect::<Result<Vec<_>, RuntimeError>>()?;
    let source_column = dataset
      .columns
      .iter()
      .find(|column| column.name == source)
      .ok_or_else(|| RuntimeError::DecodeUnknownVariable {
        variable: source.clone(),
      })?;
    if dataset.columns.iter().any(|column| column.name == generate) {
      return Err(RuntimeError::DecodeTargetExists { variable: generate });
    }
    if data_type_expression_domain(&source_column.data_type) != ExpressionDomain::Numeric {
      return Err(RuntimeError::DecodeRequiresNumeric { variable: source });
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::DecodeFailed)?;
    let next_dataset = backend
      .decode_column(&dataset, &source, &generate, &mapping)
      .map_err(|_| RuntimeError::DecodeFailed)?;
    if let Some(source_label) = variable_label(&self.label_metadata, &source) {
      let mut metadata = self.label_metadata.clone();
      upsert_variable_label(&mut metadata, generate.clone(), source_label);
      self.label_metadata = normalize_label_metadata(metadata);
    }
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Decode(DecodeResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn execute_label(
    &mut self,
    command: LabelCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "label" })?
      .clone();
    match command {
      LabelCommand::Variable { variable, text } => {
        ensure_label_variable(&dataset, &variable)?;
        let mut metadata = self.label_metadata.clone();
        metadata
          .variable_labels
          .retain(|(name, _)| name != &variable);
        let (action, message) = if let Some(text) = text {
          metadata.variable_labels.push((variable.clone(), text));
          (
            LabelResultAction::Variable,
            format!("Labeled variable {variable}"),
          )
        } else {
          (
            LabelResultAction::Variable,
            format!("Cleared variable label for {variable}"),
          )
        };
        self.label_metadata = normalize_label_metadata(metadata);
        Ok(self.label_result(action, message, &self.label_metadata))
      }
      LabelCommand::Define {
        name,
        mappings,
        replace,
      } => {
        if mappings.is_empty() {
          return Err(RuntimeError::LabelDefineNoMappings);
        }
        let mut seen = BTreeSet::new();
        for (value, _) in &mappings {
          if !seen.insert(value.clone()) {
            return Err(RuntimeError::LabelDefineDuplicateValue {
              set_name: name,
              value: value.clone(),
            });
          }
        }
        if self
          .label_metadata
          .value_sets
          .iter()
          .any(|value_set| value_set.name == name)
          && !replace
        {
          return Err(RuntimeError::LabelDefineSetExists { set_name: name });
        }
        let mut metadata = self.label_metadata.clone();
        upsert_value_label_set(
          &mut metadata,
          ValueLabelSet {
            name: name.clone(),
            mappings,
          },
        );
        self.label_metadata = normalize_label_metadata(metadata);
        let verb = if replace { "Replaced" } else { "Defined" };
        Ok(self.label_result(
          LabelResultAction::Define,
          format!("{verb} value label set {name}"),
          &self.label_metadata,
        ))
      }
      LabelCommand::Values { variable, set_name } => {
        ensure_label_variable(&dataset, &variable)?;
        let mut metadata = self.label_metadata.clone();
        metadata.attachments.retain(|(name, _)| name != &variable);
        let (message, action) = if let Some(set_name) = set_name {
          if !metadata
            .value_sets
            .iter()
            .any(|value_set| value_set.name == set_name)
          {
            return Err(RuntimeError::LabelValuesUnknownSet { set_name });
          }
          metadata
            .attachments
            .push((variable.clone(), set_name.clone()));
          (
            format!("Attached value label set {set_name} to {variable}"),
            LabelResultAction::Values,
          )
        } else {
          (
            format!("Cleared value labels for {variable}"),
            LabelResultAction::Values,
          )
        };
        self.label_metadata = normalize_label_metadata(metadata);
        Ok(self.label_result(action, message, &self.label_metadata))
      }
      LabelCommand::List { names } => {
        let metadata = if names.is_empty() {
          self.label_metadata.clone()
        } else {
          let available = self
            .label_metadata
            .value_sets
            .iter()
            .map(|value_set| value_set.name.as_str())
            .collect::<BTreeSet<_>>();
          let missing = names
            .iter()
            .filter(|name| !available.contains(name.as_str()))
            .cloned()
            .collect::<BTreeSet<_>>();
          if !missing.is_empty() {
            return Err(RuntimeError::LabelListUnknownSet {
              names: missing.into_iter().collect(),
            });
          }
          let wanted = names.iter().collect::<BTreeSet<_>>();
          LabelMetadata {
            variable_labels: self.label_metadata.variable_labels.clone(),
            value_sets: self
              .label_metadata
              .value_sets
              .iter()
              .filter(|value_set| wanted.contains(&value_set.name))
              .cloned()
              .collect(),
            attachments: self
              .label_metadata
              .attachments
              .iter()
              .filter(|(_, set_name)| wanted.contains(set_name))
              .cloned()
              .collect(),
          }
        };
        Ok(self.label_result(
          LabelResultAction::List,
          "Label dictionary".to_owned(),
          &normalize_label_metadata(metadata),
        ))
      }
      LabelCommand::Drop { names } => {
        let available = self
          .label_metadata
          .value_sets
          .iter()
          .map(|value_set| value_set.name.as_str())
          .collect::<BTreeSet<_>>();
        let missing = names
          .iter()
          .filter(|name| !available.contains(name.as_str()))
          .cloned()
          .collect::<BTreeSet<_>>();
        if !missing.is_empty() {
          return Err(RuntimeError::LabelDropUnknownSet {
            names: missing.into_iter().collect(),
          });
        }
        let dropped = names.iter().collect::<BTreeSet<_>>();
        let mut metadata = self.label_metadata.clone();
        metadata
          .value_sets
          .retain(|value_set| !dropped.contains(&value_set.name));
        metadata
          .attachments
          .retain(|(_, set_name)| !dropped.contains(set_name));
        self.label_metadata = normalize_label_metadata(metadata);
        Ok(self.label_result(
          LabelResultAction::Drop,
          format!("Dropped label set(s): {}", names.join(", ")),
          &self.label_metadata,
        ))
      }
    }
  }
  pub(crate) fn label_result(
    &self,
    action: LabelResultAction,
    message: String,
    metadata: &LabelMetadata,
  ) -> ExecutionResult {
    ExecutionResult::Label(LabelResult {
      action,
      message,
      metadata: (!metadata.is_empty()).then_some(metadata.clone()),
    })
  }
  pub(crate) fn execute_tabulate(
    &self,
    command: TabulateCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "tabulate",
      })?;
    if command.row_variables.is_empty() {
      return Err(RuntimeError::TabulateNoVariables);
    }
    if command.row_variables.len() != 1 || command.column_variables.len() > 1 {
      return Err(RuntimeError::TabulateUnsupportedDimensions);
    }

    let row_variable = &command.row_variables[0];
    let column_variable = command.column_variables.first();
    if column_variable.is_some_and(|variable| variable == row_variable) {
      return Err(RuntimeError::TabulateDuplicateVariable {
        variable: row_variable.clone(),
      });
    }
    if column_variable.is_none() && command.row_percent {
      return Err(RuntimeError::TabulatePercentageRequiresTwoWay { option: "row" });
    }
    if column_variable.is_none() && command.column_percent {
      return Err(RuntimeError::TabulatePercentageRequiresTwoWay { option: "col" });
    }

    let known_variables = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<std::collections::BTreeSet<_>>();
    let mut unknown = Vec::new();
    if !known_variables.contains(row_variable.as_str()) {
      unknown.push(row_variable.clone());
    }
    if let Some(variable) = column_variable
      && !known_variables.contains(variable.as_str())
    {
      unknown.push(variable.clone());
    }
    if !unknown.is_empty() {
      return Err(RuntimeError::TabulateUnknownVariable { variables: unknown });
    }

    let backend = self.backend.as_ref().ok_or(RuntimeError::TabulateFailed)?;
    let counts = backend
      .tabulate_counts(
        row_variable,
        column_variable.map(String::as_str),
        command.include_missing,
      )
      .map_err(|_| RuntimeError::TabulateFailed)?;

    if let Some(column_variable) = column_variable {
      let mut row_values = Vec::new();
      let mut column_values = Vec::new();
      for count in &counts {
        push_unique_tabulate_value(&mut row_values, &count.row);
        let Some(column) = count.column.as_ref() else {
          return Err(RuntimeError::TabulateFailed);
        };
        push_unique_tabulate_value(&mut column_values, column);
      }
      row_values.sort_by(tabulate_value_ordering);
      column_values.sort_by(tabulate_value_ordering);

      let mut headers = vec![row_variable.clone()];
      for column in &column_values {
        let displayed = if command.nolabel {
          column.clone()
        } else {
          tabulate_display_value(&self.label_metadata, column_variable, column)
        };
        let label = tabulate_value_text(&displayed);
        headers.push(format!("{label} Count"));
        if command.row_percent {
          headers.push(format!("{label} Row %"));
        }
        if command.column_percent {
          headers.push(format!("{label} Col %"));
        }
      }

      let mut rows = Vec::with_capacity(row_values.len());
      for row_value in &row_values {
        let displayed_row = if command.nolabel {
          row_value.clone()
        } else {
          tabulate_display_value(&self.label_metadata, row_variable, row_value)
        };
        let row_total = counts
          .iter()
          .filter(|entry| tabulate_values_equal(&entry.row, row_value))
          .map(|entry| entry.count)
          .sum::<u64>();
        let mut output = vec![displayed_row];
        for column_value in &column_values {
          let count = counts
            .iter()
            .find(|entry| {
              tabulate_values_equal(&entry.row, row_value)
                && entry
                  .column
                  .as_ref()
                  .is_some_and(|entry_column| tabulate_values_equal(entry_column, column_value))
            })
            .map_or(0, |entry| entry.count);
          output.push(CellValue::SignedInteger(i128::from(count)));
          if command.row_percent {
            output.push(CellValue::Float(if row_total == 0 {
              0.0
            } else {
              100.0 * count as f64 / row_total as f64
            }));
          }
          if command.column_percent {
            let column_total = counts
              .iter()
              .filter(|entry| {
                entry
                  .column
                  .as_ref()
                  .is_some_and(|entry_column| tabulate_values_equal(entry_column, column_value))
              })
              .map(|entry| entry.count)
              .sum::<u64>();
            output.push(CellValue::Float(if column_total == 0 {
              0.0
            } else {
              100.0 * count as f64 / column_total as f64
            }));
          }
        }
        rows.push(output);
      }
      return Ok(ExecutionResult::Tabulate(TabulateResult { headers, rows }));
    }

    let total = counts.iter().map(|entry| entry.count).sum::<u64>();
    let mut rows = Vec::with_capacity(counts.len());
    for entry in counts {
      let displayed = if command.nolabel {
        entry.row
      } else {
        tabulate_display_value(&self.label_metadata, row_variable, &entry.row)
      };
      rows.push(vec![
        displayed,
        CellValue::SignedInteger(i128::from(entry.count)),
        CellValue::Float(if total == 0 {
          0.0
        } else {
          100.0 * entry.count as f64 / total as f64
        }),
      ]);
    }
    Ok(ExecutionResult::Tabulate(TabulateResult {
      headers: vec![
        row_variable.clone(),
        "Count".to_owned(),
        "Percent".to_owned(),
      ],
      rows,
    }))
  }
  pub(crate) fn retain_label_metadata(&mut self, dataset: &DatasetInfo) {
    let columns = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<BTreeSet<_>>();
    let mut metadata = self.label_metadata.clone();
    metadata
      .variable_labels
      .retain(|(variable, _)| columns.contains(variable.as_str()));
    metadata
      .attachments
      .retain(|(variable, _)| columns.contains(variable.as_str()));
    self.label_metadata = normalize_label_metadata(metadata);
  }
  pub(crate) fn remove_value_label_attachment(&mut self, variable: &str) {
    self
      .label_metadata
      .attachments
      .retain(|(name, _)| name != variable);
    self.label_metadata = normalize_label_metadata(self.label_metadata.clone());
  }
  pub(crate) fn rename_label_metadata(&mut self, old_name: &str, new_name: &str) {
    let mut metadata = self.label_metadata.clone();
    for (name, _) in &mut metadata.variable_labels {
      if name == old_name {
        *name = new_name.to_owned();
      }
    }
    for (name, _) in &mut metadata.attachments {
      if name == old_name {
        *name = new_name.to_owned();
      }
    }
    self.label_metadata = normalize_label_metadata(metadata);
  }
}

fn ensure_label_variable(dataset: &DatasetInfo, variable: &str) -> Result<(), RuntimeError> {
  if dataset.columns.iter().any(|column| column.name == variable) {
    Ok(())
  } else {
    Err(RuntimeError::LabelUnknownVariable {
      variable: variable.to_owned(),
    })
  }
}

fn normalize_label_metadata(mut metadata: LabelMetadata) -> LabelMetadata {
  metadata
    .variable_labels
    .sort_by(|left, right| left.0.cmp(&right.0));
  metadata
    .value_sets
    .sort_by(|left, right| left.name.cmp(&right.name));
  metadata
    .attachments
    .sort_by(|left, right| left.0.cmp(&right.0));
  metadata
}

fn variable_label(metadata: &LabelMetadata, variable: &str) -> Option<String> {
  metadata
    .variable_labels
    .iter()
    .find(|(name, _)| name == variable)
    .map(|(_, text)| text.clone())
}

fn upsert_variable_label(metadata: &mut LabelMetadata, variable: String, text: String) {
  metadata
    .variable_labels
    .retain(|(name, _)| name != &variable);
  metadata.variable_labels.push((variable, text));
}

fn upsert_value_label_set(metadata: &mut LabelMetadata, value_set: ValueLabelSet) {
  metadata
    .value_sets
    .retain(|existing| existing.name != value_set.name);
  metadata.value_sets.push(value_set);
}

fn upsert_attachment(metadata: &mut LabelMetadata, variable: String, set_name: String) {
  metadata.attachments.retain(|(name, _)| name != &variable);
  metadata.attachments.push((variable, set_name));
}

fn push_unique_tabulate_value(values: &mut Vec<CellValue>, value: &CellValue) {
  if !values
    .iter()
    .any(|existing| tabulate_values_equal(existing, value))
  {
    values.push(value.clone());
  }
}

fn tabulate_values_equal(left: &CellValue, right: &CellValue) -> bool {
  match (left, right) {
    (CellValue::Float(left), CellValue::Float(right)) => {
      left == right || (left.is_nan() && right.is_nan())
    }
    _ => left == right,
  }
}

fn tabulate_value_ordering(left: &CellValue, right: &CellValue) -> std::cmp::Ordering {
  use std::cmp::Ordering;

  match (left, right) {
    (CellValue::Null, CellValue::Null) => Ordering::Equal,
    (CellValue::Null, _) => Ordering::Greater,
    (_, CellValue::Null) => Ordering::Less,
    (CellValue::Boolean(left), CellValue::Boolean(right)) => left.cmp(right),
    (CellValue::SignedInteger(left), CellValue::SignedInteger(right)) => left.cmp(right),
    (CellValue::UnsignedInteger(left), CellValue::UnsignedInteger(right)) => left.cmp(right),
    (CellValue::Float(left), CellValue::Float(right)) => left.total_cmp(right),
    (
      CellValue::Decimal {
        scale: left_scale,
        value: left,
        ..
      },
      CellValue::Decimal {
        scale: right_scale,
        value: right,
        ..
      },
    ) if left_scale == right_scale => left.cmp(right),
    (CellValue::Text(left), CellValue::Text(right)) => left.cmp(right),
    (CellValue::Bytes(left), CellValue::Bytes(right)) => left.cmp(right),
    _ => tabulate_value_text(left).cmp(&tabulate_value_text(right)),
  }
}

fn tabulate_display_value(
  metadata: &LabelMetadata,
  variable: &str,
  value: &CellValue,
) -> CellValue {
  let Some((_, set_name)) = metadata
    .attachments
    .iter()
    .find(|(name, _)| name == variable)
  else {
    return value.clone();
  };
  let Some(value_set) = metadata
    .value_sets
    .iter()
    .find(|value_set| &value_set.name == set_name)
  else {
    return value.clone();
  };
  value_set
    .mappings
    .iter()
    .find(|(label_value, _)| tabulate_label_matches_cell(label_value, value))
    .map_or_else(|| value.clone(), |(_, text)| CellValue::Text(text.clone()))
}

fn tabulate_label_matches_cell(label: &LabelValue, value: &CellValue) -> bool {
  match (label, value) {
    (LabelValue::Integer(label), CellValue::SignedInteger(value)) => i128::from(*label) == *value,
    (LabelValue::Integer(label), CellValue::UnsignedInteger(value)) => {
      *label >= 0 && u128::try_from(*label).is_ok_and(|label| label == *value)
    }
    (LabelValue::Number(label), CellValue::Float(value)) => label
      .parse::<f64>()
      .is_ok_and(|label| label == *value || (label.is_nan() && value.is_nan())),
    (LabelValue::Text(label), CellValue::Text(value)) => label == value,
    _ => false,
  }
}

fn tabulate_value_text(value: &CellValue) -> String {
  match value {
    CellValue::Null => "missing".to_owned(),
    CellValue::Boolean(value) => value.to_string(),
    CellValue::SignedInteger(value) => value.to_string(),
    CellValue::UnsignedInteger(value) => value.to_string(),
    CellValue::Float(value) => value.to_string(),
    CellValue::Decimal { value, scale, .. } => {
      if *scale == 0 {
        return value.to_string();
      }
      let negative = *value < 0;
      let digits = value.unsigned_abs().to_string();
      let scale = usize::from(*scale);
      let rendered = if digits.len() <= scale {
        format!("0.{}{}", "0".repeat(scale - digits.len()), digits)
      } else {
        let split = digits.len() - scale;
        format!("{}.{}", &digits[..split], &digits[split..])
      };
      if negative {
        format!("-{rendered}")
      } else {
        rendered
      }
    }
    CellValue::Text(value) => value.clone(),
    CellValue::Bytes(value) => format!("0x{}", hex_bytes(value)),
  }
}

fn hex_bytes(bytes: &[u8]) -> String {
  bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
