#!/usr/bin/env python3
"""Actual native GEE controls and independent-equation corruption defenses."""
import csv
import json
import math
from pathlib import Path
import shutil
import statistics
import subprocess
import tempfile
import unittest
from check_native_poisson_gee import audit, rows, EXPORT, Z95

ROOT=Path(__file__).resolve().parents[2]
SOURCE=ROOT/'tests/fixtures/mma23_conditional_fe/source.asc'
REFERENCE=ROOT/'tests/fixtures/mma23_exchangeable_gee/positive'
NEGATIVE=ROOT/'tests/fixtures/mma23_exchangeable_gee/negative'
SCRIPT=ROOT/'examples/mma23p1pannonlin/native_poisson_gee.td'
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


def rescale_inference(data,method,lookup,factor):
  inf=data[method];name='working' if method=='working' else 'cluster'
  for i in range(2):
    for j in range(2): inf['covariance'][i][j]*=factor
  for suffix,i,j in [('aa',0,0),('ab',0,1),('bb',1,1)]: lookup[name+'_'+suffix]['value']=str(inf['covariance'][i][j])
  for label,c in zip(['intercept','slope'],inf['coefficients']):
    c['standard_error']=math.sqrt(inf['covariance'][0 if label=='intercept' else 1][0 if label=='intercept' else 1]);c['statistic']=c['value']/c['standard_error']
    c['p_value']=math.erfc(abs(c['statistic'])/math.sqrt(2));c['confidence_lower']=c['value']-Z95*c['standard_error'];c['confidence_upper']=c['value']+Z95*c['standard_error']
    for dest,field in [('se','standard_error'),('z','statistic'),('lo','confidence_lower'),('hi','confidence_upper')]: lookup[f'{name}_{dest}_{label}']['value']=str(c[field])
  inf['wald_statistic']=inf['coefficients'][1]['statistic']**2;inf['wald_p_value']=inf['coefficients'][1]['p_value']


class NativePoissonGeeAudit(unittest.TestCase):
  def test_actual_positive_negative_permutation_and_missing_metadata(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp)
      for case,raw,reference in [('positive',SOURCE,REFERENCE),('negative',NEGATIVE/'source.asc',NEGATIVE)]:
        native=root/case;prepare(native,raw);run(native);self.assertEqual(audit(native,reference,raw,False),(60,12,27))
      native=root/'permuted';prepare(native);write(native/'patents.csv',list(reversed(rows(native/'patents.csv'))));run(native)
      self.assertEqual(audit(native,REFERENCE,SOURCE,False),(60,12,27));self.assertEqual((native/EXPORT).read_bytes(),(root/'positive'/EXPORT).read_bytes())
      tokens=SOURCE.read_text().split()
      for i in range(1,len(tokens),25): tokens[i]='.'
      raw=root/'missing.asc';raw.write_text(' '.join(tokens)+'\n');native=root/'missing';prepare(native,raw);run(native)
      self.assertEqual(audit(native,REFERENCE,raw,False),(60,12,27))

  def test_thirty_three_forgery_cases(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp);baseline=root/'baseline';prepare(baseline);run(baseline);self.assertEqual(audit(baseline,REFERENCE,SOURCE,False),(60,12,27))
      for corruption in range(33):
        with self.subTest(corruption=corruption):
          native=root/f'native-{corruption}';reference=root/f'reference-{corruption}';shutil.copytree(baseline,native);shutil.copytree(REFERENCE,reference)
          events=[json.loads(s) for s in (native/'result.jsonl').read_text().splitlines()];d=events[5]['data'];exported=rows(native/EXPORT)
          oracle=rows(reference/'reference.csv');lookup={r['field']:r for r in oracle};predictions=rows(reference/'predictions.csv')
          if corruption==0:
            source=rows(native/'patents.csv');source[0]['pat']='999';write(native/'patents.csv',source)
          elif corruption==1: exported.pop()
          elif corruption==2: exported[0],exported[1]=exported[1],exported[0]
          elif corruption==3: exported[0]['ardssic']='999999'
          elif corruption==4: d['working']['coefficients'][0]['value']+=.1
          elif corruption==5: d['working']['covariance'][0][1]+=.01
          elif corruption==6: d['group_count']=11
          elif corruption==7: d['zero_total_group_count']=0
          elif corruption==8: exported[0]['native_gee_mean']=str(float(exported[0]['native_gee_mean'])+1)
          elif corruption==9: exported[0]['native_gee_xb']=str(float(exported[0]['native_gee_xb'])+1)
          elif corruption==10: events[9]['data']['rows'][0]['mean']+=1
          elif corruption==11:
            for row,ref in zip(exported,predictions):
              mu=float(row['native_gee_mean'])*1.1;row['native_gee_mean']=ref['gee_mu']=str(mu);row['native_gee_residual']=ref['gee_residual']=str(float(row['pat'])-mu);row['native_gee_xb']=str(math.log(mu))
            for summary,field in zip(events[9]['data']['rows'],['native_gee_mean','native_gee_residual']):
              values=[float(r[field]) for r in exported]
              summary.update(mean=statistics.fmean(values),std_dev=statistics.stdev(values),minimum=min(values),maximum=max(values))
          elif corruption==12:
            d['working']['coefficients'][1]['statistic']+=1;lookup['working_z_slope']['value']=str(d['working']['coefficients'][1]['statistic'])
          elif corruption==13: rescale_inference(d,'working',lookup,1.25)
          elif corruption==14: rescale_inference(d,'sandwich',lookup,1.25)
          elif corruption==15: exported[0]['included_in_native_gee']='false'
          elif corruption==16: exported[0]['selected_firm_total']='999'
          elif corruption==17: d['sample']['dropped_observations']=1
          elif corruption==18: d['estimator']='pooled_poisson'
          elif corruption==19: d['working_scale']=d['pearson_normalizer']
          elif corruption==20: d['correlation']='modern_adjusted_pearson'
          elif corruption==21: events[5]['schema_version']=2
          elif corruption==22: events[5]['result_type']='PoissonResult'
          elif corruption==23: d['max_abs_scaled_newton_step']=.1
          elif corruption==24: d['sandwich_correction']=1.0
          elif corruption==25:
            d['rho']+=.01;lookup['rho']['value']=str(d['rho'])
          elif corruption==26: d['pearson_normalizer']+=1
          elif corruption==27: d['sandwich']['coefficients'][1]['p_value']*=2
          elif corruption==28:
            scores=rows(reference/'scores.csv');scores[0]['score_a']=str(float(scores[0]['score_a'])+1);write(reference/'scores.csv',scores)
          elif corruption==29: d['sample']['total_observations']-=1
          elif corruption==30: d['log_likelihood']=0.0
          elif corruption==31: exported[-1]=dict(exported[0])
          elif corruption==32: d['prediction_scope']='posterior_group_effects'
          write(native/EXPORT,exported);write(reference/'reference.csv',oracle);write(reference/'predictions.csv',predictions)
          (native/'result.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
          with self.assertRaises((AssertionError,ValueError,KeyError,IndexError)):
            audit(native,reference,SOURCE,False)


if __name__=='__main__': unittest.main()
