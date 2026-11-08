#!/usr/bin/env python3
"""Audit native Gaussian-log-effect Poisson from raw source, R and raw equations."""
from functools import lru_cache
import json
import math
from pathlib import Path
import statistics
import sys
from check_native_gamma_re import SOURCE, EXTRA, Z95, rows, near, raw_source, source_equal


def adaptive_integral(function, left, right):
  """Vector adaptive Simpson; mode-centered finite tails separately checked."""
  fa, fb, fm = function(left), function(right), function((left+right)/2)
  def panel(a,b,fa,fm,fb): return [(b-a)*(x+4*y+z)/6 for x,y,z in zip(fa,fm,fb)]
  def refine(a,b,fa,fm,fb,whole,tol,depth):
    middle=(a+b)/2;fl=function((a+middle)/2);fr=function((middle+b)/2)
    first=panel(a,middle,fa,fl,fm);second=panel(middle,b,fm,fr,fb)
    together=[x+y for x,y in zip(first,second)]
    delta=[x-y for x,y in zip(together,whole)]
    if all(abs(d)<=15*tol*(1+abs(v)) for d,v in zip(delta,together)):
      return [v+d/15 for v,d in zip(together,delta)]
    assert depth>0,'independent Simpson subdivision limit'
    return [x+y for x,y in zip(refine(a,middle,fa,fl,fm,first,tol/2,depth-1),refine(middle,b,fm,fr,fb,second,tol/2,depth-1))]
  return refine(left,right,fa,fm,fb,panel(left,right,fa,fm,fb),2e-13,24)


@lru_cache(maxsize=4096)
def posterior(s,m,v):
  left,right=-100.,100.
  for _ in range(100):
    u=(left+right)/2
    if s-m*math.exp(u)-u/v>0:left=u
    else:right=u
  mode=(left+right)/2;scale=1/math.sqrt(m*math.exp(mode)+1/v)
  def kernel(u):return s*u-m*math.exp(u)-u*u/(2*v)
  anchor=kernel(mode);intensity=m*math.exp(mode);slope=s-intensity-mode/v
  def difference(z):
    d=scale*z
    # Subtracting two O(S*u) kernels injects roundoff into tiny Simpson panels.
    # This is the same log kernel difference, evaluated without that subtraction.
    emd=d*d*(.5+d*(1/6+d*(1/24+d*(1/120+d*(1/720+d*(1/5040+d/40320)))))) if abs(d)<.01 else math.expm1(d)-d
    return slope*d-intensity*emd-d*d/(2*v)
  def weight(z):
    u=mode+scale*z;lw=difference(z)
    return u,math.exp(lw) if lw> -745 else 0.
  extent=8.
  # Gaussian prior bounds tails even for zero totals. Refuse an unverified tail,
  # rather than silently treating a fixed integration window as exact.
  while any(difference(z)> -90 for z in [-extent,extent]):
    extent*=2;assert extent<=128,'independent integration tails not negligible'
  def first(z):
    _,w=weight(z);return [w,w*math.expm1(scale*z),w*z,w*z*z]
  den,em,ez,ez2=adaptive_integral(first,-extent,extent)
  em/=den;ez/=den;ez2/=den
  multiplier=math.exp(mode);eu=multiplier*(1+em)
  u2=mode*mode+2*mode*scale*ez+scale*scale*ez2
  def centered(z):
    _,w=weight(z);dx=math.expm1(scale*z)-em
    # Dimensionless prior/intensity units keep the adaptive error budget
    # meaningful for both100000-count and tiny-mean posterior controls.
    du=(2*mode*scale*(z-ez)+scale*scale*(z*z-ez2))/v
    return [w*dx*dx,w*du*du,w*dx*du]
  vex,vu2,cross=[a/den for a in adaptive_integral(centered,-extent,extent)]
  return anchor+math.log(den*scale)-math.log(2*math.pi*v)/2,eu,u2,vex*multiplier*multiplier,vu2*v*v,cross*multiplier*v


