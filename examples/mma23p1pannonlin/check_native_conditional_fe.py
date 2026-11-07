#!/usr/bin/env python3
"""Raw-source/R and direct conditional-likelihood audit of native patents FE."""
import json
import math
from pathlib import Path
import statistics
import sys
from check_native_gamma_re import SOURCE, Z95, raw_source, rows, source_equal

EXTRA=['selected_firm_total','included_in_native_fe','native_fe_mean','native_fe_xb','native_fe_residual']
EXPORT='native_conditional_fe_predictions.csv'


def near(actual,expected,label=''):
  a,b=float(actual),float(expected);tol=1e-8*abs(b) if label.startswith('p_') and b>0 else 1e-9+1e-10*abs(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=tol,(label,a,b,tol)


def independent_equations(data,exported):
  beta=data['coefficients'][0]['value'];groups={}
  for row in exported: groups.setdefault(int(row['id']),[]).append(row)
  ll=[];null=[];score=[];info=[]
  for group in groups.values():
    y=[float(row['pat']) for row in group];total=math.fsum(y)
    if not total: continue
    x=[float(row['logr']) for row in group];eta=[beta*v for v in x];shift=max(eta)
    weights=[math.exp(v-shift) for v in eta];normalizer=math.fsum(weights);prob=[v/normalizer for v in weights]
    center=math.fsum(p*v for p,v in zip(prob,x));mu=[total*p for p in prob]
    constant=math.lgamma(total+1)-math.fsum(math.lgamma(v+1) for v in y)
    ll.append(constant+math.fsum(v*math.log(p) for v,p in zip(y,prob)))
    null.append(constant-total*math.log(len(group)))
    score.append(math.fsum((v-m)*(z-center) for v,m,z in zip(y,mu,x)))
    info.append(total*math.fsum(p*(v-center)**2 for p,v in zip(prob,x)))
    for row,expected in zip(group,mu): near(row['native_fe_mean'],expected,'independent conditional mean')
    near(math.fsum(float(row['native_fe_mean']) for row in group),total,'positive-group total')
  near(data['conditional_log_likelihood'],math.fsum(ll),'independent conditional likelihood')
  near(data['null_conditional_log_likelihood'],math.fsum(null),'independent null likelihood')
  near(data['covariance_matrix'][0][0]*math.fsum(info),1.0,'independent information inverse identity')
  assert abs(math.fsum(score))<=1e-8+1e-10*math.fsum(abs(v) for v in score),('independent score',math.fsum(score))
  lr=2*(math.fsum(ll)-math.fsum(null));near(data['likelihood_ratio'],lr,'independent LR')
  near(data['likelihood_ratio_p_value'],math.erfc(math.sqrt(lr/2)),'p_LR')


def audit(native,reference,raw,original=True):
  expected=raw_source(raw,original);source=rows(native/'patents.csv');assert len(source)==len(expected)
  seen=set()
  for row in source:
    assert list(row)==SOURCE
    key=(int(row['id']),int(row['year']));assert key not in seen;seen.add(key);source_equal(row,expected[key])
    y=float(row['pat']);x=float(row['logr']);assert math.isfinite(y) and 0<=y<=1000 and y==math.floor(y) and math.isfinite(x) and abs(x)<=20
  assert seen==set(expected)
  n=len(source);g=n//5;totals={i:sum(float(row['pat']) for key,row in expected.items() if key[0]==i) for i in range(1,g+1)}
  excluded=sum(v==0 for v in totals.values());retained=g-excluded;assert retained>=2
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','TableResult','SqlCreateResult','ConditionalPoissonResult','PredictionResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert events[0]['data']['dataset']['row_count']==n
  assert events[1]['data']=={'variables':['id','year'],'missok':False,'missing_key_rows':0,'total_rows':n,'unique_groups':n}
  assert events[2]['data']=={'headers':['source_rows','source_firms','missing_nonmodel_ardssic'],'rows':[[n,g,sum(r['ardssic']=='' for r in source)]]}
  assert events[3]['data']=={'headers':['five_year_firms'],'rows':[[g]]}
  assert events[4]['data']['table_name']=='native_conditional_fe_cohort' and events[4]['data']['dataset']['row_count']==n
  d=events[5]['data']
  for key,value in {'estimator':'conditional_poisson_fe','likelihood':'conditional_multinomial_given_selected_group_totals','outcome':'pat','predictors':['logr'],'entity_variable':'id','include_intercept':False,
    'covariance_parameter_names':['logr'],'covariance':'inverse_conditional_information','covariance_correction':1.0,'reference_distribution':'asymptotic_standard_normal','confidence_level':95,
    'missingness':'joint_finite_rows_then_zero_total_group_exclusion_after_raw_key_validation','null_model':'zero_slopes_same_selected_group_totals',
    'joint_test_reference':'asymptotic_chi_square','prediction_scope':'retained_positive_group_normalization_not_population_forecast','converged':True,
    'convergence_absolute_tolerance':1e-10,'convergence_coordinate_system':'within_first_row_rms_scaled_per_positive_group_score'}.items(): assert d[key]==value,(key,d[key])
  assert d['observation_count']==retained*5 and d['joint_finite_observation_count']==n
  assert d['candidate_group_count']==g and d['group_count']==retained
  assert d['zero_total_group_count']==excluded and d['zero_total_observation_count']==excluded*5
  assert d['sample']=={'total_observations':n,'dropped_observations':excluded*5}
  assert d['minimum_group_size']==d['maximum_group_size']==5
  assert 1<=d['iterations']<=100
  for key in ['max_abs_scaled_score','max_abs_scaled_newton_step']: assert 0<=d[key]<=1e-10
  coef=d['coefficients'];cov=d['covariance_matrix'];assert len(coef)==1 and coef[0]['name']=='logr' and len(cov)==1 and len(cov[0])==1
  c=coef[0];assert cov[0][0]>0 and 0<=c['p_value']<=1
  near(c['standard_error'],math.sqrt(cov[0][0]));near(c['statistic'],c['value']/c['standard_error']);near(c['p_value'],math.erfc(abs(c['statistic'])/math.sqrt(2)),'p_normal')
  near(c['confidence_lower'],c['value']-Z95*c['standard_error']);near(c['confidence_upper'],c['value']+Z95*c['standard_error'])
  assert d['wald_df']==d['likelihood_ratio_df']==1
  near(d['wald_statistic'],c['statistic']**2);near(d['wald_p_value'],c['p_value'],'p_Wald')
  values={'candidate_rows':n,'candidate_firms':g,'retained_rows':retained*5,'retained_firms':retained,'excluded_rows':excluded*5,'excluded_firms':excluded,
    'beta':c['value'],'se':c['standard_error'],'z':c['statistic'],'p_value':c['p_value'],'ci_lower':c['confidence_lower'],'ci_upper':c['confidence_upper'],
    'conditional_ll':d['conditional_log_likelihood'],'information':1/cov[0][0]}
  oracle={r['field']:float(r['value']) for r in rows(reference/'reference.csv')};assert len(oracle)==len(values)==14 and oracle.keys()==values.keys()
  for key,value in oracle.items(): near(values[key],value,key)
  exported=rows(native/EXPORT);prediction=rows(reference/'predictions.csv');assert len(exported)==len(prediction)==n
  for row,ref,key in zip(exported,prediction,sorted(expected)):
    assert list(row)==SOURCE+EXTRA and (int(row['id']),int(row['year']))==key
    source_equal(row,expected[key]);included=totals[key[0]]>0;assert row['included_in_native_fe']==str(included).lower()
    assert ref['included_in_fe']==str(included).upper();near(row['selected_firm_total'],totals[key[0]])
    for field in ['id','year','logr','pat']: assert float(row[field])==float(ref[field])
    near(row['native_fe_xb'],c['value']*float(row['logr']),'slope-only index')
    for field,reference_field in [('native_fe_mean','fe_mean'),('native_fe_residual','fe_residual')]:
      if not included: assert row[field]==ref[reference_field]==''
      else: near(row[field],ref[reference_field],field)
    if included: near(row['native_fe_residual'],float(row['pat'])-float(row['native_fe_mean']),'residual identity')
  positive=[row for row in exported if row['included_in_native_fe']=='true']
  assert d['zero_count']==sum(float(r['pat'])==0 for r in positive);near(d['count_sum'],sum(float(r['pat']) for r in positive))
  summary=events[9]['data']['rows'];assert [r['variable'] for r in summary]==['native_fe_mean','native_fe_residual']
  for result,field in zip(summary,['native_fe_mean','native_fe_residual']):
    samples=[float(row[field]) for row in positive];assert result['count']==len(samples)
    for key,value in [('mean',statistics.fmean(samples)),('std_dev',statistics.stdev(samples)),('minimum',min(samples)),('maximum',max(samples))]: near(result[key],value)
  assert events[10]['data']['headers']==['scope'] and len(events[10]['data']['rows'])==1
  assert events[-1]['data']['path']==EXPORT and events[-1]['data']['dataset']['row_count']==n
  independent_equations(d,exported)
  if original:
    for key,printed,half in [('beta',-.0377642,5e-8),('se',.0325518,5e-8),('p_value',.246,5e-4),('conditional_ll',-3659.5926,5e-5)]: assert abs(values[key]-printed)<=half,('selected printed fact',key)
  return n,retained*5,retained,len(values)


def main():
  assert len(sys.argv) in (4,5),'usage: check_native_conditional_fe.py native-dir reference-dir raw.asc [--control]'
  if len(sys.argv)==5: assert sys.argv[4]=='--control'
  print('validated source/retained rows, positive firms, R fields:',audit(*map(Path,sys.argv[1:4]),original=len(sys.argv)==4))


if __name__=='__main__': main()
