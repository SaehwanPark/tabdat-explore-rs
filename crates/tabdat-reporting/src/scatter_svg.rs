//! Pure, bounded scatter/line SVG generation; no file, font or browser effects.

use std::fmt::Write;

pub const SVG_WIDTH: u32 = 900;
pub const SVG_HEIGHT: u32 = 660;
pub const MAX_SCATTER_ROWS: usize = 10_000;
const LEFT: f64 = 90.0;
const TOP: f64 = 75.0;
const WIDTH: f64 = 730.0;
const HEIGHT: f64 = 400.0;

#[derive(Debug, Clone)]
pub struct ScatterSvgRow {
  pub x: Option<f64>,
  pub y: Option<f64>,
  pub lines: Vec<Option<f64>>,
}

#[derive(Debug, Clone)]
pub struct ScatterSvgData {
  pub x_variable: String,
  pub y_variable: String,
  pub x_label: String,
  pub y_label: String,
  pub title: Option<String>,
  pub line_variables: Vec<String>,
  pub line_labels: Vec<String>,
  /// Deterministic y-only display amplitude, in outcome units, not RNG state.
  pub jitter_y: f64,
  pub rows: Vec<ScatterSvgRow>,
}

#[derive(Debug, Clone)]
pub struct RenderedScatterSvg {
  pub svg: String,
  pub width: u32,
  pub height: u32,
  pub scatter_points: usize,
  pub line_points: Vec<usize>,
}

fn xml(value: &str) -> Result<String, String> {
  let mut escaped = String::new();
  for c in value.chars() {
    if !(matches!(c, '\t' | '\n' | '\r')
      || ('\u{20}'..='\u{d7ff}').contains(&c)
      || ('\u{e000}'..='\u{fffd}').contains(&c)
      || ('\u{10000}'..='\u{10ffff}').contains(&c))
    {
      return Err("label contains an invalid XML character".into());
    }
    match c {
      // Character references preserve metadata identities through XML's
      // attribute whitespace normalization (literal tabs/newlines do not).
      '\t' => escaped.push_str("&#x9;"),
      '\n' => escaped.push_str("&#xA;"),
      '\r' => escaped.push_str("&#xD;"),
      '&' => escaped.push_str("&amp;"),
      '<' => escaped.push_str("&lt;"),
      '>' => escaped.push_str("&gt;"),
      '"' => escaped.push_str("&quot;"),
      '\'' => escaped.push_str("&apos;"),
      _ => escaped.push(c),
    }
  }
  Ok(escaped)
}

fn label(value: &str) -> Result<String, String> {
  if value.is_empty() || value.chars().count() > 80 {
    return Err("plot labels must contain 1 to 80 characters".into());
  }
  xml(value)
}

