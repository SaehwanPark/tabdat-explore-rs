#!/usr/bin/env python3
"""Original Table17.8 Gompertz column: asset identity, full design, printed facts."""
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import re
import sys
from check_gompertz import audit, rows

assert len(sys.argv)==7,'usage: check_gompertz_source.py author.do author.txt ema1996.dta ema1996.asc output_directory reference_directory'
program,log,dta,ascii_data,output,reference=map(Path,sys.argv[1:])
for path,digest in [(program,'d8a395acb36d44d59308b3af510766a42b1f912ad0d52147abb1ef1b9a421e46'),
    (log,'740148da116e657c71d8eb378eecfaf6b557f2e1b12376f49a7d3e8222934141'),
    (dta,'98ba2e5e033b4d2423a5fe1c8c0218e3aaf1898c63c4ab5a89910edda5427a29'),
    (ascii_data,'933a7c2a5e39ee5b1c1a810a2e531577f430db850ce7a3e2f4b57e913082fa48')]:
  assert hashlib.sha256(path.read_bytes()).hexdigest()==digest,f'Unexpected original bytes: {path}'
gates,cells=audit(output/'duration.csv',output/'native.json',output/'gompertz_predictions.csv',reference)
source=rows(output/'duration.csv')
assert len(source)==3343 and sum(float(r['censor1'])for r in source)==1073
assert sum(float(r['spell'])for r in source)==20887
assert sum(all(float(r[f'censor{i}'])==0 for i in range(1,5))for r in source)==102
assert [float(r['source_row'])for r in source]==list(range(1,3344))
clean=re.sub(r'/\*.*?\*/',' ',program.read_text(),flags=re.S)
exported=re.search(r'outfile\s+(.*?)\s+using ema1996.asc',clean,re.S)[1].split()
assert len(exported)==42 and set(source[0])-set(exported)=={'source_row','stateur'}
tokens=ascii_data.read_text().split();assert len(tokens)==3343*42
for i,row in enumerate(source):
  for j,name in enumerate(exported):
    text=tokens[i*42+j];unit=10.0**Decimal(text).as_tuple().exponent
    assert abs(float(row[name])-float(text))<=.500000001*unit,(i,name,row[name],text)
mapping=dict(RR='reprate',DR='disrate',UI='ui',RRUI='rr_ui',DRUI='dr_ui',LOGWAGE='logwage',_cons='intercept')
xlist=re.search(r'global xlist\s+(.*?)\n\s*\* Exponential regression',clean,re.S)[1].split()
expected=[mapping.get(name,name)for name in xlist]
fit=next(json.loads(line)['data']for line in (output/'native.json').read_text().splitlines()if json.loads(line)['result_type']=='GompertzResult')
assert fit['predictors']==expected and len(expected)==40
byname={c['name']:c for c in fit['coefficients']}
text=log.read_text();first=text.split('Gompertz regression -- log relative-hazard form',1)[1].split('. estimates store bgompertz',1)[0]
fields=['value','standard_error','statistic','p_value','confidence_lower','confidence_upper']
def rounded(actual,printed):
  unit=10.0**Decimal(printed).as_tuple().exponent
  assert abs(actual-float(printed))<=.500000001*unit,(actual,printed)
facts=0;seen=set();shape=fit['shape'];gamma_seen=False
for line in first.splitlines():
  name,sep,values=line.partition('|');name=name.strip();values=values.split();native=mapping.get(name,name)
  if sep and native in byname and len(values)==6:
    assert native not in seen;seen.add(native)
    for field,value in zip(fields,values):rounded(byname[native][field],value);facts+=1
  elif sep and name=='gamma':
    assert len(values)==6 and not gamma_seen;gamma_seen=True
    for field,printed in zip(fields,values):rounded(shape[field],printed);facts+=1
assert len(seen)==41 and gamma_seen and facts==252
for actual,pattern in [(fit['normalized_log_likelihood'],r'Log pseudo-likelihood\s*=\s*([-\d.]+)'),(fit['omnibus_wald']['statistic'],r'Wald chi2\(40\)\s*=\s*([-\d.]+)')]:rounded(actual,re.search(pattern,first)[1]);facts+=1
compact=text.split('. estimates table bexponential',1)[1].split('. estimates table bcox',1)[0]
last=None;selected=0
for line in compact.splitlines():
  name,sep,values=line.partition('|');name=name.strip();values=values.split()
  if not sep or len(values)!=3:continue
  if name in mapping and mapping[name]in byname:
    last=mapping[name];rounded(byname[last]['value'],values[2]);selected+=1
  elif not name and last:
    rounded(byname[last]['statistic'],values[2]);selected+=1;last=None
assert selected==14;facts+=selected
print(f'Pinned original Gompertz audit: {gates} R gates, {cells} native cells, {3343*44} exact source cells, {facts} printed quantities including14 Table17.8 cells; all140406 rounded ASCII fields/stateur omission verified')
