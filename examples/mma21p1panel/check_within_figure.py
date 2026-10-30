#!/usr/bin/env python3
"""Audit source-scale within coordinates, FE/OLS, LOWESS and SVG."""
import csv
import hashlib
import json
import math
from pathlib import Path
import re
import sys
import xml.etree.ElementTree as ET

sys.path.insert(0,str(Path(__file__).resolve().parent.parent/'mma09p1np'))
from check_lowess import smooth

def rows(path):
  with Path(path).open(newline='') as f:return list(csv.DictReader(f))
def close(a,b,label='value'):
  a,b=float(a),float(b);tol=1e-11+1e-10*abs(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=tol,(label,a,b,tol)
def audit(work,reference,result,original=None):
  work,reference=Path(work),Path(reference)
  if original:assert hashlib.sha256(Path(original).read_bytes()).hexdigest()=='2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
  raw,source=rows(work/'MOM.csv'),rows(reference/'source.csv');data,ref=rows(work/'within_figure.csv'),rows(reference/'rows.csv');meta,=rows(reference/'meta.csv')
  assert len(raw)==len(source)==int(meta['source_rows']) and len(data)==len(ref)==int(meta['observations'])
  key=lambda r:(int(float(r['id'])),int(float(r['year'])))
  raw.sort(key=key);source.sort(key=key);assert len({key(r) for r in raw})==len(raw)
  model=[];groups={}
  for a,b in zip(raw,source):
    assert list(a)==list(b)
    for name in a:
      if b[name]=='':assert a[name]==''
      else:assert float(a[name])==float(b[name]),(key(a),name)
    try:x,y=float(a['lnwg']),float(a['lnhr'])
    except ValueError:continue
    if math.isfinite(x) and math.isfinite(y):
      assert abs(x)<=1e6 and abs(y)<=1e6 and round(x,2)==x and round(y,2)==y
      model.append(a);groups.setdefault(key(a)[0],[]).append(a)
  assert len(model)==len(data)==len(ref)
  def avg(group,name):return(sum(round(float(r[name])*100) for r in group)/100)/len(group)
  gx,gy=avg(model,'lnwg'),avg(model,'lnhr');x=[];y=[]
  for row,expected,src in zip(data,ref,model):
    assert list(row)==list(expected)==list(src)+['unit_lnhr','unit_lnwg','overall_lnhr','overall_lnwg','within_lnhr','within_lnwg','figure_linear','figure_lowess']
    for name in src:
      if src[name]=='':assert row[name]==expected[name]==''
      else:assert float(row[name])==float(expected[name])==float(src[name]),(key(src),name)
    unit=groups[key(src)[0]];ux,uy=avg(unit,'lnwg'),avg(unit,'lnhr')
    for name,value in [('unit_lnhr',uy),('unit_lnwg',ux),('overall_lnhr',gy),('overall_lnwg',gx),('within_lnhr',(float(src['lnhr'])-uy)+gy),('within_lnwg',(float(src['lnwg'])-ux)+gx)]:
      assert float(row[name])==float(expected[name])==value,(key(src),name,row[name],expected[name],value)
    x.append(float(row['within_lnwg']));y.append(float(row['within_lnhr']))
  n=len(data);f,k,constant=smooth(x,y,.8)
  assert k==int(meta['half_window']) and constant==int(meta['constant_windows']) and 0<=float(meta['max_scaled_score'])<1e-12
  assert sorted(range(n),key=lambda i:(x[i],i))==sorted(range(n),key=lambda i:(float(ref[i]['within_lnwg']),i))
  for i,(row,expected) in enumerate(zip(data,ref)):
    for name,value in [('figure_lowess',f[i]),('figure_linear',float(meta['intercept'])+float(meta['slope'])*x[i])]:
      close(row[name],value,(i,name,'equation'));close(expected[name],value,(i,name,'R'))
  events=list(map(json.loads,Path(result).read_text().splitlines()))
  def all_of(kind):return[e['data'] for e in events if e.get('result_type')==kind]
  fe,=all_of('FixedEffectsResult');reg,=all_of('RegressionResult');low,=all_of('LowessResult');feline,olsline=all_of('LincomResult')
  assert fe['covariance']=='cluster(id)' and fe['common_intercept_inference']=='normalization_point_only' and fe['observation_count']==n
  assert reg['covariance']=='cluster(id)' and reg['observation_count']==n and reg['sample']['dropped_observations']==0
  for actual,name in [(fe['coefficients'][0]['value'],'fe_slope'),(fe['covariance_matrix'][0][0],'fe_cov'),(feline['estimate'],'fe_slope'),(feline['standard_error'],'fe_se'),(feline['statistic'],'fe_t'),(reg['coefficients'][0]['value'],'intercept'),(reg['coefficients'][1]['value'],'slope'),(reg['covariance_matrix'][0][0],'cov00'),(reg['covariance_matrix'][0][1],'cov01'),(reg['covariance_matrix'][1][1],'cov11'),(olsline['estimate'],'slope'),(olsline['standard_error'],'slope_se'),(olsline['statistic'],'slope_t'),(olsline['df_residual'],'inference_df')]:close(actual,meta[name],name)
  for actual,name in [(feline['p_value'],'fe_p'),(olsline['p_value'],'slope_p')]:assert abs(actual-float(meta[name]))<=1e-9*float(meta[name])
  assert low['algorithm']=='centered_rank_local_linear_tricube' and low['bandwidth']==.8 and low['half_window']==k and low['local_constant_windows']==constant
  assert low['estimation_state']=='unchanged' and low['sample']['retained_indices']==list(range(n))
  check_svg(work/'within_figure.svg',data)
  return len(raw),n

def check_svg(path,data):
  root=ET.parse(path).getroot();a=root.attrib;n=len(data)
  assert (a['width'],a['height'],a['viewBox'],a['role'])==('900','660','0 0 900 660','img')
  assert a['data-candidate-rows']==str(n) and float(a['data-jitter-y'])==0 and a['data-coordinate-precision']=='float64'
  for key,value in [('left',90),('top',75),('width',730),('height',400)]:assert float(a['data-plot-'+key])==value
  x=[float(r['within_lnwg']) for r in data];series=['figure_lowess','figure_linear'];y=[float(r[name]) for r in data for name in ['within_lnhr',*series]]
  for axis,values in [('x',x),('y',y)]:
    lo,hi=min(values),max(values);pad=(hi-lo)*.05 if hi>lo else max(abs(lo),1)*.05
    close(a['data-'+axis+'-min'],lo-pad);close(a['data-'+axis+'-max'],hi+pad)
  def pixel(xx,yy):return(90+(xx-float(a['data-x-min']))/(float(a['data-x-max'])-float(a['data-x-min']))*730,75+(float(a['data-y-max'])-yy)/(float(a['data-y-max'])-float(a['data-y-min']))*400)
  text=[e.text for e in root.iter() if e.text]
  for name in ['Within (Fixed Effects) Regression','Within log wage (overall mean restored)','Within log hours (overall mean restored)','Nonparametric fit (span 0.8)','Within linear fit']:assert name in text
  order=sorted(range(n),key=lambda i:(x[i],i));lines=[e for e in root.iter() if e.attrib.get('class')=='line-series'];assert [e.attrib['data-variable'] for e in lines]==series
  num=r'[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?'
  for node,name in zip(lines,series):
    b=node.attrib;assert list(map(int,b['data-row-indices'].split(',')))==order
    xx,yy=b['data-x-values'].split(','),b['data-y-values'].split(',');assert re.fullmatch(f'M{num} {num}(?:L{num} {num}){{{n-1}}}',b['d'])
    points=re.findall(f'[ML]({num}) ({num})',b['d']);assert len(xx)==len(yy)==len(points)==n
    for i,vx,vy,point in zip(order,xx,yy,points):
      value=float(data[i][name]);close(vx,x[i]);close(vy,value);assert all(abs(float(c)-v)<.001 for c,v in zip(point,pixel(x[i],value)))
  marks=[e for e in root.iter() if e.attrib.get('class')=='scatter-point'];assert len(marks)==n
  for i,node in enumerate(marks):
    b=node.attrib;value=float(data[i]['within_lnhr']);assert int(b['data-row-index'])==i and b['data-x-variable']=='within_lnwg' and b['data-y-variable']=='within_lnhr'
    close(b['data-x'],x[i]);close(b['data-y'],value);close(b['data-display-y'],value);assert all(abs(float(b[key])-v)<.001 for key,v in zip(('cx','cy'),pixel(x[i],value)))

if __name__=='__main__':
  if len(sys.argv) not in (4,5):raise SystemExit('usage: check_within_figure.py work-dir reference-dir result.jsonl [MOM.dat]')
  raw,n=audit(*sys.argv[1:]);print(f'PASS: {raw} exact source rows, {n} within/QR/equation rows, {n} SVG markers and {2*n} connected vertices; historical tied-curve parity unclaimed')
