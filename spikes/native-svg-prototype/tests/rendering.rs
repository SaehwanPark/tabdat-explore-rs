#![forbid(unsafe_code)]

use tabdat_native_svg_spike::{Chart, HEIGHT, PngRenderer, WIDTH, render_svg, scatter_fixture};

fn marks(svg: &str) -> Vec<String> {
  roxmltree::Document::parse(svg)
    .unwrap()
    .descendants()
    .filter(|node| node.attribute("class") == Some("mark"))
    .map(|node| node.attribute("aria-label").unwrap().to_owned())
    .collect()
}

fn histogram() -> Chart {
  Chart::Histogram {
    label: "value".into(),
    edges: vec![0.0, 1.0, 2.0, 3.0],
    values: vec![Some(0.2), Some(0.4), Some(1.1), Some(2.0), Some(2.8), None],
  }
}

fn bar() -> Chart {
  Chart::Bar {
    label: "category".into(),
    counts: vec![("b".into(), 3), ("<missing>".into(), 1), ("a".into(), 2)],
  }
}

#[test]
fn semantics_match_pinned_python_artifact_marks() {
  let expected: serde_json::Value =
    serde_json::from_str(include_str!("../fixtures/python-marks.json")).unwrap();
  for (name, chart) in [
    ("scatter", scatter_fixture()),
    ("histogram", histogram()),
    ("bar", bar()),
  ] {
    let actual = marks(&render_svg(&chart).unwrap());
    let wanted: Vec<_> = expected[name]
      .as_array()
      .unwrap()
      .iter()
      .map(|v| v.as_str().unwrap().to_owned())
      .collect();
    assert_eq!(actual, wanted, "{name} mark data/order changed");
  }
}

#[test]
fn scatter_geometry_preserves_linear_scales_and_zero_baseline() {
  let svg = render_svg(&scatter_fixture()).unwrap();
  let doc = roxmltree::Document::parse(&svg).unwrap();
  let points: Vec<_> = doc
    .descendants()
    .filter(|n| n.has_tag_name("circle"))
    .collect();
  for (point, (x, y)) in points
    .iter()
    .zip([(145.0, 160.0), (245.0, 235.0), (345.0, 10.0)])
  {
    assert_eq!(point.attribute("cx").unwrap().parse::<f64>().unwrap(), x);
    assert_eq!(point.attribute("cy").unwrap().parse::<f64>().unwrap(), y);
  }
  assert_eq!(doc.root_element().attribute("width"), Some("351"));
  assert_eq!(doc.root_element().attribute("height"), Some("347"));
  assert!(svg.contains(">x</text>"));
  assert!(svg.contains(">y</text>"));
}

#[test]
fn missing_points_are_omitted_and_negative_constant_data_is_supported() {
  let chart = Chart::Scatter {
    x_label: "x".into(),
    y_label: "y".into(),
    rows: vec![
      (None, Some(3.0)),
      (Some(2.0), None),
      (Some(-2.0), Some(-1.0)),
      (Some(-2.0), Some(-1.0)),
    ],
  };
  let svg = render_svg(&chart).unwrap();
  assert_eq!(marks(&svg).len(), 2);
  assert!(!svg.contains("NaN"));
  let zero = Chart::Scatter {
    x_label: "x".into(),
    y_label: "y".into(),
    rows: vec![(Some(0.0), Some(0.0))],
  };
  assert!(render_svg(&zero).is_ok());
}

#[test]
fn explicit_histogram_edges_include_final_endpoint_and_empty_bins() {
  let chart = Chart::Histogram {
    label: "v".into(),
    edges: vec![0.0, 1.0, 2.0, 3.0],
    values: vec![Some(0.0), Some(1.0), Some(3.0)],
  };
  assert_eq!(
    marks(&render_svg(&chart).unwrap()),
    [
      "v: 0 – 1; Count: 1",
      "v: 1 – 2; Count: 1",
      "v: 2 – 3; Count: 1"
    ]
  );
  let empty_bin = Chart::Histogram {
    label: "v".into(),
    edges: vec![0.0, 1.0, 2.0],
    values: vec![Some(0.5)],
  };
  assert!(marks(&render_svg(&empty_bin).unwrap())[1].ends_with("Count: 0"));
}

