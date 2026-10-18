use tabdat_reporting::scatter_svg::{
  MAX_SCATTER_ROWS, ScatterSvgData, ScatterSvgRow, render_scatter_svg,
};

fn fixture() -> ScatterSvgData {
  ScatterSvgData {
    x_variable: "x".into(),
    y_variable: "y".into(),
    x_label: "X".into(),
    y_label: "Y".into(),
    title: None,
    line_variables: vec!["pred".into()],
    line_labels: vec!["Model".into()],
    jitter_y: 0.0,
    rows: [(2.0, 4.0, 3.0), (0.0, 0.0, 1.0), (1.0, 1.0, 2.0)]
      .into_iter()
      .map(|(x, y, p)| ScatterSvgRow {
        x: Some(x),
        y: Some(y),
        lines: vec![Some(p)],
      })
      .collect(),
  }
}
fn marks<'a>(svg: &'a str, class: &str) -> Vec<&'a str> {
  svg
    .lines()
    .filter(|line| line.contains(&format!("class=\"{class}\"")))
    .collect()
}
fn attr<'a>(element: &'a str, name: &str) -> &'a str {
  element
    .split_once(&format!("{name}=\""))
    .unwrap()
    .1
    .split_once('"')
    .unwrap()
    .0
}
fn number(element: &str, name: &str) -> f64 {
  attr(element, name).parse().unwrap()
}
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-10, "{a} != {b}");
}

#[test]
fn geometry_is_independently_derived_and_original_scatter_order_is_retained() {
  let rendered = render_scatter_svg(&fixture()).unwrap();
  assert_eq!((rendered.width, rendered.height), (900, 660));
  assert_eq!(rendered.scatter_points, 3);
  assert_eq!(rendered.line_points, [3]);
  let points = marks(&rendered.svg, "scatter-point");
  assert_eq!(
    points
      .iter()
      .map(|p| attr(p, "data-row-index"))
      .collect::<Vec<_>>(),
    ["0", "1", "2"]
  );
  // Extrema (x=0..2,y=0..4) padded 5%; 730x400 plot at (90,75).
  close(number(points[0], "cx"), 786.8181818181818);
  close(number(points[0], "cy"), 93.18181818181819);
  close(number(points[1], "cx"), 123.18181818181819);
  close(number(points[1], "cy"), 456.8181818181818);
  let lines = marks(&rendered.svg, "line-series");
  assert_eq!(attr(lines[0], "data-row-indices"), "1,2,0");
  assert_eq!(attr(lines[0], "data-x-values"), "0e0,1e0,2e0");
  assert_eq!(attr(lines[0], "data-y-values"), "1e0,2e0,3e0");
  assert!(attr(lines[0], "d").contains("L455 275"));
  assert_eq!(rendered.svg, render_scatter_svg(&fixture()).unwrap().svg);
}

#[test]
fn finite_selection_is_independent_and_missing_line_values_break_paths() {
  let mut data = fixture();
  data.rows[1].y = None;
  data.rows[2].lines[0] = None;
  data.rows.extend([
    ScatterSvgRow {
      x: Some(3.0),
      y: Some(f64::NAN),
      lines: vec![Some(4.0)],
    },
    ScatterSvgRow {
      x: None,
      y: Some(0.0),
      lines: vec![Some(0.0)],
    },
    ScatterSvgRow {
      x: Some(f64::INFINITY),
      y: Some(2.0),
      lines: vec![Some(2.0)],
    },
  ]);
  let rendered = render_scatter_svg(&data).unwrap();
  assert_eq!(rendered.scatter_points, 2);
  assert_eq!(rendered.line_points, [3]);
  let line = marks(&rendered.svg, "line-series")[0];
  assert_eq!(attr(line, "data-row-indices"), "1,0,3");
  assert_eq!(attr(line, "d").matches('M').count(), 2);
  assert_eq!(attr(line, "d").matches('L').count(), 1);
  assert!(!rendered.svg.contains("NaN"));
  assert!(!rendered.svg.contains("inf"));
}

#[test]
fn ties_are_sorted_by_original_row_index_and_jitter_retains_raw_outcomes() {
  let mut data = fixture();
  data.rows[2].x = Some(0.0);
  data.jitter_y = 0.03;
  let rendered = render_scatter_svg(&data).unwrap();
  let line = marks(&rendered.svg, "line-series")[0];
  assert_eq!(attr(line, "data-row-indices"), "1,2,0");
  let points = marks(&rendered.svg, "scatter-point");
  close(number(points[0], "data-y"), 4.0);
  close(number(points[0], "data-display-y"), 3.97);
  close(
    number(points[2], "data-display-y"),
    1.0 + 0.03 * (146.0 / 125.0 - 1.0),
  );
  assert_eq!(data.rows[0].y, Some(4.0));
  assert_eq!(rendered.svg, render_scatter_svg(&data).unwrap().svg);
}

