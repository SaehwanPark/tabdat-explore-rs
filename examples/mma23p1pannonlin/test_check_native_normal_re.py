#!/usr/bin/env python3
"""Native controls and hostile artifacts, including coherent native/R errors."""
import json
import math
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import check_native_normal_re as checker
from check_native_normal_re import audit, posterior, rows, Z95
from test_check_native_gamma_re import write

ROOT=Path(__file__).resolve().parents[2]
SOURCE=ROOT/'tests/fixtures/mma23_gamma_re_bootstrap/source.asc'
REFERENCE=ROOT/'tests/fixtures/mma23_normal_re'
SCRIPT=ROOT/'examples/mma23p1pannonlin/native_normal_re.td'
BIN=ROOT/'target/debug/tabdat-explore-rs'


def prepare(native,source=SOURCE):
  native.mkdir();subprocess.run(['sh',str(ROOT/'examples/mma23p1pannonlin/prepare.sh'),str(source),str(native/'patents.csv')],check=True,capture_output=True)


def run(native):
  out=subprocess.run([str(BIN),'--json','-f',str(SCRIPT)],cwd=native,capture_output=True,text=True)
  assert out.returncode==0,out.stderr
  (native/'result.jsonl').write_text(out.stdout)


class NativeNormalAudit(unittest.TestCase):
  def test_independent_simpson_against_six_direct_r_posterior_controls(self):
    fixture=ROOT/'crates/tabdat-stats/tests/fixtures/normal_poisson/posterior.csv'
    for row in rows(fixture):
      actual=posterior(float(row['total']),float(row['mean_sum']),math.exp(float(row['log_variance'])))
      for value,key in zip(actual,['log_integral','mean_exp_u','mean_u2','var_exp_u','var_u2','cov_exp_u_u2']):checker.near(value,float(row[key]),key)

  def test_real_control_permutation_and_wholly_missing_nonmodel_metadata(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp);baseline=root/'baseline';prepare(baseline);run(baseline)
      self.assertEqual(audit(baseline,REFERENCE,SOURCE,False),(200,40,42))
      permuted=root/'permuted';prepare(permuted);write(permuted/'patents.csv',list(reversed(rows(permuted/'patents.csv'))));run(permuted)
      self.assertEqual(audit(permuted,REFERENCE,SOURCE,False),(200,40,42))
      self.assertEqual((baseline/'native_normal_re_predictions.csv').read_bytes(),(permuted/'native_normal_re_predictions.csv').read_bytes())
      tokens=SOURCE.read_text().split()
      for i in range(1,len(tokens),25):tokens[i]='.'
      raw=root/'missing.asc';raw.write_text(' '.join(tokens)+'\n');native=root/'missing';prepare(native,raw);run(native)
      self.assertEqual(audit(native,REFERENCE,raw,False),(200,40,42))

  def test_thirty_one_forgeries_and_mocked_guard_demonstrations(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp);baseline=root/'baseline';prepare(baseline);run(baseline)
      self.assertEqual(audit(baseline,REFERENCE,SOURCE,False),(200,40,42))
      for corruption in range(31):
        with self.subTest(corruption=corruption):
          native=root/f'native-{corruption}';reference=root/f'reference-{corruption}';shutil.copytree(baseline,native);shutil.copytree(REFERENCE,reference)
          events=[json.loads(s) for s in (native/'result.jsonl').read_text().splitlines()];d=events[5]['data'];exported=rows(native/'native_normal_re_predictions.csv');oracle=rows(reference/'reference.csv');lookup={r['field']:r for r in oracle}
          if corruption==0:
            source=rows(native/'patents.csv');source[0]['pat']='999';write(native/'patents.csv',source)
          elif corruption==1:exported.pop()
          elif corruption==2:exported[0],exported[1]=exported[1],exported[0]
          elif corruption==3:exported[0]['ardssic']='999999'
          elif corruption==4:d['coefficients'][0]['value']+=.1
          elif corruption==5:d['covariance_matrix'][0][1]+=.01
          elif corruption==6:d['group_count']=39
          elif corruption==7:d['zero_total_group_count']=0
          elif corruption==8:exported[0]['native_re_mean']=str(float(exported[0]['native_re_mean'])+1)
          elif corruption==9:exported[0]['native_re_xb']=str(float(exported[0]['native_re_xb'])+1)
          elif corruption==10:events[9]['data']['rows'][0]['mean']+=1
          elif corruption==11:
            events[9]['data']['rows'][0]['mean']+=1;lookup['mean_mu']['value']=str(events[9]['data']['rows'][0]['mean'])
          elif corruption==12:
            d['coefficients'][1]['statistic']+=1;lookup['z_slope']['value']=str(d['coefficients'][1]['statistic'])
          elif corruption==13:
            # Coherent wrong native/R intercept inference passes agreement and
            # all elementary SE/z/p/CI identities, but not raw information.
            d['covariance_matrix'][0][0]*=1.25;lookup['covariance_aa']['value']=str(d['covariance_matrix'][0][0])
            c=d['coefficients'][0];c['standard_error']*=math.sqrt(1.25);c['statistic']=c['value']/c['standard_error'];c['p_value']=math.erfc(abs(c['statistic'])/math.sqrt(2))
            c['confidence_lower']=c['value']-Z95*c['standard_error'];c['confidence_upper']=c['value']+Z95*c['standard_error']
            for prefix,field in [('se','standard_error'),('z','statistic'),('p','p_value'),('lower','confidence_lower'),('upper','confidence_upper')]:lookup[prefix+'_intercept']['value']=str(c[field])
          elif corruption==14:
            exported[0]['native_re_mean']=str(float(exported[0]['native_re_mean'])*1.1);exported[0]['native_re_residual']=str(float(exported[0]['pat'])-float(exported[0]['native_re_mean']))
          elif corruption==15:exported[0]['included_in_native_re']='false'
          elif corruption==16:exported[0]['selected_firm_total']='999'
          elif corruption==17:d['sample']['dropped_observations']=1
          elif corruption==18:d['estimator']='pooled_poisson'
          elif corruption==19:d['covariance']='inverse_mean_block_only'
          elif corruption==20:d['prediction_scope']='posterior_group_effects'
          elif corruption==21:events[5]['schema_version']=2
          elif corruption==22:events[5]['result_type']='GammaPoissonResult'
          elif corruption==23:d['max_abs_scaled_score']=.1
          elif corruption==24:d['ln_variance_test_null']='variance=0'
          elif corruption==25:d['variance_likelihood_ratio_reference']='chi_square_1'
          elif corruption==26:
            self.assertGreater(d['variance_likelihood_ratio_p_value'],0);d['variance_likelihood_ratio_p_value']*=2;lookup['p_lr_variance_zero']['value']=str(d['variance_likelihood_ratio_p_value'])
          elif corruption==27:d['max_relative_quadrature_difference']=1e-3
          elif corruption==28:
            d['sigma_standard_error']*=2;lookup['se_sigma']['value']=str(d['sigma_standard_error'])
          elif corruption==29:
            d['sigma_confidence_lower']*=.8;lookup['lower_sigma']['value']=str(d['sigma_confidence_lower'])
          elif corruption==30:
            # Coherent wrong null/LR/p passes with the raw null guard disabled.
            d['poisson_log_likelihood']-=1;d['variance_likelihood_ratio']+=2
            d['variance_likelihood_ratio_p_value']=.5*math.erfc(math.sqrt(d['variance_likelihood_ratio']/2))
            for dest,field in [('poisson_log_likelihood','poisson_log_likelihood'),('lr_variance_zero','variance_likelihood_ratio'),('p_lr_variance_zero','variance_likelihood_ratio_p_value')]:lookup[dest]['value']=str(d[field])
          write(native/'native_normal_re_predictions.csv',exported);write(reference/'reference.csv',oracle);(native/'result.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
          if corruption in (13,30):
            score=[float(lookup['score_'+name]['value']) for name in ['intercept','slope','lnvariance']]
            with patch.object(checker,'independent_identity',return_value=score):self.assertEqual(audit(native,reference,SOURCE,False),(200,40,42))
          with self.assertRaises((AssertionError,ValueError,KeyError,IndexError)):audit(native,reference,SOURCE,False)


if __name__=='__main__':unittest.main()
