#![forbid(unsafe_code)]

use std::{hint::black_box, time::Instant};
use tabdat_native_svg_spike::{Chart, PngRenderer, render_svg, scatter_fixture};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<_> = std::env::args().skip(1).collect();
  if args.first().is_some_and(|arg| arg == "--metadata") {
    println!("native SVG / resvg 0.48.1 spike; no font discovery");
    return Ok(());
  }
  let chart = scatter_fixture();
  let start = Instant::now();
  let svg = render_svg(&chart)?;
  println!("first_svg_us={:.3}", start.elapsed().as_secs_f64() * 1e6);
  let start = Instant::now();
  for _ in 0..1_000 {
    black_box(render_svg(black_box(&chart))?);
  }
  println!("repeated_svg_us={:.3}", start.elapsed().as_secs_f64() * 1e3);
  let start = Instant::now();
  let path = std::env::var("TABDAT_SPIKE_FONT").unwrap_or_else(|_| {
    if cfg!(target_os = "macos") {
      "/System/Library/Fonts/Supplemental/Arial.ttf".into()
    } else {
      "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".into()
    }
  });
  let renderer = PngRenderer::from_font_data(std::fs::read(path)?)?;
  println!(
    "font_read_init_ms={:.3}",
    start.elapsed().as_secs_f64() * 1e3
  );
  let start = Instant::now();
  let png = renderer.render(&chart)?;
  println!("first_png_ms={:.3}", start.elapsed().as_secs_f64() * 1e3);
  let start = Instant::now();
  for _ in 0..100 {
    black_box(renderer.render(black_box(&chart))?);
  }
  println!(
    "repeated_png_ms={:.3}",
    start.elapsed().as_secs_f64() * 10.0
  );
  if let Some(directory) = args.first() {
    let directory = std::path::Path::new(directory);
    std::fs::create_dir_all(directory)?;
    std::fs::write(directory.join("scatter.svg"), svg)?;
    std::fs::write(directory.join("scatter.png"), png)?;
    for (name, chart) in [
      (
        "histogram",
        Chart::Histogram {
          label: "value".into(),
          edges: vec![0.0, 1.0, 2.0, 3.0],
          values: vec![Some(0.2), Some(0.4), Some(1.1), Some(2.0), Some(2.8), None],
        },
      ),
      (
        "bar",
        Chart::Bar {
          label: "category".into(),
          counts: vec![("b".into(), 3), ("<missing>".into(), 1), ("a".into(), 2)],
        },
      ),
    ] {
      std::fs::write(directory.join(format!("{name}.svg")), render_svg(&chart)?)?;
      std::fs::write(
        directory.join(format!("{name}.png")),
        renderer.render(&chart)?,
      )?;
    }
  }
  Ok(())
}
