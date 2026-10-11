#![forbid(unsafe_code)]

use serde_json::{Value, json};
use vl_convert_rs::{VlVersion, converter::VlOpts};

/// Inline synthetic data only. Matches the pinned Python scatter chart's
/// mark, axes, field titles and quantitative encodings for ordinary x/y names.
pub fn scatter_spec() -> Value {
  json!({
    "$schema": "https://vega.github.io/schema/vega-lite/v6.4.1.json",
    "config": {"view": {"continuousWidth": 300, "continuousHeight": 300}},
    "data": {"values": [{"x": 1, "y": 2}, {"x": 2, "y": 1}, {"x": 3, "y": 4}]},
    "mark": {"type": "point"},
    "encoding": {
      "x": {"field": "x", "title": "x", "type": "quantitative"},
      "y": {"field": "y", "title": "y", "type": "quantitative"}
    }
  })
}

pub fn options() -> VlOpts {
  VlOpts {
    vl_version: VlVersion::v6_4,
    allowed_base_urls: Some(vec![]),
    ..Default::default()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn specification_uses_inline_data_and_explicit_axes() {
    let spec = scatter_spec();
    assert_eq!(spec["data"]["values"].as_array().unwrap().len(), 3);
    assert!(spec["data"].get("url").is_none());
    assert_eq!(spec["mark"]["type"], "point");
    assert_eq!(spec["encoding"]["x"]["field"], "x");
    assert_eq!(spec["encoding"]["y"]["field"], "y");
    assert_eq!(spec["encoding"]["x"]["type"], "quantitative");
  }

  #[test]
  fn specification_matches_pinned_python_scatter() {
    let expected: Value =
      serde_json::from_str(include_str!("../fixtures/python-scatter.json")).unwrap();
    assert_eq!(scatter_spec(), expected);
  }

  #[tokio::test]
  async fn renders_svg_and_png_without_python() {
    let mut converter = vl_convert_rs::VlConverter::new();
    let svg = converter
      .vegalite_to_svg(scatter_spec(), options())
      .await
      .unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("role-mark"));
    let png = converter
      .vegalite_to_png(scatter_spec(), options(), None, None)
      .await
      .unwrap();
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert!(u32::from_be_bytes(png[16..20].try_into().unwrap()) > 300);
    assert!(u32::from_be_bytes(png[20..24].try_into().unwrap()) > 300);
  }
}
