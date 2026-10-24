"""Development-only temporary-copy corruptions of the truncated-source audit.
Usage: python3 check_truncreg_failures.py VERIFIED_WORK_DIRECTORY
Never changes verified inputs; shared native/R corruptions must fail independent gates.
"""
import csv
import hashlib
import json
import math
import pathlib
import shutil
import subprocess
import sys
import tempfile
from check_truncreg import likelihood

FILES = ['mma16p1tobit.asc','mma16p1tobit.csv','truncreg.jsonl','truncreg_predictions.csv','R_truncreg_quantities.csv','R_truncreg_predictions.csv']

def table(work,name,change):
  path = work/name
  with path.open() as stream:
    reader = csv.DictReader(stream);fields = reader.fieldnames;rows = list(reader)
  change(rows)
  with path.open('w',newline='') as stream:
    writer = csv.DictWriter(stream,fieldnames=fields);writer.writeheader();writer.writerows(rows)

def events(work,change):
  path = work/'truncreg.jsonl';records = [json.loads(line) for line in path.read_text().splitlines()];change(records);path.write_text(''.join(json.dumps(r)+'\n' for r in records))

def reference(work,q,i,j,value):
  def change(rows):
    found = [r for r in rows if r['quantity']==q and int(r['i'])==i and int(r['j'])==j];assert len(found)==1;found[0]['value']=repr(value)
  table(work,'R_truncreg_quantities.csv',change)

def shared_covariance(work):
  rows = [json.loads(line) for line in (work/'truncreg.jsonl').read_text().splitlines()];value = rows[1]['data']['covariance_matrix'][0][1]*1.2
  events(work,lambda r:r[1]['data']['covariance_matrix'][0].__setitem__(1,value));reference(work,'joint_covariance',1,2,value)

def shared_zero_tail(work):
  events(work,lambda r:r[1]['data']['coefficients'][0].__setitem__('p_value',0));reference(work,'coefficient_p_value',1,0,0)

def forged_stationarity(work):
  records = [json.loads(line) for line in (work/'truncreg.jsonl').read_text().splitlines()];fit = records[1]['data'];b = fit['coefficients'][0];b['value'] += 100
  b['statistic'] = b['value']/b['standard_error'];b['p_value'] = math.erfc(abs(b['statistic'])/math.sqrt(2));b['confidence_lower'] = b['value']-1.959963984540054*b['standard_error'];b['confidence_upper'] = b['value']+1.959963984540054*b['standard_error']
  for key in ['value','standard_error','statistic','p_value','confidence_lower','confidence_upper']:
    reference(work,'coefficient_'+key,1,0,b[key])
  source = [line.split() for line in (work/'mma16p1tobit.asc').read_text().splitlines() if line.strip()];selected = [r for r in source if r[3]!='.']
  beta = [r['value'] for r in fit['coefficients']];fit['log_likelihood'] = likelihood(beta,fit['scale']['value'],selected)[0];reference(work,'log_likelihood',0,0,fit['log_likelihood'])
  fit['max_abs_scaled_score']=0;fit['max_abs_scaled_newton_step']=0
  for q in ['scaled_score','scaled_step','null_scaled_score','null_scaled_step']:
    reference(work,q,0,0,0)
  (work/'truncreg.jsonl').write_text(''.join(json.dumps(r)+'\n' for r in records))

def shared_prediction(work,field):
  for name in ['truncreg_predictions.csv','R_truncreg_predictions.csv']:
    table(work,name,lambda rows:rows[0].__setitem__(field,str(float(rows[0][field])+(.1 if field=='retention_probability' else 100))))

def shared_ordinal(work):
  for name in ['truncreg_predictions.csv','R_truncreg_predictions.csv']:
    table(work,name,lambda rows:rows[0].__setitem__('source_row','2'))

def run(work):
  script = pathlib.Path(__file__).with_name('check_truncreg.py')
  return subprocess.run([sys.executable,str(script),str(work)],capture_output=True,text=True,timeout=20)

if __name__ == '__main__':
  verified = pathlib.Path(sys.argv[1]);before = {n:hashlib.sha256((verified/n).read_bytes()).hexdigest() for n in FILES};baseline = run(verified);assert baseline.returncode==0,baseline.stderr
  cases = [
    ('shared_covariance',shared_covariance,'independent full observed inverse'),
    ('shared_zero_tail',shared_zero_tail,'positive normal p-value'),
    ('forged_native_and_R_stationarity',forged_stationarity,'independent observed score certificate'),
    ('shared_mean_prediction',lambda w:shared_prediction(w,'conditional_mean'),'independent conditional_mean'),
    ('shared_population_probability',lambda w:shared_prediction(w,'retention_probability'),None),
    ('shared_ordinal',shared_ordinal,None),
    ('duplicate_reference',lambda w:table(w,'R_truncreg_quantities.csv',lambda rows:rows.append(rows[0].copy())),None),
    ('missing_reference',lambda w:table(w,'R_truncreg_quantities.csv',lambda rows:rows.pop()),'exact reference inventory'),
    ('changed_prepared_source',lambda w:table(w,'mma16p1tobit.csv',lambda rows:rows[0].__setitem__('lnwage','99')),None),
    ('changed_exported_source',lambda w:table(w,'truncreg_predictions.csv',lambda rows:rows[0].__setitem__('ystar','99')),None),
    ('changed_raw_source',lambda w:(w/'mma16p1tobit.asc').write_bytes((w/'mma16p1tobit.asc').read_bytes()+b' '),'original source SHA256'),
  ]
  with tempfile.TemporaryDirectory(prefix='tabdat-truncreg-corruption-') as root:
    for name,change,gate in cases:
      work = pathlib.Path(root)/name;shutil.copytree(verified,work);change(work);result = run(work)
      assert result.returncode!=0 and 'Validated45' not in result.stdout,(name,result.stdout,result.stderr)
      if gate:
        assert gate in result.stderr,(name,gate,result.stderr)
      print(name,'rejected')
  assert before=={n:hashlib.sha256((verified/n).read_bytes()).hexdigest() for n in FILES}
  print('All11 temporary-copy corruptions rejected;6 verified input fingerprints unchanged.')
