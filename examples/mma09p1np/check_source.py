#!/usr/bin/env python3
"""Pin original author assets, source integers and historical float-generated summaries."""
from decimal import Decimal
import hashlib
import math
from pathlib import Path
import re
import statistics
import struct
import sys
from check_reference import RAW, audit, close, rows
from prepare import FIELDS

PINS = {
  'program':'fdd77b2a381aa4f62c41ea570388764b4f20a54ce0c2fd91af8a83bd8d525ae4',
  'log':'7f126f34b82db86480c3fe47e86e241c74271b650bd7e708757785ef559c01f8',
  'data':'2f24024edd599e74305924165e981790f96badb5f03048587a1b57cb74ce8346',
}


def printed(actual,text):
  expected=Decimal(text)
  assert math.isfinite(actual)
  assert abs(Decimal(str(actual))-expected)<=Decimal(5).scaleb(expected.as_tuple().exponent-1),(actual,text)


def f32(value):
  return struct.unpack('<f',struct.pack('<f',value))[0]


def source_check(program,log,data,directory,reference):
  for key,path in [('program',program),('log',log),('data',data)]:
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==PINS[key],(key,'source pin mismatch')
  directory,reference=Path(directory),Path(reference)
  original=[]
  for i,line in enumerate(Path(data).read_text(encoding='ascii').splitlines(),1):
    assert len(line)==25
    original.append(dict(source_row=i,**{name:int(line[a:b])if line[a:b].strip()else None
      for name,a,b in FIELDS}))
  actual=rows(directory/'psid.csv');r_source=rows(reference/'source.csv')
  assert len(original)==len(actual)==len(r_source)==4856
  for expected,a,b in zip(original,actual,r_source):
    assert list(a)==list(b)==RAW
    for key,value in expected.items():
      if value is None:assert a[key]==b[key]==''
      else:assert int(a[key])==int(b[key])==value
  selected=[r for r in original if r['earnings']>0 and r['age']==36 and r['educatn']not in(None,0,98,99)]
  assert len(selected)==177
  assert sum(r['earnings']==0 for r in original)==1204
  assert sum(r['educatn']is None for r in original)==1
  assert sum(r['educatn']in(0,98,99)for r in original)==290
  assert all(r['hours']>0 and r['sex']==2 for r in selected)
  assert all(r['married']not in(8,9)for r in selected)
  # Keep historical generated-float and native-double paths distinct.
  legacy=[]
  for r in selected:
    value=dict(r);value['earnings']=min(value['earnings'],100000)
    value['married']=int(value['married']==1)
    value['hwage']=f32(value['earnings']/value['hours'])
    value['lnhwage']=f32(math.log(value['hwage']));legacy.append(value)
  summaries=[]
  for sample,names in [(original,[name for name,_,_ in FIELDS]),
      (legacy,['intnum','persnum','age','educatn','earnings','hours','sex','married','hwage','lnhwage'])]:
    for name in names:
      v=[r[name]for r in sample if r[name]is not None]
      summaries.append((name,len(v),statistics.mean(v),statistics.stdev(v),min(v),max(v)))
  text=Path(log).read_text()
  matches=re.findall(r'^\s*(\w+)\s*\|\s+(\d+)\s+([.\-\d]+)\s+([.\-\d]+)\s+([.\-\d]+)\s+([.\-\d]+)\s*$',text,re.M)
  matches=[m for m in matches if m[0]in RAW+['hwage','lnhwage']]
  assert len(matches)==len(summaries)==19
  author_checks=0
  for actual,record in zip(summaries,matches):
    assert actual[0]==record[0]
    for value,printed_value in zip(actual[1:],record[1:]):
      printed(value,printed_value);author_checks+=1
  v=sorted(r['lnhwage']for r in legacy)
  scale=min(statistics.stdev(v),(v[132]-v[44])/1.349)
  bandwidth=1.3643*1.7188*scale/177**.2
  result=re.search(r'^sadj: ([.\d]+) iqr/1349: ([.\d]+) stdev: ([.\d]+)$',text,re.M)
  assert result
  for value,token in zip([scale,(v[132]-v[44])/1.349,statistics.stdev(v)],result.groups()):
    printed(value,token);author_checks+=1
  for label,value in [('Bandwidth:',bandwidth),('Usual Epanechnikov (epan2):',bandwidth),
      ('Gaussian:',1.3643*.7764*scale/177**.2),
      ('Quartic or biweight:',1.3643*2.0362*scale/177**.2),
      ('Uniform or rectangular:',.5*1.3643*1.3510*scale/177**.2)]:
    match=re.search(r'^'+re.escape(label)+r'\s+([.\d]+)\s*$',text,re.M);assert match,label
    printed(value,match[1]);author_checks+=1
  legacy_reference=rows(reference/'legacy_statistics.csv');assert len(legacy_reference)==1
  close(float(legacy_reference[0]['bandwidth']),bandwidth)
  close(float(legacy_reference[0]['sample_sd']),statistics.stdev(v))
  checks,vertices=audit(directory,directory/'psid.csv',reference)
  assert author_checks==103
  return checks,vertices,author_checks


if __name__=='__main__':
  assert len(sys.argv)==6,'usage: check_source.py mma09p1np.do mma09p1np.txt psidf3050.dat analysis_directory reference_directory'
  checks,vertices,facts=source_check(*sys.argv[1:])
  print(f'Validated48560 exact original/R CSV fields,177-row cohort,{checks} direct-R cells,1536 independent KDE ordinates,{vertices} SVG vertices/512 grid marks and{facts} author printed facts on separate float32 path')
