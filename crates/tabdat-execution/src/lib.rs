#![forbid(unsafe_code)]
//! Session coordination over owned domain contracts and narrow effect adapters.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use tabdat_data::*;
pub use tabdat_domain::*;
use tabdat_io::{
  LocalInputFormat, local_input_format, resolve_script_path, validate_local_input_path,
};
use tabdat_language::script::{
  ControlFlowDirective, ScriptBlockState, ScriptContext, ScriptDirective, expand_script_macros,
  parse_control_flow_directive, parse_script_directive, read_script,
};
mod binary;
mod bootstrap;
mod estimates;
mod fixed_effects;
mod multinomial;
mod random_effects;
mod row_conversion;
use row_conversion::*;
mod count;
mod inspection;
mod labels;
mod nb2;
mod plotting;
mod poisson;
mod quantile;
mod scripts;
mod statistics;
mod transforms;

struct AffineCombination {
  offset: f64,
  weights: Vec<f64>,
}

/// A session holding optional active metadata and a private DuckDB adapter.
pub struct Session {
  backend: Option<DuckDbBackend>,
  active_dataset: Option<DatasetInfo>,
  active_table_name: Option<String>,
  named_tables: HashMap<String, DatasetInfo>,
  label_metadata: LabelMetadata,
  estimation: Option<EstimationState>,
  stored_estimates: BTreeMap<String, EstimateSnapshot>,
}