#[derive(Clone, Copy)]
struct Domain {
  low: f64,
  high: f64,
}
impl Domain {
  fn padded(values: impl Iterator<Item = f64>) -> Result<Self, String> {
    let (mut low, mut high) = (f64::INFINITY, f64::NEG_INFINITY);
    for value in values {
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
    if !low.is_finite() || !high.is_finite() || !(high - low).is_finite() || high <= low {
      return Err("plot range cannot be represented with positive finite padding".into());
    }
    Ok(Self { low, high })
  }
  fn fraction(self, value: f64) -> f64 {
    (value - self.low) / (self.high - self.low)
  }
}

fn finite(value: Option<f64>) -> Option<f64> {
  value.filter(|v| v.is_finite())
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
fn px(domain: Domain, value: f64) -> f64 {
  LEFT + WIDTH * domain.fraction(value)
}
fn py(domain: Domain, value: f64) -> f64 {
  TOP + HEIGHT * (1.0 - domain.fraction(value))
}

/// Finite selection is mark-specific; a missing line value at a finite x breaks
/// that line. Sorting is internal and stable by x/original row index. Metadata
/// retains raw coordinates and original candidate-row indices, including when
/// display-only jitter is requested. No estimator, seed or dataset is changed.
pub fn render_scatter_svg(data: &ScatterSvgData) -> Result<RenderedScatterSvg, String> {
  if data.rows.len() > MAX_SCATTER_ROWS {
    return Err("scatter exceeds 10000 candidate rows".into());
  }
  let series = data.line_variables.len();
  if series > 3
    || data.line_labels.len() != series
    || data.rows.iter().any(|r| r.lines.len() != series)
  {
    return Err("scatter requires at most three aligned line series".into());
  }
  if data
    .line_variables
    .iter()
    .collect::<std::collections::BTreeSet<_>>()
    .len()
    != series
  {
    return Err("line variables must be unique".into());
  }
  if !data.jitter_y.is_finite() || data.jitter_y < 0.0 {
    return Err("jitter must be finite and nonnegative".into());
  }
  let x_label = label(&data.x_label)?;
  let y_label = label(&data.y_label)?;
  let default_title = if series == 0 {
    "Scatter plot"
  } else {
    "Scatter plot with line overlays"
  };
  let title = label(data.title.as_deref().unwrap_or(default_title))?;
  let x_variable = xml(&data.x_variable)?;
  let y_variable = xml(&data.y_variable)?;
  let line_labels = data
    .line_labels
    .iter()
    .map(|s| label(s))
    .collect::<Result<Vec<_>, _>>()?;
  let variables = data
    .line_variables
    .iter()
    .map(|s| xml(s))
    .collect::<Result<Vec<_>, _>>()?;
  let mut points = Vec::new();
  for (index, row) in data.rows.iter().enumerate() {
    if let (Some(x), Some(y)) = (finite(row.x), finite(row.y)) {
      // An ordinal visual pattern, deliberately not a statistical RNG/seed.
      let display_y = y + data.jitter_y * ((index * 73 % 251) as f64 / 125.0 - 1.0);
      if !display_y.is_finite() {
        return Err("jittered coordinate is nonfinite".into());
      }
      points.push((index, x, y, display_y));
    }
  }
  if points.is_empty() {
    return Err("scatter has no complete finite points".into());
  }
  let mut lines = Vec::new();
  let mut line_points = Vec::new();
  for j in 0..series {
    let mut rows: Vec<_> = data
      .rows
      .iter()
      .enumerate()
      .filter_map(|(i, r)| Some((i, finite(r.x)?, finite(r.lines[j]))))
      .collect();
    // Signed zeros are numeric ties, not distinct total_cmp categories.
    rows.sort_by(|a, b| {
      if a.1 == b.1 {
        a.0.cmp(&b.0)
      } else {
        a.1.total_cmp(&b.1)
      }
    });
    if !rows
      .windows(2)
      .any(|r| r[0].2.is_some() && r[1].2.is_some())
    {
      return Err(format!(
        "line {} has no adjacent finite pair after x sorting",
        data.line_variables[j]
      ));
    }
    line_points.push(rows.iter().filter(|r| r.2.is_some()).count());
    lines.push(rows);
  }
  let xd = Domain::padded(
    points.iter().map(|p| p.1).chain(
      lines
        .iter()
        .flatten()
        .filter(|p| p.2.is_some())
        .map(|p| p.1),
    ),
  )?;
  let yd = Domain::padded(
    points
      .iter()
      .flat_map(|p| [p.2, p.3])
      .chain(lines.iter().flatten().filter_map(|p| p.2)),
  )?;
  let mut svg = format!(
    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{SVG_WIDTH}\" height=\"{SVG_HEIGHT}\" viewBox=\"0 0 {SVG_WIDTH} {SVG_HEIGHT}\" role=\"img\" aria-labelledby=\"plot-title plot-description\" data-x-min=\"{:e}\" data-x-max=\"{:e}\" data-y-min=\"{:e}\" data-y-max=\"{:e}\" data-plot-left=\"{LEFT}\" data-plot-top=\"{TOP}\" data-plot-width=\"{WIDTH}\" data-plot-height=\"{HEIGHT}\" data-candidate-rows=\"{}\" data-coordinate-precision=\"float64\" data-jitter-y=\"{:e}\"><title id=\"plot-title\">{title}</title><desc id=\"plot-description\">Finite scatter observations with optional line overlays. Lines sorted by x then original row index; missing line values break paths. Jitter changes display y only, not raw float64 coordinates. Numeric coordinates are float64, not lossless wide-integer or decimal identities. Overlaid values are not clipped.</desc><rect width=\"100%\" height=\"100%\" fill=\"white\"/><g font-family=\"sans-serif\" font-size=\"12\" fill=\"#222\">",
    xd.low,
    xd.high,
    yd.low,
    yd.high,
    data.rows.len(),
    data.jitter_y
  );
  writeln!(
    svg,
    "<text x=\"450\" y=\"30\" text-anchor=\"middle\" font-size=\"18\">{title}</text>"
  )
  .unwrap();
  for i in 0..=5 {
    let f = f64::from(i) / 5.0;
    let (x, y) = (LEFT + WIDTH * f, TOP + HEIGHT * (1.0 - f));
    writeln!(
      svg,
      "<path d=\"M{LEFT} {y}h{WIDTH}M{x} {TOP}v{HEIGHT}\" fill=\"none\" stroke=\"#ddd\"/>"
    )
    .unwrap();
    writeln!(svg,"<text x=\"{}\" y=\"{}\" text-anchor=\"end\">{}</text><text x=\"{x}\" y=\"{}\" text-anchor=\"middle\">{}</text>",LEFT-8.0,y+4.0,tick(yd.low+(yd.high-yd.low)*f),TOP+HEIGHT+20.0,tick(xd.low+(xd.high-xd.low)*f)).unwrap();
  }
  writeln!(svg,"<path d=\"M{LEFT} {TOP}v{HEIGHT}h{WIDTH}\" fill=\"none\" stroke=\"#444\"/><text x=\"455\" y=\"525\" text-anchor=\"middle\">{x_label}</text><text transform=\"translate(20 275) rotate(-90)\" text-anchor=\"middle\">{y_label}</text>").unwrap();
  for &(i, x, y, display_y) in &points {
    writeln!(svg,"<circle class=\"scatter-point\" data-row-index=\"{i}\" data-x=\"{x:e}\" data-y=\"{y:e}\" data-display-y=\"{display_y:e}\" data-x-variable=\"{x_variable}\" data-y-variable=\"{y_variable}\" cx=\"{}\" cy=\"{}\" r=\"2.4\" fill=\"#707070\" opacity=\"0.45\"/>",px(xd,x),py(yd,display_y)).unwrap();
  }
  let colors = ["#0072B2", "#D55E00", "#009E73"];
  let dashes = ["none", "8 4", "3 3"];
  writeln!(svg,"<circle cx=\"100\" cy=\"553\" r=\"2.4\" fill=\"#707070\"/><text x=\"125\" y=\"557\">{y_label} (scatter)</text>").unwrap();
  for (j, rows) in lines.iter().enumerate() {
    let mut path = String::new();
    let (mut indices, mut xs, mut ys) = (Vec::new(), Vec::new(), Vec::new());
    let mut pen = false;
    for &(i, x, y) in rows {
      if let Some(y) = y {
        write!(
          path,
          "{}{} {}",
          if pen { "L" } else { "M" },
          px(xd, x),
          py(yd, y)
        )
        .unwrap();
        pen = true;
        indices.push(i.to_string());
        xs.push(format!("{x:e}"));
        ys.push(format!("{y:e}"));
      } else {
        pen = false;
      }
    }
    writeln!(svg,"<path class=\"line-series\" data-variable=\"{}\" data-row-indices=\"{}\" data-x-values=\"{}\" data-y-values=\"{}\" d=\"{path}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\" stroke-dasharray=\"{}\"/>",variables[j],indices.join(","),xs.join(","),ys.join(","),colors[j],dashes[j]).unwrap();
    let y = 576 + j * 19;
    writeln!(svg,"<path d=\"M90 {y}h25\" stroke=\"{}\" stroke-width=\"2\" stroke-dasharray=\"{}\"/><text x=\"125\" y=\"{}\">{}</text>",colors[j],dashes[j],y+4,line_labels[j]).unwrap();
  }
  writeln!(svg,"<text x=\"90\" y=\"642\">Display-only y jitter amplitude: {:e}; raw coordinates retained in mark metadata.</text></g></svg>",data.jitter_y).unwrap();
  Ok(RenderedScatterSvg {
    svg,
    width: SVG_WIDTH,
    height: SVG_HEIGHT,
    scatter_points: points.len(),
    line_points,
  })
}
