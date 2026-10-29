#!/usr/bin/env python3
"""Generate the inspectable 40-firm gamma-RE bootstrap fixture in a new file."""
import math
from pathlib import Path
import sys

if len(sys.argv) != 2:
  raise SystemExit('usage: gamma_re_bootstrap_synthetic.py output.asc')
out = Path(sys.argv[1])
with out.open('x') as f:
  for i in range(1, 41):
    amplitude = [0, 1, 3, 6, 12][(i - 1) % 5]
    logr = [-.4 + .021 * i + .08 * t + .01 * ((i * t) % 7) for t in range(10)]
    patents = [round(amplitude * math.exp(.2 * x) * (.85 + .05 * ((t + i) % 7)))
               for t, x in enumerate(logr)]
    fields = [i + 1000, '.' if i == 3 else 10 + i % 5, i % 2,
              f'{1 + i / 100:.5f}', sum(patents),
              *(f'{x:.5f}' for x in logr), *patents]
    print(*fields, file=f)
