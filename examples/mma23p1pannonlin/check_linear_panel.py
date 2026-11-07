#!/usr/bin/env python3
"""Audit exact source/design, three linear-panel fits and normalized FE inference."""
import csv
import json
import math
from pathlib import Path
import sys
from check_native_gamma_re import SOURCE, raw_source, source_equal


def rows(path):
  with path.open(newline='') as stream: return list(csv.DictReader(stream))


def keyed(records,label):
  result={}
  for row in records:
    key=(int(row['id']),int(row['year']));assert key not in result,('duplicate',label,key);result[key]=row
  return result


def close(actual,expected,label):
  actual,expected=float(actual),float(expected);tol=1e-9+1e-10*abs(expected)
  assert math.isfinite(actual) and math.isfinite(expected) and abs(actual-expected)<=tol,(label,actual,expected,tol)


def student_two_tail(statistic,dof):
  """Independent incomplete-beta tail for this source auditor, checked against R."""
  assert math.isfinite(statistic) and dof>0
  squared=statistic*statistic
  if squared==0: return 1.0
  if not math.isfinite(squared): return 0.0
  # Computing both arguments directly avoids cancellation in 1-x near t=0.
  x=dof/(dof+squared);complement=squared/(dof+squared)
  if x==0: return 0.0
  def fraction(a,b,z):
    tiny=1e-300;c=1.;d=1-(a+b)*z/(a+1)
    if abs(d)<tiny: d=math.copysign(tiny,d)
    d=1/d;h=d
    for m in range(1,501):
      for coefficient in [m*(b-m)*z/((a+2*m-1)*(a+2*m)),
        -(a+m)*(a+b+m)*z/((a+2*m)*(a+2*m+1))]:
        d=1+coefficient*d;c=1+coefficient/c
        if abs(d)<tiny: d=math.copysign(tiny,d)
        if abs(c)<tiny: c=math.copysign(tiny,c)
        d=1/d;delta=d*c;h*=delta
      if abs(delta-1)<3e-14: return h
    raise AssertionError('independent Student-t continued fraction did not converge')
  a=dof/2;b=.5
  prefactor=math.exp(math.lgamma(a+b)-math.lgamma(a)-math.lgamma(b)+a*math.log(x)+b*math.log(complement))
  result=prefactor*fraction(a,b,x)/a if x<(a+1)/(a+b+2) else 1-prefactor*fraction(b,a,complement)/b
  assert math.isfinite(result) and 0<=result<=1
  return result