def independent_identity(data,exported):
  a,b,t=[data['coefficients'][0]['value'],data['coefficients'][1]['value'],data['ln_variance']['value']]
  v=math.exp(t);groups={}
  for row in exported:groups.setdefault(int(row['id']),[]).append(row)
  ll=[];scores=[[],[],[]];info=[[[] for _ in range(3)] for _ in range(3)]
  for group in groups.values():
    x=[float(r['logr']) for r in group];y=[int(float(r['pat'])) for r in group]
    eta=[a+b*z for z in x];assert all(-40<=z<=40 for z in eta)
    mu=[math.exp(z) for z in eta];s=sum(y);m=math.fsum(mu);h=math.fsum(z*q for z,q in zip(mu,x));j=math.fsum(z*q*q for z,q in zip(mu,x))
    logz,eu,u2,vex,vu2,cross=posterior(s,m,v)
    ll.append(math.fsum(q*z-math.lgamma(q+1) for q,z in zip(y,eta))+logz)
    scores[0].append(s-m*eu);scores[1].append(math.fsum(q*z for q,z in zip(y,x))-h*eu);scores[2].append((u2/v-1)/2)
    info[0][0].append(m*eu-m*m*vex);info[0][1].append(h*eu-m*h*vex);info[1][1].append(j*eu-h*h*vex)
    info[0][2].append(m*cross/(2*v));info[1][2].append(h*cross/(2*v));info[2][2].append(u2/(2*v)-vu2/(4*v*v))
  near(data['log_likelihood'],math.fsum(ll),'independent integrated likelihood')
  score=[math.fsum(z) for z in scores]
  for total,terms in zip(score,scores):assert abs(total)<=1e-8+1e-10*math.fsum(abs(z) for z in terms),('raw integrated score',score)
  matrix=[[0.]*3 for _ in range(3)]
  for i in range(3):
    for j in range(i,3):matrix[i][j]=matrix[j][i]=math.fsum(info[i][j])
  cov=data['covariance_matrix']
  for i in range(3):
    for j in range(3):
      terms=[matrix[i][k]*cov[k][j] for k in range(3)]
      assert abs(math.fsum(terms)-int(i==j))<=1e-8*(1+math.fsum(abs(z) for z in terms)),('raw information inverse',i,j)
  # A second raw-source defense for the null, independent of the reported R LL.
  beta=[a+v/2,b]
  for _ in range(50):
    aa=[];ab=[];bb=[];sa=[];sb=[]
    for row in exported:
      x=float(row['logr']);y=float(row['pat']);mu=math.exp(beta[0]+beta[1]*x)
      aa.append(mu);ab.append(mu*x);bb.append(mu*x*x);sa.append(y-mu);sb.append((y-mu)*x)
    aa,ab,bb,sa,sb=map(math.fsum,[aa,ab,bb,sa,sb]);det=aa*bb-ab*ab;assert det>0
    step=[(bb*sa-ab*sb)/det,(aa*sb-ab*sa)/det];beta=[q+d for q,d in zip(beta,step)]
    if max(abs(d) for d in step)<1e-13:break
  else:raise AssertionError('independent pooled null failed')
  pooled=math.fsum(float(r['pat'])*(beta[0]+beta[1]*float(r['logr']))-math.exp(beta[0]+beta[1]*float(r['logr']))-math.lgamma(float(r['pat'])+1) for r in exported)
  near(data['poisson_log_likelihood'],pooled,'raw refitted pooled null')
  return score


