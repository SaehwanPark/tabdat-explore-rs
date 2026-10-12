"""External pinned-oracle probe; Python is not a runtime renderer dependency.

PYTHONPATH=../tabdat-explore/src ../tabdat-explore/.venv/bin/python \
  spikes/native-svg-prototype/fixtures/python_charts.py OUTPUT_DIR
"""
import json
import sys
from pathlib import Path
from xml.etree import ElementTree as ET
from tabdat import visualization

output = Path(sys.argv[1])
output.mkdir(parents=True, exist_ok=True)
visualization.save_scatter(((2, 1), (1, 2), (4, 3)), "y", "x", output / "scatter.svg")
visualization.save_scatter(((2, 1), (1, 2), (4, 3)), "y", "x", output / "scatter.png")
visualization.save_histogram(((0.2,), (0.4,), (1.1,), (2.0,), (2.8,), (None,)), "value", output / "histogram.svg", bins=3)
visualization.save_histogram(((0.2,), (0.4,), (1.1,), (2.0,), (2.8,), (None,)), "value", output / "histogram.png", bins=3)
visualization.save_bar((("b", 3), (None, 1), ("a", 2)), "category", output / "bar.svg")
visualization.save_bar((("b", 3), (None, 1), ("a", 2)), "category", output / "bar.png")
marks = {}
for name in ("scatter", "histogram", "bar"):
  root = ET.parse(output / f"{name}.svg").getroot()
  marks[name] = [node.attrib["aria-label"] for node in root.iter() if node.attrib.get("role") == "graphics-symbol" and node.attrib.get("aria-roledescription") in ("point", "bar")]
(output / "python-marks.json").write_text(json.dumps(marks, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(marks, ensure_ascii=False, indent=2))