def normalized_checks(fe,source):
  """Raw within/mean identities, independent of the R dummy-covariance artifact."""
  groups={}
  for row in source.values():
    x=float(row['logr']);pat=float(row['pat'])
    assert math.isfinite(x) and abs(x)<=20 and math.isfinite(pat) and 0<=pat<=1000 and pat==math.floor(pat)
    groups.setdefault(int(row['id']),[]).append((x,math.log(.5 if pat==0 else pat)))
  n=len(source);g=len(groups);df=n-g-1;assert df>0 and all(len(v)==5 for v in groups.values())
  centered=[]
  for group in groups.values():
    mx=math.fsum(x for x,y in group)/len(group);my=math.fsum(y for x,y in group)/len(group)
    centered.extend((x-mx,y-my) for x,y in group)
  sxx=math.fsum(x*x for x,y in centered);assert sxx>0
  beta=math.fsum(x*y for x,y in centered)/sxx
  xbar=math.fsum(x for group in groups.values() for x,y in group)/n
  ybar=math.fsum(y for group in groups.values() for x,y in group)/n;common=ybar-xbar*beta
  sigma2=math.fsum((y-beta*x)**2 for x,y in centered)/df;assert sigma2>0
  level=fe['normalized_level'];stats=fe['common_intercept_statistics']
  assert level['parameter_names']==['_cons','logr'] and level['reference_distribution']=='student_t'
  assert level['inference_df']==stats['degrees_of_freedom']==fe['inference_df']==fe['df_residual']==df
  assert stats['name']=='_cons' and stats['confidence_level']==95
  assert fe['unit_effect_normalization']=='estimation_observation_weighted_zero_mean'
  assert fe['observation_count']==n and fe['group_count']==g
  assert len(level['parameters'])==2 and len(level['covariance_matrix'])==2 and all(len(r)==2 for r in level['covariance_matrix'])
  for value,expected in zip(level['parameters'],[common,beta]): close(value,expected,'independent normalized parameter')
  close(fe['common_intercept'],common,'independent normalized intercept')
  cov=level['covariance_matrix'];assert cov[0][0]>0 and cov[1][1]>0
  # Dimensionless inverse identity avoids absolute tolerances hiding small variances.
  information=[[n/sigma2,n*xbar/sigma2],[n*xbar/sigma2,(sxx+n*xbar*xbar)/sigma2]]
  for i in range(2):
    for j in range(2):
      assert math.isfinite(cov[i][j]);terms=[cov[i][k]*information[k][j] for k in range(2)]
      assert abs(math.fsum(terms)-int(i==j))<=1e-9*(1+math.fsum(abs(v) for v in terms)),('independent normalized covariance inverse',i,j)
  close(cov[0][1],cov[1][0],'normalized symmetry')
  se=math.sqrt(sigma2*(1/n+xbar*xbar/sxx))
  close(stats['value'],common,'independent normalized reported value');close(stats['standard_error'],se,'independent normalized SE')
  close(stats['statistic'],common/se,'independent normalized t')
  expected_p=student_two_tail(common/se,df)
  assert 0<=stats['p_value']<=1 and abs(stats['p_value']-expected_p)<=1e-9*expected_p,('independent normalized Student-t p',stats['p_value'],expected_p)
  assert stats['ci_lower']<stats['value']<stats['ci_upper']
  close(stats['value']-stats['ci_lower'],stats['ci_upper']-stats['value'],'symmetric normalized CI')
  assert abs(student_two_tail((stats['ci_upper']-common)/se,df)-.05)<=1e-9*.05,'independent normalized Student-t CI'


