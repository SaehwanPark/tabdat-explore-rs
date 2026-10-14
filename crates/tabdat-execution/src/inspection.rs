use crate::*;

impl Session {
  pub(crate) fn execute_describe(&self) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "describe",
      })?;
    Ok(ExecutionResult::Describe(DescribeResult {
      dataset: dataset.clone(),
    }))
  }
  pub(crate) fn execute_count(&self) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "count" })?
      .clone();
    Ok(ExecutionResult::Count(CountResult {
      row_count: dataset.row_count,
    }))
  }
  pub(crate) fn execute_summarize(
    &self,
    variables: Vec<String>,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.execute_summary_query(variables, None)
  }
  pub(crate) fn execute_summary_query(
    &self,
    variables: Vec<String>,
    condition: Option<GenerateExpression>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "summarize",
      })?;
    let predicate = condition
      .as_ref()
      .map(|condition| {
        let predicate = bounded_expression_to_assert(condition)
          .ok_or(RuntimeError::SummaryUnsupportedPredicate)?;
        let unknown = expression_identifiers(&predicate)
          .into_iter()
          .filter(|name| !dataset.columns.iter().any(|column| column.name == *name))
          .collect::<Vec<_>>();
        if !unknown.is_empty() {
          return Err(RuntimeError::SummaryUnknownVariable { variables: unknown });
        }
        let domain = expression_domain(&predicate, dataset).map_err(|error| match error {
          RuntimeError::AssertTypeMismatch { message } => {
            RuntimeError::SummaryPredicateTypeMismatch { message }
          }
          other => other,
        })?;
        if !matches!(domain, ExpressionDomain::Boolean | ExpressionDomain::Null) {
          return Err(RuntimeError::SummaryRequiresBoolean);
        }
        Ok(predicate)
      })
      .transpose()?;
    let column_types = dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect::<std::collections::HashMap<_, _>>();
    let unknown = variables
      .iter()
      .filter(|variable| !column_types.contains_key(variable.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::SummaryUnknownVariable { variables: unknown });
    }

    let requested = if variables.is_empty() {
      dataset
        .columns
        .iter()
        .filter(|column| is_numeric_data_type(&column.data_type))
        .map(|column| column.name.clone())
        .collect::<Vec<_>>()
    } else {
      variables
    };
    if requested.is_empty() {
      return Err(RuntimeError::SummaryNoNumericColumns);
    }

    let non_numeric = requested
      .iter()
      .filter(|variable| {
        column_types
          .get(variable.as_str())
          .is_none_or(|data_type| !is_numeric_data_type(data_type))
      })
      .cloned()
      .collect::<Vec<_>>();
    if !non_numeric.is_empty() {
      return Err(RuntimeError::SummaryRequiresNumeric {
        variables: non_numeric,
      });
    }

    let backend = self
      .backend
      .as_ref()
      .ok_or_else(|| RuntimeError::SummaryFailed {
        variable: requested[0].clone(),
      })?;
    let rows = requested
      .iter()
      .map(|variable| {
        backend
          .summarize_variable_if(
            variable,
            predicate.as_ref().map(|predicate| (predicate, dataset)),
          )
          .map_err(|_| RuntimeError::SummaryFailed {
            variable: variable.clone(),
          })
      })
      .collect::<Result<Vec<_>, _>>()?;
    Ok(ExecutionResult::Summarize(SummarizeResult { rows }))
  }
  pub(crate) fn execute_codebook(
    &self,
    variables: Vec<String>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "codebook",
      })?;
    let column_types = dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect::<std::collections::HashMap<_, _>>();
    let unknown = variables
      .iter()
      .filter(|variable| !column_types.contains_key(variable.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::CodebookUnknownVariable { variables: unknown });
    }

    let requested = if variables.is_empty() {
      dataset
        .columns
        .iter()
        .map(|column| column.name.clone())
        .collect::<Vec<_>>()
    } else {
      variables
    };
    let backend = self
      .backend
      .as_ref()
      .ok_or_else(|| RuntimeError::CodebookFailed {
        variable: requested
          .first()
          .cloned()
          .unwrap_or_else(|| "<none>".to_owned()),
      })?;
    let rows = requested
      .iter()
      .map(|variable| {
        let data_type = column_types
          .get(variable.as_str())
          .copied()
          .ok_or_else(|| RuntimeError::CodebookFailed {
            variable: variable.clone(),
          })?;
        backend
          .codebook_variable(variable, data_type)
          .map_err(|_| RuntimeError::CodebookFailed {
            variable: variable.clone(),
          })
      })
      .collect::<Result<Vec<_>, _>>()?;
    Ok(ExecutionResult::Codebook(CodebookResult { rows }))
  }
  pub(crate) fn execute_missing(
    &self,
    variables: Vec<String>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "missing" })?;
    let column_types = dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect::<std::collections::HashMap<_, _>>();
    let unknown = variables
      .iter()
      .filter(|variable| !column_types.contains_key(variable.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::MissingUnknownVariable { variables: unknown });
    }

    let requested = if variables.is_empty() {
      dataset
        .columns
        .iter()
        .map(|column| column.name.clone())
        .collect::<Vec<_>>()
    } else {
      variables
    };
    let typed_requested = requested
      .iter()
      .map(|variable| {
        Ok((
          variable.as_str(),
          column_types
            .get(variable.as_str())
            .copied()
            .ok_or(RuntimeError::MissingFailed)?,
        ))
      })
      .collect::<Result<Vec<_>, RuntimeError>>()?;
    let backend = self.backend.as_ref().ok_or(RuntimeError::MissingFailed)?;
    let rows = backend
      .missingness(&typed_requested)
      .map_err(|_| RuntimeError::MissingFailed)?;
    Ok(ExecutionResult::Missing(MissingResult { rows }))
  }
  pub(crate) fn execute_duplicates(
    &self,
    variables: Vec<String>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "duplicates",
      })?;
    let known = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<std::collections::HashSet<_>>();
    let unknown = variables
      .iter()
      .filter(|variable| !known.contains(variable.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::DuplicatesUnknownVariable { variables: unknown });
    }

    let requested = if variables.is_empty() {
      dataset
        .columns
        .iter()
        .map(|column| column.name.clone())
        .collect::<Vec<_>>()
    } else {
      variables
    };
    let backend = self
      .backend
      .as_ref()
      .ok_or(RuntimeError::DuplicatesFailed)?;
    backend
      .duplicates(&requested)
      .map(ExecutionResult::Duplicates)
      .map_err(|_| RuntimeError::DuplicatesFailed)
  }
  pub(crate) fn execute_isid(
    &self,
    variables: Vec<String>,
    missok: bool,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "isid" })?;
    if variables.is_empty() {
      return Err(RuntimeError::IsidNoVariables);
    }
    let known = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<std::collections::HashSet<_>>();
    let unknown = variables
      .iter()
      .filter(|variable| !known.contains(variable.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::IsidUnknownVariable { variables: unknown });
    }

    let backend = self.backend.as_ref().ok_or(RuntimeError::IsidFailed)?;
    let (total_rows, unique_groups, duplicate_groups, duplicate_rows, missing_key_rows) = backend
      .isid_counts(&variables)
      .map_err(|_| RuntimeError::IsidFailed)?;
    if (!missok && missing_key_rows > 0) || duplicate_groups > 0 {
      return Err(RuntimeError::IsidSemanticFailure {
        missing_key_rows,
        duplicate_rows,
        duplicate_groups,
        missok,
      });
    }
    Ok(ExecutionResult::Isid(IsidResult {
      variables,
      total_rows,
      unique_groups,
      missing_key_rows,
      missok,
    }))
  }
  pub(crate) fn execute_datasignature(&self) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "datasignature",
      })?;
    let backend = self
      .backend
      .as_ref()
      .ok_or(RuntimeError::DatasignatureFailed)?;
    let (signature, row_count) = backend
      .datasignature(&dataset.columns)
      .map_err(|_| RuntimeError::DatasignatureFailed)?;
    let column_count =
      u64::try_from(dataset.columns.len()).map_err(|_| RuntimeError::DatasignatureFailed)?;
    Ok(ExecutionResult::Datasignature(DatasignatureResult {
      algorithm: "sha256".to_owned(),
      signature,
      row_count,
      column_count,
    }))
  }
  pub(crate) fn execute_assert(
    &self,
    expression: AssertExpression,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "assert" })?;
    if let Some(variable) = expression_identifiers(&expression)
      .into_iter()
      .find(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == *variable)
      })
    {
      return Err(RuntimeError::AssertUnknownVariable {
        variables: vec![variable],
      });
    }
    let domain = expression_domain(&expression, dataset)?;
    if !matches!(domain, ExpressionDomain::Boolean | ExpressionDomain::Null) {
      return Err(RuntimeError::AssertRequiresBoolean);
    }
    let backend = self.backend.as_ref().ok_or(RuntimeError::AssertFailed)?;
    let (checked, failed) = backend
      .assert_rows(&expression, dataset)
      .map_err(|_| RuntimeError::AssertFailed)?;
    if failed > 0 {
      return Err(RuntimeError::AssertSemanticFailure { checked, failed });
    }
    Ok(ExecutionResult::Assert(AssertResult { checked, failed }))
  }
  pub(crate) fn execute_head(&self, limit: RowLimit) -> Result<ExecutionResult, RuntimeError> {
    self
      .execute_preview(limit, "head", false)
      .map(ExecutionResult::Head)
  }
  pub(crate) fn execute_tail(&self, limit: RowLimit) -> Result<ExecutionResult, RuntimeError> {
    self
      .execute_preview(limit, "tail", true)
      .map(ExecutionResult::Tail)
  }
  pub(crate) fn execute_preview(
    &self,
    limit: RowLimit,
    command: &'static str,
    from_end: bool,
  ) -> Result<PreviewResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command })?;
    let limit = limit
      .as_decimal()
      .parse::<i64>()
      .map_err(|_| RuntimeError::PreviewFailed { command })?;
    let columns = dataset
      .columns
      .iter()
      .map(|column| column.name.clone())
      .collect::<Vec<_>>();
    if limit == 0 {
      return Ok(PreviewResult {
        columns,
        rows: Vec::new(),
      });
    }

    let backend = self
      .backend
      .as_ref()
      .ok_or(RuntimeError::PreviewFailed { command })?;
    let rows = backend
      .preview_rows(limit, dataset.columns.len(), from_end)
      .map_err(|_| RuntimeError::PreviewFailed { command })?;
    Ok(PreviewResult { columns, rows })
  }
  pub(crate) fn execute_use(
    &mut self,
    source: DataSource,
    execution_mode: ExecutionMode,
    lazy_engine: Option<LazyEngine>,
    delimiter: Option<String>,
    has_header: Option<bool>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let DataSource::LocalPath(raw_path) = source else {
      return Err(RuntimeError::UnsupportedUseConfiguration);
    };

    let path = PathBuf::from(&raw_path);
    if self.named_tables.contains_key(&raw_path) {
      if execution_mode != ExecutionMode::Eager
        || lazy_engine.is_some()
        || delimiter.is_some()
        || has_header.is_some()
      {
        return Err(RuntimeError::UseOptionsNotSupportedForNamedTable);
      }
      let backend = self
        .backend
        .as_mut()
        .ok_or(RuntimeError::BackendInitialization)?;
      let activated = backend.activate_named_table(&raw_path)?;
      self.label_metadata = LabelMetadata::default();
      self.active_dataset = Some(activated.clone());
      self.active_table_name = Some(raw_path.clone());
      self
        .named_tables
        .insert(raw_path.clone(), activated.clone());
      return Ok(ExecutionResult::Activate(ActivateResult {
        table_name: raw_path,
        dataset: activated,
      }));
    }

    if execution_mode == ExecutionMode::Eager
      && lazy_engine.is_none()
      && delimiter.is_none()
      && has_header.is_none()
      && !path.exists()
      && path.extension().is_none()
      && !raw_path.contains('/')
      && !raw_path.contains('\\')
    {
      return Err(RuntimeError::UnknownTable { name: raw_path });
    }

    if execution_mode != ExecutionMode::Eager || lazy_engine.is_some() {
      return Err(RuntimeError::UnsupportedUseConfiguration);
    }

    let input_format = match local_input_format(&path) {
      Ok(input_format) => input_format,
      Err(RuntimeError::UnsupportedFormat { .. })
        if delimiter.is_some() || has_header.is_some() =>
      {
        return Err(RuntimeError::UnsupportedUseConfiguration);
      }
      Err(error) => return Err(error),
    };
    if input_format == LocalInputFormat::Parquet && (delimiter.is_some() || has_header.is_some()) {
      return Err(RuntimeError::UnsupportedUseConfiguration);
    }
    validate_local_input_path(&path)?;

    let dataset = if let Some(backend) = self.backend.as_mut() {
      backend.load_eager_input(&path, input_format, delimiter.as_deref(), has_header)?
    } else {
      let mut backend = DuckDbBackend::new()?;
      let dataset =
        backend.load_eager_input(&path, input_format, delimiter.as_deref(), has_header)?;
      self.backend = Some(backend);
      dataset
    };
    self.label_metadata = LabelMetadata::default();
    self.active_table_name = None;
    self.active_dataset = Some(dataset.clone());
    Ok(ExecutionResult::Load(LoadResult { dataset }))
  }
}
