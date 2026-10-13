use crate::*;

impl Session {
  /// Execute a SQL query, either returning a TableResult or registering and activating a named table.
  pub fn execute_sql(
    &mut self,
    query: &str,
    into: Option<&str>,
  ) -> Result<ExecutionResult, RuntimeError> {
    if self.active_dataset.is_none() {
      return Err(RuntimeError::NoActiveDataset { command: "sql" });
    }
    validate_sql_query(query)?;

    if let Some(target_table) = into {
      let backend = self
        .backend
        .as_mut()
        .ok_or(RuntimeError::BackendInitialization)?;
      let next_dataset = backend.create_named_table_from_sql(query, target_table)?;
      self.label_metadata = LabelMetadata::default();
      self.active_dataset = Some(next_dataset.clone());
      self.active_table_name = Some(target_table.to_owned());
      self
        .named_tables
        .insert(target_table.to_owned(), next_dataset.clone());
      Ok(ExecutionResult::SqlCreate(SqlCreateResult {
        table_name: target_table.to_owned(),
        dataset: next_dataset,
      }))
    } else {
      let backend = self
        .backend
        .as_ref()
        .ok_or(RuntimeError::BackendInitialization)?;
      let (headers, rows) = backend.run_sql(query)?;
      Ok(ExecutionResult::Table(TableResult { headers, rows }))
    }
  }
  /// Execute a join command against an active dataset and a named table.
  pub fn execute_join(&mut self, command: &JoinCommand) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "join" })?
      .clone();
    let right_dataset = self
      .named_tables
      .get(&command.table_name)
      .ok_or_else(|| RuntimeError::UnknownTable {
        name: command.table_name.clone(),
      })?
      .clone();

    let backend = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::BackendInitialization)?;

    let next_dataset = backend.join_named_table(&dataset, &right_dataset, command)?;

    self.retain_label_metadata(&next_dataset);
    let synced = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Join(JoinResult { dataset: synced }))
  }
  /// Execute an append command against an active dataset and a named table.
  pub fn execute_append(&mut self, table_name: &str) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "append" })?
      .clone();
    let append_dataset = self
      .named_tables
      .get(table_name)
      .ok_or_else(|| RuntimeError::UnknownTable {
        name: table_name.to_owned(),
      })?
      .clone();

    let backend = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::BackendInitialization)?;

    let next_dataset = backend.append_named_table(&dataset, &append_dataset, table_name)?;

    self.retain_label_metadata(&next_dataset);
    self.active_table_name = None;
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Append(AppendResult {
      dataset: next_dataset,
    }))
  }
  /// Execute a reshape command against the active dataset.
  pub fn execute_reshape(
    &mut self,
    command: &ReshapeCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "reshape" })?
      .clone();

    let backend = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::BackendInitialization)?;

    let next_dataset = match command.direction {
      ReshapeDirection::Long => backend.reshape_long(
        &dataset,
        &command.variables,
        &command.identifiers,
        &command.j_variable,
      )?,
      ReshapeDirection::Wide => backend.reshape_wide(
        &dataset,
        &command.variables,
        &command.identifiers,
        &command.j_variable,
      )?,
    };

    self.retain_label_metadata(&next_dataset);
    self.active_table_name = None;
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Reshape(ReshapeResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn sync_active_dataset(
    &mut self,
    dataset: DatasetInfo,
  ) -> Result<DatasetInfo, RuntimeError> {
    if let Some(ref current_table_name) = self.active_table_name {
      let backend = self
        .backend
        .as_mut()
        .ok_or(RuntimeError::BackendInitialization)?;
      backend
        .store_active_as_named_table(current_table_name)
        .map_err(|_| RuntimeError::BackendInitialization)?;
      self
        .named_tables
        .insert(current_table_name.clone(), dataset.clone());
    }
    self.active_dataset = Some(dataset.clone());
    Ok(dataset)
  }
  pub(crate) fn execute_generate(
    &mut self,
    variable: String,
    expression: GenerateExpression,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "generate",
      })?
      .clone();
    if dataset.columns.iter().any(|column| column.name == variable) {
      return Err(RuntimeError::GenerateTargetExists { variable });
    }

    let unknown = generate_expression_identifiers(&expression)
      .into_iter()
      .filter(|name| !dataset.columns.iter().any(|column| column.name == *name))
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::GenerateUnknownVariable { variables: unknown });
    }

    let expression = generate_expression_to_assert(&expression)?;
    match expression_domain(&expression, &dataset) {
      Ok(ExpressionDomain::Numeric) => {}
      Ok(_) => {
        return Err(RuntimeError::GenerateTypeMismatch {
          message: "expression type mismatch: arithmetic requires numeric operands".to_owned(),
        });
      }
      Err(RuntimeError::AssertTypeMismatch { message }) => {
        return Err(RuntimeError::GenerateTypeMismatch { message });
      }
      Err(RuntimeError::AssertUnknownVariable { variables }) => {
        return Err(RuntimeError::GenerateUnknownVariable { variables });
      }
      Err(_) => return Err(RuntimeError::GenerateFailed),
    }
    validate_generate_expression(&expression, &dataset)?;
    let backend = self.backend.as_mut().ok_or(RuntimeError::GenerateFailed)?;
    let next_dataset = backend
      .generate_column(&dataset, &variable, &expression)
      .map_err(|_| RuntimeError::GenerateFailed)?;
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Generate(GenerateResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn execute_replace(
    &mut self,
    variable: String,
    expression: GenerateExpression,
    condition: Option<GenerateExpression>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "replace" })?
      .clone();
    if !dataset.columns.iter().any(|column| column.name == variable) {
      return Err(RuntimeError::ReplaceTargetUnknownVariable { variable });
    }

    let replacement = replace_expression_to_assert(&expression)?;
    let condition = condition
      .as_ref()
      .map(replace_expression_to_assert)
      .transpose()?;
    let identifiers = replace_expression_identifiers(&expression)
      .into_iter()
      .chain(
        condition
          .as_ref()
          .map(expression_identifiers)
          .into_iter()
          .flatten(),
      )
      .filter(|name| !dataset.columns.iter().any(|column| column.name == *name))
      .collect::<Vec<_>>();
    if !identifiers.is_empty() {
      return Err(RuntimeError::ReplaceUnknownVariable {
        variables: identifiers,
      });
    }

    let target_type = dataset
      .columns
      .iter()
      .find(|column| column.name == variable)
      .map(|column| column.data_type.as_str())
      .ok_or_else(|| RuntimeError::ReplaceTargetUnknownVariable {
        variable: variable.clone(),
      })?;
    let target_domain = data_type_expression_domain(target_type);
    if !matches!(
      target_domain,
      ExpressionDomain::Numeric | ExpressionDomain::String
    ) {
      return Err(RuntimeError::ReplaceTypeMismatch {
        message: format!(
          "replace target {variable} has unsupported domain: {}",
          expression_domain_name(target_domain)
        ),
      });
    }

    let replacement_domain =
      expression_domain(&replacement, &dataset).map_err(map_replace_error)?;
    if replacement_domain != ExpressionDomain::Null && replacement_domain != target_domain {
      return Err(RuntimeError::ReplaceTypeMismatch {
        message: format!(
          "replace target {variable} is {} but expression is {}",
          expression_domain_name(target_domain),
          expression_domain_name(replacement_domain)
        ),
      });
    }
    if let Some(condition) = condition.as_ref() {
      let condition_domain = expression_domain(condition, &dataset).map_err(map_replace_error)?;
      if !matches!(
        condition_domain,
        ExpressionDomain::Boolean | ExpressionDomain::Null
      ) {
        return Err(RuntimeError::ReplaceRequiresBoolean);
      }
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::ReplaceFailed)?;
    let next_dataset = backend
      .replace_column(
        &dataset,
        &variable,
        target_type,
        &replacement,
        condition.as_ref(),
      )
      .map_err(|_| RuntimeError::ReplaceFailed)?;
    self.remove_value_label_attachment(&variable);
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Replace(ReplaceResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn execute_rename(
    &mut self,
    old_name: String,
    new_name: String,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "rename" })?
      .clone();
    if !dataset.columns.iter().any(|column| column.name == old_name) {
      return Err(RuntimeError::RenameUnknownVariable { variable: old_name });
    }
    if dataset.columns.iter().any(|column| column.name == new_name) {
      return Err(RuntimeError::RenameTargetExists { variable: new_name });
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::RenameFailed)?;
    let next_dataset = backend
      .rename_column(&dataset, &old_name, &new_name)
      .map_err(|_| RuntimeError::RenameFailed)?;
    self.rename_label_metadata(&old_name, &new_name);
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Rename(RenameResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn execute_sort(
    &mut self,
    variables: Vec<String>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "sort" })?
      .clone();
    if variables.is_empty() {
      return Err(RuntimeError::SortNoVariables);
    }
    let unknown = variables
      .iter()
      .filter(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      })
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::SortUnknownVariable { variables: unknown });
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::SortFailed)?;
    let next_dataset = backend
      .sort_rows(&dataset, &variables)
      .map_err(|_| RuntimeError::SortFailed)?;
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Sort(SortResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn execute_gsort(
    &mut self,
    keys: Vec<SortKey>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "gsort" })?
      .clone();
    if keys.is_empty() {
      return Err(RuntimeError::GsortNoVariables);
    }
    let variables = keys
      .iter()
      .map(|key| key.variable.clone())
      .collect::<Vec<_>>();
    let directions = keys.iter().map(|key| key.descending).collect::<Vec<_>>();
    let unknown = variables
      .iter()
      .filter(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      })
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::GsortUnknownVariable { variables: unknown });
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::GsortFailed)?;
    let next_dataset = backend
      .sort_rows_directed(&dataset, &variables, &directions)
      .map_err(|_| RuntimeError::GsortFailed)?;
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Gsort(GsortResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn execute_recode(
    &mut self,
    variables: Vec<String>,
    rules: Vec<RecodeRule>,
    target: RecodeTarget,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "recode" })?
      .clone();
    if variables.is_empty() {
      return Err(RuntimeError::RecodeNoVariables);
    }
    if rules.is_empty() {
      return Err(RuntimeError::RecodeNoRules);
    }

    let unknown = variables
      .iter()
      .filter(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      })
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::RecodeUnknownVariable { variables: unknown });
    }

    if let Some(range_variable) = variables.iter().find(|variable| {
      let data_type = dataset
        .columns
        .iter()
        .find(|column| column.name == **variable)
        .map(|column| column.data_type.as_str())
        .expect("source variable validation precedes range validation");
      !is_numeric_data_type(data_type)
        && rules.iter().any(|rule| {
          rule
            .inputs
            .iter()
            .any(|input| matches!(input, RecodeInput::Range { .. }))
        })
    }) {
      return Err(RuntimeError::RecodeRangeRequiresNumeric {
        variable: (*range_variable).clone(),
      });
    }

    if let RecodeTarget::Generate {
      variables: generated,
    } = &target
    {
      if generated.len() != variables.len() {
        return Err(RuntimeError::RecodeGenerateCountMismatch);
      }
      if generated.is_empty() {
        return Err(RuntimeError::RecodeGenerateCountMismatch);
      }
      if generated
        .iter()
        .enumerate()
        .any(|(index, variable)| generated[..index].contains(variable))
      {
        return Err(RuntimeError::RecodeGenerateDuplicateVariable);
      }
      if let Some(variable) = generated.iter().find(|variable| {
        dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      }) {
        return Err(RuntimeError::RecodeGenerateTargetExists {
          variable: variable.clone(),
        });
      }
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::RecodeFailed)?;
    let next_dataset = backend
      .recode_columns(&dataset, &variables, &rules, &target)
      .map_err(|_| RuntimeError::RecodeFailed)?;
    if matches!(target, RecodeTarget::Replace) {
      for variable in &variables {
        self.remove_value_label_attachment(variable);
      }
    }
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Recode(RecodeResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn execute_by(&self, command: ByCommand) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "by" })?;
    if command.groups.is_empty() {
      return Err(RuntimeError::ByNoGroups);
    }

    let known_variables = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<BTreeSet<_>>();
    let unknown_groups = command
      .groups
      .iter()
      .filter(|group| !known_variables.contains(group.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown_groups.is_empty() {
      return Err(RuntimeError::ByUnknownVariable {
        variables: unknown_groups,
      });
    }

    match *command.command {
      Command::Summarize { variables } => {
        let unknown_variables = variables
          .iter()
          .filter(|variable| !known_variables.contains(variable.as_str()))
          .cloned()
          .collect::<Vec<_>>();
        if !unknown_variables.is_empty() {
          return Err(RuntimeError::ByUnknownVariable {
            variables: unknown_variables,
          });
        }

        let requested = if variables.is_empty() {
          dataset
            .columns
            .iter()
            .filter(|column| {
              is_numeric_data_type(&column.data_type)
                && !command.groups.iter().any(|group| group == &column.name)
            })
            .map(|column| column.name.clone())
            .collect::<Vec<_>>()
        } else {
          variables
        };
        if requested.is_empty() {
          return Err(RuntimeError::BySummarizeNoNumericColumns);
        }

        let non_numeric = requested
          .iter()
          .filter(|variable| {
            dataset
              .columns
              .iter()
              .find(|column| column.name == **variable)
              .is_none_or(|column| !is_numeric_data_type(&column.data_type))
          })
          .cloned()
          .collect::<Vec<_>>();
        if !non_numeric.is_empty() {
          return Err(RuntimeError::BySummarizeRequiresNumeric {
            variables: non_numeric,
          });
        }

        let backend = self.backend.as_ref().ok_or(RuntimeError::ByFailed)?;
        let rows = backend
          .grouped_summarize(&command.groups, &requested)
          .map_err(|_| RuntimeError::ByFailed)?;
        let headers = command
          .groups
          .iter()
          .cloned()
          .chain(requested.iter().map(|variable| format!("mean_{variable}")))
          .collect();
        Ok(ExecutionResult::By(ByResult { headers, rows }))
      }
      Command::Count => {
        let backend = self.backend.as_ref().ok_or(RuntimeError::ByFailed)?;
        let rows = backend
          .grouped_count(&command.groups)
          .map_err(|_| RuntimeError::ByFailed)?;
        let headers = command
          .groups
          .iter()
          .cloned()
          .chain(std::iter::once("Count".to_owned()))
          .collect();
        Ok(ExecutionResult::By(ByResult { headers, rows }))
      }
      _ => Err(RuntimeError::ByUnsupportedCommand),
    }
  }
  pub(crate) fn execute_collapse(
    &mut self,
    command: CollapseCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "collapse",
      })?
      .clone();
    if command.variables.is_empty() {
      return Err(RuntimeError::CollapseNoVariables);
    }
    if command.groups.is_empty() {
      return Err(RuntimeError::CollapseNoGroups);
    }

    let known_columns = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<BTreeSet<_>>();
    let unknown = command
      .groups
      .iter()
      .chain(&command.variables)
      .filter(|variable| !known_columns.contains(variable.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::CollapseUnknownVariable { variables: unknown });
    }

    if command.statistic != CollapseStatistic::Count {
      let non_numeric = command
        .variables
        .iter()
        .filter(|variable| {
          dataset
            .columns
            .iter()
            .find(|column| column.name == **variable)
            .is_some_and(|column| !is_numeric_data_type(&column.data_type))
        })
        .cloned()
        .collect::<Vec<_>>();
      if !non_numeric.is_empty() {
        return Err(RuntimeError::CollapseRequiresNumeric {
          variables: non_numeric,
        });
      }
    }

    let next_dataset = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::CollapseFailed)?
      .collapse(
        &dataset,
        command.statistic,
        &command.variables,
        &command.groups,
      )
      .map_err(|_| RuntimeError::CollapseFailed)?;
    self.retain_label_metadata(&next_dataset);
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Collapse(CollapseResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn execute_keep(
    &mut self,
    variables: Vec<String>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "keep" })?
      .clone();
    if variables.is_empty() {
      return Err(RuntimeError::KeepNoVariables);
    }
    let unknown = variables
      .iter()
      .filter(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      })
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::KeepUnknownVariable { variables: unknown });
    }
    let backend = self.backend.as_mut().ok_or(RuntimeError::KeepFailed)?;
    let next_dataset = backend
      .project_columns(&dataset, &variables)
      .map_err(|_| RuntimeError::KeepFailed)?;
    self.retain_label_metadata(&next_dataset);
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Keep(KeepResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn execute_drop(
    &mut self,
    variables: Vec<String>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "drop" })?
      .clone();
    if variables.is_empty() {
      return Err(RuntimeError::DropNoVariables);
    }
    let unknown = variables
      .iter()
      .filter(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      })
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::DropUnknownVariable { variables: unknown });
    }
    let remaining = dataset
      .columns
      .iter()
      .filter(|column| !variables.iter().any(|variable| variable == &column.name))
      .map(|column| column.name.clone())
      .collect::<Vec<_>>();
    if remaining.is_empty() {
      return Err(RuntimeError::DropWouldRemoveEveryColumn);
    }
    let backend = self.backend.as_mut().ok_or(RuntimeError::DropFailed)?;
    let next_dataset = backend
      .project_columns(&dataset, &remaining)
      .map_err(|_| RuntimeError::DropFailed)?;
    self.retain_label_metadata(&next_dataset);
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Drop(DropResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn execute_select(
    &mut self,
    variables: Vec<String>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "select" })?
      .clone();
    if variables.is_empty() {
      return Err(RuntimeError::SelectNoVariables);
    }
    let unknown = variables
      .iter()
      .filter(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      })
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::SelectUnknownVariable { variables: unknown });
    }
    let backend = self.backend.as_mut().ok_or(RuntimeError::SelectFailed)?;
    let next_dataset = backend
      .project_columns(&dataset, &variables)
      .map_err(|_| RuntimeError::SelectFailed)?;
    self.retain_label_metadata(&next_dataset);
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Select(SelectResult {
      dataset: next_dataset,
    }))
  }
  pub(crate) fn execute_save(
    &self,
    raw_path: String,
    replace: bool,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "save" })?;
    let path = PathBuf::from(raw_path);
    tabdat_io::prepare_output_path(&path, replace, tabdat_io::OutputFormat::Parquet)?;

    self
      .backend
      .as_ref()
      .ok_or_else(|| RuntimeError::SaveFailed { path: path.clone() })?
      .save_active_parquet(&path)
      .map_err(|_| RuntimeError::SaveFailed { path: path.clone() })?;

    let saved_dataset = DatasetInfo {
      source: path.clone(),
      row_count: dataset.row_count,
      columns: dataset.columns.clone(),
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    };
    Ok(ExecutionResult::Save(SaveResult {
      path,
      dataset: saved_dataset,
    }))
  }
  pub(crate) fn execute_export(
    &self,
    raw_path: String,
    replace: bool,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "export" })?;
    let path = PathBuf::from(raw_path);
    tabdat_io::prepare_output_path(&path, replace, tabdat_io::OutputFormat::Csv)?;

    self
      .backend
      .as_ref()
      .ok_or_else(|| RuntimeError::ExportFailed { path: path.clone() })?
      .export_active_csv(&path)
      .map_err(|_| RuntimeError::ExportFailed { path: path.clone() })?;

    let exported_dataset = DatasetInfo {
      source: path.clone(),
      row_count: dataset.row_count,
      columns: dataset.columns.clone(),
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    };
    Ok(ExecutionResult::Export(ExportResult {
      path,
      dataset: exported_dataset,
    }))
  }
}
