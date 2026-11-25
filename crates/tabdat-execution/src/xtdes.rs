use crate::*;
use std::collections::BTreeMap;

pub(crate) const XT_DES_MAX_PERIODS: usize = 512;

impl Session {
  pub(crate) fn execute_xtdes(
    &self,
    command: XtDesCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if !(1..=100).contains(&command.patterns) {
      return Err(RuntimeError::XtDesInvalidPatternLimit);
    }
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "xtdes" })?;
    let Some(id_column) = dataset
      .columns
      .iter()
      .find(|column| column.name == command.id_variable)
    else {
      return Err(RuntimeError::XtDesUnknownPanelVariable {
        variable: command.id_variable,
      });
    };
    let id_numeric = is_numeric_data_type(&id_column.data_type);
    let id_string = matches!(id_column.data_type.as_str(), "VARCHAR" | "TEXT" | "STRING");
    if !id_numeric && !id_string {
      return Err(RuntimeError::XtDesPanelIdentifierType {
        variable: id_column.name.clone(),
      });
    }
    let Some(time_column) = dataset
      .columns
      .iter()
      .find(|column| column.name == command.time_variable)
    else {
      return Err(RuntimeError::XtDesUnknownTimeVariable {
        variable: command.time_variable,
      });
    };
    if !is_numeric_data_type(&time_column.data_type) {
      return Err(RuntimeError::XtDesTimeVariableType {
        variable: time_column.name.clone(),
      });
    }

    let backend = self.backend.as_ref().ok_or(RuntimeError::XtDesFailed)?;
    let (invalid_id, invalid_time) = backend
      .xtdes_invalid_keys(&command.id_variable, &command.time_variable, id_numeric)
      .map_err(|_| RuntimeError::XtDesKeyCheckFailed)?;
    if invalid_id {
      return Err(RuntimeError::XtDesInvalidPanelIdentifier {
        variable: command.id_variable,
      });
    }
    if invalid_time {
      return Err(RuntimeError::XtDesInvalidTimeValue {
        variable: command.time_variable,
      });
    }

    let summary = backend
      .xtdes_key_summary(&command.id_variable, &command.time_variable)
      .map_err(|_| RuntimeError::XtDesFailed)?;
    if summary.time_values.len() > XT_DES_MAX_PERIODS {
      return Err(RuntimeError::XtDesTooManyPeriods {
        limit: XT_DES_MAX_PERIODS,
      });
    }
    let pair_counts = backend
      .xtdes_pair_counts(&command.id_variable, &command.time_variable)
      .map_err(|_| RuntimeError::XtDesFailed)?;
    let result = assemble_xtdes_result(command, summary, &pair_counts)?;
    Ok(ExecutionResult::XtDes(result))
  }
}

