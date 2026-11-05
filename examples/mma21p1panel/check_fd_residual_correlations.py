#!/usr/bin/env python3
"""Audit annual FD fit/correlations against raw MOM, R and author log."""
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


def close(actual,expected,label):
  tolerance=abs(expected)*1e-8 if str(label).endswith('p_value') else 1e-9
  assert math.isfinite(actual) and math.isfinite(expected) and abs(actual-expected)<=tolerance,(label,actual,expected)


def pointer(data,path):
  for key in path.split('/')[1:]:data=data[int(key)] if isinstance(data,list) else data[key]
  return data


def author_values(path):
  assert hashlib.sha256(path.read_bytes()).hexdigest()=='925aff11176cb78bbacf71abc69c2bef4876bdcc0e5544c001c2df4f5f8f56b5'
  text=path.read_text();found={}
  for variable,author,end in [('dlnhr','dlnhr','. corr dlnwg1980'),('dlnwg','dlnwg','. corr ufdiff1980'),('fd_residual','ufdiff','. ************ (3) ANALYSIS')]:
    block=text[text.index('. corr '+author+'1980'):text.index(end)];seen=set()
    for line in block.splitlines():
      match=re.match(r'\s*'+author+r'(198\d)\s*\|\s*(.*)',line)
      if not match:continue
      year=int(match.group(1));assert year not in seen;seen.add(year)
      for i,value in enumerate(match.group(2).split()):
        other=1980+i
        if other<year:found[(variable,other,year)]=float(value)
    assert seen==set(range(1980,1989))
  assert len(found)==108
  block=text[text.index('. regress dlnhr dlnwg'):text.index('. predict ufdiff, residuals')]
  fit={}
  for name,author in [('intercept','_cons'),('dlnwg','dlnwg')]:
    match=re.search(r'^\s*'+author+r'\s*\|\s*(.*)$',block,re.MULTILINE);assert match
    values=match.group(1).split();assert len(values)==6
    for key,value in zip(['value','standard_error','statistic','p_value','ci_lower','ci_upper'],values):fit[(name,key)]=float(value)
  assert 'Number of obs =    4788' in block and 'F(  1,  4786)' in block
  return found,fit


def audit(native,reference,raw,author_log=None,synthetic=False):
  if not synthetic:
    assert hashlib.sha256(raw.read_bytes()).hexdigest()=='2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
    assert author_log is not None
  values=[float(v) for v in raw.read_text().split()];assert len(values)%8==0 and all(math.isfinite(v) for v in values)
  source=[dict(source_row=i//8+1,**dict(zip(SOURCE,values[i:i+8]))) for i in range(0,len(values),8)]
  source.sort(key=lambda x:(x['id'],x['year']));g=len({r['id'] for r in source});n=len(source)-g;assert 3<=g<=532 and n==9*g
  annual=[(a,b) for a,b in zip(source[1:],source[:-1]) if a['id']==b['id']];assert len(annual)==n
  events=[json.loads(l) for l in (native/'fd_residual_correlations.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  select=lambda kind:[e['data'] for e in events if e['result_type']==kind]
  fit,=select('RegressionResult');lincom,=select('LincomResult')
  assert fit['outcome']=='dlnhr' and fit['predictors']==['dlnwg'] and fit['covariance']=='nonrobust' and fit['include_intercept']
  assert fit['sample']==dict(total_observations=n,dropped_observations=0)
  refs=records(reference/'reference.csv');assert len(refs)==len({(r['result_type'],r['path']) for r in refs})==25
  for row in refs:close(pointer({'RegressionResult':fit,'LincomResult':lincom}[row['result_type']],row['path']),float(row['value']),row['path'])
  assert [e['result_type'] for e in events][-5:]==['ExportResult','SqlCreateResult','TableResult','LincomResult','ExportResult']
  assert 'descriptive, not a test or covariance correction' in json.dumps(events)
  exports=select('ExportResult');assert len(exports)==2
  for event,name,count in zip(exports,['panel_fd_residual_rows.csv','panel_fd_residual_correlations.csv'],[n,243]):
    assert event['path']==event['dataset']['path']==name and event['dataset']['row_count']==count
  actual=records(native/'panel_fd_residual_rows.csv');expected=records(reference/'rows.csv')
  assert len(actual)==len(expected)==n and list(actual[0])==list(expected[0]) and len(actual[0])==17
  for (a,b),(original,previous) in zip(zip(actual,expected),annual):
    assert original['year']==previous['year']+1
    for key in RAW:assert float(a[key])==float(b[key])==original[key],key
    for key in ['source_row','year','lnhr','lnwg']:assert float(a['previous_'+key])==float(b['previous_'+key])==previous[key],key
    close(float(a['lnhr'])-float(a['previous_lnhr']),float(a['dlnhr']),'hours difference')
    close(float(a['lnwg'])-float(a['previous_lnwg']),float(a['dlnwg']),'wage difference')
    close(float(a['dlnhr'])-float(a['fd_fitted']),float(a['fd_residual']),'residual identity')
    for key in ['dlnhr','dlnwg','fd_fitted','fd_residual']:close(float(a[key]),float(b[key]),key)
  actual=records(native/'panel_fd_residual_correlations.csv');expected=records(reference/'correlations.csv')
  table=select('TableResult')[-1];assert table['headers']==['variable','year_a','year_b','people','correlation'] and len(table['rows'])==243
  assert len(actual)==len(expected)==243 and list(actual[0])==list(expected[0])==table['headers']
  matrix={};max_r=0
  for index,(a,b,row) in enumerate(zip(actual,expected,table['rows'])):
    variable=['dlnhr','dlnwg','fd_residual'][index//81];x,y=1980+(index%81)//9,1980+index%9
    assert a['variable']==b['variable']==row[0]==variable
    assert float(a['year_a'])==float(b['year_a'])==row[1]==x and float(a['year_b'])==float(b['year_b'])==row[2]==y
    assert float(a['people'])==float(b['people'])==row[3]==g
    value=float(a['correlation']);max_r=max(max_r,abs(value-float(b['correlation'])))
    close(value,float(b['correlation']),(variable,x,y));close(row[4],value,(variable,x,y));matrix[(variable,x,y)]=value
  for (variable,x,y),value in matrix.items():
    close(value,matrix[(variable,y,x)],'symmetry')
    if x==y:close(value,1,'diagonal')
  max_log=0
  if author_log is not None:
    printed,printed_fit=author_values(author_log)
    for key,value in printed.items():
      difference=abs(matrix[key]-value);max_log=max(max_log,difference);assert difference<=0.00005,(key,matrix[key],value)
    historical=records(reference/'historical_float32.csv');assert len(historical)==12
    for row in historical:
      key=(row['name'],row['field']);tolerance={'statistic':0.005,'p_value':0.0005}.get(row['field'],0.00000005)
      assert abs(float(row['value'])-printed_fit[key])<=tolerance,(key,row['value'],printed_fit[key])
  print(f'PASS {"synthetic" if synthetic else "original MOM"}: 25 R fields, {n}x17 annual/prediction cells, 243 R/table cells (max {max_r:.12g})'+(f', 108 author cells (max {max_log:.12g}), 12 float32 fit facts' if author_log else ''))


if __name__=='__main__':
  parser=argparse.ArgumentParser(description=__doc__)
  for name in ['native','reference','raw']:parser.add_argument(name,type=Path)
  parser.add_argument('--author-log',type=Path);parser.add_argument('--synthetic',action='store_true')
  args=parser.parse_args();audit(args.native,args.reference,args.raw,args.author_log,args.synthetic)
