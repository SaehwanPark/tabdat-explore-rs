#!/usr/bin/env python3
"""Audit RE conditional residual correlations against raw MOM, plm and author log."""
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
  text = path.read_text(); text = text[text.index('. corr ure1979'):text.index('. corr ufe1979')]
  found = {}; seen = set()
  for line in text.splitlines():
    match = re.match(r'\s*ure(19\d\d)\s*\|\s*(.*)',line)
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
  events = [json.loads(l) for l in (native/'random_effects_residual_correlations.jsonl').read_text().splitlines()]
  assert all(e['schema_version'] == 1 and 'error' not in e for e in events)
  select = lambda kind: [e['data'] for e in events if e['result_type'] == kind]
  fit, = select('RandomEffectsResult')
  assert fit['outcome'] == 'lnhr' and fit['predictors'] == ['lnwg'] and fit['estimator'] == 'balanced_swamy_arora_random_effects_gls'
  assert fit['entity_variable'] == 'id' and fit['time_variable'] == 'year'
  assert fit['variance_component_method'] == 'balanced_swamy_arora'
  assert fit['covariance'] == 'nonrobust' and fit['reference_distribution'] == 'normal'
  assert fit['covariance_convention'] == 'quasi_demeaned_rss_over_n_minus_k'
  assert fit['sample'] == dict(total_observations=n,dropped_observations=0)
  refs = records(reference/'reference.csv'); assert len(refs) == len({r['path'] for r in refs}) == 30
  for row in refs:
    close(pointer(fit,row['path']),float(row['value']),row['path'])
  kinds = [e['result_type'] for e in events]
  assert kinds[-6:] == ['TableResult','ActivateResult','PredictionResult','TableResult','ActivateResult','ExportResult']
  retained = [v for v in select('TableResult') if v['headers'] == ['retained_re_prediction_rows']]
  assert len(retained) == 1 and retained[0]['rows'] == [[n]]
  assert 'descriptive across people, not a test or covariance correction' in json.dumps(events)
  exports = select('ExportResult'); assert len(exports) == 2
  for event,name,count in zip(exports,['panel_re_residual_rows.csv','panel_re_residual_correlations.csv'],[n,100]):
    assert event['path'] == event['dataset']['path'] == name and event['dataset']['row_count'] == count
  actual = records(native/'panel_re_residual_rows.csv'); expected = records(reference/'rows.csv')
  assert len(actual)==len(expected)==n and list(actual[0])==list(expected[0]) and len(actual[0])==24
  people = {}
  for i,(a,b,original) in enumerate(zip(actual,expected,source),1):
    people.setdefault(original['id'],[]).append(a)
    close(float(a['lnhr'])-float(a['population_xb']),float(a['population_residual']),'population residual identity')
    close(float(a['population_xb'])+float(a['re_unit_effect']),float(a['conditional_xbu']),'conditional fit identity')
    close(float(a['lnhr'])-float(a['conditional_xbu']),float(a['re_e']),'idiosyncratic residual identity')
    close(float(a['shrink']),1-(1-float(a['theta']))**2,'shrink identity')
    close(float(a['re_unit_effect']),float(a['shrink'])*float(a['mean_population_residual']),'unit-effect identity')
    for key in ['sigma_e_squared','raw_sigma_u_squared','sigma_u_squared','theta']:
      close(float(a[key]),fit[key],key)
    for key in RAW:
      assert float(a[key]) == float(b[key]) == (i if key == 'source_row' else original[key]), (i,key)
    for key in a.keys()-set(RAW):
      close(float(a[key]),float(b[key]),key)
  assert len(people) == g
  for group in people.values():
    assert len(group) == 10
    mean = math.fsum(float(r['population_residual']) for r in group)/10
    for r in group:
      close(float(r['mean_population_residual']),mean,'person residual mean')
  actual = records(native/'panel_re_residual_correlations.csv'); expected = records(reference/'correlations.csv')
  tables = [v for v in select('TableResult') if v['headers'] == ['year_a','year_b','people','correlation']]
  assert len(tables) == 1
  table = tables[0]
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
  print(f'PASS {"synthetic" if synthetic else "original MOM"}:30 plm/ranef fields,{n}x24 source/prediction cells,100 R/table matrix cells'+(' and45 author-log cells' if author_log else ''))


if __name__ == '__main__':
  parser=argparse.ArgumentParser(description=__doc__)
  for name in ['native','reference','raw']:
    parser.add_argument(name,type=Path)
  parser.add_argument('--author-log',type=Path)
  parser.add_argument('--synthetic',action='store_true')
  args=parser.parse_args();audit(args.native,args.reference,args.raw,args.author_log,args.synthetic)
