#!/usr/bin/env python3
"""Full joint Weibull/source/point audit; independent q-Hessian and scalar null."""
import json
import math
from pathlib import Path
import sys
from check_reference import rows, close, inverse, multiply


def audit(input_csv,native_json,native_predictions,reference_directory):
  ref=Path(reference_directory)
  raw,pred,expected,hexes=rows(input_csv),rows(native_predictions),rows(ref/'predictions.csv'),rows(ref/'source_hex.csv')
  assert len(raw)==len(pred)==len(expected)==len(hexes)>42
  assert list(raw[0])==list(hexes[0])
  assert set(pred[0])==set(expected[0])|{'rr_ui','dr_ui','original_ui'}
  for source,actual,hexcells in zip(raw,pred,hexes):
    for key,value in hexcells.items():
      assert float(source[key])==float(actual[key])==float.fromhex(value),(key,source[key],actual[key],value)
  events=[json.loads(line)for line in Path(native_json).read_text().splitlines()]
  assert all(e.get('error')is None for e in events)
  fits=[e['data']for e in events if e['result_type']=='WeibullResult'];assert len(fits)==1
  fit=fits[0];names=(ref/'parameter_names.txt').read_text().splitlines();k=len(names)-1
  assert k==41 and names[-1]=='lnshape' and names[0]=='intercept'
  assert fit['covariance_parameter_names']==names and fit['predictors']==names[1:-1]
  assert [c['name']for c in fit['coefficients']]==names[:-1]
  assert fit['estimator']=='streg_weibull' and fit['robust'] and fit['include_intercept']
  assert fit['time_variable']=='spell' and fit['failure_variable']=='censor1'
  assert len(fit['covariance_matrix'])==k+1 and all(len(row)==k+1 for row in fit['covariance_matrix'])
  assert fit['null_model']=='intercept_and_shape_refitted_same_sample'
  assert fit['covariance']=='robust_full_joint_observed_information_score_sandwich'
  assert fit['reference_distribution']=='asymptotic_standard_normal'
  assert fit['likelihood_ratio_p_value']is None and fit['likelihood_ratio_reference_distribution']is None
  assert fit['confidence_level']==95 and fit['converged']
  sample=fit['sample'];n=len(raw)
  assert sample['total_observations']==n and sample['dropped_observations']==0
  assert sample['retained_indices']==list(range(n))and sample['weights']is None and sample['cluster_groups']is None
  gates=rows(ref/'robust.csv');assert len(gates)==2031
  for gate in gates:
    value=fit
    for part in gate['path'].strip('/').split('/'):
      value=value[int(part)]if isinstance(value,list)else value[part]
    close(float(value),float(gate['value']),gate['path'].endswith('p_value'))
  for actual,expected_row in zip(pred,expected):
    for name,value in expected_row.items():
      positive=('hazard'in name or 'survival'in name)and float(value)>0
      close(float(actual[name]),float(value),positive)
  beta=[c['value']for c in fit['coefficients']];a=fit['shape']['value'];assert a>0
  X=[];H=[];res=[];lnt=[];D=[]
  for source in pred:
    r={key:float(value)for key,value in source.items()}
    assert r['ui']==r['original_ui'] and r['rr_ui']==r['reprate']*r['ui'] and r['dr_ui']==r['disrate']*r['ui']
    x=[1.0]+[r[name]for name in names[1:-1]];logt=math.log(r['spell'])
    for prefix,ui in [('',r['ui']),('ui0_',0),('ui1_',1)]:
      candidate=dict(r,ui=ui,rr_ui=r['reprate']*ui,dr_ui=r['disrate']*ui)
      eta=beta[0]+math.fsum(candidate[name]*b for name,b in zip(names[1:-1],beta[1:]))
      cumulative=math.exp(eta+a*logt);hazard=math.exp(math.log(a)+eta+(a-1)*logt)
      for field,value in [('xb',eta),('hazard',hazard),('cumulative_hazard',cumulative),('net_survival',math.exp(-cumulative))]:
        close(r[prefix+field],value,field!='xb'and value>0)
    X.append(x);H.append(r['cumulative_hazard']);res.append(r['censor1']-H[-1]);lnt.append(logt);D.append(r['censor1'])
  # Normalize coordinates for numerical inversion, not for a different model.
  scales=[math.sqrt(math.fsum(row[j]**2 for row in X))for j in range(k)]
  scales.append(math.sqrt(math.fsum((a*t)**2 for t in lnt)+sum(D)))
  Z=[[value/scale for value,scale in zip(row+[a*t],scales)]for row,t in zip(X,lnt)]
  scores=[[z*r for z in row[:-1]]+[(d+r*a*t)/scales[-1]]for row,r,d,t in zip(Z,res,D,lnt)]
  assert max(abs(math.fsum(row[j]for row in scores))for j in range(k+1))<1e-9,'independent joint score failed'
  info=[[math.fsum(h*row[i]*row[j]for row,h in zip(Z,H))for j in range(k+1)]for i in range(k+1)]
  # Actual logshape curvature: H*(a log t)^2 - (d-H)*a log t.
  info[k][k]-=math.fsum(r*a*t for r,t in zip(res,lnt))/scales[-1]**2
  bread=inverse(info);identity=multiply(info,bread)
  assert max(abs(v-float(i==j))for i,row in enumerate(identity)for j,v in enumerate(row))<1e-8
  meat=[[math.fsum(row[i]*row[j]for row in scores)for j in range(k+1)]for i in range(k+1)]
  v=multiply(multiply(bread,meat),bread)
  for i in range(k+1):
    for j in range(k+1):close(fit['covariance_matrix'][i][j],v[i][j]*n/(n-1)/scales[i]/scales[j])
  close(fit['covariance_correction'],n/(n-1))
  for i,c in enumerate(fit['coefficients']):
    se=math.sqrt(fit['covariance_matrix'][i][i]);z=beta[i]/se
    for field,value in [('standard_error',se),('statistic',z),('confidence_lower',beta[i]-1.959963984540054*se),('confidence_upper',beta[i]+1.959963984540054*se)]:close(c[field],value)
    close(c['p_value'],math.erfc(abs(z)/math.sqrt(2)),True)
  sq=math.sqrt(fit['covariance_matrix'][k][k]);q=math.log(a)
  for field,value in [('log_value',q),('log_standard_error',sq),('standard_error',a*sq),('log_statistic',q/sq),('confidence_lower',math.exp(q-1.959963984540054*sq)),('confidence_upper',math.exp(q+1.959963984540054*sq))]:close(fit['shape'][field],value)
  close(fit['shape']['log_p_value'],math.erfc(abs(q/sq)/math.sqrt(2)),True)
  restricted=inverse([row[1:k]for row in fit['covariance_matrix'][1:k]])
  wald=math.fsum(beta[i+1]*restricted[i][j]*beta[j+1]for i in range(40)for j in range(40))
  close(fit['omnibus_wald']['statistic'],wald);assert fit['omnibus_wald']['degrees_of_freedom']==40
  y=wald/2;terms=[j*math.log(y)-math.lgamma(j+1)for j in range(20)];maximum=max(terms)
  close(fit['omnibus_wald']['p_value'],math.exp(-y+maximum+math.log(math.fsum(math.exp(t-maximum)for t in terms))),True)
  constant=math.fsum(d*t for d,t in zip(D,lnt))
  density=math.fsum(d*(math.log(a)+math.fsum(v*b for v,b in zip(row,beta))+(a-1)*t)-h for row,d,t,h in zip(X,D,lnt,H))
  # Independent scalar PROFILE null; conditional baseline exp(beta0)=D/sum t^a.
  def null(shape):
    values=[shape*t for t in lnt];maximum=max(values);weights=[math.exp(v-maximum)for v in values]
    logsum=maximum+math.log(math.fsum(weights));weighted=math.fsum(w*t for w,t in zip(weights,lnt))/math.fsum(weights)
    score=sum(D)/shape+constant-sum(D)*weighted
    ll=sum(D)*(math.log(shape)+math.log(sum(D))-logsum)+shape*constant-sum(D)-constant
    return score,ll
  lo,hi=1e-12,1.0
  while null(hi)[0]>0:hi*=2
  for _ in range(100):
    middle=(lo+hi)/2
    if null(middle)[0]>0:lo=middle
    else:hi=middle
  null_density=null((lo+hi)/2)[1]
  close(fit['log_likelihood'],density);close(fit['normalized_log_likelihood'],density+constant)
  close(fit['null_log_likelihood'],null_density);close(fit['null_normalized_log_likelihood'],null_density+constant)
  close(fit['likelihood_ratio'],2*(density-null_density));assert fit['likelihood_ratio_df']==40
  for ui in (0,1):assert abs(math.fsum(r for source,r in zip(pred,res)if float(source['ui'])==ui))<1e-7
  return len(gates),len(pred)*len(pred[0])


if __name__=='__main__':
  assert len(sys.argv)==5,'usage: check_weibull.py duration.csv native.json weibull_predictions.csv reference_directory'
  gates,cells=audit(*sys.argv[1:]);print(f'Validated {gates} R gates/all1764 joint covariance entries, {cells} source/design/projection cells, exact source hex, independent joint score/Hessian/sandwich and scalar profile null')