impl Session {
  /// Construct a session without initializing DuckDB.
  pub fn new() -> Self {
    Self {
      backend: None,
      active_dataset: None,
      active_table_name: None,
      named_tables: HashMap::new(),
      label_metadata: LabelMetadata::default(),
      estimation: None,
      stored_estimates: BTreeMap::new(),
    }
  }
  /// Execute one command supported by this bounded runtime slice.
  pub fn execute(&mut self, command: Command) -> Result<ExecutionResult, RuntimeError> {
    let command_name = command_name(&command);
    match command {
      Command::Use {
        source,
        execution_mode,
        lazy_engine,
        delimiter,
        has_header,
      } => self.execute_use(source, execution_mode, lazy_engine, delimiter, has_header),
      Command::Status => Ok(self.execute_status()),
      Command::Estimates { command } => self.execute_estimates(&command),
      Command::Describe => self.execute_describe(),
      Command::Count => self.execute_count(),
      Command::CountQuery { condition } => self.execute_count_query(condition),
      Command::Summarize { variables } => self.execute_summarize(variables),
      Command::SummarizeQuery {
        variables,
        condition,
        options,
      } if options.is_empty() || summarize_detail_requested(&options) => {
        self.execute_summary_query(variables, condition, summarize_detail_requested(&options))
      }
      Command::Codebook { variables } => self.execute_codebook(variables),
      Command::Missing { variables } => self.execute_missing(variables),
      Command::Duplicates { variables } => self.execute_duplicates(variables),
      Command::Isid { variables, missok } => self.execute_isid(variables, missok),
      Command::Datasignature => self.execute_datasignature(),
      Command::Assert { expression } => self.execute_assert(expression),
      Command::Generate {
        variable,
        expression,
      } => self.execute_generate(variable, expression),
      Command::Replace {
        variable,
        expression,
        condition,
      } => self.execute_replace(variable, expression, condition),
      Command::Rename { old_name, new_name } => self.execute_rename(old_name, new_name),
      Command::Sort { variables } => self.execute_sort(variables),
      Command::Gsort { keys } => self.execute_gsort(keys),
      Command::Recode {
        variables,
        rules,
        target,
      } => self.execute_recode(variables, rules, target),
      Command::Encode {
        source,
        generate,
        label,
      } => self.execute_encode(source, generate, label),
      Command::Decode { source, generate } => self.execute_decode(source, generate),
      Command::Label { command } => self.execute_label(command),
      Command::Tabulate { command } => self.execute_tabulate(command),
      Command::TabulateQuery {
        command,
        condition,
        aggregation: None,
      } => self.execute_frequency_query(command, condition),
      Command::By { command } => self.execute_by(command),
      Command::Collapse { command } => self.execute_collapse(command),
      Command::RowFilter { action, condition } => self.execute_row_filter(action, condition),
      Command::Keep { variables } => self.execute_keep(variables),
      Command::Drop { variables } => self.execute_drop(variables),
      Command::Select { variables } => self.execute_select(variables),
      Command::Save { path, replace } => self.execute_save(path, replace),
      Command::Export { path, replace } => self.execute_export(path, replace),
      Command::Scatter { command } => self.execute_scatter(&command),
      Command::Head { limit } => self.execute_head(limit),
      Command::Tail { limit } => self.execute_tail(limit),
      Command::Run { path } => self.execute_run(path),
      Command::Sql { command } => self.execute_sql(&command.query, command.into.as_deref()),
      Command::Join { command } => self.execute_join(&command),
      Command::Append { table_name } => self.execute_append(&table_name),
      Command::Reshape { command } => self.execute_reshape(&command),
      Command::Regress { command } => self.execute_regress(&command),
      Command::RegressQuery { command, condition } => {
        self.execute_regress_query(&command, &condition)
      }
      Command::IvRegress { command } => self.execute_ivregress(&command),
      Command::XtReg { command } if command.estimator == XtRegEstimator::FixedEffects => {
        self.execute_fixed_effects(&command)
      }
      Command::XtReg { command } if command.id_variable.is_some() => {
        self.execute_random_effects(&command)
      }
      Command::Bootstrap { command } => self.execute_bootstrap(&command),
      Command::QuantileBootstrap { command } => self.execute_quantile_bootstrap(&command),
      Command::Logit { command } => self.execute_logit(&command),
      Command::Probit { command } => self.execute_probit(&command),
      Command::Mlogit { command } => self.execute_mlogit(&command),
      Command::Poisson { command } => self.execute_poisson(&command),
      Command::Nbreg { command } => self.execute_nbreg(&command),
      Command::Qreg { command } => self.execute_qreg(&command),
      Command::Predict { command } => self.execute_predict(&command),
      Command::Lincom { command } => self.execute_lincom(&command),
      Command::Test { command } => self.execute_test(&command),
      Command::Ttest { command } => self.execute_ttest(&command),
      Command::Estat { command } => self.execute_estat(&command),
      _ => Err(RuntimeError::UnsupportedCommand { name: command_name }),
    }
  }
  fn linear_state(&self) -> Option<&LinearRegressionState> {
    self.estimation.as_ref().and_then(EstimationState::linear)
  }
  fn linear_inference_result(&self) -> Option<&tabdat_stats::LeastSquaresResult> {
    if let Some(EstimationState::FixedEffects(state)) = &self.estimation {
      Some(&state.result.within)
    } else {
      self.linear_state().map(|state| &state.result)
    }
  }
  /// Read an owned domain snapshot without initializing a backend.
  /// Panel/operation/materialization tracking are not executed by this slice.
  pub fn state(&self) -> ExecutionState {
    ExecutionState {
      active_dataset: self.active_dataset.clone(),
      active_table_name: self.active_table_name.clone(),
      named_tables: self
        .named_tables
        .iter()
        .map(|(name, dataset)| (name.clone(), dataset.clone()))
        .collect(),
      labels: self.label_metadata.clone(),
      estimation: self.estimation.clone(),
      stored_estimates: self.stored_estimates.clone(),
      ..ExecutionState::default()
    }
  }
  /// Report initialized heavyweight capabilities without initializing them.
  pub fn initialized_capabilities(&self) -> &[Capability] {
    if self.backend.is_some() {
      &[Capability::Data]
    } else {
      &[]
    }
  }

  /// Return the currently published dataset metadata, if any.
  pub fn active_dataset(&self) -> Option<&DatasetInfo> {
    self.active_dataset.as_ref()
  }
  /// Return the result from the current regression attempt, if any.
  /// A new attempt clears the prior result before validation begins.
  pub fn last_regression(&self) -> Option<&tabdat_stats::LeastSquaresResult> {
    self.linear_state().map(|state| &state.result)
  }
  /// Return the currently published named table registry.
  pub fn named_tables(&self) -> &HashMap<String, DatasetInfo> {
    &self.named_tables
  }
  /// Return the name of the currently active named table, if active dataset was loaded from one.
  pub fn active_table_name(&self) -> Option<&str> {
    self.active_table_name.as_deref()
  }
  /// Return the currently published session-local label metadata, if any.
  pub fn active_label_metadata(&self) -> Option<&LabelMetadata> {
    (!self.label_metadata.is_empty()).then_some(&self.label_metadata)
  }
}

impl Default for Session {
  fn default() -> Self {
    Self::new()
  }
}

fn command_name(command: &Command) -> &'static str {
  command.name()
}

#[cfg(test)]
mod tests;