def audit(work,reference_dir,output,original=None):
  references=rows(reference_dir/'reference.csv');assert len(references)==106,'expected106 R fields; regenerate with current linear_panel_reference.R'
  assert len({(r['model'],r['path']) for r in references})==106
  predictions=keyed(rows(reference_dir/'predictions.csv'),'R predictions')
  source_records=rows(work/'patents.csv');source=keyed(source_records,'source');assert source
  assert all(list(row)==SOURCE for row in source_records)
  if original is not None:
    expected=raw_source(original,True);assert source.keys()==expected.keys()
    for key,row in source.items(): source_equal(row,expected[key])
  events=[json.loads(line) for line in output.read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  def select(kind,count):
    values=[e['data'] for e in events if e.get('result_type')==kind];assert len(values)==count,(kind,len(values));return values
  pool,fe,re=[select(kind,1)[0] for kind in ['RegressionResult','FixedEffectsResult','RandomEffectsResult']]
  pl,fl=select('LincomResult',2);pt,ft=select('TestResult',2)
  models={'pooled':pool,'fe':fe,'re':re,'pooled_lincom':pl,'fe_lincom':fl,'pooled_test':pt,'fe_test':ft}
  assert pool['covariance']=='cluster(id)' and pool['estimator']=='ols'
  assert fe['covariance']=='nonrobust' and fe['common_intercept_inference']=='classical_student_t'
  assert re['estimator']=='balanced_swamy_arora_random_effects_gls' and re['covariance']=='nonrobust'
  assert re['reference_distribution']=='normal' and fe['reference_distribution']=='student_t'
  assert all(m['sample']['dropped_observations']==0 for m in (pool,fe,re))
  normalized_checks(fe,source)
  for row in references:
    value=models[row['model']]
    for key in row['path'].strip('/').split('/'): value=value[int(key)] if isinstance(value,list) else value[key]
    expected=float(row['value'])
    if row['path'].endswith('p_value') and expected>0:
      assert math.isfinite(float(value)) and abs(float(value)-expected)<=1e-9*expected,(row['model'],row['path'],value,expected,'relative1e-9')
    else: close(value,expected,row['model']+row['path'])
  count=0;previous=None
  for row in rows(work/'linear_panel_predictions.csv'):
    key=(int(row['id']),int(row['year']));assert previous is None or key>previous,'prediction rows not strictly ordered';previous=key
    actual_source=source.pop(key);expected=predictions.pop(key);source_equal(row,actual_source);source_equal(expected,actual_source)
    for name in ['lpat','pooled_xb','pooled_residual','fe_xb','fe_xbu','fe_u','fe_residual','re_xb','re_residual']: close(row[name],expected[name],f'{name} {key}')
    count+=1
  assert not source and not predictions and count==pool['observation_count'],'missing/extra source/prediction rows'
  facts=[]
  if original is not None:
    # Half-last-printed-unit gates are not exact historical storage/optimizer parity.
    facts=[
      ('pooled slope',pool['coefficients'][1]['value'],.8340745,5e-8),('pooled slope SE',pool['coefficients'][1]['standard_error'],.0228655,5e-8),
      ('pooled intercept',pool['coefficients'][0]['value'],.7954785,5e-8),('pooled intercept SE',pool['coefficients'][0]['standard_error'],.0579246,5e-8),('pooled F',pt['statistic'],1330.60,.005),
      ('within slope',fe['coefficients'][0]['value'],.1067505,5e-8),('within slope SE',fe['coefficients'][0]['standard_error'],.0560364,5e-8),('within normalized intercept',fe['common_intercept'],1.709116,5e-7),
      ('within R squared',fe['within_r_squared'],.0026,.00005),('within F',ft['statistic'],3.63,.005),('within effects F',fe['unit_effects_f_test']['f_statistic'],16.96,.005),
      ('RE slope',re['coefficients'][1]['value'],.7202377,5e-8),('RE slope SE',re['coefficients'][1]['standard_error'],.0237986,5e-8),('RE intercept',re['coefficients'][0]['value'],.9384761,5e-8),('RE intercept SE',re['coefficients'][0]['standard_error'],.0599584,5e-8),
      ('RE sigma_u',math.sqrt(re['sigma_u_squared']),.90057544,5e-9),('RE sigma_e',math.sqrt(re['sigma_e_squared']),.51119065,5e-9),('RE rho',re['rho'],.7563152,5e-8),('RE Wald',re['wald_statistic'],915.90,.005),
      ('within intercept SE',fe['common_intercept_statistics']['standard_error'],.0714557,5e-8),('within intercept t',fe['common_intercept_statistics']['statistic'],23.92,.005),
      ('within intercept p rounds to0.000',fe['common_intercept_statistics']['p_value'],0.,.0005),('within intercept CI lower',fe['common_intercept_statistics']['ci_lower'],1.568943,5e-7),('within intercept CI upper',fe['common_intercept_statistics']['ci_upper'],1.849289,5e-7)]
    for name,actual,printed,half in facts: assert abs(actual-printed)<=half,('author printed',name,actual,printed)
  return count,len(references),len(facts)


def main():
  if len(sys.argv) not in (4,5): raise SystemExit('usage: check_linear_panel.py work-dir reference-dir result.jsonl [original-source.asc]')
  count,quantities,facts=audit(*map(Path,sys.argv[1:4]),original=Path(sys.argv[4]) if len(sys.argv)==5 else None)
  if facts: print(f'PASS: {facts} selected author-log printed facts')
  print(f'PASS: {count} exact source/design and R prediction rows, {quantities} independent fit/test quantities')


if __name__=='__main__': main()
