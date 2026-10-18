use crate::*;
use tabdat_reporting::scatter_svg::{
  MAX_SCATTER_ROWS, ScatterSvgData, ScatterSvgRow, render_scatter_svg,
};

fn scatter_error(message: impl Into<String>) -> RuntimeError {
  RuntimeError::ScatterFailed {
    message: message.into(),
  }
}

impl Session {
  pub(crate) fn execute_scatter(
    &self,
    command: &ScatterCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let path = command.saving.as_deref().map(Path::new).ok_or_else(|| {
      scatter_error(
        "requires explicit saving(\"file.svg\") noopen; automatic paths/opening are not installed",
      )
    })?;
    if command.open_artifact {
      return Err(scatter_error(
        "automatic opening is not installed; add noopen",
      ));
    }
    tabdat_io::validate_svg_path(path)?;
    if command.line_variables.len() > 3 {
      return Err(scatter_error("lines accepts at most three variables"));
    }
    let jitter_y = command
      .jitter_y
      .as_deref()
      .unwrap_or("0")
      .parse::<f64>()
      .ok()
      .filter(|v| v.is_finite() && *v >= 0.0)
      .ok_or_else(|| scatter_error("jitter must be finite and nonnegative"))?;
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "scatter" })?;
    let variables: Vec<_> = std::iter::once(&command.x_variable)
      .chain(std::iter::once(&command.y_variable))
      .chain(command.line_variables.iter())
      .collect();
    for variable in &variables {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == **variable)
        .ok_or_else(|| scatter_error(format!("unknown variable: {variable}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(scatter_error(format!(
          "requires numeric coordinates; {variable} has type {}",
          column.data_type
        )));
      }
    }
    if dataset.row_count > MAX_SCATTER_ROWS as u64 {
      return Err(scatter_error(
        "exceeds 10000 candidate rows; select an explicit plotting cohort first",
      ));
    }
    let mut rows = Vec::with_capacity(dataset.row_count as usize);
    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE} LIMIT {}",
      variables
        .iter()
        .map(|s| quote_identifier(s))
        .collect::<Vec<_>>()
        .join(", "),
      MAX_SCATTER_ROWS + 1
    );
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "scatter" })?
      .visit_rows(
        &query,
        |row| {
          if rows.len() == MAX_SCATTER_ROWS {
            return Err(scatter_error("exceeds 10000 candidate rows"));
          }
          let values = (0..variables.len())
            .map(|i| {
              row
                .cell(i)
                .map(cell_to_f64)
                .map_err(|_| scatter_error("could not read numeric coordinates"))
            })
            .collect::<Result<Vec<_>, _>>()?;
          rows.push(ScatterSvgRow {
            x: values[0],
            y: values[1],
            lines: values[2..].to_vec(),
          });
          Ok(())
        },
        |_| scatter_error("could not read plotting data"),
      )?;
    let display_label = |variable: &str| {
      self
        .label_metadata
        .variable_labels
        .iter()
        .find(|(name, _)| name == variable)
        .map_or_else(|| variable.to_owned(), |(_, text)| text.clone())
    };
    let data = ScatterSvgData {
      x_variable: command.x_variable.clone(),
      y_variable: command.y_variable.clone(),
      x_label: display_label(&command.x_variable),
      y_label: display_label(&command.y_variable),
      title: command.title.clone(),
      line_variables: command.line_variables.clone(),
      line_labels: command
        .line_variables
        .iter()
        .map(|s| display_label(s))
        .collect(),
      jitter_y,
      rows,
    };
    let rendered = render_scatter_svg(&data).map_err(scatter_error)?;
    // Publication is the only effect, after all schema/data/render validation.
    // In particular this command never clears a retained estimator or sorts data.
    tabdat_io::publish_svg(path, &rendered.svg)?;
    let candidate_rows = data.rows.len();
    Ok(ExecutionResult::Plot(PlotArtifactResult {
      path: path.to_owned(),
      format: "svg".into(),
      width: rendered.width,
      height: rendered.height,
      candidate_rows,
      x_variable: command.x_variable.clone(),
      scatter: PlotSeriesSummary {
        variable: command.y_variable.clone(),
        point_count: rendered.scatter_points,
        omitted_rows: candidate_rows - rendered.scatter_points,
      },
      lines: command
        .line_variables
        .iter()
        .zip(rendered.line_points)
        .map(|(variable, count)| PlotSeriesSummary {
          variable: variable.clone(),
          point_count: count,
          omitted_rows: candidate_rows - count,
        })
        .collect(),
      jitter_y,
      jitter_convention: "deterministic_row_index_pattern_outcome_units".into(),
      coordinate_precision: "float64".into(),
    }))
  }
}
