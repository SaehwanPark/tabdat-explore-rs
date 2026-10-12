#![forbid(unsafe_code)]

use std::fmt::Write;

pub const WIDTH: u32 = 351;
pub const HEIGHT: u32 = 347;
const SIZE: f64 = 300.0;
const LEFT: f64 = 45.0;
const TOP: f64 = 10.0;

/// Prepared data only: category disambiguation and automatic bin selection belong
/// upstream. Histogram intervals are left-closed, with the final edge inclusive.
#[derive(Clone, Debug)]
pub enum Chart {
  Scatter {
    x_label: String,
    y_label: String,
    rows: Vec<(Option<f64>, Option<f64>)>,
  },
  Histogram {
    label: String,
    edges: Vec<f64>,
    values: Vec<Option<f64>>,
  },
  Bar {
    label: String,
    counts: Vec<(String, u64)>,
  },
}

#[derive(Debug, PartialEq, Eq)]
pub struct RenderError(pub String);

impl std::fmt::Display for RenderError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl std::error::Error for RenderError {}

type Result<T> = std::result::Result<T, RenderError>;

fn error(message: &str) -> RenderError {
  RenderError(message.into())
}

fn finite(value: f64) -> Result<f64> {
  if value.is_finite() {
    Ok(value)
  } else {
    Err(error("nonfinite numeric data"))
  }
}

fn xml(value: &str) -> Result<String> {
  let mut escaped = String::new();
  for c in value.chars() {
    if !(matches!(c, '\t' | '\n' | '\r')
      || ('\u{20}'..='\u{d7ff}').contains(&c)
      || ('\u{e000}'..='\u{fffd}').contains(&c)
      || ('\u{10000}'..='\u{10ffff}').contains(&c))
    {
      return Err(error("label contains an invalid XML character"));
    }
    escaped.push_str(match c {
      '&' => "&amp;",
      '<' => "&lt;",
      '>' => "&gt;",
      '"' => "&quot;",
      '\'' => "&apos;",
      _ => {
        escaped.push(c);
        continue;
      }
    });
  }
  Ok(escaped)
}

#[derive(Clone, Copy)]
struct Domain {
  low: f64,
  high: f64,
}

impl Domain {
  fn new(low: f64, high: f64) -> Result<Self> {
    let domain = Self { low, high };
    if !low.is_finite() || !high.is_finite() || !(high - low).is_finite() || high <= low {
      return Err(error("numeric range must have a finite positive span"));
    }
    Ok(domain)
  }

  fn including_zero(values: impl Iterator<Item = f64>) -> Result<Self> {
    let (mut low, mut high) = (0.0_f64, 0.0_f64);
    for value in values {
      finite(value)?;
      low = low.min(value);
      high = high.max(value);
    }
    if low == high {
      high = 1.0;
    }
    Self::new(low, high)
  }

  fn fraction(self, value: f64) -> f64 {
    (value - self.low) / (self.high - self.low)
  }
}

fn text(svg: &mut String, x: f64, y: f64, value: &str, anchor: &str) -> Result<()> {
  write!(
    svg,
    "<text x=\"{x}\" y=\"{y}\" text-anchor=\"{anchor}\">{}</text>",
    xml(value)?
  )
  .unwrap();
  Ok(())
}

fn y_position(domain: Domain, value: f64) -> f64 {
  TOP + SIZE * (1.0 - domain.fraction(value))
}

