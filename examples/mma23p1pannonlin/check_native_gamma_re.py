#!/usr/bin/env python3
"""Audit native patents shared-gamma RE against raw source, R and likelihood identities."""
import csv
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import statistics
import sys

SOURCE=['id','year','cusip','ardssic','scisect','logk','sumpat','logr','pat']
EXTRA=['selected_firm_total','included_in_native_re','native_re_mean','native_re_xb','native_re_residual']
HASH='87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8'
Z95=1.959963984540054


def rows(path):
  with path.open(newline='') as f: return list(csv.DictReader(f))


def near(actual,expected,label=''):
  a,b=float(actual),float(expected)
  tolerance=1e-9*b if label.startswith('p_') and b>0 else 1e-9+1e-10*abs(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=tolerance,(label,a,b,tolerance)


def raw_source(raw,original):
  if original: assert hashlib.sha256(raw.read_bytes()).hexdigest()==HASH
  tokens=raw.read_text(encoding='ascii').split();assert len(tokens)%25==0
  firms=len(tokens)//25;assert 2<=firms<=346
  if original: assert firms==346
  expected={}
  for i in range(firms):
    record=tokens[i*25:i*25+25]
    for t,year in enumerate(range(1975,1980)):
      values=[str(i+1),str(year),*record[:5],record[10+t],record[20+t]]
      expected[(i+1,year)]={k:'' if v=='.' else v for k,v in zip(SOURCE,values)}
  return expected


def source_equal(actual,expected):
  for key in SOURCE:
    if expected[key]=='': assert actual[key]=='',(key,actual[key])
    else: assert Decimal(actual[key])==Decimal(expected[key]),(key,actual[key],expected[key])


def independent_identity(data,exported):
  a,b,t=[data['coefficients'][0]['value'],data['coefficients'][1]['value'],data['ln_alpha']['value']]
  r=math.exp(-t);ll=[];scores=[[],[],[]];info=[[[] for _ in range(3)] for _ in range(3)]
  groups={}
  for row in exported: groups.setdefault(int(row['id']),[]).append(row)
  for group in groups.values():
    x=[float(v['logr']) for v in group];y=[int(float(v['pat'])) for v in group]
    eta=[a+b*v for v in x];mu=[math.exp(v) for v in eta];s=sum(y);m=math.fsum(mu)
    h=math.fsum(u*v for u,v in zip(mu,x));q=(r+s)/(r+m)
    dg=math.fsum(1/(r+j) for j in range(s))+math.log(r)+1-math.log(r+m)-(r+s)/(r+m)
    dgp=-math.fsum(1/(r+j)**2 for j in range(s))+1/r-1/(r+m)+(s-m)/(r+m)**2
    ll.append(math.lgamma(r+s)-math.lgamma(r)+r*math.log(r)-(r+s)*math.log(r+m)+math.fsum(v*z-math.lgamma(v+1) for v,z in zip(y,eta)))
    scores[0].append(s-q*m);scores[1].append(math.fsum(v*z for v,z in zip(y,x))-q*h);scores[2].append(-r*dg)
    info[0][0].append(q*m*r/(r+m));info[0][1].append(q*h*r/(r+m))
    info[1][1].append(q*(math.fsum(u*v*v for u,v in zip(mu,x))-h*h/(r+m)))
    info[0][2].append(-r*(m-s)*m/(r+m)**2);info[1][2].append(-r*(m-s)*h/(r+m)**2)
    info[2][2].append(-r*dg-r*r*dgp)
  near(data['log_likelihood'],math.fsum(ll),'independent likelihood')
  for score in scores: assert abs(math.fsum(score))<=1e-8+1e-10*math.fsum(abs(v) for v in score)
  matrix=[[0.0]*3 for _ in range(3)]
  for i in range(3):
    for j in range(i,3): matrix[i][j]=matrix[j][i]=math.fsum(info[i][j])
  covariance=data['covariance_matrix']
  for i in range(3):
    for j in range(3):
      terms=[matrix[i][k]*covariance[k][j] for k in range(3)]
      assert abs(math.fsum(terms)-int(i==j))<=1e-8*(1+math.fsum(abs(v) for v in terms)),('observed information inverse',i,j)


def audit(native,reference,raw,original=True):
  expected=raw_source(raw,original);source=rows(native/'patents.csv')
  assert len(source)==len(expected)
  seen=set()
  for row in source:
    assert list(row)==SOURCE
    key=(int(row['id']),int(row['year']));assert key not in seen;seen.add(key)
    source_equal(row,expected[key])
  assert seen==set(expected)
  events=[json.loads(s) for s in (native/'result.jsonl').read_text().splitlines()]
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','TableResult','SqlCreateResult','GammaPoissonResult','PredictionResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  n=len(source);g=n//5
  assert events[0]['data']['dataset']['row_count']==n
  assert events[1]['data']=={'variables':['id','year'],'missok':False,'missing_key_rows':0,'total_rows':n,'unique_groups':n}
  assert events[2]['data']=={'headers':['source_rows','source_firms','missing_nonmodel_ardssic'],
    'rows':[[n,g,sum(v['ardssic']=='' for v in source)]]}
  assert events[3]['data']=={'headers':['five_year_firms'],'rows':[[g]]}
  assert events[4]['data']['table_name']=='native_gamma_re_cohort'
  assert events[4]['data']['dataset']['row_count']==n
  d=events[5]['data'];coef=d['coefficients'];assert len(coef)==2
  assert [v['name'] for v in coef]==['_cons','logr'];assert d['ln_alpha']['name']=='lnalpha'
  assert d['covariance_parameter_names']==['_cons','logr','lnalpha']
  for key,value in {'outcome':'pat','predictors':['logr'],'entity_variable':'id','include_intercept':True,
    'estimator':'shared_gamma_poisson_re','random_effects_distribution':'mean_one_gamma',
    'likelihood':'integrated_shared_mean_one_gamma_poisson','covariance':'inverse_full_joint_observed_information',
    'covariance_correction':1.0,'reference_distribution':'asymptotic_standard_normal',
    'confidence_level':95,'coefficient_test_null':0.0,'converged':True,
    'ln_alpha_test_null':'ln_alpha=0_equivalently_alpha=1_not_zero_variance',
    'dispersion_likelihood_ratio_null':'alpha=0_same_sample_refitted_pooled_poisson',
    'dispersion_likelihood_ratio_reference':'equal_mixture_chi_square_0_1',
    'prediction_scope':'population_marginal_not_posterior_group_effects'}.items(): assert d[key]==value,(key,d[key])
  assert d['sample']=={'dropped_observations':0,'total_observations':n}
  assert d['observation_count']==n and d['group_count']==g and d['singleton_group_count']==0
  assert d['minimum_group_size']==d['maximum_group_size']==5
  assert 1<=d['iterations']<=100 and d['convergence_absolute_tolerance']==1e-10
  assert d['convergence_coordinate_system']=='raw_predictor_rms_and_log_alpha_per_group_score'
  for key in ('max_abs_scaled_score','max_abs_scaled_newton_step'): assert 0<=d[key]<=1e-10
  cov=d['covariance_matrix'];assert len(cov)==3 and all(len(row)==3 for row in cov)
  for i,value in enumerate(coef+[d['ln_alpha']]):
    assert cov[i][i]>0;near(value['standard_error'],math.sqrt(cov[i][i]))
    near(value['statistic'],value['value']/value['standard_error'])
    near(value['p_value'],math.erfc(abs(value['statistic'])/math.sqrt(2)),'p_normal')
    near(value['confidence_lower'],value['value']-Z95*value['standard_error'])
    near(value['confidence_upper'],value['value']+Z95*value['standard_error'])
    for j in range(3): near(cov[i][j],cov[j][i])
  near(d['alpha'],math.exp(d['ln_alpha']['value']))
  near(d['alpha_standard_error'],d['alpha']*d['ln_alpha']['standard_error'])
  near(d['alpha_confidence_lower'],math.exp(d['ln_alpha']['confidence_lower']))
  near(d['alpha_confidence_upper'],math.exp(d['ln_alpha']['confidence_upper']))
  near(d['dispersion_likelihood_ratio'],2*(d['log_likelihood']-d['poisson_log_likelihood']))
  near(d['dispersion_likelihood_ratio_p_value'],.5*math.erfc(math.sqrt(d['dispersion_likelihood_ratio']/2)),'p_lr_mixture')
  near(d['wald_statistic'],coef[1]['statistic']**2);assert d['wald_df']==1
  near(d['wald_p_value'],coef[1]['p_value'],'p_wald')
  summary=events[9]['data']['rows'];assert [v['variable'] for v in summary]==['native_re_mean','native_re_residual']
  values={'candidate_rows':n,'firms':g,'zero_total_firms':d['zero_total_group_count'],
    'alpha':d['alpha'],'se_alpha':d['alpha_standard_error'],'lower_alpha':d['alpha_confidence_lower'],'upper_alpha':d['alpha_confidence_upper'],
    'log_likelihood':d['log_likelihood'],'poisson_log_likelihood':d['poisson_log_likelihood'],
    'lr_alpha_zero':d['dispersion_likelihood_ratio'],'p_lr_alpha_zero':d['dispersion_likelihood_ratio_p_value'],
    'mean_prediction':summary[0]['mean'],'mean_residual':summary[1]['mean']}
  for name,value in zip(['intercept','slope','lnalpha'],coef+[d['ln_alpha']]):
    for dest,field in [('estimate','value'),('se','standard_error'),('z','statistic'),('p','p_value'),('lower','confidence_lower'),('upper','confidence_upper')]: values[f'{dest}_{name}']=value[field]
  for name,i,j in [('aa',0,0),('ab',0,1),('at',0,2),('bb',1,1),('bt',1,2),('tt',2,2)]: values['covariance_'+name]=cov[i][j]
  oracle={v['field']:float(v['value']) for v in rows(reference/'reference.csv')}
  assert len(values)==len(oracle)==37 and values.keys()==oracle.keys()
  for key,value in oracle.items(): near(values[key],value,key)
  exported=rows(native/'native_gamma_re_predictions.csv');prediction=rows(reference/'predictions.csv')
  assert len(exported)==len(prediction)==n
  total={i:sum(float(v['pat']) for k,v in expected.items() if k[0]==i) for i in range(1,g+1)}
  for row,independent,key in zip(exported,prediction,sorted(expected)):
    assert list(row)==SOURCE+EXTRA
    assert (int(row['id']),int(row['year']))==key
    source_equal(row,expected[key]);assert row['included_in_native_re']=='true'
    near(row['selected_firm_total'],total[key[0]])
    for field in ['id','year','logr','pat']: assert float(row[field])==float(independent[field])
    mu=float(row['native_re_mean']);xb=float(row['native_re_xb']);res=float(row['native_re_residual'])
    near(mu,independent['marginal_mu']);near(res,independent['marginal_residual'])
    near(xb,coef[0]['value']+coef[1]['value']*float(row['logr']));near(mu,math.exp(xb));near(res,float(row['pat'])-mu)
  assert d['zero_total_group_count']==sum(v==0 for v in total.values())
  assert d['zero_count']==sum(float(v['pat'])==0 for v in exported)
  near(d['count_sum'],sum(float(v['pat']) for v in exported));near(d['maximum_group_count_sum'],max(total.values()))
  for result,field in zip(summary,['native_re_mean','native_re_residual']):
    samples=[float(v[field]) for v in exported];assert result['count']==n
    for label,value in [('mean',statistics.fmean(samples)),('std_dev',statistics.stdev(samples)),('minimum',min(samples)),('maximum',max(samples))]: near(result[label],value)
  assert events[10]['data']['headers']==['scope'] and len(events[10]['data']['rows'])==1
  assert events[-1]['data']['path']=='native_gamma_re_predictions.csv'
  assert events[-1]['data']['dataset']['row_count']==n
  independent_identity(d,exported)
  if original:
    for field,printed,half in [('estimate_slope',.3487832,5e-8),('estimate_intercept',2.312705,5e-7),('se_slope',.0332254,5e-8),('se_intercept',.124758,5e-7),('se_lnalpha',.0899144,5e-8),('log_likelihood',-5553.1787,5e-5)]:
      assert abs(values[field]-printed)<=half,('printed author fact',field)
  return n,g,len(values)


def main():
  assert len(sys.argv) in (4,5),'usage: check_native_gamma_re.py native-dir reference-dir raw.asc [--control]'
  if len(sys.argv)==5: assert sys.argv[4]=='--control'
  print('validated source/prediction rows, firms, independent R fields:',audit(*map(Path,sys.argv[1:4]),original=len(sys.argv)==4))


if __name__=='__main__': main()
