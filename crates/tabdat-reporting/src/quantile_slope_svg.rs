use std::fmt::Write;
use tabdat_stats::QuantileSlopeBandResult;

const WIDTH: u32 = 900;
const HEIGHT: u32 = 660;
const LEFT: f64 = 90.0;
const TOP: f64 = 75.0;
const PLOT_WIDTH: f64 = 730.0;
const PLOT_HEIGHT: f64 = 400.0;

#[derive(Debug, Clone)]
pub struct RenderedQuantileSlopeSvg {
  /// Complete standalone document, returned only after all labels and domains validate.
  pub svg: String,
  /// SVG width in user units.
  pub width: u32,
  /// SVG height in user units.
  pub height: u32,
}

fn xml(value: &str) -> Result<String, String> {
  let mut escaped = String::new();
  for character in value.chars() {
    if !(matches!(character, '\t' | '\n' | '\r')
      || ('\u{20}'..='\u{d7ff}').contains(&character)
      || ('\u{e000}'..='\u{fffd}').contains(&character)
      || ('\u{10000}'..='\u{10ffff}').contains(&character))
    {
      return Err("plot label contains an invalid XML character".into());
    }
    match character {
      '\t' => escaped.push_str("&#x9;"),
      '\n' => escaped.push_str("&#xA;"),
      '\r' => escaped.push_str("&#xD;"),
      '&' => escaped.push_str("&amp;"),
      '<' => escaped.push_str("&lt;"),
      '>' => escaped.push_str("&gt;"),
      '"' => escaped.push_str("&quot;"),
      '\'' => escaped.push_str("&apos;"),
      _ => escaped.push(character),
    }
  }
  Ok(escaped)
}

fn checked_label(value: &str) -> Result<String, String> {
  if value.is_empty() || value.chars().count() > 80 {
    return Err("plot labels must contain 1 to 80 characters".into());
  }
  xml(value)
}

fn tick(value: f64) -> String {
  if value != 0.0 && (value.abs() < 0.01 || value.abs() >= 1e4) {
    format!("{value:.2e}")
  } else {
    let text = format!("{value:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
      "0".into()
    } else {
      text.into()
    }
  }
}

#[derive(Clone, Copy)]
struct Domain {
  low: f64,
  high: f64,
}

#[derive(Clone, Copy)]
struct SeriesStyle {
  name: &'static str,
  color: &'static str,
  dash: &'static str,
}

impl Domain {
  fn padded(values: impl Iterator<Item = f64>) -> Result<Self, String> {
    let (mut low, mut high) = (f64::INFINITY, f64::NEG_INFINITY);
    for value in values {
      if !value.is_finite() {
        return Err("plot coordinates must be finite".into());
      }
      low = low.min(value);
      high = high.max(value);
    }
    let span = high - low;
    if !span.is_finite() || span < 0.0 {
      return Err("plot range has no finite span".into());
    }
    let padding = if span == 0.0 {
      low.abs().max(1.0) * 0.05
    } else {
      span * 0.05
    };
    low -= padding;
    high += padding;
    if !low.is_finite() || !high.is_finite() || high <= low || !(high - low).is_finite() {
      return Err("plot range cannot be represented with positive finite padding".into());
    }
    Ok(Self { low, high })
  }

  fn fraction(self, value: f64) -> f64 {
    (value - self.low) / (self.high - self.low)
  }
}

fn map_x(domain: Domain, value: f64) -> f64 {
  LEFT + PLOT_WIDTH * domain.fraction(value)
}

fn map_y(domain: Domain, value: f64) -> f64 {
  TOP + PLOT_HEIGHT * (1.0 - domain.fraction(value))
}

