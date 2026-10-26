#!/usr/bin/env python3
"""Audit source membership, direct-R KDE and independently reconstructed SVG geometry."""
import csv
import math
from pathlib import Path
import re
import statistics
import sys
import xml.etree.ElementTree as ET

RAW = ['source_row','intnum','persnum','age','educatn','earnings','hours','sex','kids','married']
SERIES = ['density_half','density_plugin','density_double']


def rows(path):
  with open(path,newline='') as stream:
    return list(csv.DictReader(stream))


def close(actual,expected):
  assert math.isfinite(actual) and math.isfinite(expected),(actual,expected)
  assert abs(actual-expected)<=1e-11+1e-10*abs(expected),(actual,expected)


def quantile(values,p):
  at=(len(values)-1)*p;left=math.floor(at);right=math.ceil(at)
  return values[left]+(values[right]-values[left])*(at-left)


def kde(values,point,width):
  return math.fsum(.75*(1-u*u) if abs(u)<1 else 0
    for u in ((point-v)/width for v in values))/(len(values)*width)


def audit(directory,input_csv,reference_directory):
  directory,reference=Path(directory),Path(reference_directory)
  raw=rows(input_csv);assert 2<=len(raw)<=10000 and list(raw[0])==RAW
  ordinals=[int(r['source_row'])for r in raw]
  assert len(set(ordinals))==len(raw) and all(0<i<=10000 for i in ordinals)
  expected=[]
  for r in raw:
    if int(r['age'])!=36 or int(r['earnings'])==0 or r['educatn'] in ('','0','98','99'):
      continue
    assert int(r['hours'])>0
    capped=min(int(r['earnings']),100000);wage=capped/int(r['hours'])
    assert math.isfinite(wage) and wage>0
    expected.append(dict(r,capped_earnings=str(capped),hwage=str(wage),lnhwage=str(math.log(wage))))
  expected.sort(key=lambda r:int(r['source_row']))
  selected=rows(directory/'psid_selected.csv');assert len(expected)==len(selected)>=2
  assert list(selected[0])==list(expected[0])
  for actual,wanted in zip(selected,expected):
    for key in wanted:
      if key in RAW or key=='capped_earnings':
        assert actual[key]==wanted[key],(key,actual[key],wanted[key])
      else:close(float(actual[key]),float(wanted[key]))
  values=sorted(float(r['lnhwage'])for r in expected);n=len(values)
  q25,q75=quantile(values,.25),quantile(values,.75)
  scale=min(statistics.stdev(values),(q75-q25)/1.349)
  width=1.3643*1.7188*scale/n**.2
  assert math.isfinite(width) and width>0
  summary=dict(n=n,sample_mean=statistics.mean(values),sample_sd=statistics.stdev(values),
    q25=q25,q75=q75,scale=scale,bandwidth=width,min_x=min(values),max_x=max(values))
  native_summary=rows(directory/'psid_bandwidth.csv');assert len(native_summary)==1
  assert list(native_summary[0])==list(summary)
  for key,value in summary.items():close(float(native_summary[0][key]),value)
  table=rows(directory/'psid_density.csv')
  assert len(table)==512 and list(table[0])==['grid_index','log_wage_grid',*SERIES]
  lower=min(values)-2*width;span=max(values)-min(values)+4*width
  for i,row in enumerate(table):
    assert int(row['grid_index'])==i
    point=lower+span*i/511
    close(float(row['log_wage_grid']),point)
    for name,multiplier in zip(SERIES,[.5,1,2]):
      actual=float(row[name]);assert actual>=0
      close(actual,kde(values,point,multiplier*width))
  # Every compact-kernel piece is quadratic; Simpson integrates it exactly.
  for multiplier in [.5,1,2]:
    h=multiplier*width;knots=sorted(set([v-h for v in values]+[v+h for v in values]))
    mass=math.fsum((b-a)*(kde(values,a,h)+4*kde(values,(a+b)/2,h)+kde(values,b,h))/6
      for a,b in zip(knots,knots[1:]))
    assert abs(mass-1)<1e-12,(multiplier,mass)
  checks=0
  for filename,reference_name in [('psid_selected.csv','sample.csv'),
      ('psid_bandwidth.csv','bandwidth.csv'),('psid_density.csv','density.csv')]:
    actual,wanted=rows(directory/filename),rows(reference/reference_name)
    assert len(actual)==len(wanted)>0 and list(actual[0])==list(wanted[0])
    for a,b in zip(actual,wanted):
      for key in a:
        if not a[key] or not b[key]:assert a[key]==b[key]
        else:close(float(a[key]),float(b[key]))
        checks+=1
  return checks,check_svg(directory/'psid_density.svg',table)


def check_svg(path,table):
  root=ET.parse(path).getroot();attrs=root.attrib
  assert attrs['data-candidate-rows']=='512' and float(attrs['data-jitter-y'])==0
  assert attrs['data-coordinate-precision']=='float64'
  xs=[float(r['log_wage_grid'])for r in table]
  ys=[float(r[name])for r in table for name in SERIES]
  for axis,values in [('x',xs),('y',ys)]:
    span=max(values)-min(values);assert span>0
    close(float(attrs[f'data-{axis}-min']),min(values)-.05*span)
    close(float(attrs[f'data-{axis}-max']),max(values)+.05*span)
  def pixel(x,y):
    a=attrs
    return (float(a['data-plot-left'])+(x-float(a['data-x-min']))/
      (float(a['data-x-max'])-float(a['data-x-min']))*float(a['data-plot-width']),
      float(a['data-plot-top'])+(float(a['data-y-max'])-y)/
      (float(a['data-y-max'])-float(a['data-y-min']))*float(a['data-plot-height']))
  lines=[node for node in root.iter()if node.attrib.get('class')=='line-series']
  assert [node.attrib['data-variable']for node in lines]==SERIES
  for node,name in zip(lines,SERIES):
    assert list(map(int,node.attrib['data-row-indices'].split(',')))==list(range(512))
    xx=node.attrib['data-x-values'].split(',');yy=node.attrib['data-y-values'].split(',')
    points=re.findall(r'[ML]([\d.eE+\-]+)[ ,]([\d.eE+\-]+)',node.attrib['d'])
    assert len(xx)==len(yy)==len(points)==512
    for i,(x,y,point)in enumerate(zip(xx,yy,points)):
      close(float(x),xs[i]);close(float(y),float(table[i][name]))
      assert all(abs(float(a)-b)<.001 for a,b in zip(point,pixel(xs[i],float(table[i][name]))))
  marks=[node for node in root.iter()if node.attrib.get('class')=='scatter-point']
  assert len(marks)==512
  for i,node in enumerate(marks):
    a=node.attrib;y=float(table[i]['density_plugin'])
    assert int(a['data-row-index'])==i
    assert a['data-x-variable']=='log_wage_grid' and a['data-y-variable']=='density_plugin'
    close(float(a['data-x']),xs[i]);close(float(a['data-y']),y);close(float(a['data-display-y']),y)
    assert all(abs(float(a[k])-b)<.001 for k,b in zip(['cx','cy'],pixel(xs[i],y)))
  return 1536


if __name__=='__main__':
  assert len(sys.argv)==4,'usage: check_reference.py analysis_directory psid.csv reference_directory'
  checks,vertices=audit(*sys.argv[1:])
  print(f'Validated {checks} direct-R cells, exact cohort/source values,1536 independent KDE ordinates, three unit masses, {vertices} SVG vertices and512 evaluation marks')
