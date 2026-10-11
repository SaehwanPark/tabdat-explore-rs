"""External oracle probe; run with the clean pinned Python checkout environment.

Usage: PYTHONPATH=/path/to/oracle/src python python_scatter.py OUTPUT_DIRECTORY
Python is validation tooling only, never part of the Rust render path.
"""
import json
import sys
from pathlib import Path
from tabdat import visualization

output = Path(sys.argv[1])
output.mkdir(parents=True, exist_ok=True)
original_save = visualization._save_chart


def capture(chart, path):
  (output / "scatter.json").write_text(json.dumps(chart.to_dict(), indent=2) + "\n")
  return original_save(chart, path)


visualization._save_chart = capture
rows = ((2, 1), (1, 2), (4, 3))  # Python scatter rows are ordered (y, x).
visualization.save_scatter(rows, "y", "x", output / "scatter.svg")
visualization.save_scatter(rows, "y", "x", output / "scatter.png")
