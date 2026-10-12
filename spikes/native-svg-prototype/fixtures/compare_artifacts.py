"""Compare owned Rust artifacts against Python artifacts (stdlib only).

python compare_artifacts.py PYTHON_DIRECTORY RUST_DIRECTORY
This validates semantics/relative scatter geometry, not pixel or byte parity.
"""
import json
import re
import struct
import sys
from pathlib import Path
from xml.etree import ElementTree as ET

python_dir, rust_dir = map(Path, sys.argv[1:])
results = {}
for name in ("scatter", "histogram", "bar"):
  expected = ET.parse(python_dir / f"{name}.svg").getroot()
  actual = ET.parse(rust_dir / f"{name}.svg").getroot()
  python_marks = [node for node in expected.iter() if node.attrib.get("role") == "graphics-symbol" and node.attrib.get("aria-roledescription") in ("point", "bar")]
  rust_marks = [node for node in actual.iter() if node.attrib.get("class") == "mark"]
  assert [node.attrib["aria-label"] for node in python_marks] == [node.attrib["aria-label"] for node in rust_marks], name
  if name == "scatter":
    for original, replacement in zip(python_marks, rust_marks, strict=True):
      x, y = map(float, re.fullmatch(r"translate\(([^,]+),([^\)]+)\)", original.attrib["transform"]).groups())
      assert abs(x - (float(replacement.attrib["cx"]) - 45)) <= 1e-10
      assert abs(y - (float(replacement.attrib["cy"]) - 10)) <= 1e-10
  dimensions = []
  for directory in (python_dir, rust_dir):
    png = (directory / f"{name}.png").read_bytes()
    assert png[:8] == b"\x89PNG\r\n\x1a\n"
    dimensions.append(struct.unpack(">II", png[16:24]))
  results[name] = {"mark_semantics_equal": True, "python_dimensions": dimensions[0], "rust_dimensions": dimensions[1], "png_bytes_equal": (python_dir / f"{name}.png").read_bytes() == (rust_dir / f"{name}.png").read_bytes()}
print(json.dumps(results, indent=2))
