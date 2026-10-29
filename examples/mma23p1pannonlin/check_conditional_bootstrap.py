#!/usr/bin/env python3
"""Compare TabDat's exported whole-firm draws and report with independent references."""
import csv
import json
import math
from pathlib import Path
import sys

if len(sys.argv)!=5:
  raise SystemExit('usage: check_conditional_bootstrap.py oracle.csv reference-dir draws.csv json-output')
oracle,refdir,actual,json_output=map(Path,sys.argv[1:])
with (refdir/'reference.csv').open(newline='') as f:
  ref={row['field']:float(row['value']) for row in csv.DictReader(f)}
with (refdir/'replicates.csv').open(newline='') as f:
  betas={int(row['replicate']):float(row['beta']) for row in csv.DictReader(f)}
with (refdir/'groups.csv').open(newline='') as f:
  groups={int(row['index0']):int(row['firm_id']) for row in csv.DictReader(f)}
with (refdir/'design.csv').open(newline='') as f:
  design=list(csv.DictReader(f))
with (actual.parent/'patents.csv').open(newline='') as f:
  source=list(csv.DictReader(f))
if len(source)!=len(design): raise AssertionError('source row count mismatch')
totals={}
for row in source:
  firm=int(row['id']); totals[firm]=totals.get(firm,0)+float(row['pat'])
for row,expected in zip(source,design):
  for field in ('id','year','pat','logr'):
    if float(row[field])!=float(expected[field]): raise AssertionError(f'source {field} mismatch')
  firm=int(row['id'])
  if totals[firm]!=float(expected['firm_total']) or (firm in groups.values())!=(expected['included']=='TRUE'):
    raise AssertionError(f'firm {firm} total or inclusion mismatch')

def check_float(value,expected,name):
  value=float(value)
  if not math.isfinite(value) or abs(value-expected)>1e-10:
    raise AssertionError(f'{name}: {value!r} != {expected!r} (absolute tolerance 1e-10)')

checked=0
with oracle.open(newline='') as f,actual.open(newline='') as g:
  expected_rows=csv.DictReader(f); actual_rows=csv.DictReader(g)
  for expected in expected_rows:
    row=next(actual_rows,None)
    if row is None: raise AssertionError('missing draw row')
    rep=int(expected['replicate']); pos=int(expected['position'])
    for field in ('replicate','position','step'):
      if int(row[field])!=int(expected[field]): raise AssertionError(f'draw {rep}:{pos} {field} mismatch')
    if int(row['firm_id'])!=groups[int(expected['index0'])]: raise AssertionError(f'draw {rep}:{pos} firm mismatch')
    check_float(row['replicate_beta'],betas[rep],f'replicate {rep}')
    for field in ('estimate','replications','firms_per_draw','candidate_rows','candidate_firms','retained_rows'):
      check_float(row[field],ref[field],field)
    checked+=1
  if next(actual_rows,None) is not None: raise AssertionError('extra draw rows')
if checked!=int(ref['firms_per_draw']*ref['replications']): raise AssertionError('wrong draw count')
report=None
with json_output.open() as f:
  for line in f:
    event=json.loads(line)
    if event.get('result_type')=='TableResult' and 'bootstrap_se' in event['data']['headers']:
      data=event['data']; assert len(data['rows'])==1
      report=dict(zip(data['headers'],data['rows'][0]))
if report is None: raise AssertionError('bootstrap summary JSON missing')
for field,value in ref.items(): check_float(report[field],value,'report '+field)
assert report['base_seed']==10001 and report['method']=='whole_firm_pairs'
assert report['interval_method']=='type7_percentile_95'
assert report['rng']=='xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1'
print(f'PASS: {len(source)} original-source design rows, {checked} ordered firm draws and beta cells, 500 R refits, {len(ref)} report values')