#[test]
fn labels_are_escaped_without_adding_markup_or_external_resources() {
  let mut data = fixture();
  data.title = Some("<script> & \"quoted\" 'title'".into());
  data.y_variable = "a\" onload=\"bad".into();
  data.line_labels[0] = "Δ <model>".into();
  let svg = render_scatter_svg(&data).unwrap().svg;
  assert!(svg.contains("&lt;script&gt; &amp; &quot;quoted&quot; &apos;title&apos;"));
  assert!(svg.contains("a&quot; onload=&quot;bad"));
  assert!(svg.contains("Δ &lt;model&gt;"));
  assert!(!svg.contains("<script"));
  assert!(!svg.contains(" onload=\""));
  assert!(!svg.contains("href="));
  assert!(!svg.contains("<image"));
  assert!(svg.contains("aria-labelledby=\"plot-title plot-description\""));
  data.y_label = "\0".into();
  assert!(
    render_scatter_svg(&data)
      .unwrap_err()
      .contains("invalid XML")
  );
  data.y_label = "a".repeat(81);
  assert!(render_scatter_svg(&data).unwrap_err().contains("1 to 80"));
}

#[test]
fn degenerate_and_tiny_domains_work_but_overflowing_ranges_fail() {
  let mut data = fixture();
  for row in &mut data.rows {
    row.x = Some(0.0);
    row.y = Some(0.0);
    row.lines[0] = Some(0.0);
  }
  let svg = render_scatter_svg(&data).unwrap().svg;
  close(number(marks(&svg, "scatter-point")[0], "cx"), 455.0);
  close(number(marks(&svg, "scatter-point")[0], "cy"), 275.0);
  for (i, row) in data.rows.iter_mut().enumerate() {
    row.x = Some((i + 1) as f64 * 1e-300);
    row.y = row.x;
    row.lines[0] = row.x;
  }
  let svg = render_scatter_svg(&data).unwrap().svg;
  assert!(svg.contains("e-300"));
  assert!(!svg.contains("NaN"));
  data.rows[0].x = Some(-f64::MAX);
  data.rows[1].x = Some(f64::MAX);
  assert!(render_scatter_svg(&data).unwrap_err().contains("range"));
}

#[test]
fn three_curves_have_distinct_color_and_dash_styles_and_are_not_clipped() {
  let mut data = fixture();
  data.line_variables = vec!["logit".into(), "probit".into(), "ols".into()];
  data.line_labels = data.line_variables.clone();
  for row in &mut data.rows {
    row.lines = vec![Some(0.2), Some(0.3), Some(1.3)];
  }
  data.rows[0].lines[2] = Some(-0.2);
  let rendered = render_scatter_svg(&data).unwrap();
  let lines = marks(&rendered.svg, "line-series");
  assert_eq!(lines.len(), 3);
  assert_eq!(rendered.line_points, [3, 3, 3]);
  let colors: std::collections::BTreeSet<_> = lines.iter().map(|p| attr(p, "stroke")).collect();
  let dashes: std::collections::BTreeSet<_> =
    lines.iter().map(|p| attr(p, "stroke-dasharray")).collect();
  assert_eq!((colors.len(), dashes.len()), (3, 3));
  assert!(attr(lines[2], "data-y-values").contains("-2e-1"));
  assert!(attr(lines[2], "data-y-values").contains("1.3e0"));
  assert!(number(&rendered.svg, "data-y-min") < -0.2);
  assert!(number(&rendered.svg, "data-y-max") > 1.3);
}

#[test]
fn invalid_requests_fail_and_plain_scatter_does_not_claim_fitted_lines() {
  let mut data = fixture();
  data.line_variables.clear();
  data.line_labels.clear();
  for row in &mut data.rows {
    row.lines.clear();
  }
  let svg = render_scatter_svg(&data).unwrap().svg;
  assert!(svg.contains("<title id=\"plot-title\">Scatter plot</title>"));
  assert!(marks(&svg, "line-series").is_empty());
  data.rows.clear();
  assert!(
    render_scatter_svg(&data)
      .unwrap_err()
      .contains("no complete finite")
  );
  let mut data = fixture();
  data.rows[1].lines[0] = None;
  data.rows[2].lines[0] = None;
  assert!(
    render_scatter_svg(&data)
      .unwrap_err()
      .contains("adjacent finite pair")
  );
  data.rows = vec![data.rows[0].clone(); MAX_SCATTER_ROWS + 1];
  assert!(render_scatter_svg(&data).unwrap_err().contains("10000"));
  for jitter in [-0.1, f64::INFINITY, f64::NAN] {
    let mut data = fixture();
    data.jitter_y = jitter;
    assert!(render_scatter_svg(&data).unwrap_err().contains("jitter"));
  }
  let mut data = fixture();
  data.rows[0].lines.clear();
  assert!(render_scatter_svg(&data).unwrap_err().contains("aligned"));
}