/// Build a standalone coefficient plot without writing files or changing state.
///
/// Raw quantile and coefficient values are embedded on each series so callers
/// can independently verify the rendered screen coordinates. Invalid bands,
/// domains, labels or XML characters return an error before a document is
/// returned.
pub fn render_quantile_slope_svg(
  result: &QuantileSlopeBandResult,
  title: Option<&str>,
) -> Result<RenderedQuantileSlopeSvg, String> {
  if result.points.len() < 2
    || result.quantiles.len() != result.points.len()
    || result
      .points
      .iter()
      .zip(&result.quantiles)
      .any(|(point, quantile)| {
        point.quantile != *quantile
          || !point.quantile.is_finite()
          || !(0.05..=0.95).contains(&point.quantile)
          || !point.estimate.is_finite()
          || !point.standard_error.is_finite()
          || point.standard_error < 0.0
          || !point.lower.is_finite()
          || !point.upper.is_finite()
          || point.lower > point.estimate
          || point.estimate > point.upper
      })
    || result.quantiles.windows(2).any(|pair| pair[0] >= pair[1])
    || !result.ols_coefficient.is_finite()
  {
    return Err("quantile slope plot requires two or more ordered finite band estimates".into());
  }
  let title = checked_label(title.unwrap_or("Slope Estimates as Quantile Varies"))?;
  let outcome = checked_label(&result.outcome)?;
  let coefficient = checked_label(&result.coefficient)?;
  let x_domain = Domain::padded(result.quantiles.iter().copied())?;
  let y_domain = Domain::padded(
    result
      .points
      .iter()
      .flat_map(|point| [point.lower, point.estimate, point.upper])
      .chain([result.ols_coefficient]),
  )?;

  let mut svg = format!(
    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{WIDTH}\" height=\"{HEIGHT}\" viewBox=\"0 0 {WIDTH} {HEIGHT}\" role=\"img\" aria-labelledby=\"plot-title plot-description\" data-x-min=\"{:e}\" data-x-max=\"{:e}\" data-y-min=\"{:e}\" data-y-max=\"{:e}\" data-plot-left=\"{LEFT}\" data-plot-top=\"{TOP}\" data-plot-width=\"{PLOT_WIDTH}\" data-plot-height=\"{PLOT_HEIGHT}\" data-point-count=\"{}\" data-outcome=\"{outcome}\" data-coefficient=\"{coefficient}\" data-band-method=\"pointwise_normal_fixed_1.96\" data-simultaneous-coverage=\"false\"><title id=\"plot-title\">{title}</title><desc id=\"plot-description\">Pointwise 95% normal-approximation coefficient bands, estimate plus or minus 1.96 bootstrap standard errors. No simultaneous-coverage guarantee. Outcome {outcome}; coefficient {coefficient}.</desc><rect width=\"100%\" height=\"100%\" fill=\"white\"/><g font-family=\"sans-serif\" font-size=\"12\" fill=\"#222\">",
    x_domain.low,
    x_domain.high,
    y_domain.low,
    y_domain.high,
    result.points.len()
  );
  writeln!(
    svg,
    "<text x=\"450\" y=\"30\" text-anchor=\"middle\" font-size=\"18\">{title}</text>"
  )
  .unwrap();
  for index in 0..=5 {
    let fraction = f64::from(index) / 5.0;
    let (x, y) = (
      LEFT + PLOT_WIDTH * fraction,
      TOP + PLOT_HEIGHT * (1.0 - fraction),
    );
    writeln!(
      svg,
      "<path d=\"M{LEFT} {y}h{PLOT_WIDTH}M{x} {TOP}v{PLOT_HEIGHT}\" fill=\"none\" stroke=\"#ddd\"/>"
    )
    .unwrap();
    writeln!(
      svg,
      "<text x=\"{}\" y=\"{}\" text-anchor=\"end\">{}</text><text x=\"{x}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
      LEFT - 8.0,
      y + 4.0,
      tick(y_domain.low + (y_domain.high - y_domain.low) * fraction),
      TOP + PLOT_HEIGHT + 20.0,
      tick(x_domain.low + (x_domain.high - x_domain.low) * fraction)
    )
    .unwrap();
  }
  writeln!(
    svg,
    "<path d=\"M{LEFT} {TOP}v{PLOT_HEIGHT}h{PLOT_WIDTH}\" fill=\"none\" stroke=\"#444\"/><text x=\"455\" y=\"525\" text-anchor=\"middle\">Quantile</text><text transform=\"translate(20 275) rotate(-90)\" text-anchor=\"middle\">Slope and confidence bands</text>"
  )
  .unwrap();

  for (style, value) in [
    (
      SeriesStyle {
        name: "upper_95_band",
        color: "#999999",
        dash: "5 3",
      },
      result
        .points
        .iter()
        .map(|point| point.upper)
        .collect::<Vec<_>>(),
    ),
    (
      SeriesStyle {
        name: "quantile_slope",
        color: "#0072B2",
        dash: "none",
      },
      result
        .points
        .iter()
        .map(|point| point.estimate)
        .collect::<Vec<_>>(),
    ),
    (
      SeriesStyle {
        name: "lower_95_band",
        color: "#999999",
        dash: "5 3",
      },
      result
        .points
        .iter()
        .map(|point| point.lower)
        .collect::<Vec<_>>(),
    ),
  ] {
    write_series(
      &mut svg,
      style,
      &result.quantiles,
      &value,
      x_domain,
      y_domain,
    );
  }
  let ols_x = [result.quantiles[0], *result.quantiles.last().unwrap()];
  let ols_y = [result.ols_coefficient; 2];
  write_series(
    &mut svg,
    SeriesStyle {
      name: "ols_slope",
      color: "#D55E00",
      dash: "8 4",
    },
    &ols_x,
    &ols_y,
    x_domain,
    y_domain,
  );

  for (index, (name, color, dash)) in [
    ("Upper 95% confidence band", "#999999", "5 3"),
    ("Quantile slope coefficient", "#0072B2", "none"),
    ("Lower 95% confidence band", "#999999", "5 3"),
    ("OLS slope coefficient", "#D55E00", "8 4"),
  ]
  .into_iter()
  .enumerate()
  {
    let y = 555 + index * 20;
    writeln!(
      svg,
      "<path d=\"M90 {y}h25\" stroke=\"{color}\" stroke-width=\"2\" stroke-dasharray=\"{dash}\"/><text x=\"125\" y=\"{}\">{}</text>",
      y + 4,
      checked_label(name)?
    )
    .unwrap();
  }
  svg.push_str("</g></svg>");
  Ok(RenderedQuantileSlopeSvg {
    svg,
    width: WIDTH,
    height: HEIGHT,
  })
}

fn write_series(
  svg: &mut String,
  style: SeriesStyle,
  quantiles: &[f64],
  values: &[f64],
  x_domain: Domain,
  y_domain: Domain,
) {
  let x_values = quantiles
    .iter()
    .map(|value| format!("{value:e}"))
    .collect::<Vec<_>>();
  let y_values = values
    .iter()
    .map(|value| format!("{value:e}"))
    .collect::<Vec<_>>();
  let path = quantiles
    .iter()
    .zip(values)
    .enumerate()
    .map(|(index, (x, y))| {
      format!(
        "{}{} {}",
        if index == 0 { "M" } else { "L" },
        map_x(x_domain, *x),
        map_y(y_domain, *y)
      )
    })
    .collect::<Vec<_>>()
    .join("");
  writeln!(
    svg,
    "<path class=\"quantile-slope-series\" data-series=\"{}\" data-quantile-values=\"{}\" data-coefficient-values=\"{}\" d=\"{path}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\" stroke-dasharray=\"{}\"/>",
    style.name,
    x_values.join(","),
    y_values.join(","),
    style.color,
    style.dash
  )
  .unwrap();
}