fn numeric_axes(svg: &mut String, x: Option<Domain>, y: Domain) -> Result<()> {
  for tick in 0..=4 {
    let fraction = f64::from(tick) / 4.0;
    let py = TOP + SIZE * (1.0 - fraction);
    write!(svg, "<path d=\"M{LEFT} {py}h{SIZE}\" stroke=\"#ddd\"/>").unwrap();
    let value = y.low + (y.high - y.low) * fraction;
    text(svg, LEFT - 5.0, py + 3.0, &format!("{value:.2}"), "end")?;
    if let Some(domain) = x {
      let px = LEFT + SIZE * fraction;
      write!(svg, "<path d=\"M{px} {TOP}v{SIZE}\" stroke=\"#ddd\"/>").unwrap();
      let value = domain.low + (domain.high - domain.low) * fraction;
      text(svg, px, TOP + SIZE + 13.0, &format!("{value:.2}"), "middle")?;
    }
  }
  write!(
    svg,
    "<path d=\"M{LEFT} {TOP}v{SIZE}h{SIZE}\" fill=\"none\" stroke=\"#888\"/>"
  )
  .unwrap();
  Ok(())
}

fn count_value(count: u64) -> Result<f64> {
  if count > (1_u64 << 53) {
    return Err(error("count exceeds exact floating-point integer range"));
  }
  Ok(count as f64)
}

fn bar_mark(
  svg: &mut String,
  x: f64,
  width: f64,
  count: u64,
  y: Domain,
  description: &str,
) -> Result<()> {
  let top = y_position(y, count_value(count)?);
  let height = (TOP + SIZE - top).max(0.0);
  write!(svg, "<rect class=\"mark\" aria-label=\"{}\" x=\"{x}\" y=\"{top}\" width=\"{width}\" height=\"{height}\" fill=\"#4c78a8\"/>", xml(description)?).unwrap();
  Ok(())
}

/// Pure SVG generation. No file, URL, JavaScript, or font access occurs here.
pub fn render_svg(chart: &Chart) -> Result<String> {
  let mut svg = format!(
    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{WIDTH}\" height=\"{HEIGHT}\" viewBox=\"0 0 {WIDTH} {HEIGHT}\" role=\"img\"><rect width=\"100%\" height=\"100%\" fill=\"white\"/><g font-family=\"Arial, sans-serif\" font-size=\"10\" fill=\"#333\">"
  );
  let (x_label, y_label) = match chart {
    Chart::Scatter {
      x_label,
      y_label,
      rows,
    } => {
      // Validate present values even when the other coordinate is missing.
      for (x, y) in rows {
        for value in [x, y].into_iter().flatten() {
          finite(*value)?;
        }
      }
      let points: Vec<_> = rows
        .iter()
        .filter_map(|(x, y)| Some(((*x)?, (*y)?)))
        .collect();
      if points.is_empty() {
        return Err(error("scatter has no complete points"));
      }
      let xd = Domain::including_zero(points.iter().map(|p| p.0))?;
      let yd = Domain::including_zero(points.iter().map(|p| p.1))?;
      numeric_axes(&mut svg, Some(xd), yd)?;
      for (x, y) in points {
        let px = LEFT + SIZE * xd.fraction(x);
        let py = y_position(yd, y);
        let description = xml(&format!("{x_label}: {x}; {y_label}: {y}"))?;
        write!(svg, "<circle class=\"mark\" aria-label=\"{description}\" cx=\"{px}\" cy=\"{py}\" r=\"2.739\" fill=\"none\" stroke=\"#4c78a8\" stroke-width=\"2\" opacity=\"0.7\"/>").unwrap();
      }
      (x_label.as_str(), y_label.as_str())
    }
    Chart::Histogram {
      label,
      edges,
      values,
    } => {
      if edges.len() < 2 || edges.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(error("histogram needs strictly increasing edges"));
      }
      let xd = Domain::new(edges[0], edges[edges.len() - 1])?;
      for edge in edges {
        finite(*edge)?;
      }
      let mut counts = vec![0_u64; edges.len() - 1];
      let mut present = false;
      for value in values.iter().flatten() {
        finite(*value)?;
        if *value < xd.low || *value > xd.high {
          return Err(error("histogram value outside explicit edges"));
        }
        present = true;
        let bin = edges
          .partition_point(|edge| edge <= value)
          .saturating_sub(1)
          .min(counts.len() - 1);
        counts[bin] += 1;
      }
      if !present {
        return Err(error("histogram has no numeric values"));
      }
      let yd = Domain::including_zero(counts.iter().map(|count| *count as f64))?;
      numeric_axes(&mut svg, Some(xd), yd)?;
      for (pair, count) in edges.windows(2).zip(counts) {
        let x = LEFT + SIZE * xd.fraction(pair[0]);
        let width = SIZE * (xd.fraction(pair[1]) - xd.fraction(pair[0]));
        bar_mark(
          &mut svg,
          x + 0.5,
          (width - 1.0).max(0.0),
          count,
          yd,
          &format!("{label}: {} – {}; Count: {count}", pair[0], pair[1]),
        )?;
      }
      (label.as_str(), "Count")
    }
    Chart::Bar { label, counts } => {
      if counts.is_empty() {
        return Err(error("bar has no categories"));
      }
      let mut seen = std::collections::HashSet::new();
      let mut numeric_counts = Vec::with_capacity(counts.len());
      for (category, count) in counts {
        xml(category)?;
        if !seen.insert(category) {
          return Err(error("bar labels must already be disambiguated"));
        }
        numeric_counts.push(count_value(*count)?);
      }
      let yd = Domain::including_zero(numeric_counts.into_iter())?;
      numeric_axes(&mut svg, None, yd)?;
      let band = SIZE / counts.len() as f64;
      for (index, (category, count)) in counts.iter().enumerate() {
        let x = LEFT + band * index as f64;
        bar_mark(
          &mut svg,
          x + band * 0.1,
          band * 0.8,
          *count,
          yd,
          &format!("{label}: {category}; Count: {count}"),
        )?;
        text(
          &mut svg,
          x + band / 2.0,
          TOP + SIZE + 13.0,
          category,
          "middle",
        )?;
      }
      (label.as_str(), "Count")
    }
  };
  text(
    &mut svg,
    LEFT + SIZE / 2.0,
    f64::from(HEIGHT) - 3.0,
    x_label,
    "middle",
  )?;
  write!(
    svg,
    "<text transform=\"translate(12 160) rotate(-90)\" text-anchor=\"middle\">{}</text></g></svg>",
    xml(y_label)?
  )
  .unwrap();
  Ok(svg)
}

