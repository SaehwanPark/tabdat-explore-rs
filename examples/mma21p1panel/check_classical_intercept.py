#!/usr/bin/env python3
"""Source-bound QR audit of native classical FE intercept and joint inference."""
import hashlib
import json
from pathlib import Path
import sys
from check_random_effects_gls_refit_bootstrap import rows, close, integer, FIELDS


def audit(native,reference,source,synthetic=False):
  raw=source.read_bytes()
  if not synthetic:
    assert hashlib.sha256(raw).hexdigest()=='2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
  tokens=raw.split();assert tokens and len(tokens)%8==0
  original=[dict(zip(FIELDS,map(float,tokens[i:i+8]))) for i in range(0,len(tokens),8)]
  events=[json.loads(line) for line in (native/'output.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  fit=[e['data'] for e in events if e['result_type']=='FixedEffectsResult'];assert len(fit)==1
  lincom=[e['data'] for e in events if e['result_type']=='LincomResult'];assert len(lincom)==2
  tests=[e['data'] for e in events if e['result_type']=='TestResult'];assert len(tests)==1
  fit=fit[0];data=dict(fit=fit,common=lincom[0],mean=lincom[1],joint=tests[0])
  assert fit['common_intercept_inference']=='classical_student_t'
  assert fit['unit_effect_normalization']=='estimation_observation_weighted_zero_mean'
  assert fit['covariance']=='nonrobust' and fit['covariance_parameter_names']==['lnwg']
  assert len(fit['coefficients'])==len(fit['covariance_matrix'])==1
  assert fit['normalized_level']['parameter_names']==['_cons','lnwg']
  assert len(fit['normalized_level']['parameters'])==2
  matrix=fit['normalized_level']['covariance_matrix']
  assert len(matrix)==2 and all(len(row)==2 for row in matrix)
  assert fit['normalized_level']['reference_distribution']=='student_t'
  assert fit['common_intercept_statistics']['name']=='_cons'
  assert tests[0]['is_chi2'] is False
  reference_fields=rows(reference/'reference.csv');assert len(reference_fields)==39
  for r in reference_fields:
    value=data[r['kind']]
    for key in r['path'].split('/')[1:]:value=value[int(key)] if isinstance(value,list) else value[key]
    expected=float(r['value']);tolerance=1e-9
    if r['path'].endswith('p_value'):tolerance=abs(expected)*1e-8
    if r['kind']=='joint' and r['path']=='/statistic':tolerance+=1e-10*abs(expected)
    close(value,expected,(r['kind'],r['path']),tolerance)
  assert fit['sample']=={'total_observations':len(original),'dropped_observations':0}
  assert fit['observation_count']==len(original)
  close(fit['common_intercept_statistics']['standard_error']**2,fit['normalized_level']['covariance_matrix'][0][0],'intercept variance identity',1e-12)
  close(fit['normalized_level']['covariance_matrix'][1][1],fit['covariance_matrix'][0][0],'unchanged slope covariance',1e-12)
  p=rows(native/'panel_classical_fe_intercept_predictions.csv');rp=rows(reference/'predictions.csv')
  assert len(p)==len(rp)==len(original)
  for i,(a,b,src) in enumerate(zip(p,rp,original),1):
    assert set(a)==set(b)==set(FIELDS)|{'source_row','fe_xb','fe_xbu','fe_u','fe_residual'}
    assert integer(a['source_row'])==integer(b['source_row'])==i
    for key in FIELDS:assert float(a[key])==float(b[key])==src[key],(i,key,'raw source')
    for key in ['fe_xb','fe_xbu','fe_u','fe_residual']:close(a[key],b[key],(i,key))
  print(f'PASS: 39 full QR inference fields, {len(p)} retained FE prediction rows/{8*len(p)} exact raw fields')


if __name__=='__main__':
  if len(sys.argv) not in (4,5) or len(sys.argv)==5 and sys.argv[4]!='--synthetic':
    raise SystemExit('usage: check_classical_intercept.py native_directory reference_directory MOM.dat [--synthetic]')
  audit(*map(Path,sys.argv[1:4]),synthetic=len(sys.argv)==5)