fn assemble_xtdes_result(
  command: XtDesCommand,
  summary: XtDesKeySummary,
  pair_counts: &[XtDesPairCount],
) -> Result<XtDesResult, RuntimeError> {
  let panel_count = usize::try_from(summary.panel_count).map_err(|_| RuntimeError::XtDesFailed)?;
  let period_count = summary.time_values.len();
  let mut observations_per_panel = vec![0_u64; panel_count];
  let mut periods_per_panel = vec![0_u64; panel_count];
  let mut pattern_frequencies: BTreeMap<Vec<(usize, u64)>, u64> = BTreeMap::new();
  let mut current_panel = None;
  let mut current_pattern = Vec::new();

  for pair in pair_counts {
    let panel_index = usize::try_from(pair.panel_index).map_err(|_| RuntimeError::XtDesFailed)?;
    let period_index = usize::try_from(pair.period_index).map_err(|_| RuntimeError::XtDesFailed)?;
    if panel_index >= panel_count || period_index >= period_count || pair.observations == 0 {
      return Err(RuntimeError::XtDesFailed);
    }
    if current_panel.is_some_and(|previous| previous != panel_index) {
      add_pattern_frequency(&mut pattern_frequencies, &mut current_pattern)?;
    }
    if current_panel != Some(panel_index) {
      current_panel = Some(panel_index);
      current_pattern.clear();
    }
    let panel_observations = &mut observations_per_panel[panel_index];
    *panel_observations = panel_observations
      .checked_add(pair.observations)
      .ok_or(RuntimeError::XtDesFailed)?;
    periods_per_panel[panel_index] = periods_per_panel[panel_index]
      .checked_add(1)
      .ok_or(RuntimeError::XtDesFailed)?;
    current_pattern.push((period_index, pair.observations));
  }
  if current_panel.is_some() {
    add_pattern_frequency(&mut pattern_frequencies, &mut current_pattern)?;
  }
  if observations_per_panel.contains(&0) {
    return Err(RuntimeError::XtDesFailed);
  }

  let missing_panel_periods = periods_per_panel.iter().try_fold(0_u64, |total, count| {
    let missing = u64::try_from(period_count)
      .map_err(|_| RuntimeError::XtDesFailed)?
      .checked_sub(*count)
      .ok_or(RuntimeError::XtDesFailed)?;
    total.checked_add(missing).ok_or(RuntimeError::XtDesFailed)
  })?;
  let balanced =
    (panel_count > 0).then_some(missing_panel_periods == 0 && summary.duplicate_key_pairs == 0);
  let mut ranked_patterns = pattern_frequencies.into_iter().collect::<Vec<_>>();
  ranked_patterns.sort_by(|(left, left_count), (right, right_count)| {
    right_count
      .cmp(left_count)
      .then_with(|| compare_dense_patterns(left, right, period_count))
  });
  let distinct_pattern_count =
    u64::try_from(ranked_patterns.len()).map_err(|_| RuntimeError::XtDesFailed)?;
  let returned_pattern_count = command.patterns.min(ranked_patterns.len());
  let omitted_pattern_count = distinct_pattern_count
    .checked_sub(u64::try_from(returned_pattern_count).map_err(|_| RuntimeError::XtDesFailed)?)
    .ok_or(RuntimeError::XtDesFailed)?;
  let denominator = summary.panel_count as f64;
  let mut cumulative_panels = 0_u64;
  let mut patterns = Vec::with_capacity(returned_pattern_count);
  for (sparse_counts, pattern_panels) in ranked_patterns.into_iter().take(returned_pattern_count) {
    cumulative_panels = cumulative_panels
      .checked_add(pattern_panels)
      .ok_or(RuntimeError::XtDesFailed)?;
    let percent = pattern_panels as f64 * 100.0 / denominator;
    let cumulative_percent = if cumulative_panels == summary.panel_count {
      100.0
    } else {
      cumulative_panels as f64 * 100.0 / denominator
    };
    let mut counts_by_period = vec![0_u64; period_count];
    let mut observations_per_pattern_panel = 0_u64;
    for (period_index, observations) in &sparse_counts {
      counts_by_period[*period_index] = *observations;
      observations_per_pattern_panel = observations_per_pattern_panel
        .checked_add(*observations)
        .ok_or(RuntimeError::XtDesFailed)?;
    }
    patterns.push(XtDesPatternRow {
      counts_by_period,
      panel_count: pattern_panels,
      percent,
      cumulative_percent,
      periods_observed: u64::try_from(sparse_counts.len())
        .map_err(|_| RuntimeError::XtDesFailed)?,
      observations_per_panel: observations_per_pattern_panel,
    });
  }

  let time_min = summary.time_values.first().cloned();
  let time_max = summary.time_values.last().cloned();
  Ok(XtDesResult {
    id_variable: command.id_variable,
    time_variable: command.time_variable,
    observation_count: summary.observation_count,
    panel_count: summary.panel_count,
    panel_min: summary.panel_min,
    panel_max: summary.panel_max,
    period_count: u64::try_from(period_count).map_err(|_| RuntimeError::XtDesFailed)?,
    time_min,
    time_max,
    duplicate_key_pairs: summary.duplicate_key_pairs,
    extra_pair_observations: summary.extra_pair_observations,
    missing_panel_periods,
    balanced,
    observations_per_panel: count_distribution(&observations_per_panel),
    periods_per_panel: count_distribution(&periods_per_panel),
    time_values: summary.time_values,
    distinct_pattern_count,
    requested_patterns: command.patterns,
    omitted_pattern_count,
    patterns,
  })
}

fn compare_dense_patterns(
  left: &[(usize, u64)],
  right: &[(usize, u64)],
  period_count: usize,
) -> std::cmp::Ordering {
  let mut left_position = 0;
  let mut right_position = 0;
  for period in 0..period_count {
    let left_count = left
      .get(left_position)
      .filter(|(index, _)| *index == period)
      .map_or(0, |(_, count)| *count);
    let right_count = right
      .get(right_position)
      .filter(|(index, _)| *index == period)
      .map_or(0, |(_, count)| *count);
    match left_count.cmp(&right_count) {
      std::cmp::Ordering::Equal => {}
      order => return order,
    }
    if left
      .get(left_position)
      .is_some_and(|(index, _)| *index == period)
    {
      left_position += 1;
    }
    if right
      .get(right_position)
      .is_some_and(|(index, _)| *index == period)
    {
      right_position += 1;
    }
  }
  std::cmp::Ordering::Equal
}

fn add_pattern_frequency(
  frequencies: &mut BTreeMap<Vec<(usize, u64)>, u64>,
  pattern: &mut Vec<(usize, u64)>,
) -> Result<(), RuntimeError> {
  let frequency = frequencies.entry(std::mem::take(pattern)).or_default();
  *frequency = frequency.checked_add(1).ok_or(RuntimeError::XtDesFailed)?;
  Ok(())
}

fn count_distribution(values: &[u64]) -> XtDesCountDistribution {
  if values.is_empty() {
    return XtDesCountDistribution {
      minimum: None,
      p5: None,
      p25: None,
      p50: None,
      p75: None,
      p95: None,
      maximum: None,
    };
  }
  let mut sorted = values.to_vec();
  sorted.sort_unstable();
  XtDesCountDistribution {
    minimum: sorted.first().copied(),
    p5: type7_percentile(&sorted, 0.05),
    p25: type7_percentile(&sorted, 0.25),
    p50: type7_percentile(&sorted, 0.50),
    p75: type7_percentile(&sorted, 0.75),
    p95: type7_percentile(&sorted, 0.95),
    maximum: sorted.last().copied(),
  }
}

fn type7_percentile(sorted: &[u64], probability: f64) -> Option<f64> {
  let last = sorted.len().checked_sub(1)?;
  let position = last as f64 * probability;
  let lower = position.floor() as usize;
  let upper = position.ceil() as usize;
  let lower_value = *sorted.get(lower)? as f64;
  let upper_value = *sorted.get(upper)? as f64;
  Some(lower_value + (position - lower as f64) * (upper_value - lower_value))
}
