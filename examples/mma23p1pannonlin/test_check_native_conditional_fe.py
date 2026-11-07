#!/usr/bin/env python3
"""Actual native FE controls and raw-equation corruption defenses; no R required."""
import csv
import json
import math
from pathlib import Path
import shutil
import statistics
import subprocess
import tempfile
import unittest
from check_native_conditional_fe import audit, independent_equations, rows, EXPORT, Z95

ROOT=Path(__file__).resolve().parents[2]
REFERENCE=ROOT/'tests/fixtures/mma23_conditional_fe'
SOURCE=REFERENCE/'source.asc'
SCRIPT=ROOT/'examples/mma23p1pannonlin/native_conditional_fe.td'
BIN=ROOT/'target/debug/tabdat-explore-rs'


def write(path,records):
  with path.open('w',newline='') as stream:
    writer=csv.DictWriter(stream,fieldnames=list(records[0]));writer.writeheader();writer.writerows(records)


def prepare(native,source=SOURCE):
  native.mkdir();subprocess.run(['sh',str(ROOT/'examples/mma23p1pannonlin/prepare.sh'),str(source),str(native/'patents.csv')],check=True,capture_output=True)


def run(native):
  out=subprocess.run([str(BIN),'--json','-f',str(SCRIPT)],cwd=native,capture_output=True,text=True)
  assert out.returncode==0,(out.stdout,out.stderr)
  (native/'result.jsonl').write_text(out.stdout)


def summary(events,exported):
  for result,field in zip(events[9]['data']['rows'],['native_fe_mean','native_fe_residual']):
    values=[float(r[field]) for r in exported if r[field]!='']
    result.update(mean=statistics.fmean(values),std_dev=statistics.stdev(values),minimum=min(values),maximum=max(values))


def rescale_inference(data,lookup,factor):
  data['covariance_matrix'][0][0]*=factor;c=data['coefficients'][0]
  c['standard_error']=math.sqrt(data['covariance_matrix'][0][0]);c['statistic']=c['value']/c['standard_error']
  c['p_value']=math.erfc(abs(c['statistic'])/math.sqrt(2));c['confidence_lower']=c['value']-Z95*c['standard_error'];c['confidence_upper']=c['value']+Z95*c['standard_error']
  for field,name in [('se','standard_error'),('z','statistic'),('p_value','p_value'),('ci_lower','confidence_lower'),('ci_upper','confidence_upper')]: lookup[field]['value']=str(c[name])
  lookup['information']['value']=str(1/data['covariance_matrix'][0][0]);data['wald_statistic']=c['statistic']**2;data['wald_p_value']=c['p_value']