#[test]
fn labels_are_xml_escaped_not_active_markup() {
  let label = "<script>&\"' café";
  let chart = Chart::Bar {
    label: label.into(),
    counts: vec![(label.into(), 1)],
  };
  let svg = render_svg(&chart).unwrap();
  assert!(!svg.contains("<script>"));
  assert!(!svg.contains("href="));
  assert_eq!(marks(&svg), [format!("{label}: {label}; Count: 1")]);
}

#[test]
fn malformed_and_nonfinite_inputs_are_errors() {
  let mut invalid = vec![
    Chart::Bar {
      label: "c".into(),
      counts: vec![],
    },
    Chart::Bar {
      label: "c".into(),
      counts: vec![("a".into(), 1), ("a".into(), 2)],
    },
    Chart::Bar {
      label: "c".into(),
      counts: vec![("a".into(), (1_u64 << 53) + 1)],
    },
    Chart::Bar {
      label: "\u{1}".into(),
      counts: vec![("a".into(), 1)],
    },
    Chart::Scatter {
      x_label: "x".into(),
      y_label: "y".into(),
      rows: vec![(None, None)],
    },
    Chart::Scatter {
      x_label: "x".into(),
      y_label: "y".into(),
      rows: vec![(Some(-f64::MAX), Some(0.0)), (Some(f64::MAX), Some(0.0))],
    },
  ];
  for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
    invalid.push(Chart::Scatter {
      x_label: "x".into(),
      y_label: "y".into(),
      rows: vec![(Some(value), None)],
    });
  }
  for edges in [
    vec![],
    vec![1.0],
    vec![1.0, 1.0],
    vec![2.0, 1.0],
    vec![0.0, f64::NAN],
    vec![0.0, f64::INFINITY],
  ] {
    invalid.push(Chart::Histogram {
      label: "v".into(),
      edges,
      values: vec![Some(0.5)],
    });
  }
  for values in [
    vec![],
    vec![None],
    vec![Some(-1.0)],
    vec![Some(3.0)],
    vec![Some(f64::NAN)],
  ] {
    invalid.push(Chart::Histogram {
      label: "v".into(),
      edges: vec![0.0, 2.0],
      values,
    });
  }
  for chart in invalid {
    assert!(render_svg(&chart).is_err(), "accepted {chart:?}");
  }
  assert!(render_svg(&scatter_fixture()).is_ok());
}

#[test]
fn png_renders_real_marks_for_all_chart_types_and_recovers_after_errors() {
  assert!(PngRenderer::from_font_data(vec![]).is_err());
  assert!(PngRenderer::from_font_data(b"not a font".to_vec()).is_err());
  let path = std::env::var("TABDAT_SPIKE_FONT").unwrap_or_else(|_| {
    if cfg!(target_os = "macos") {
      "/System/Library/Fonts/Supplemental/Arial.ttf".into()
    } else {
      "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".into()
    }
  });
  let renderer = PngRenderer::from_font_data(std::fs::read(path).unwrap()).unwrap();
  for chart in [scatter_fixture(), histogram(), bar()] {
    let png = renderer.render(&chart).unwrap();
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    let pixmap = resvg::tiny_skia::Pixmap::decode_png(&png).unwrap();
    assert_eq!((pixmap.width(), pixmap.height()), (WIDTH, HEIGHT));
    let colored = pixmap
      .pixels()
      .iter()
      .filter(|p| u16::from(p.blue()) > u16::from(p.red()) + 20)
      .count();
    assert!(colored > 30, "raster lost the blue marks");
    assert_eq!(png, renderer.render(&chart).unwrap());
  }
  let bad = Chart::Bar {
    label: "c".into(),
    counts: vec![],
  };
  assert!(renderer.render(&bad).is_err());
  assert!(renderer.render(&scatter_fixture()).is_ok());
}
