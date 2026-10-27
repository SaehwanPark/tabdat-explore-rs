#!/usr/bin/env python3
"""Development-only reproducibility probe; produces no inferential interval."""
import csv
import hashlib
import json
import math
import random
import subprocess
import sys
import tempfile
from pathlib import Path

if len(sys.argv) not in (3,4) or (len(sys.argv)==4 and sys.argv[3]!='--verify-r'):
  raise SystemExit('usage: nearest_subsample_probe.py nswpsid.da1 tabdat-binary [--verify-r]')
source,binary=map(Path,sys.argv[1:3])
verify_r=len(sys.argv)==4
binary=binary.resolve()
script=Path(__file__).with_name('nearest_matching.td').resolve()
raw=source.read_bytes()
assert hashlib.sha256(raw).hexdigest()=='810f6dbfbe0ffbcc52676946c330f49d4d8bf5cafc06980e912ca945ac7f4e44'
original=[line.split() for line in raw.decode('ascii').splitlines() if line.strip()]
assert len(original)==2675 and all(len(row)==11 for row in original)
fields=['source_row','treat','age','educ','black','hisp','marr','re74','re75','re78','u74_source','u75_source']
rng=random.Random(3982)
results=[]
with tempfile.TemporaryDirectory(prefix='tabdat-nearest-probe-') as root:
  for size in (700,1000,1400):
    successes=[];failures=[]
    for rep in range(20):
      selected=sorted(rng.sample(range(len(original)),size))
      directory=Path(root)/f'{size}-{rep}';directory.mkdir()
      with (directory/'nswpsid.csv').open('w',newline='') as stream:
        writer=csv.writer(stream,lineterminator='\n');writer.writerow(fields)
        writer.writerows([[i,*original[j]] for i,j in enumerate(selected,1)])
      process=subprocess.run([str(binary),'--json','-f',str(script)],cwd=directory,text=True,capture_output=True)
      if process.returncode:
        failure={'replication':rep,'returncode':process.returncode,
          'diagnostic':process.stderr.strip().replace(str(script),'nearest_matching.td')}
        marker='sql """\nSELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(isfinite(pscore)'
        parts=script.read_text().split(marker)
        assert len(parts)==2
        diagnostic=directory/'score_boundary.td'
        diagnostic.write_text(parts[0]+'''sql """WITH bounds AS (SELECT greatest(min(pscore) FILTER (WHERE treat=0),min(pscore) FILTER (WHERE treat=1)) AS lo,
least(max(pscore) FILTER (WHERE treat=0),max(pscore) FILTER (WHERE treat=1)) AS hi FROM active)
SELECT count(*) FILTER (WHERE pscore=0) AS zero_score,count(*) FILTER (WHERE pscore=1) AS one_score,
count(*) FILTER (WHERE NOT coalesce(isfinite(pscore),FALSE)) AS nonfinite_score,
count(*) FILTER (WHERE pscore IN (0,1) AND pscore>=lo AND pscore<=hi) AS saturated_in_range FROM active CROSS JOIN bounds"""\n''')
        probe=subprocess.run([str(binary),'--json','-f',str(diagnostic)],cwd=directory,text=True,capture_output=True)
        if probe.returncode:
          failure['boundary_probe_error']=probe.stderr.strip().replace(str(diagnostic),'score_boundary.td')
        else:
          leaves=[json.loads(line) for line in probe.stdout.splitlines()]
          tables=[v['data'] for v in leaves if v.get('result_type')=='TableResult' and 'zero_score' in v['data']['headers']]
          assert len(tables)==1 and len(tables[0]['rows'])==1
          failure['boundary_scores']=dict(zip(tables[0]['headers'],tables[0]['rows'][0]))
        failures.append(failure)
      else:
        leaves=[json.loads(line) for line in process.stdout.splitlines()]
        tables=[v['data'] for v in leaves if v.get('result_type')=='TableResult' and 'matched_treated' in v['data']['headers']]
        assert len(tables)==1 and len(tables[0]['rows'])==1
        row=dict(zip(tables[0]['headers'],tables[0]['rows'][0]));assert row['standard_error'] is None
        boundaries=[v['data'] for v in leaves if v.get('result_type')=='TableResult' and 'rounded_boundary_scores' in v['data']['headers']]
        assert len(boundaries)==1 and len(boundaries[0]['rows'])==1
        boundary=dict(zip(boundaries[0]['headers'],boundaries[0]['rows'][0]))
        assert boundary['rounded_boundary_scores']==boundary['rounded_boundary_excluded']
        success={'replication':rep,'contrast':row['contrast'],'matched_treated':row['matched_treated'],
          'rounded_boundary_excluded':boundary['rounded_boundary_excluded']}
        if verify_r and boundary['rounded_boundary_excluded']:
          reference=Path(__file__).with_name('nearest_matching_reference.R').resolve()
          oracle=subprocess.run(['Rscript',str(reference),'--prepared','nswpsid.csv','reference'],cwd=directory,text=True,capture_output=True)
          if oracle.returncode: raise RuntimeError(f'R reference failed at {size}/{rep}: {oracle.stderr}')
          with (directory/'nearest_pairs.csv').open(newline='') as stream: actual=list(csv.DictReader(stream))
          with (directory/'reference'/'pairs.csv').open(newline='') as stream: expected=list(csv.DictReader(stream))
          assert len(actual)==len(expected)
          for a,e in zip(actual,expected):
            for key in ('treated_id','control_id','tied_controls'): assert a[key]==e[key],(size,rep,key,a[key],e[key])
            for key in ('treated_score','control_score','distance','matched_outcome'):
              av,ev=float(a[key]),float(e[key])
              assert math.isfinite(av) and math.isfinite(ev) and abs(av-ev)<=1e-8+1e-9*abs(ev),(size,rep,key,av,ev)
          success['reference_pairs_checked']=len(actual)
          success['reference_warnings']=oracle.stderr.strip()
        successes.append(success)
    results.append({'size':size,'attempted':20,'successful':successes,'failed':failures})
print(json.dumps({'purpose':'feasibility only; not an uncertainty interval or coverage study','seed':3982,'verify_r':verify_r,'results':results},indent=2))
