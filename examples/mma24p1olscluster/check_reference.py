#!/usr/bin/env python3
"""Audit the actual structured run, source CSV and predictions against R."""
import csv
import hashlib
import json
import math
import shlex
import sys
from pathlib import Path


def csv_rows(path):
  with path.open(newline='') as f:
    return list(csv.DictReader(f))


def close(a, b, pvalue=False):
  a, b = float(a), float(b)
  assert math.isfinite(a) and math.isfinite(b), (a, b)
  tol = (abs(b) * 1e-8 if b else 0) if pvalue else 1e-9
  assert abs(a-b) <= tol, (a, b, tol)


def main():
  if len(sys.argv) != 4:
    raise SystemExit('usage: check_reference.py run-directory R-reference-directory vietnam_ex1.asc')
  run, ref, source = map(Path, sys.argv[1:])
  assert hashlib.sha256(source.read_bytes()).hexdigest() == '74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061'
  events = [json.loads(line) for line in (run/'run.jsonl').read_text().splitlines()]
  assert all(e['schema_version'] == 1 and 'error' not in e for e in events)
  select = lambda name: [e['data'] for e in events if e['result_type'] == name]
  fits = select('RegressionResult')
  assert len(fits) == 3
  expected_cov = ['nonrobust', 'robust', 'cluster(commune)']
  models = dict(zip(['nonrobust','robust','cluster'], fits))
  for fit, cov in zip(fits, expected_cov):
    assert fit['estimator'] == 'ols' and fit['covariance'] == cov
    assert fit['outcome'] == 'lnexp12m' and fit['include_intercept']
    assert fit['predictors'] == ['lnhhexp','age','sex','hhsize','farm','educ']
    assert fit['observation_count'] == 5006 and fit['df_residual'] == 4999
    assert fit['sample'] == {'total_observations': 5006, 'dropped_observations': 0}
  fields = csv_rows(ref/'fits.csv')
  assert len(fields) == 240 and len({(x['model'], x['path']) for x in fields}) == 240
  for row in fields:
    actual = models[row['model']]
    for key in row['path'].strip('/').split('/'):
      actual = actual[int(key)] if isinstance(actual, list) else actual[key]
    close(actual, row['value'], row['path'].endswith('p_value'))
  source_rows, source_expected = csv_rows(run/'vietnam.csv'), csv_rows(ref/'source.csv')
  assert len(source_rows) == len(source_expected) == 5999
  assert sum(r['lnexp12m']=='' for r in source_rows) == 993
  tokens = shlex.split(source.read_text(encoding='ascii'))
  assert len(tokens)==5999*8
  farms = {'non farm':0, 'farm':1}
  educ = {'Never':0, 'Nhatre':1, '<cap I':2, 'Cap I':3, 'Cap II':4,
          'Cap III':5, 'Nghe SC':6, 'THCN':7, 'DHDC':8, 'DHCD':9,
          'Thac sy':10, 'PTS':11, 'TS':12}
  for i, (a,b) in enumerate(zip(source_rows,source_expected),1):
    assert set(a)==set(b) and int(a['source_row'])==i
    y,age,sex,size,farm,education,x,commune = tokens[8*(i-1):8*i]
    original = {'lnexp12m': '' if y=='.' else y, 'age':age, 'sex':sex,
                'hhsize':size, 'farm':farms[farm], 'educ':educ[education],
                'lnhhexp':x, 'commune':commune}
    for k, value in original.items():
      if value=='':
        assert a[k]==''
      else:
        assert float(a[k])==float(value), (i,k,a[k],value)
    for k in a:
      if b[k] == '':
        assert a[k] == ''
      else:
        assert float(a[k]) == float(b[k]), (i,k,a[k],b[k])
  actual, expected = csv_rows(run/'household_ols_predictions.csv'), csv_rows(ref/'predictions.csv')
  assert len(actual)==len(expected)==5006
  for a,b in zip(actual, expected):
    assert set(a)==set(b)
    for k in a:
      if k in ('household_fitted','household_residual'):
        close(a[k],b[k])
      else:
        assert float(a[k]) == float(b[k]), (k,a[k],b[k])
    close(float(a['lnexp12m'])-float(a['household_fitted']),a['household_residual'])
  table = select('EstimateTableResult')[0]['models']
  assert [m['inference_degrees_of_freedom'] for m in table] == [4999,4999,193]
  assert table[2]['cluster_count']==194 and table[2]['cluster_variable']=='commune'
  test, lincom = select('TestResult')[0], select('LincomResult')[0]
  assert not test['is_chi2'] and test['df']==1 and test['df_residual']==lincom['df_residual']==193
  post = csv_rows(ref/'post.csv')
  assert len(post)==8 and len({(r['result'],r['path']) for r in post})==8
  for row in post:
    result = {'test':test, 'lincom':lincom}[row['result']]
    close(result[row['path'][1:]],row['value'],row['path'].endswith('p_value'))
  coefficient = fits[2]['coefficients'][1]
  for field in ('standard_error','statistic','p_value'):
    close(lincom[field],coefficient[field],field=='p_value')
  close(lincom['estimate'],coefficient['value'])
  close(test['statistic'],lincom['statistic']**2)
  close(test['p_value'],lincom['p_value'],True)
  assert select('ExportResult')[0]['dataset']['row_count']==5006
  print('validated 240 R fit fields, 8 post-estimation fields, 5999 source rows and 5006 prediction rows')


if __name__ == '__main__':
  main()
