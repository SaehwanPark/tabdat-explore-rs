#!/usr/bin/env python3
"""Independent PSID preparation/window solve, direct QR and actual SVG audit."""
from decimal import Decimal
import hashlib
import math
from pathlib import Path
import re
import sys
import xml.etree.ElementTree as ET
from check_reference import RAW, close, rows
from check_source import PINS
from prepare import FIELDS

SERIES=['lowess_08','lowess_04','lowess_01']
SPANS=[.8,.4,.1]
BASE=RAW+['capped_earnings','hwage','lnhwage']


def integer(text):
  value=Decimal(text)
  assert value.is_finite() and value==value.to_integral_value(),text
  return int(value)


def prepare(path):
  raw=rows(path);assert 2<=len(raw)<=10000
  limits=[(1,10000),(0,9999),(0,999),(30,50),(0,99),(0,999999),(0,9999),(2,2),(0,99),(1,9)]
  for row in raw:
    assert list(row)==RAW
    for key,(low,high)in zip(RAW,limits):
      if key=='educatn'and row[key]=='':continue
      assert low<=integer(row[key])<=high,(key,row[key])
  assert len({integer(r['source_row'])for r in raw})==len(raw)
  selected=[]
  for r in raw:
    if integer(r['age'])!=36 or integer(r['earnings'])<=0 or r['educatn']=='' or integer(r['educatn'])in(0,98,99):continue
    hours=integer(r['hours']);assert hours>0
    capped=min(integer(r['earnings']),100000);wage=capped/hours
    selected.append(dict(r,capped_earnings=capped,hwage=wage,lnhwage=math.log(wage)))
  selected.sort(key=lambda r:integer(r['source_row']))
  assert len(selected)>=5,'three spans require N*0.1>=0.5'
  return raw,selected


def smooth(x,y,span):
  n=len(x);k=math.floor((n*span-.5)/2)
  assert 2<=n<=10000 and n*span>=.5 and n*min(n,2*k+1)<=20000000
  order=sorted(range(n),key=lambda i:(x[i],i));result=[None]*n;constant=0
  for i,index in enumerate(order):
    at=order[max(0,i-k):min(n,i+k+1)]
    radius=max(abs(x[j]-x[index])for j in at)
    if radius==0:
      constant+=1;result[index]=math.fsum(y[j]for j in at)/len(at);continue
    scale=max(abs(y[j])for j in at)or 1
    z=[(x[j]-x[index])/radius for j in at]
    w=[(1-(abs(t)/1.0001)**3)**3 for t in z]
    v=[y[j]/scale for j in at]
    # Solve the independent 2x2 normal equations, not the Rust centred-moment
    # calculation or R Householder QR. Target z=0 has weight1 in every window.
    a=math.fsum(w);b=math.fsum(t*u for t,u in zip(w,z))
    c=math.fsum(t*u*u for t,u in zip(w,z));d=math.fsum(t*u for t,u in zip(w,v))
    e=math.fsum(t*u*q for t,u,q in zip(w,z,v));det=a*c-b*b
    assert det>0 and math.isfinite(det)
    result[index]=(c*d-b*e)/det*scale
  assert all(math.isfinite(v)for v in result)
  return result,k,constant


def check_svg(path,table):
  root=ET.parse(path).getroot();attrs=root.attrib;n=len(table)
  assert attrs['width']=='900'and attrs['height']=='660'and attrs['viewBox']=='0 0 900 660'
  assert attrs['data-candidate-rows']==str(n)and float(attrs['data-jitter-y'])==0
  assert attrs['data-coordinate-precision']=='float64'
  for key,value in [('left',90),('top',75),('width',730),('height',400)]:assert float(attrs['data-plot-'+key])==value
  xs=[float(r['educatn'])for r in table]
  ys=[float(r[key])for r in table for key in ['lnhwage',*SERIES]]
  for axis,values in [('x',xs),('y',ys)]:
    low,high=min(values),max(values);padding=(high-low)*.05 if high>low else max(abs(low),1)*.05
    close(float(attrs[f'data-{axis}-min']),low-padding);close(float(attrs[f'data-{axis}-max']),high+padding)
  def pixel(x,y):
    return(90+(x-float(attrs['data-x-min']))/(float(attrs['data-x-max'])-float(attrs['data-x-min']))*730,
      75+(float(attrs['data-y-max'])-y)/(float(attrs['data-y-max'])-float(attrs['data-y-min']))*400)
  order=sorted(range(n),key=lambda i:(xs[i],i))
  lines=[e for e in root.iter()if e.attrib.get('class')=='line-series']
  assert [e.attrib['data-variable']for e in lines]==SERIES
  for node,name in zip(lines,SERIES):
    a=node.attrib;assert list(map(int,a['data-row-indices'].split(',')))==order
    xx=a['data-x-values'].split(',');yy=a['data-y-values'].split(',')
    points=re.findall(r'[ML]([\d.eE+\-]+)[ ,]([\d.eE+\-]+)',a['d'])
    assert len(xx)==len(yy)==len(points)==n
    for i,px,py,point in zip(order,xx,yy,points):
      y=float(table[i][name]);close(float(px),xs[i]);close(float(py),y)
      assert all(abs(float(a)-b)<.001 for a,b in zip(point,pixel(xs[i],y)))
  marks=[e for e in root.iter()if e.attrib.get('class')=='scatter-point'];assert len(marks)==n
  for i,node in enumerate(marks):
    a=node.attrib;y=float(table[i]['lnhwage'])
    assert integer(a['data-row-index'])==i
    assert a['data-x-variable']=='educatn'and a['data-y-variable']=='lnhwage'
    close(float(a['data-x']),xs[i]);close(float(a['data-y']),y);close(float(a['data-display-y']),y)
    assert all(abs(float(a[key])-b)<.001 for key,b in zip(['cx','cy'],pixel(xs[i],y)))
  return 3*n,n


