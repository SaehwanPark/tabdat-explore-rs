use tabdat_reporting::quantile_slope_svg::render_quantile_slope_svg;
use tabdat_stats::{QuantileSlopeBandPoint, QuantileSlopeBandResult};

fn result() -> QuantileSlopeBandResult {
  QuantileSlopeBandResult {
    outcome: "log-medical".into(),
    coefficient: "log-total".into(),
    quantiles: vec![0.25, 0.75],
    observation_count: 20,
    replications: 40,
    seed: 7,
    rng: "test".into(),
    confidence_level: 95.0,
    reference_distribution: "asymptotic_standard_normal".into(),
    critical_value: 1.96,
    band_method: "pointwise_normal_fixed_1.96".into(),
    covariance_convention: "common_iid_pairs_b_minus_1".into(),
    simultaneous_coverage: false,
    ols_coefficient: 0.55,
    points: vec![
      QuantileSlopeBandPoint {
        quantile: 0.25,
        estimate: 0.6,
        standard_error: 0.1 / 1.96,
        lower: 0.5,
        upper: 0.7,
      },
      QuantileSlopeBandPoint {
        quantile: 0.75,
        estimate: 0.7,
        standard_error: 0.2 / 1.96,
        lower: 0.5,
        upper: 0.9,
      },
    ],
    sample: None,
  }
}

fn series_values(svg: &str, series: &str, attribute: &str) -> Vec<f64> {
  let marker = format!("data-series=\"{series}\"");
  let start = svg.find(&marker).unwrap();
  let tail = &svg[start..];
  let marker = format!("{attribute}=\"");
  let start = tail.find(&marker).unwrap() + marker.len();
  tail[start..]
    .split_once('"')
    .unwrap()
    .0
    .split(',')
    .map(|value| value.parse().unwrap())
    .collect()
}

#[test]
fn renders_ordered_pointwise_bands_ols_reference_and_auditable_geometry() {
  let data = result();
  let rendered = render_quantile_slope_svg(&data, Some("Slope & bands")).unwrap();
  assert_eq!((rendered.width, rendered.height), (900, 660));
  assert!(
    rendered
      .svg
      .contains("<title id=\"plot-title\">Slope &amp; bands</title>")
  );
  assert!(
    rendered
      .svg
      .contains("data-band-method=\"pointwise_normal_fixed_1.96\"")
  );
  assert!(
    rendered
      .svg
      .contains("data-simultaneous-coverage=\"false\"")
  );
  assert!(rendered.svg.contains("No simultaneous-coverage guarantee"));
  assert_eq!(
    rendered
      .svg
      .matches("class=\"quantile-slope-series\"")
      .count(),
    4
  );
  for (name, expected) in [
    ("upper_95_band", [0.7, 0.9]),
    ("quantile_slope", [0.6, 0.7]),
    ("lower_95_band", [0.5, 0.5]),
  ] {
    let actual = series_values(&rendered.svg, name, "data-coefficient-values");
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
      assert!((a - e).abs() < 1e-15);
    }
  }
  assert!(rendered.svg.contains("data-series=\"ols_slope\""));
  let ols_x = series_values(&rendered.svg, "ols_slope", "data-quantile-values");
  let ols_y = series_values(&rendered.svg, "ols_slope", "data-coefficient-values");
  assert_eq!(ols_x, [0.25, 0.75]);
  assert!((ols_y[0] - 0.55).abs() < 1e-15 && ols_y[0] == ols_y[1]);
  assert!(rendered.svg.contains("Quantile slope coefficient"));
  assert!(rendered.svg.contains("OLS slope coefficient"));
}

#[test]
fn rejects_misaligned_or_invalid_band_rows_before_svg_publication() {
  let mut data = result();
  data.quantiles[1] = data.quantiles[0];
  assert!(render_quantile_slope_svg(&data, None).is_err());
  let mut data = result();
  data.points[0].lower = data.points[0].estimate + 0.1;
  assert!(render_quantile_slope_svg(&data, None).is_err());
  let mut data = result();
  data.outcome = "bad\u{1}".into();
  assert!(render_quantile_slope_svg(&data, None).is_err());
}
