#!/usr/bin/env python3
"""Audit BP LM source cohort, original pooled fit, residual moments and exports."""
import csv
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import re
import shlex
import sys


def rows(path):
  with path.open(newline='') as f:
    return list(csv.DictReader(f))


def near(actual, expected, p=False):
  a, b = float(actual), float(expected)
  assert math.isfinite(a) and math.isfinite(b), (a, b)
  tolerance = abs(b)*1e-8 if p else 1e-9
  assert abs(a-b) <= tolerance, (a, b, tolerance)


def audit_source(native, source_reference, raw, mode='--ascii'):
  hashes = {'--ascii': '74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061',
            '--dta': '0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf'}
  if mode in hashes:
    assert hashlib.sha256(raw.read_bytes()).hexdigest() == hashes[mode]
  source = rows(native/'vietnam.csv')
  expected_source = rows(source_reference)
  keys = ['source_row','lnexp12m','age','sex','hhsize','farm','educ','lnhhexp','commune']
  assert len(source) == len(expected_source) > 7
  if mode in hashes:
    assert len(source) == 5999
  if mode == '--csv':
    assert rows(raw) == source
  tokens = shlex.split(raw.read_text(encoding='ascii')) if mode == '--ascii' else None
  farms = {'non farm': 0, 'farm': 1}
  educ = {'Never':0,'Nhatre':1,'<cap I':2,'Cap I':3,'Cap II':4,'Cap III':5,'Nghe SC':6,
          'THCN':7,'DHDC':8,'DHCD':9,'Thac sy':10,'PTS':11,'TS':12}
  if tokens is not None:
    assert len(tokens) == len(source)*8
  for i, (a, b) in enumerate(zip(source, expected_source)):
    assert list(a) == list(b) == keys
    for key in keys:
      if a[key] == '':
        assert b[key] == ''
      else:
        assert float(a[key]) == float(b[key]), (i, key)
    if tokens is not None:
      assert int(a['source_row']) == i+1
      y, age, sex, size, farm, education, x, group = tokens[i*8:i*8+8]
      values = [i+1, '' if y == '.' else y, age, sex, size, farms[farm], educ[education], x, group]
      for key, value in zip(keys, values):
        if value == '':
          assert a[key] == ''
        else:
          assert Decimal(a[key]) == Decimal(str(value)), (i, key)
  return source


def audit(native, reference, raw, mode='--ascii', author=None):
  source = audit_source(native, reference/'source.csv', raw, mode)
  keys = list(source[0])
  events = [json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version'] == 1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events] == ['LoadResult','IsidResult','TableResult',
    'SqlCreateResult','TableResult','RegressionResult','PredictionResult','PredictionResult',
    'SqlCreateResult','TableResult','ExportResult','ActivateResult','SummarizeResult','ExportResult']
  fit = events[5]['data']
  assert fit['estimator'] == 'ols' and fit['covariance'] == 'nonrobust'
  names = ['intercept','lnhhexp','age','sex','hhsize','farm','educ']
  assert fit['covariance_parameter_names'] == names
  assert [c['name'] for c in fit['coefficients']] == names
  refs = rows(reference/'fit.csv')
  assert len(refs) == len({r['path'] for r in refs}) == 56
  for r in refs:
    value = fit
    for key in r['path'].strip('/').split('/'):
      value = value[int(key)] if isinstance(value, list) else value[key]
    near(value, r['value'])
  selected = sorted((r for r in source if r['lnexp12m'] != ''), key=lambda r: int(r['source_row']))
  actual, expected = rows(native/'household_bp_predictions.csv'), rows(reference/'predictions.csv')
  assert len(actual) == len(expected) == len(selected) == fit['observation_count']
  assert fit['sample'] == {'total_observations':len(selected),'dropped_observations':0}
  assert fit['df_residual'] == len(selected)-7
  groups = {}
  for a, b, s in zip(actual, expected, selected):
    assert list(a) == list(b) == keys+['bp_xb','bp_residual']
    for key in keys:
      assert float(a[key]) == float(b[key]) == float(s[key]), (s['source_row'], key)
    for key in ('bp_xb','bp_residual'):
      near(a[key], b[key])
    xb = fit['coefficients'][0]['value'] + math.fsum(fit['coefficients'][j]['value']*float(s[name])
      for j, name in enumerate(names[1:], 1))
    near(a['bp_xb'], xb)
    near(float(a['bp_xb'])+float(a['bp_residual']), s['lnexp12m'])
    groups.setdefault(s['commune'], []).append(float(a['bp_residual']))
  diag, refdiag = rows(native/'household_bp_diagnostic.csv'), rows(reference/'diagnostic.csv')
  assert len(diag) == len(refdiag) == 1
  diag, refdiag = diag[0], refdiag[0]
  assert len(refdiag) == 13
  assert set(diag) == set(refdiag)|{'method','reference_distribution','scope'}
  for key, value in refdiag.items():
    near(diag[key], value, key == 'p_value')
  assert diag['method'] == 'pooled_residual_breusch_pagan_unbalanced'
  assert diag['reference_distribution'] == 'legacy_squared_score_chi_square_1'
  assert 'not robust' in diag['scope'] and 'one-sided' in diag['scope']
  n = len(selected)
  rss = math.fsum(e*e for es in groups.values() for e in es)
  q = math.fsum(math.fsum(es)**2 for es in groups.values())
  m = sum(len(es)**2 for es in groups.values())
  z = n*(q/rss-1)/math.sqrt(2*(m-n))
  for key, value in {'observations':n,'communes':len(groups),'sum_group_size_squared':m,
      'residual_sum_of_squares':rss,'group_residual_sum_squares':q,
      'signed_score':z,'lm_statistic':z*z,'df':1}.items():
    near(diag[key], value)
  near(diag['p_value'], math.erfc(abs(z)/math.sqrt(2)), True)
  if mode in ('--ascii','--dta'):
    assert (n,len(groups),min(map(len,groups.values())),max(map(len,groups.values()))) == (5006,194,1,39)
  if author is not None:
    assert mode == '--dta'
    assert hashlib.sha256(author.read_bytes()).hexdigest() == '5e74c8de58eaf833b0070d68b289d61fad43831f3eb8b57c06818a6bf549837e'
    block = author.read_text().split('Breusch and Pagan Lagrangian multiplier test for random effects:')[1].split('. * Hausman')[0]
    printed = float(re.search(r'chi2\(1\)\s*=\s*([0-9.]+)', block)[1])
    assert abs(float(diag['lm_statistic'])-printed) <= .005
  assert events[-1]['data']['dataset']['row_count'] == n
  return n, len(groups)


def main():
  if len(sys.argv) not in (4,5,6):
    raise SystemExit('usage: check_household_bp_lm.py native-dir reference-dir source [--ascii|--dta|--csv] [author-log]')
  mode = sys.argv[4] if len(sys.argv)>4 else '--ascii'
  assert mode in ('--ascii','--dta','--csv')
  n, g = audit(*map(Path,sys.argv[1:4]),mode,Path(sys.argv[5]) if len(sys.argv)==6 else None)
  print('validated 69 independent R QR/plm quantities, all source rows and',n,'prediction rows;',g,'groups')


if __name__ == '__main__':
  main()
