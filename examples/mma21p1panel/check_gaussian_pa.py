#!/usr/bin/env python3
"""Source-bound Gaussian PA audit, including every unit influence and covariance."""
import math
from pathlib import Path
import sys
from check_random_effects_gls_refit_bootstrap import rows, close, integer, FIELDS
import hashlib


def audit(native, reference, source, synthetic=False):
  raw = source.read_bytes()
  if not synthetic:
    assert hashlib.sha256(raw).hexdigest() == '2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
  tokens = raw.split()
  assert tokens and len(tokens) % 8 == 0
  original = [dict(zip(FIELDS,map(float,tokens[i:i+8]))) for i in range(0,len(tokens),8)]
  ids = sorted({r['id'] for r in original})
  assert 3 <= len(ids) <= 532 and len(original) == 10*len(ids)
  assert len({(r['id'],r['year']) for r in original}) == len(original)
  assert all(math.isfinite(v) for r in original for v in r.values())
  f = rows(native/'panel_gaussian_pa_fit.csv'); rf = rows(reference/'fit.csv')
  assert len(f) == len(rf) == 1
  f, rf = f[0], rf[0]
  assert set(f) == set(rf) | {'gee_join_key','iterations','convention','unit_covariance','inference'}
  for k,v in rf.items():
    close(f[k],v,('fit',k))
  assert integer(f['observations']) == len(original) and integer(f['people']) == len(ids)
  assert integer(f['years_per_person']) == 10 and integer(f['gee_join_key']) == 1
  assert 1 <= integer(f['iterations']) < 100
  assert f['convention'] == 'legacy_pearson_exchangeable_rss_over_n'
  assert f['unit_covariance'] == 'uncorrected_unit_score_sandwich'
  assert f['inference'] == 'normal_95_no_pvalues'
  rho = float(f['working_correlation']);phi = float(f['scale'])
  assert -1/9 < rho < 1 and phi > 0
  close(f['within_eigenvariance'],phi*(1-rho),'within eigenvalue')
  close(f['between_eigenvariance'],phi*(1+9*rho),'between eigenvalue')
  for key in ['intercept_step','slope_step']:
    assert abs(float(f[key])) <= 1e-10*(1+abs(float(f[key.split('_')[0]])))
  coefficients = rows(native/'panel_gaussian_pa_coefficients.csv')
  reference_coefficients = rows(reference/'coefficients.csv')
  def indexed(data):
    assert len(data)==4
    result = {(r['covariance'],r['parameter']):r for r in data}
    assert len(result)==4
    assert set(result)=={(v,p) for v in ['working_model','uncorrected_unit_sandwich'] for p in ['_cons','lnwg']}
    return result
  c, rc = indexed(coefficients), indexed(reference_coefficients)
  for key,r in c.items():
    assert set(r)==set(rc[key])
    for field in set(r)-{'covariance','parameter'}:
      close(r[field],rc[key][field],(key,field))
    assert float(r['standard_error'])>0
  p, rp = rows(native/'panel_gaussian_pa_predictions.csv'), rows(reference/'predictions.csv')
  assert len(p)==len(rp)==len(original)
  expected_fields = set(FIELDS) | {'source_row','pols_mean','pols_residual','gee_mean','gee_residual'}
  for i,(a,b,rawrow) in enumerate(zip(p,rp,original),1):
    assert set(a)==set(b)==expected_fields
    assert integer(a['source_row'])==integer(b['source_row'])==i
    for key in FIELDS:
      assert float(a[key])==float(b[key])==rawrow[key],(i,key,'raw source mismatch')
    for key in ['pols_mean','pols_residual','gee_mean','gee_residual']:
      close(a[key],b[key],(i,key))
    close(a['gee_mean'],float(f['intercept'])+float(f['slope'])*rawrow['lnwg'],(i,'mean identity'))
  units, runits = rows(native/'panel_gaussian_pa_unit_scores.csv'), rows(reference/'unit_scores.csv')
  assert len(units)==len(runits)==len(ids)
  assert [integer(r['id']) for r in units]==list(map(int,ids))
  for a,b in zip(units,runits):
    assert set(a)==set(b)
    assert integer(a['observations'])==10
    for key in a:
      close(a[key],b[key],('unit',a['id'],key))
  for left,right,field in [('influence_intercept','influence_intercept','unit_covariance_aa'),
    ('influence_intercept','influence_slope','unit_covariance_ab'),('influence_slope','influence_slope','unit_covariance_bb')]:
    close(sum(float(r[left])*float(r[right]) for r in units),f[field],('unit sandwich sum',field))
  printed = 0
  if not synthetic:
    working_a=c[('working_model','_cons')];working_b=c[('working_model','lnwg')]
    facts=[(f['intercept'],'7.345479'),(f['slope'],'.1195474'),(f['scale'],'.0805511'),
      (working_a['standard_error'],'.0364481'),(working_b['standard_error'],'.0136507'),
      (working_a['z'],'201.53'),(working_b['z'],'8.76'),
      (working_a['normal95_lower'],'7.274042'),(working_a['normal95_upper'],'7.416916'),
      (working_b['normal95_lower'],'.0927925'),(working_b['normal95_upper'],'.1463023'),
      (float(working_b['z'])**2,'76.70')]
    for actual,text in facts:
      close(actual,text,('author printed',text),.5*10**(-len(text.split('.')[1])))
    printed=len(facts)
  print(f'PASS: {len(rf)} R fit fields, four full inference rows, {len(units)} complete unit scores/influences, {len(p)} prediction rows/{8*len(p)} exact raw fields, {printed} author facts')


if __name__=='__main__':
  if len(sys.argv) not in (4,5) or len(sys.argv)==5 and sys.argv[4]!='--synthetic':
    raise SystemExit('usage: check_gaussian_pa.py native_directory reference_directory MOM.dat [--synthetic]')
  audit(*map(Path,sys.argv[1:4]),synthetic=len(sys.argv)==5)
