#!/usr/bin/env python3
"""Audit full-decade source/derivations, independent R summaries and boundaries."""
import csv
import hashlib
import json
import math
from pathlib import Path
import statistics
import sys

SOURCE=['id','year','cusip','ardssic','scisect','logk','sumpat','logr','pat']
FIELDS=SOURCE+['lpat','dpat','randd']+[f'logrl{j}' for j in range(1,6)]+[f'dyear{j}' for j in range(2,6)]
HASH='87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8'


def rows(path):
  with path.open() as file:return list(csv.DictReader(file))


def near(actual,expected):
  if expected in ('',None):assert actual in ('',None),(actual,expected);return
  actual,expected=float(actual),float(expected)
  assert math.isfinite(actual) and math.isfinite(expected)
  assert abs(actual-expected)<=1e-10+1e-12*abs(expected),(actual,expected)


def expected_rows(raw,original):
  if original:assert hashlib.sha256(raw.read_bytes()).hexdigest()==HASH
  tokens=raw.read_text().split();assert tokens and len(tokens)%25==0
  expected=[]
  for i in range(len(tokens)//25):
    record=tokens[25*i:25*(i+1)];v=[None if j==1 and value=='.' else float(value) for j,value in enumerate(record)]
    assert all(x is None or math.isfinite(x) for x in v)
    for t in range(10):
      y=v[15+t];x=v[5+t]
      assert 0<=y<=1e9 and y==math.floor(y) and abs(x)<=40
      values=[i+1,1970+t]+v[:5]+[x,y,math.log(.5 if y==0 else y),int(y>0),math.exp(x)]
      values += [v[5+t-j] if t>=j else None for j in range(1,6)]
      values += [int(1970+t==1974+j) for j in range(2,6)]
      expected.append(dict(zip(FIELDS,values)))
  assert 2<=len(expected)//10<=346
  return expected


def audit(native,raw,reference,original=True):
  expected=expected_rows(raw,original);g=len(expected)//10
  source=rows(native/'patents_decade.csv');assert len(source)==len(expected)
  by_key={(int(row['id']),int(row['year'])):row for row in expected};seen=set()
  for row in source:
    assert list(row)==SOURCE;key=(int(row['id']),int(row['year']));assert key not in seen;seen.add(key)
    for field in SOURCE:
      actual=None if row[field]=='' else float(row[field]);assert actual==by_key[key][field],(key,field)
  assert seen==by_key.keys()
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','TableResult','SqlCreateResult']+['LabelResult']*11+['SummarizeResult','ExportResult','KeepResult','SummarizeResult','ExportResult']
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert dict(events[15]['data']['metadata']['variable_labels'])==dict(zip(['id','pat','logr','lpat','dpat','randd']+[f'logrl{j}' for j in range(1,6)],['id','Patents','Ln(R&D)','Ln(Patents)','Patent Indicator','R&D']+[f'Ln(R&D) lagged {word}' for word in ['once','twice','three times','four times','five times']]))
  facts=rows(reference/'summaries.csv');assert len(facts)==42
  for cohort,event_index,export,rfile,subset in [('full',16,'patents_decade_prepared.csv','full.csv',expected),('selected',19,'patents_decade_analysis.csv','selected.csv',[r for r in expected if r['year']>=1975])]:
    actual=rows(native/export);independent=rows(reference/rfile);assert len(actual)==len(independent)==len(subset)
    for row,r,source in zip(actual,independent,subset):
      assert list(row)==list(r)==FIELDS
      for field in FIELDS:
        if field in SOURCE:
          value=None if row[field]=='' else float(row[field]);assert value==source[field],field
        else:near(row[field],source[field])
        near(row[field],r[field])
    summary=events[event_index]['data']['rows'];assert [r['variable'] for r in summary]==FIELDS
    oracle=[r for r in facts if r['cohort']==cohort];assert [r['variable'] for r in oracle]==FIELDS
    for actual,r in zip(summary,oracle):
      values=[row[r['variable']] for row in subset if row[r['variable']] is not None]
      n=len(values)
      raw_facts={'count':n,'mean':statistics.fmean(values) if n else None,'std_dev':statistics.stdev(values) if n>1 else None,'minimum':min(values) if n else None,'maximum':max(values) if n else None}
      for field in raw_facts:
        near(actual[field],raw_facts[field]);near(actual[field],r[field])
    exported=events[event_index+1]['data'];assert exported['path']==export and exported['dataset']['row_count']==len(subset)
  return len(expected),len(expected)//2


if __name__=='__main__':
  assert len(sys.argv) in (4,5)
  if len(sys.argv)==5:assert sys.argv[4]=='--control'
  print('PASS full/selected rows',audit(*map(Path,sys.argv[1:4]),original=len(sys.argv)==4),'all21 fields/42 variable-wise R summaries/pre-restriction lags')
