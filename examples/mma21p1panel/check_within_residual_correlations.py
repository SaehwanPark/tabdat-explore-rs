#!/usr/bin/env python3
"""Audit FE conditional residual correlations against raw MOM, full-dummy QR and author log."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import re

SOURCE = 'lnhr lnwg kids ageh agesq disab id year'.split()
RAW = ['source_row'] + SOURCE


def records(path):
  with path.open(newline='') as f:
    return list(csv.DictReader(f))


def close(actual, expected, label):
  tolerance = abs(expected)*1e-8 if str(label).endswith('p_value') else 1e-9
  assert math.isfinite(actual) and math.isfinite(expected) and abs(actual-expected) <= tolerance, (label,actual,expected)


def pointer(data, path):
  for key in path.split('/')[1:]:
    data = data[int(key)] if isinstance(data,list) else data[key]
  return data


def author_values(path):
  assert hashlib.sha256(path.read_bytes()).hexdigest() == '925aff11176cb78bbacf71abc69c2bef4876bdcc0e5544c001c2df4f5f8f56b5'
  text = path.read_text(); text = text[text.index('. corr ufe1979'):text.index('. regress lnhr1979')]
  found = {}; seen = set()
  for line in text.splitlines():
    match = re.match(r'\s*ufe(19\d\d)\s*\|\s*(.*)',line)
    if not match:
      continue
    year = int(match.group(1))
    if year in seen:
      continue  # Wrapped final diagonal is displayed in a second table block.
    seen.add(year)
    for i,value in enumerate(match.group(2).split()):
      other = 1979+i
      if other < year:
        found[(other,year)] = float(value)
  assert seen == set(range(1979,1989)) and len(found) == 45
  return found


def audit(native,reference,raw,author_log=None,synthetic=False):
  if not synthetic:
    assert hashlib.sha256(raw.read_bytes()).hexdigest() == '2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
    assert author_log is not None
  values = [float(v) for v in raw.read_text().split()]
  assert len(values)%8 == 0 and all(math.isfinite(v) for v in values)
  source = [dict(zip(SOURCE,values[i:i+8])) for i in range(0,len(values),8)]
  n,g = len(source),len({r['id'] for r in source})
  assert 3 <= g <= 532 and n == 10*g
  events = [json.loads(l) for l in (native/'within_residual_correlations.jsonl').read_text().splitlines()]
  assert all(e['schema_version'] == 1 and 'error' not in e for e in events)
  select = lambda kind: [e['data'] for e in events if e['result_type'] == kind]
  fit, = select('FixedEffectsResult'); lincom, = select('LincomResult')
  assert fit['outcome'] == 'lnhr' and fit['predictors'] == ['lnwg'] and fit['estimator'] == 'within_fixed_effects'
  assert fit['entity_variable'] == 'id' and fit['time_variable'] == 'year'
  assert fit['unit_effect_normalization'] == 'estimation_observation_weighted_zero_mean'
  assert fit['covariance'] == 'nonrobust' and fit['reference_distribution'] == 'student_t'
  assert fit['sample'] == dict(total_observations=n,dropped_observations=0)
  refs = records(reference/'reference.csv'); assert len(refs) == len({(r['result_type'],r['path']) for r in refs}) == 25
  for row in refs:
    close(pointer({'FixedEffectsResult':fit,'LincomResult':lincom}[row['result_type']],row['path']),float(row['value']),row['path'])
  kinds = [e['result_type'] for e in events]
  assert kinds[-4:] == ['SqlCreateResult','TableResult','LincomResult','ExportResult']
  assert 'descriptive, not a test or covariance correction' in json.dumps(events)
  exports = select('ExportResult'); assert len(exports) == 2
  for event,name,count in zip(exports,['panel_within_residual_rows.csv','panel_within_residual_correlations.csv'],[n,100]):
    assert event['path'] == event['dataset']['path'] == name and event['dataset']['row_count'] == count
  actual = records(native/'panel_within_residual_rows.csv'); expected = records(reference/'rows.csv')
  assert len(actual)==len(expected)==n and list(actual[0])==list(expected[0]) and len(actual[0])==13
  person_residuals = {}
  for i,(a,b,original) in enumerate(zip(actual,expected,source),1):
    person_residuals.setdefault(original['id'],[]).append(float(a['fe_residual']))
    close(float(a['lnhr'])-float(a['fe_xbu']),float(a['fe_residual']),'residual identity')
    close(float(a['fe_xb'])+float(a['fe_u']),float(a['fe_xbu']),'conditional fitted identity')
    for key in RAW:
      assert float(a[key]) == float(b[key]) == (i if key == 'source_row' else original[key]), (i,key)
    for key in a.keys()-set(RAW):
      close(float(a[key]),float(b[key]),key)
  assert len(person_residuals) == g
  for residuals in person_residuals.values():
    assert len(residuals) == 10 and abs(math.fsum(residuals)) <= 1e-9
  actual = records(native/'panel_within_residual_correlations.csv'); expected = records(reference/'correlations.csv')
  table = select('TableResult')[-1]
  assert table['headers'] == ['year_a','year_b','people','correlation'] and len(table['rows'])==100
  assert len(actual)==len(expected)==100 and list(actual[0]) == list(expected[0]) == table['headers']
  matrix = {}
  for index,(a,b,row) in enumerate(zip(actual,expected,table['rows'])):
    x,y = 1979+index//10,1979+index%10
    assert float(a['year_a']) == float(b['year_a']) == row[0] == x
    assert float(a['year_b']) == float(b['year_b']) == row[1] == y
    assert float(a['people']) == float(b['people']) == row[2] == g
    close(float(a['correlation']),float(b['correlation']),(x,y));close(row[3],float(a['correlation']),(x,y))
    matrix[(x,y)] = float(a['correlation'])
  for (x,y),value in matrix.items():
    close(value,matrix[(y,x)],'symmetry')
    if x==y: close(value,1.0,'diagonal')
  if author_log is not None:
    printed = author_values(author_log)
    for key,value in printed.items():
      assert abs(matrix[key]-value) <= 0.00005,(key,matrix[key],value)
  print(f'PASS {"synthetic" if synthetic else "original MOM"}:25 full-dummy QR fields,{n}x13 source/prediction cells,100 R/table matrix cells'+(' and45 author-log cells' if author_log else ''))


if __name__ == '__main__':
  parser=argparse.ArgumentParser(description=__doc__)
  for name in ['native','reference','raw']:
    parser.add_argument(name,type=Path)
  parser.add_argument('--author-log',type=Path)
  parser.add_argument('--synthetic',action='store_true')
  args=parser.parse_args();audit(args.native,args.reference,args.raw,args.author_log,args.synthetic)
