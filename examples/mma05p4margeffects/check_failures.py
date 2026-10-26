#!/usr/bin/env python3
"""Exercise preflight and export failures against the actual CLI script."""
import csv
from pathlib import Path
import subprocess
import sys
import tempfile

if len(sys.argv) != 2:
  raise SystemExit('usage: check_failures.py /absolute/path/to/tabdat-explore-rs')
binary = str(Path(sys.argv[1]).resolve())
script = Path(__file__).resolve().with_name('workflow.td')
cases = [
  [('1', '1', '0'), ('2', '0', '1'), ('3', '2', '2')],  # invalid response
  [('1', '1', '0'), ('1', '2', '1'), ('3', '3', '2')],  # duplicate identity
  [('1', '1', '0'), ('2', '2', '0'), ('3', '3', '0')],  # nonidentified slope
  [('1', '1', '0'), ('2', '2', '1'), ('4', '3', '2')],  # missing identity
]
for rows in cases:
  with tempfile.TemporaryDirectory() as directory:
    root = Path(directory)
    with (root / 'nls.csv').open('w', newline='') as f:
      writer = csv.writer(f)
      writer.writerow(['source_row', 'y', 'x'])
      writer.writerows(rows)
    result = subprocess.run([binary, '--json', '-f', str(script)], cwd=root, capture_output=True, text=True)
    assert result.returncode != 0, (rows, result.stdout, result.stderr)
    assert not (root / 'exponential_effects.csv').exists()
print(f'{len(cases)} invalid-input cases fail before export')
