#!/usr/bin/env python3
"""Development-only native NLS cross-check of representative bootstrap refits."""
import csv
import json
from pathlib import Path
import subprocess
import sys
import tempfile

if len(sys.argv)!=5:
  raise SystemExit('usage: check_nls_bootstrap_native_refits.py patents.csv oracle.csv reference-dir tabdat-binary')
source,oracle,refdir,binary=map(lambda p:Path(p).resolve(),sys.argv[1:])
with source.open(newline='') as f: rows=list(csv.DictReader(f))
if not rows: raise AssertionError('empty source')
by_firm={}
for row in rows: by_firm.setdefault(int(row['id']),[]).append(row)
firm_ids=sorted(by_firm)
if any(len(group)!=5 for group in by_firm.values()): raise AssertionError('unbalanced source')
with (refdir/'fits.csv').open(newline='') as f: refs=list(csv.DictReader(f))
if len(refs)!=500: raise AssertionError('expected 500 reference refits')
choices=sorted({1,*[int(fn(refs,key=lambda row:float(row[field]))['replicate'])
  for fn in (min,max) for field in ('intercept','slope')]})
draws={i:[] for i in choices}
with oracle.open(newline='') as f:
  for row in csv.DictReader(f):
    rep=int(row['replicate'])
    if rep in draws: draws[rep].append(firm_ids[int(row['index0'])])
max_diff=0.
with tempfile.TemporaryDirectory(prefix='tabdat-nls-refits-') as name:
  directory=Path(name)
  for rep in choices:
    if len(draws[rep])!=len(firm_ids): raise AssertionError('draw count mismatch')
    data=directory/'replicate.csv'
    with data.open('w',newline='') as f:
      writer=csv.DictWriter(f,fieldnames=rows[0].keys());writer.writeheader()
      for position,firm in enumerate(draws[rep],1):
        for row in by_firm[firm]: writer.writerow(dict(row,id=position))
    script=directory/'refit.td'
    script.write_text('use replicate.csv\nnl pat = exp(a+b*logr), params(a b) start(2.5 0.5)\n')
    proc=subprocess.run([str(binary),'--json','-f',str(script)],cwd=directory,text=True,capture_output=True)
    if proc.returncode: raise RuntimeError(f'native replicate {rep} failed: {proc.stderr}')
    events=[json.loads(line) for line in proc.stdout.splitlines()]
    fit=next(e['data'] for e in events if e['result_type']=='NlsResult')
    if fit['observation_count']!=len(rows) or fit['sample']['dropped_observations']!=0:
      raise AssertionError('native refit sample mismatch')
    coefficients={c['name']:c['value'] for c in fit['coefficients']};ref=refs[rep-1]
    if int(ref['replicate'])!=rep: raise AssertionError('reference ordering mismatch')
    for name,field in [('a','intercept'),('b','slope')]:
      error=abs(coefficients[name]-float(ref[field]));max_diff=max(max_diff,error)
      if error>1e-9+1e-10*abs(float(ref[field])): raise AssertionError(f'native replicate {rep} {name} mismatch')
print(f'PASS: {len(choices)} native repeated-row NLS refits {choices}; max coefficient difference {max_diff:.3g}')
