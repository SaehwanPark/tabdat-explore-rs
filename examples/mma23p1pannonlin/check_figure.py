#!/usr/bin/env python3
"""Audit source/design, independent LOWESS equations, R QR and actual SVG geometry."""
import csv
import hashlib
import json
import math
from pathlib import Path
import re
import sys
import xml.etree.ElementTree as ET

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'mma09p1np'))
from check_lowess import smooth  # independent 2x2 normal equations

RAW = ['id', 'year', 'cusip', 'ardssic', 'scisect', 'logk', 'sumpat', 'logr', 'pat']
DERIVED = ['lpat', 'figure_linear', 'figure_lowess']
SERIES = ['figure_lowess', 'figure_linear']

def rows(path):
  with Path(path).open(newline='') as f:
    return list(csv.DictReader(f))

def close(a, b, label='value'):
  a, b = float(a), float(b)
  tolerance = 1e-11 + 1e-10 * abs(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a - b) <= tolerance, (label, a, b, tolerance)

def audit(work, reference, result, original=None):
  work, reference = Path(work), Path(reference)
  if original:
    assert hashlib.sha256(Path(original).read_bytes()).hexdigest() == '87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8'
  source = rows(work / 'patents.csv')
  table = rows(work / 'figure_data.csv')
  ref = rows(reference / 'rows.csv')
  meta, = rows(reference / 'meta.csv')
  n = len(source)
  assert n == len(table) == len(ref) == int(meta['observations'])
  assert list(table[0]) == list(ref[0]) == RAW + DERIVED
  src = {(int(r['id']), int(r['year'])): r for r in source}
  assert len(src) == n
  keys = [(int(r['id']), int(r['year'])) for r in table]
  assert keys == sorted(src) and len(keys) == n
  x = [float(r['logr']) for r in table]
  y = [math.log(float(r['pat']) if float(r['pat']) > 0 else .5) for r in table]
  fit, k, constant = smooth(x, y, .8)
  assert k == int(meta['half_window']) and constant == int(meta['constant_windows'])
  assert 0 <= float(meta['max_scaled_score']) < 1e-12
  for i, (row, expected) in enumerate(zip(table, ref)):
    key = keys[i]
    assert key == (int(expected['id']), int(expected['year']))
    for name in RAW:
      value = src[key][name]
      if value == '':
        assert row[name] == expected[name] == ''
      else:
        assert float(row[name]) == float(value) == float(expected[name]), (key, name)
    for name, calculated in [('lpat', y[i]), ('figure_lowess', fit[i])]:
      close(row[name], calculated, (key, name, 'equation'))
      close(expected[name], calculated, (key, name, 'R'))
    close(row['figure_linear'], expected['figure_linear'], (key, 'linear'))
  events = list(map(json.loads, Path(result).read_text().splitlines()))
  def one(kind):
    found = [e['data'] for e in events if e.get('result_type') == kind]
    assert len(found) == 1, (kind, len(found))
    return found[0]
  regression, linear, lowess = one('RegressionResult'), one('LincomResult'), one('LowessResult')
  assert regression['covariance'] == 'cluster(id)' and regression['observation_count'] == n
  assert regression['sample']['dropped_observations'] == 0
  for actual, name in [(regression['coefficients'][0]['value'], 'intercept'), (regression['coefficients'][1]['value'], 'slope'), (regression['covariance_matrix'][0][0], 'cov00'), (regression['covariance_matrix'][0][1], 'cov01'), (regression['covariance_matrix'][1][1], 'cov11'), (linear['estimate'], 'slope'), (linear['standard_error'], 'slope_se'), (linear['statistic'], 'slope_t'), (linear['df_residual'], 'inference_df')]:
    close(actual, meta[name], name)
  assert abs(linear['p_value'] - float(meta['slope_p'])) <= 1e-9 * float(meta['slope_p'])
  assert lowess['algorithm'] == 'centered_rank_local_linear_tricube' and lowess['bandwidth'] == .8
  assert lowess['half_window'] == k and lowess['local_constant_windows'] == constant
  assert lowess['estimation_state'] == 'unchanged' and lowess['sample']['retained_indices'] == list(range(n))
  check_svg(work / 'figure.svg', table)
  return n

def check_svg(path, table):
  root = ET.parse(path).getroot(); a = root.attrib; n = len(table)
  assert (a['width'], a['height'], a['viewBox'], a['role']) == ('900', '660', '0 0 900 660', 'img')
  assert a['data-candidate-rows'] == str(n) and float(a['data-jitter-y']) == 0
  assert a['data-coordinate-precision'] == 'float64'
  for key, value in [('left', 90), ('top', 75), ('width', 730), ('height', 400)]:
    assert float(a['data-plot-' + key]) == value
  x = [float(r['logr']) for r in table]
  y = [float(r[key]) for r in table for key in ['lpat', *SERIES]]
  for axis, values in [('x', x), ('y', y)]:
    lo, hi = min(values), max(values)
    pad = (hi - lo) * .05 if hi > lo else max(abs(lo), 1) * .05
    close(a['data-' + axis + '-min'], lo - pad, axis + ' min')
    close(a['data-' + axis + '-max'], hi + pad, axis + ' max')
  def pixel(xx, yy):
    return (90 + (xx-float(a['data-x-min']))/(float(a['data-x-max'])-float(a['data-x-min']))*730,
            75 + (float(a['data-y-max'])-yy)/(float(a['data-y-max'])-float(a['data-y-min']))*400)
  text = [e.text for e in root.iter() if e.text]
  for label in ['Pooled (Overall) Regression', 'Log R&D spending', 'Log patents (zero mapped to ln(0.5))', 'Nonparametric fit (span 0.8)', 'Pooled linear fit']:
    assert label in text, label
  order = sorted(range(n), key=lambda i: (x[i], i))
  lines = [e for e in root.iter() if e.attrib.get('class') == 'line-series']
  assert [e.attrib['data-variable'] for e in lines] == SERIES
  number = r'[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?'
  for node, name in zip(lines, SERIES):
    b = node.attrib
    assert list(map(int, b['data-row-indices'].split(','))) == order
    xx, yy = b['data-x-values'].split(','), b['data-y-values'].split(',')
    assert re.fullmatch(f'M{number} {number}(?:L{number} {number}){{{n-1}}}', b['d'])
    points = re.findall(f'[ML]({number}) ({number})', b['d'])
    assert len(xx) == len(yy) == len(points) == n
    for i, vx, vy, point in zip(order, xx, yy, points):
      value = float(table[i][name]); close(vx, x[i]); close(vy, value)
      assert all(abs(float(c)-v)<.001 for c, v in zip(point, pixel(x[i], value)))
  marks = [e for e in root.iter() if e.attrib.get('class') == 'scatter-point']
  assert len(marks) == n
  for i, node in enumerate(marks):
    b = node.attrib; value = float(table[i]['lpat'])
    assert int(b['data-row-index']) == i and b['data-x-variable'] == 'logr' and b['data-y-variable'] == 'lpat'
    close(b['data-x'], x[i]); close(b['data-y'], value); close(b['data-display-y'], value)
    assert all(abs(float(b[key])-v)<.001 for key, v in zip(('cx', 'cy'), pixel(x[i], value)))

if __name__ == '__main__':
  if len(sys.argv) not in (4, 5):
    raise SystemExit('usage: check_figure.py work-dir reference-dir result.jsonl [original-source.asc]')
  n = audit(*sys.argv[1:])
  print(f'PASS: {n} exact source/QR/normal-equation/linear rows, {n} SVG markers, {2*n} connected SVG vertices; historical tied-curve parity unclaimed')