def audit(native,reference,raw,original=True):
  expected=raw_source(raw,original);source=rows(native/'patents.csv');n=len(source);g=n//5
  assert len(expected)==n
  seen=set()
  for row in source:
    assert list(row)==SOURCE;key=(int(row['id']),int(row['year']));assert key not in seen;seen.add(key);source_equal(row,expected[key])
  assert seen==set(expected)
  events=[json.loads(s) for s in (native/'result.jsonl').read_text().splitlines()]
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','TableResult','SqlCreateResult','NormalPoissonResult','PredictionResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert events[0]['data']['dataset']['row_count']==n
  assert events[1]['data']=={'variables':['id','year'],'missok':False,'missing_key_rows':0,'total_rows':n,'unique_groups':n}
  assert events[2]['data']=={'headers':['source_rows','source_firms','missing_nonmodel_ardssic'],'rows':[[n,g,sum(r['ardssic']=='' for r in source)]]}
  assert events[3]['data']=={'headers':['five_year_firms'],'rows':[[g]]}
  assert events[4]['data']['table_name']=='native_normal_re_cohort' and events[4]['data']['dataset']['row_count']==n
  d=events[5]['data'];coef=d['coefficients'];assert len(coef)==2 and [c['name'] for c in coef]==['_cons','logr'];assert d['ln_variance']['name']=='lnsigma2'
  assert d['covariance_parameter_names']==['_cons','logr','lnsigma2']
  for key,value in {'outcome':'pat','predictors':['logr'],'entity_variable':'id','include_intercept':True,'estimator':'normal_random_intercept_poisson_re',
    'random_effects_distribution':'mean_zero_gaussian_log_intercept','likelihood':'integrated_mean_zero_gaussian_log_effect_poisson','variance_parameterization':'lnsigma2_is_log_variance_of_gaussian_log_effect',
    'covariance':'inverse_full_joint_observed_information','covariance_correction':1.0,'reference_distribution':'asymptotic_standard_normal','confidence_level':95,'coefficient_test_null':0.0,
    'ln_variance_test_null':'lnsigma2=0_equivalently_variance=1_not_zero_variance','variance_likelihood_ratio_null':'variance=0_same_sample_refitted_pooled_poisson',
    'variance_likelihood_ratio_reference':'equal_mixture_chi_square_0_1','prediction_scope':'population_marginal_exp_xbeta_plus_variance_over_two_not_posterior_group_effects',
    'quadrature':'posterior_adaptive_paired_physicists_gauss_hermite_80_96','quadrature_relative_discrepancy_limit':1e-9,'quadrature_discrepancy_is_error_bound':False,'converged':True}.items():assert d[key]==value,(key,d[key])
  assert d['sample']=={'total_observations':n,'dropped_observations':0}
  assert (d['observation_count'],d['group_count'],d['singleton_group_count'],d['minimum_group_size'],d['maximum_group_size'])==(n,g,0,5,5)
  assert 1<=d['iterations']<=100
  for key,limit in [('max_abs_scaled_score',1e-10),('max_abs_scaled_newton_step',1e-12),('max_relative_quadrature_difference',1e-9)]:assert 0<=d[key]<=limit
  cov=d['covariance_matrix'];assert len(cov)==3 and all(len(r)==3 for r in cov)
  for i,c in enumerate(coef+[d['ln_variance']]):
    assert cov[i][i]>0;near(c['standard_error'],math.sqrt(cov[i][i]));near(c['statistic'],c['value']/c['standard_error']);near(c['p_value'],math.erfc(abs(c['statistic'])/math.sqrt(2)),'p_normal')
    near(c['confidence_lower'],c['value']-Z95*c['standard_error']);near(c['confidence_upper'],c['value']+Z95*c['standard_error'])
    for j in range(3):near(cov[i][j],cov[j][i])
  t=d['ln_variance'];assert -8<t['value']<8
  near(d['sigma'],math.exp(t['value']/2));near(d['sigma_standard_error'],d['sigma']*t['standard_error']/2)
  near(d['sigma_confidence_lower'],math.exp(t['confidence_lower']/2));near(d['sigma_confidence_upper'],math.exp(t['confidence_upper']/2))
  near(d['variance_likelihood_ratio'],2*(d['log_likelihood']-d['poisson_log_likelihood']));near(d['variance_likelihood_ratio_p_value'],.5*math.erfc(math.sqrt(d['variance_likelihood_ratio']/2)),'p_lr_mixture')
  near(d['wald_statistic'],coef[1]['statistic']**2);assert d['wald_df']==1;near(d['wald_p_value'],coef[1]['p_value'],'p_wald')
  exported=rows(native/'native_normal_re_predictions.csv');predicted=rows(reference/'predictions.csv');assert len(exported)==len(predicted)==n
  totals={i:sum(float(v['pat']) for k,v in expected.items() if k[0]==i) for i in range(1,g+1)}
  for row,independent,key in zip(exported,predicted,sorted(expected)):
    assert list(row)==SOURCE+EXTRA;assert (int(row['id']),int(row['year']))==key;source_equal(row,expected[key]);assert row['included_in_native_re']=='true';near(row['selected_firm_total'],totals[key[0]])
    for field in ['id','year','logr','pat']:assert float(row[field])==float(independent[field])
    mu,xb,res=[float(row[f]) for f in ['native_re_mean','native_re_xb','native_re_residual']]
    assert -700<=xb+math.exp(t['value'])/2<=700
    near(mu,independent['marginal_mu']);near(res,independent['marginal_residual']);near(xb,coef[0]['value']+coef[1]['value']*float(row['logr']));near(mu,math.exp(xb+math.exp(t['value'])/2));near(res,float(row['pat'])-mu)
  assert d['zero_total_group_count']==sum(v==0 for v in totals.values());assert d['zero_count']==sum(float(r['pat'])==0 for r in exported)
  near(d['count_sum'],sum(float(r['pat']) for r in exported));near(d['maximum_group_count_sum'],max(totals.values()))
  summary=events[9]['data']['rows'];assert [s['variable'] for s in summary]==['native_re_mean','native_re_residual']
  for result,field in zip(summary,['native_re_mean','native_re_residual']):
    samples=[float(r[field]) for r in exported];assert result['count']==n
    for label,value in [('mean',statistics.fmean(samples)),('std_dev',statistics.stdev(samples)),('minimum',min(samples)),('maximum',max(samples))]:near(result[label],value)
  assert events[10]['data']['headers']==['scope'] and len(events[10]['data']['rows'])==1
  assert events[-1]['data']['path']=='native_normal_re_predictions.csv' and events[-1]['data']['dataset']['row_count']==n
  score=independent_identity(d,exported)
  values={'candidate_rows':n,'firms':g,'zero_total_firms':d['zero_total_group_count'],'sigma':d['sigma'],'se_sigma':d['sigma_standard_error'],'lower_sigma':d['sigma_confidence_lower'],'upper_sigma':d['sigma_confidence_upper'],
    'log_likelihood':d['log_likelihood'],'poisson_log_likelihood':d['poisson_log_likelihood'],'lr_variance_zero':d['variance_likelihood_ratio'],'p_lr_variance_zero':d['variance_likelihood_ratio_p_value'],
    'mean_mu':summary[0]['mean'],'min_mu':summary[0]['minimum'],'max_mu':summary[0]['maximum'],'mean_residual':summary[1]['mean']}
  for name,c,s in zip(['intercept','slope','lnvariance'],coef+[t],score):
    for dest,field in [('estimate','value'),('se','standard_error'),('z','statistic'),('p','p_value'),('lower','confidence_lower'),('upper','confidence_upper')]:values[f'{dest}_{name}']=c[field]
    values['score_'+name]=s
  for name,i,j in [('aa',0,0),('ab',0,1),('at',0,2),('bb',1,1),('bt',1,2),('tt',2,2)]:values['covariance_'+name]=cov[i][j]
  oracle={r['field']:float(r['value']) for r in rows(reference/'reference.csv')};assert len(oracle)==len(values)==42 and values.keys()==oracle.keys()
  for key,value in oracle.items():near(values[key],value,key)
  return n,g,len(values)


if __name__=='__main__':
  assert len(sys.argv) in (4,5)
  original=len(sys.argv)==4
  if not original:assert sys.argv[4]=='--control'
  n,g,k=audit(*map(Path,sys.argv[1:4]),original=original)
  print(f'PASS: {n} exact source/prediction rows/{g} firms/{k} independent R fields/full covariance; raw direct-integration scores/information/null identities')
