#![forbid(unsafe_code)]

use std::{fs, time::Instant};
use tabdat_visualization_spike::{options, scatter_spec};
use vl_convert_rs::VlConverter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let output = std::env::args_os()
    .nth(1)
    .expect("usage: measure OUTPUT_DIRECTORY");
  fs::create_dir_all(&output)?;
  let output = std::path::PathBuf::from(output);
  let spec = scatter_spec();
  fs::write(
    output.join("scatter.json"),
    serde_json::to_vec_pretty(&spec)?,
  )?;
  let start = Instant::now();
  let mut converter = VlConverter::new();
  let construction_us = start.elapsed().as_micros();
  let start = Instant::now();
  let svg = converter.vegalite_to_svg(spec.clone(), options()).await?;
  let first_svg_us = start.elapsed().as_micros();
  fs::write(output.join("scatter.svg"), svg)?;
  let start = Instant::now();
  let png = converter
    .vegalite_to_png(spec.clone(), options(), None, None)
    .await?;
  let png_us = start.elapsed().as_micros();
  fs::write(output.join("scatter.png"), png)?;
  let start = Instant::now();
  for _ in 0..10 {
    let _ = converter.vegalite_to_svg(spec.clone(), options()).await?;
  }
  println!(
    "construction_us={construction_us}\nfirst_svg_us={first_svg_us}\npng_us={png_us}\nrepeated_10_svg_us={}",
    start.elapsed().as_micros()
  );
  Ok(())
}
