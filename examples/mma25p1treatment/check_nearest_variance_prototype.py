#!/usr/bin/env python3
"""Smoke checks for the development-only, untrimmed variance prototype."""
import csv
import math
import random
import subprocess
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name('nearest_variance_prototype.R')
randomizer = random.Random(356)
rows = []
for i in range(300):
  x1, x2 = randomizer.uniform(-1, 1), randomizer.uniform(-1, 1)
  score = 1 / (1 + math.exp(-(-0.3 + 0.7 * x1 - 0.4 * x2)))
  treatment = int(randomizer.random() < score)
  outcome = 1.5 * treatment + x1 + x2 + randomizer.gauss(0, 1)
  rows.append([outcome, treatment, x1, x2])


def run(data):
  with tempfile.TemporaryDirectory() as directory:
    path = Path(directory) / 'data.csv'
    with path.open('w', newline='') as stream:
      writer = csv.writer(stream)
      writer.writerow(['y', 't', 'x1', 'x2'])
      writer.writerows(data)
    return subprocess.run(['Rscript', str(SCRIPT), str(path)], text=True, capture_output=True)


def values(result):
  assert result.returncode == 0, result.stderr
  return {key: float(value) for key, value in
          (item.split('=') for item in result.stdout.strip().split())}


original = values(run(rows))
randomizer.shuffle(rows)
permuted = values(run(rows))
for key in original:
  assert math.isclose(original[key], permuted[key], rel_tol=1e-8, abs_tol=1e-9), (key, original, permuted)
assert original['se'] > 0
assert original['base'] > 0
assert original['c_adjust'] >= 0 and original['d_adjust'] >= 0
bad = rows.copy()
bad[1] = bad[0].copy()
result = run(bad)
assert result.returncode != 0 and 'scores must be interior and distinct' in result.stderr, result.stderr
bad = rows.copy()
bad[0] = bad[0].copy()
bad[0][0] = float('nan')
result = run(bad)
assert result.returncode != 0 and 'invalid data' in result.stderr, result.stderr
print('PASS: order invariance, finite variance components, score ties and invalid-data rejection')