def check_values(directory,input_csv,reference):
  directory,reference=Path(directory),Path(reference)
  _,sample=prepare(input_csv);n=len(sample)
  table=rows(directory/'psid_lowess.csv');assert len(table)==n and list(table[0])==BASE+SERIES
  r_sample=rows(reference/'sample.csv');r_fitted=rows(reference/'fitted.csv');metadata=rows(reference/'metadata.csv')
  assert len(r_sample)==len(r_fitted)==n and len(metadata)==3
  assert list(r_sample[0])==BASE and list(r_fitted[0])==['source_row','x','y',*SERIES]
  x=[integer(r['educatn'])for r in sample];y=[r['lnhwage']for r in sample]
  fits=[smooth(x,y,span)for span in SPANS]
  for j,(span,(_,k,constant))in enumerate(zip(SPANS,fits)):
    m=metadata[j];assert list(m)==['bandwidth','observations','half_window','local_constant_windows','max_scaled_score']
    assert float(m['bandwidth'])==span and integer(m['observations'])==n
    assert integer(m['half_window'])==k and integer(m['local_constant_windows'])==constant
    assert math.isfinite(float(m['max_scaled_score']))and 0<=float(m['max_scaled_score'])<1e-12
  checks=0
  for i,(native,expected,rs,rf)in enumerate(zip(table,sample,r_sample,r_fitted)):
    for key in BASE:
      if key in RAW or key=='capped_earnings':assert integer(str(native[key]))==integer(str(expected[key]))==integer(str(rs[key])),(i,key)
      else:close(float(native[key]),expected[key]);close(float(rs[key]),expected[key])
      checks+=1
    assert integer(rf['source_row'])==integer(expected['source_row'])
    assert float(rf['x'])==x[i];close(float(rf['y']),y[i])
    for name,(values,_,_)in zip(SERIES,fits):
      close(float(native[name]),values[i]);close(float(rf[name]),values[i]);checks+=1
  return checks,table


def audit(directory,input_csv,reference):
  checks,table=check_values(directory,input_csv,reference)
  vertices,marks=check_svg(Path(directory)/'psid_lowess.svg',table)
  return checks,vertices,marks


def source_check(program,log,data,input_csv,reference):
  for key,path in [('program',program),('log',log),('data',data)]:assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==PINS[key],(key,'pin mismatch')
  for path in [program,log]:
    spans=re.findall(r'lowess\s+lnhwage\s+educatn,\s+bwidth\((0\.[841])\)',Path(path).read_text())
    assert spans[:3]==['0.8','0.4','0.1']
  original=[]
  for i,line in enumerate(Path(data).read_text(encoding='ascii').splitlines(),1):
    assert len(line)==25
    original.append(dict(source_row=i,**{name:int(line[a:b])if line[a:b].strip()else None for name,a,b in FIELDS}))
  raw=rows(input_csv);r_source=rows(Path(reference)/'source.csv');assert len(raw)==len(original)==len(r_source)==4856
  for expected,native,r in zip(original,raw,r_source):
    assert list(native)==list(r)==RAW
    for key,value in expected.items():
      if value is None:assert native[key]==r[key]==''
      else:assert integer(native[key])==integer(r[key])==value
  assert len(prepare(input_csv)[1])==177
  return 48560


if __name__=='__main__':
  assert len(sys.argv)in(4,7),'usage: check_lowess.py analysis_directory psid.csv reference_directory [program log data]'
  checks,vertices,marks=audit(*sys.argv[1:4])
  source=''
  if len(sys.argv)==7:source=f', {source_check(*sys.argv[4:7],sys.argv[2],sys.argv[3])} exact original/R source fields and author spans'
  print(f'Validated {checks} distinct exported R cells, {vertices} independent QR/normal-equation fits and actual SVG vertices, {marks} raw observations{source}; historical tied-curve parity unclaimed')