/// Explicit first-use loading of one caller-supplied font; never needed for SVG
/// or metadata paths. Avoid scanning the system's potentially enormous font set.
/// Only SVG generated above is accepted: no arbitrary SVG/file/URL input surface.
pub struct PngRenderer {
  options: resvg::usvg::Options<'static>,
}

impl PngRenderer {
  pub fn from_font_data(data: Vec<u8>) -> Result<Self> {
    let mut options = resvg::usvg::Options::default();
    options.fontdb_mut().load_font_data(data);
    let family = options
      .fontdb
      .faces()
      .next()
      .and_then(|face| face.families.first())
      .map(|(name, _)| name.clone())
      .ok_or_else(|| error("PNG text requires valid font data with a named family"))?;
    options.fontdb_mut().set_sans_serif_family(family.clone());
    options.font_family = family;
    Ok(Self { options })
  }

  pub fn render(&self, chart: &Chart) -> Result<Vec<u8>> {
    let svg = render_svg(chart)?;
    let tree = resvg::usvg::Tree::from_str(&svg, &self.options)
      .map_err(|err| RenderError(format!("generated SVG parse failed: {err}")))?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(WIDTH, HEIGHT)
      .ok_or_else(|| error("PNG canvas allocation failed"))?;
    resvg::render(
      &tree,
      resvg::tiny_skia::Transform::identity(),
      &mut pixmap.as_mut(),
    );
    pixmap
      .encode_png()
      .map_err(|err| RenderError(format!("PNG encoding failed: {err}")))
  }
}

pub fn scatter_fixture() -> Chart {
  Chart::Scatter {
    x_label: "x".into(),
    y_label: "y".into(),
    rows: vec![
      (Some(1.0), Some(2.0)),
      (Some(2.0), Some(1.0)),
      (Some(3.0), Some(4.0)),
    ],
  }
}
