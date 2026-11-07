#!/usr/bin/env python3
"""Audit patents native GEE against raw source, R and independent mean/score/bread identities."""
import json
import math
from pathlib import Path
import statistics
import sys
from check_native_gamma_re import SOURCE, Z95, raw_source, rows, source_equal

EXTRA=['selected_firm_total','included_in_native_gee','native_gee_mean','native_gee_xb','native_gee_residual']
EXPORT='native_poisson_gee_predictions.csv'


def near(actual,expected,label=''):
  a,b=float(actual),float(expected)
  tol=1e-8*abs(b) if label.startswith('p_') and b>0 else 1e-9+1e-10*abs(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=tol,(label,a,b,tol)


def independent_equations(data,exported,reference):
  beta=[c['value'] for c in data['working']['coefficients']]
  groups={};ss=[];sg=[];score={};bread=[[[] for _ in range(2)] for _ in range(2)]
  for row in exported: groups.setdefault(int(row['id']),[]).append(row)
  rho=data['rho'];adjust=rho/(1+4*rho)
  assert -.25<rho<1
  for key,group in groups.items():
    assert len(group)==5
    x=[[1.0,float(row['logr'])] for row in group];y=[float(row['pat']) for row in group]
    mu=[math.exp(beta[0]+beta[1]*row[1]) for row in x]
    e=[(v-m)/math.sqrt(m) for v,m in zip(y,mu)]
    z=[[math.sqrt(m)*v for v in row] for row,m in zip(x,mu)]
    sums=[math.fsum(row[j] for row in z) for j in range(2)];se=math.fsum(e)
    ss.extend(v*v for v in e);sg.append(se*se)
    score[key]=[(math.fsum(row[j]*v for row,v in zip(z,e))-adjust*sums[j]*se)/(1-rho) for j in range(2)]
    for j in range(2):
      for k in range(2): bread[j][k].append((math.fsum(row[j]*row[k] for row in z)-adjust*sums[j]*sums[k])/(1-rho))
  ss=math.fsum(ss);near(data['pearson_normalizer'],ss/len(exported),'independent Pearson normalizer')
  near(rho,(math.fsum(sg)-ss)/(4*ss),'independent legacy rho')
  for j in range(2):
    values=[v[j] for v in score.values()]
    assert abs(math.fsum(values))<=1e-8+1e-10*math.fsum(abs(v) for v in values),('independent GEE score',j)
  matrix=[[math.fsum(cell) for cell in row] for row in bread]
  det=matrix[0][0]*matrix[1][1]-matrix[0][1]*matrix[1][0];assert det>0 and math.isfinite(det)
  inverse=[[matrix[1][1]/det,-matrix[0][1]/det],[-matrix[1][0]/det,matrix[0][0]/det]]
  influence=[[math.fsum(inverse[j][k]*u[k] for k in range(2)) for j in range(2)] for u in score.values()]
  correction=len(groups)/(len(groups)-1)
  for j in range(2):
    for k in range(2):
      near(data['working']['covariance'][j][k],inverse[j][k],'independent working inverse')
      near(data['sandwich']['covariance'][j][k],correction*math.fsum(u[j]*u[k] for u in influence),'independent group sandwich')
  oracle=rows(reference/'scores.csv');assert len(oracle)==len(groups)
  assert len({int(r['id']) for r in oracle})==len(groups)
  for row in oracle:
    for j,field in enumerate(['score_a','score_b']):
      a,b=score[int(row['id'])][j],float(row[field])
      assert abs(a-b)<=1e-8+1e-10*abs(b),('R group score',row['id'],field,a,b)


def audit(native,reference,raw,original=True):
  expected=raw_source(raw,original);source=rows(native/'patents.csv');assert len(source)==len(expected)
  seen=set()
  for row in source:
    assert list(row)==SOURCE
    key=(int(row['id']),int(row['year']));assert key not in seen;seen.add(key);source_equal(row,expected[key])
  assert seen==set(expected)
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','TableResult','SqlCreateResult','PoissonGeeResult','PredictionResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  n=len(source);g=n//5;assert 3<=g<=346
  for row in source:
    y=float(row['pat']);x=float(row['logr']);assert math.isfinite(y) and 0<=y<=1000 and y==math.floor(y) and math.isfinite(x) and abs(x)<=20
  assert events[0]['data']['dataset']['row_count']==n
  assert events[1]['data']=={'variables':['id','year'],'missok':False,'missing_key_rows':0,'total_rows':n,'unique_groups':n}
  assert events[2]['data']=={'headers':['source_rows','source_firms','missing_nonmodel_ardssic'],'rows':[[n,g,sum(v['ardssic']=='' for v in source)]]}
  assert events[3]['data']=={'headers':['five_year_firms'],'rows':[[g]]}
  assert events[4]['data']['table_name']=='native_poisson_gee_cohort' and events[4]['data']['dataset']['row_count']==n
  d=events[5]['data']
  for key,value in {'outcome':'pat','predictors':['logr'],'entity_variable':'id','family':'poisson','link':'log','include_intercept':True,
    'estimator':'poisson_log_exchangeable_gee','correlation':'legacy_balanced_normalized_pearson','working_scale':1.0,
    'working_covariance':'inverse_gee_bread_fixed_scale1','sandwich_covariance':'group_scores_G_over_G_minus_1',
    'independent_unit':'group','reference_distribution':'asymptotic_standard_normal','joint_test_reference':'asymptotic_chi_square',
    'confidence_level':95,'missingness':'joint_finite_outcome_predictors_and_group','prediction_scope':'population_marginal_mean_log_link','converged':True}.items(): assert d[key]==value,(key,d[key])
  assert not any(key in d for key in ['log_likelihood','likelihood_ratio','r_squared','ln_alpha','fitted_means'])
  assert d['observation_count']==n and d['group_count']==g and d['group_size']==5
  assert d['sample']=={'total_observations':n,'dropped_observations':0}
  near(d['sandwich_correction'],g/(g-1));assert 0<=d['iterations']<=100
  assert 0<=d['max_abs_scaled_score']<=1e-10 and 0<=d['max_abs_scaled_newton_step']<=1e-12
  values={'observations':n,'firms':g,'intercept':d['working']['coefficients'][0]['value'],'slope':d['working']['coefficients'][1]['value'],'rho':d['rho']}
  for name,method in [('working','working'),('cluster','sandwich')]:
    inference=d[method];coef=inference['coefficients'];cov=inference['covariance']
    assert len(coef)==2 and [c['name'] for c in coef]==['_cons','logr']
    assert len(cov)==2 and all(len(row)==2 for row in cov)
    assert [c['value'] for c in coef]==[values['intercept'],values['slope']]
    for j,c in enumerate(coef):
      assert cov[j][j]>0;near(c['standard_error'],math.sqrt(cov[j][j]));near(c['statistic'],c['value']/c['standard_error'])
      assert 0<=c['p_value']<=1
      near(c['p_value'],math.erfc(abs(c['statistic'])/math.sqrt(2)),'p_normal')
      near(c['confidence_lower'],c['value']-Z95*c['standard_error']);near(c['confidence_upper'],c['value']+Z95*c['standard_error'])
      for k in range(2): near(cov[j][k],cov[k][j])
    assert inference['wald_df']==1;near(inference['wald_statistic'],coef[1]['statistic']**2)
    assert 0<=inference['wald_p_value']<=1
    near(inference['wald_p_value'],coef[1]['p_value'],'p_wald')
    for suffix,j,k in [('aa',0,0),('ab',0,1),('bb',1,1)]: values[name+'_'+suffix]=cov[j][k]
    for label,c in zip(['intercept','slope'],coef):
      for prefix,field in [('se','standard_error'),('z','statistic'),('lo','confidence_lower'),('hi','confidence_upper')]: values[f'{name}_{prefix}_{label}']=c[field]
  oracle={r['field']:float(r['value']) for r in rows(reference/'reference.csv')}
  assert len(values)==len(oracle)==27 and values.keys()==oracle.keys()
  for key,value in oracle.items(): near(values[key],value,key)
  exported=rows(native/EXPORT);predictions=rows(reference/'predictions.csv');assert len(exported)==len(predictions)==n
  totals={i:sum(int(float(v['pat'])) for key,v in expected.items() if key[0]==i) for i in range(1,g+1)}
  for row,ref,key in zip(exported,predictions,sorted(expected)):
    assert list(row)==SOURCE+EXTRA and (int(row['id']),int(row['year']))==key
    source_equal(row,expected[key]);assert row['included_in_native_gee']=='true';near(row['selected_firm_total'],totals[key[0]])
    for field in ['id','year','logr','pat']: assert float(row[field])==float(ref[field])
    mu=float(row['native_gee_mean']);xb=float(row['native_gee_xb']);res=float(row['native_gee_residual'])
    near(mu,ref['gee_mu']);near(res,ref['gee_residual']);near(xb,values['intercept']+values['slope']*float(row['logr']));near(mu,math.exp(xb));near(res,float(row['pat'])-mu)
  assert d['zero_total_group_count']==sum(v==0 for v in totals.values())
  assert d['zero_count']==sum(int(float(v['pat']))==0 for v in exported);near(d['count_sum'],sum(int(float(v['pat'])) for v in exported))
  summary=events[9]['data']['rows'];assert [row['variable'] for row in summary]==['native_gee_mean','native_gee_residual']
  for result,field in zip(summary,['native_gee_mean','native_gee_residual']):
    samples=[float(v[field]) for v in exported];assert result['count']==n
    for key,value in [('mean',statistics.fmean(samples)),('std_dev',statistics.stdev(samples)),('minimum',min(samples)),('maximum',max(samples))]: near(result[key],value)
  assert events[10]['data']['headers']==['scope'] and len(events[10]['data']['rows'])==1
  assert events[-1]['data']['path']==EXPORT and events[-1]['data']['dataset']['row_count']==n
  independent_equations(d,exported,reference)
  return n,g,len(values)


def main():
  assert len(sys.argv) in (4,5),'usage: check_native_poisson_gee.py native-dir reference-dir raw.asc [--control]'
  if len(sys.argv)==5: assert sys.argv[4]=='--control'
  print('validated source/prediction rows, firms, R fields:',audit(*map(Path,sys.argv[1:4]),original=len(sys.argv)==4))


if __name__=='__main__': main()