class NativeConditionalFeAudit(unittest.TestCase):
  def test_equations_keep_finite_log_likelihood_when_probability_underflows(self):
    # Constructed identity, not a native/R fit: one degenerate probability block
    # has zero observed counts at underflowed probabilities; the other has info.
    beta=1000.;a=math.log(2)/beta;exported=[]
    for i in range(5):
      exported.append({'id':'1','logr':str(float(i)),'pat':str(10 if i==4 else 0),'native_fe_mean':str(10 if i==4 else 0)})
      exported.append({'id':'2','logr':str(i*a),'pat':str(2**i),'native_fe_mean':str(2**i)})
    constant=math.lgamma(32)-sum(math.lgamma(2**i+1) for i in range(5))
    ll=constant+sum(2**i*math.log(2**i/31) for i in range(5));null=constant-41*math.log(5)
    center=sum(2**i*i*a/31 for i in range(5));info=sum(2**i*(i*a-center)**2 for i in range(5));lr=2*(ll-null)
    data={'coefficients':[{'value':beta}],'covariance_matrix':[[1/info]],'conditional_log_likelihood':ll,
      'null_conditional_log_likelihood':null,'likelihood_ratio':lr,'likelihood_ratio_p_value':math.erfc(math.sqrt(lr/2))}
    independent_equations(data,exported)

  def test_actual_source_reflection_permutation_and_missing_metadata(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp);native=root/'baseline';prepare(native);run(native);self.assertEqual(audit(native,REFERENCE,SOURCE,False),(60,50,10,14))
      native=root/'permuted';prepare(native);write(native/'patents.csv',list(reversed(rows(native/'patents.csv'))));run(native)
      self.assertEqual(audit(native,REFERENCE,SOURCE,False),(60,50,10,14));self.assertEqual((native/EXPORT).read_bytes(),(root/'baseline'/EXPORT).read_bytes())
      tokens=SOURCE.read_text().split()
      for i in range(1,len(tokens),25): tokens[i]='.'
      raw=root/'missing.asc';raw.write_text(' '.join(tokens)+'\n');native=root/'missing';prepare(native,raw);run(native)
      self.assertEqual(audit(native,REFERENCE,raw,False),(60,50,10,14))
      # Exact sign-reflection identity of the frozen R model, not a new oracle.
      tokens=SOURCE.read_text().split()
      for base in range(0,len(tokens),25):
        for i in range(10,15): tokens[base+i]=str(-float(tokens[base+i]))
      raw=root/'reflection.asc';raw.write_text(' '.join(tokens)+'\n');reference=root/'reflection-reference';reference.mkdir()
      oracle=rows(REFERENCE/'reference.csv');lookup={r['field']:r for r in oracle};lower=lookup['ci_lower']['value'];upper=lookup['ci_upper']['value']
      for field in ['beta','z']: lookup[field]['value']=str(-float(lookup[field]['value']))
      lookup['ci_lower']['value']=str(-float(upper));lookup['ci_upper']['value']=str(-float(lower));write(reference/'reference.csv',oracle)
      predictions=rows(REFERENCE/'predictions.csv')
      for row in predictions: row['logr']=str(-float(row['logr']))
      write(reference/'predictions.csv',predictions);native=root/'reflection';prepare(native,raw);run(native)
      self.assertEqual(audit(native,reference,raw,False),(60,50,10,14))

  def test_thirty_one_forgery_cases(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp);baseline=root/'baseline';prepare(baseline);run(baseline);self.assertEqual(audit(baseline,REFERENCE,SOURCE,False),(60,50,10,14))
      for corruption in range(31):
        with self.subTest(corruption=corruption):
          native=root/f'native-{corruption}';reference=root/f'reference-{corruption}';shutil.copytree(baseline,native);shutil.copytree(REFERENCE,reference)
          events=[json.loads(s) for s in (native/'result.jsonl').read_text().splitlines()];d=events[5]['data'];exported=rows(native/EXPORT)
          oracle=rows(reference/'reference.csv');lookup={r['field']:r for r in oracle};predictions=rows(reference/'predictions.csv')
          positive=next(i for i,r in enumerate(exported) if r['included_in_native_fe']=='true');zero=next(i for i,r in enumerate(exported) if r['included_in_native_fe']=='false')
          if corruption==0:
            source=rows(native/'patents.csv');source[0]['pat']='999';write(native/'patents.csv',source)
          elif corruption==1: exported.pop()
          elif corruption==2: exported[0],exported[1]=exported[1],exported[0]
          elif corruption==3: exported[0]['ardssic']='999999'
          elif corruption==4: d['coefficients'][0]['value']+=.1
          elif corruption==5: d['covariance_matrix'][0][0]*=1.1
          elif corruption==6: d['group_count']-=1
          elif corruption==7: d['zero_total_group_count']=0
          elif corruption==8: exported[positive]['native_fe_mean']=str(float(exported[positive]['native_fe_mean'])+1)
          elif corruption==9: exported[zero]['native_fe_xb']='0'
          elif corruption==10: events[9]['data']['rows'][0]['mean']+=1
          elif corruption==11:
            for row,ref in zip(exported,predictions):
              if row['included_in_native_fe']=='true':
                mu=float(row['native_fe_mean'])*1.1;row['native_fe_mean']=ref['fe_mean']=str(mu);row['native_fe_residual']=ref['fe_residual']=str(float(row['pat'])-mu)
            summary(events,exported)
          elif corruption==12:
            # Preserve firm totals but forge the within-firm allocation in both outputs.
            for i,delta in [(positive,.01),(positive+1,-.01)]:
              mu=float(exported[i]['native_fe_mean'])+delta;exported[i]['native_fe_mean']=predictions[i]['fe_mean']=str(mu)
              exported[i]['native_fe_residual']=predictions[i]['fe_residual']=str(float(exported[i]['pat'])-mu)
            summary(events,exported)
          elif corruption==13: rescale_inference(d,lookup,1.25)
          elif corruption==14:
            d['conditional_log_likelihood']+=1;lookup['conditional_ll']['value']=str(d['conditional_log_likelihood']);d['likelihood_ratio']+=2;d['likelihood_ratio_p_value']=math.erfc(math.sqrt(d['likelihood_ratio']/2))
          elif corruption==15: exported[zero]['included_in_native_fe']='true'
          elif corruption==16: exported[0]['selected_firm_total']='999'
          elif corruption==17: d['sample']['dropped_observations']=0
          elif corruption==18: d['estimator']='pooled_poisson'
          elif corruption==19: d['covariance']='firm_cluster_robust'
          elif corruption==20: d['likelihood']='unconditional_poisson'
          elif corruption==21: events[5]['schema_version']=2
          elif corruption==22: events[5]['result_type']='PoissonResult'
          elif corruption==23: d['max_abs_scaled_newton_step']=.1
          elif corruption==24: d['covariance_correction']=1.2
          elif corruption==25: exported[zero]['native_fe_mean']=predictions[zero]['fe_mean']='0'
          elif corruption==26: d['null_conditional_log_likelihood']+=1
          elif corruption==27:
            d['coefficients'][0]['p_value']*=.5;d['wald_p_value']=d['coefficients'][0]['p_value'];lookup['p_value']['value']=str(d['wald_p_value'])
          elif corruption==28: d['likelihood_ratio_p_value']*=.5
          elif corruption==29: d['sample']['total_observations']-=1
          elif corruption==30: d['prediction_scope']='current_subset_re_normalization'
          write(native/EXPORT,exported);write(reference/'reference.csv',oracle);write(reference/'predictions.csv',predictions)
          (native/'result.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
          with self.assertRaises((AssertionError,ValueError,KeyError,IndexError)): audit(native,reference,SOURCE,False)


if __name__=='__main__': unittest.main()
