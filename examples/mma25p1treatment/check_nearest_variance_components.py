#!/usr/bin/env python3
"""Compare prototype components with a pinned, independently implemented reference.

Development-only; requires NumPy and pandas and a separately downloaded StatsPAI
_ai2016.py. The repository does not depend on these packages at runtime.
"""
import hashlib
import importlib.util
from pathlib import Path
import subprocess
import sys

import numpy as np
import pandas as pd

if len(sys.argv) != 3:
  raise SystemExit('usage: check_nearest_variance_components.py psm.csv statspai_ai2016.py')
data_path, reference_path = map(Path, sys.argv[1:])
expected_hash = '2f278d2700585a1de87e79de883e1e90c4d7a4dcef8aa6cb5e348865435099cc'
if hashlib.sha256(reference_path.read_bytes()).hexdigest() != expected_hash:
  raise SystemExit('unexpected reference implementation checksum')
spec = importlib.util.spec_from_file_location('independent_ai', reference_path)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
data = pd.read_csv(data_path)
x = data[['x1', 'x2']].to_numpy()
z = np.column_stack((np.ones(len(x)), x))
t = data.t.to_numpy(dtype=int)
y = data.y.to_numpy()
beta = np.zeros(z.shape[1])
for _ in range(30):
  p = 1 / (1 + np.exp(-z @ beta))
  info = z.T @ ((p * (1 - p))[:, None] * z)
  step = np.linalg.solve(info, z.T @ (t - p))
  beta += step
  if np.max(np.abs(step)) < 1e-13:
    break
else:
  raise RuntimeError('independent logit did not converge')
p = 1 / (1 + np.exp(-z @ beta))
info = z.T @ ((p * (1 - p))[:, None] * z)
treated, controls = np.flatnonzero(t == 1), np.flatnonzero(t == 0)
matches = []
for i in treated:
  distance = np.abs(p[controls] - p[i])
  matches.append(np.flatnonzero(distance == distance.min()))
weights = [np.full(len(m), 1 / len(m)) for m in matches]
se, components = module.abadie_imbens_2016_se(
  y, t, p, z, np.linalg.inv(info), x, treated, controls, matches, weights, 1)
script = Path(__file__).with_name('nearest_variance_prototype.R')
result = subprocess.run(['Rscript', str(script), str(data_path)], text=True, capture_output=True, check=True)
actual = {key: float(value) for key, value in
          (item.split('=') for item in result.stdout.strip().split())}
expected = {'effect': components['att'], 'base': components['base_var'],
            'c_adjust': components['c_V_c'], 'd_adjust': components['d_V_d'], 'se': se}
for key, value in expected.items():
  if not np.isclose(actual[key], value, rtol=0, atol=5.1e-11):
    raise AssertionError((key, actual[key], value))
print('PASS: independent logit, point estimate and all three variance components agree')
