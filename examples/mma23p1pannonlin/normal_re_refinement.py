#!/usr/bin/env python3
"""Build the development-only 96/128-node quadrature refinement script."""
from pathlib import Path
import sys

if len(sys.argv) != 4:
  raise SystemExit('usage: normal_re_refinement.py normal_re_poisson.td nodes96-128.sql output.td')
source, nodes, output = map(Path, sys.argv[1:])
text = source.read_text()
marker = 'quadrature(level,node,z,weight) AS (VALUES\n'
start, rest = text.split(marker)
_, end = rest.split('\n),\nsearch(', 1)
assert end.count('high.level=96 AND low.level=80') == 1
text = start + marker + nodes.read_text().rstrip() + '\n),\nsearch(' + end.replace('high.level=96 AND low.level=80', 'high.level=128 AND low.level=96')
with output.open('x') as f:
  f.write(text)
