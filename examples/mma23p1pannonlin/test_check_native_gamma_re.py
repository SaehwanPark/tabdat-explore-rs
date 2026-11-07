#!/usr/bin/env python3
"""Real native RE controls and targeted corruptions, including common native/R errors."""
import csv
import json
import math
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from check_native_gamma_re import audit, rows, Z95

ROOT=Path(__file__).resolve().parents[2]
SOURCE=ROOT/'tests/fixtures/mma23_conditional_fe/source.asc'
REFERENCE=ROOT/'tests/fixtures/mma23_gamma_re'
SCRIPT=ROOT/'examples/mma23p1pannonlin/native_gamma_re.td'
BIN=ROOT/'target/debug/tabdat-explore-rs'


def write(path,records):
  with path.open('w',newline='') as f:
    writer=csv.DictWriter(f,fieldnames=list(records[0]));writer.writeheader();writer.writerows(records)


def prepare(native,source=SOURCE):
  native.mkdir()
  subprocess.run(['sh',str(ROOT/'examples/mma23p1pannonlin/prepare.sh'),str(source),str(native/'patents.csv')],check=True,capture_output=True)


def run(native):
  out=subprocess.run([str(BIN),'--json','-f',str(SCRIPT)],cwd=native,capture_output=True,text=True)
  assert out.returncode==0,out.stderr
  (native/'result.jsonl').write_text(out.stdout)


class NativeGammaAudit(unittest.TestCase):
  def test_real_control_permutation_and_wholly_missing_nonmodel_metadata(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp);baseline=root/'baseline';prepare(baseline);run(baseline)
      self.assertEqual(audit(baseline,REFERENCE,SOURCE,False),(60,12,37))
      permuted=root/'permuted';prepare(permuted)
      write(permuted/'patents.csv',list(reversed(rows(permuted/'patents.csv'))));run(permuted)
      self.assertEqual(audit(permuted,REFERENCE,SOURCE,False),(60,12,37))
      self.assertEqual((baseline/'native_gamma_re_predictions.csv').read_bytes(),(permuted/'native_gamma_re_predictions.csv').read_bytes())
      tokens=SOURCE.read_text().split()
      for i in range(1,len(tokens),25): tokens[i]='.'
      raw=root/'missing.asc';raw.write_text(' '.join(tokens)+'\n')
      native=root/'missing';prepare(native,raw);run(native)
      self.assertEqual(audit(native,REFERENCE,raw,False),(60,12,37))

  def test_twenty_seven_forgeries(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp);baseline=root/'baseline';prepare(baseline);run(baseline)
      self.assertEqual(audit(baseline,REFERENCE,SOURCE,False),(60,12,37))
      for corruption in range(27):
        with self.subTest(corruption=corruption):
          native=root/f'native-{corruption}';reference=root/f'reference-{corruption}'
          shutil.copytree(baseline,native);shutil.copytree(REFERENCE,reference)
          events=[json.loads(s) for s in (native/'result.jsonl').read_text().splitlines()]
          data=events[5]['data'];exported=rows(native/'native_gamma_re_predictions.csv')
          oracle=rows(reference/'reference.csv');lookup={r['field']:r for r in oracle}
          if corruption==0:
            source=rows(native/'patents.csv');source[0]['pat']='999';write(native/'patents.csv',source)
          elif corruption==1: exported.pop()
          elif corruption==2: exported[0],exported[1]=exported[1],exported[0]
          elif corruption==3: exported[0]['ardssic']='999999'
          elif corruption==4: data['coefficients'][0]['value']+=.1
          elif corruption==5: data['covariance_matrix'][0][1]+=.01
          elif corruption==6: data['group_count']=11
          elif corruption==7: data['zero_total_group_count']=0
          elif corruption==8: exported[0]['native_re_mean']=str(float(exported[0]['native_re_mean'])+1)
          elif corruption==9: exported[0]['native_re_xb']=str(float(exported[0]['native_re_xb'])+1)
          elif corruption==10: events[9]['data']['rows'][0]['mean']+=1
          elif corruption==11:
            events[9]['data']['rows'][0]['mean']+=1
            lookup['mean_prediction']['value']=str(events[9]['data']['rows'][0]['mean'])
          elif corruption==12:
            data['coefficients'][1]['statistic']+=1
            lookup['z_slope']['value']=str(data['coefficients'][1]['statistic'])
          elif corruption==13:
            data['covariance_matrix'][0][0]*=1.25
            lookup['covariance_aa']['value']=str(data['covariance_matrix'][0][0])
            co=data['coefficients'][0];co['standard_error']*=math.sqrt(1.25)
            co['statistic']=co['value']/co['standard_error'];co['p_value']=math.erfc(abs(co['statistic'])/math.sqrt(2))
            co['confidence_lower']=co['value']-Z95*co['standard_error'];co['confidence_upper']=co['value']+Z95*co['standard_error']
            for prefix,field in [('se','standard_error'),('z','statistic'),('p','p_value'),('lower','confidence_lower'),('upper','confidence_upper')]: lookup[prefix+'_intercept']['value']=str(co[field])
          elif corruption==14:
            exported[0]['native_re_mean']=str(float(exported[0]['native_re_mean'])*1.1)
            exported[0]['native_re_residual']=str(float(exported[0]['pat'])-float(exported[0]['native_re_mean']))
          elif corruption==15: exported[0]['included_in_native_re']='false'
          elif corruption==16: exported[0]['selected_firm_total']='999'
          elif corruption==17: data['sample']['dropped_observations']=1
          elif corruption==18: data['estimator']='pooled_poisson'
          elif corruption==19: data['covariance']='inverse_mean_block_only'
          elif corruption==20: data['prediction_scope']='posterior_group_effects'
          elif corruption==21: events[5]['schema_version']=2
          elif corruption==22: events[5]['result_type']='PoissonResult'
          elif corruption==23: data['max_abs_scaled_score']=.1
          elif corruption==24: data['ln_alpha_test_null']='alpha=0'
          elif corruption==25: data['dispersion_likelihood_ratio_reference']='chi_square_1'
          elif corruption==26:
            self.assertGreater(data['dispersion_likelihood_ratio_p_value'],0)
            data['dispersion_likelihood_ratio_p_value']*=2
            lookup['p_lr_alpha_zero']['value']=str(data['dispersion_likelihood_ratio_p_value'])
          write(native/'native_gamma_re_predictions.csv',exported);write(reference/'reference.csv',oracle)
          (native/'result.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
          with self.assertRaises((AssertionError,ValueError,KeyError,IndexError)):
            audit(native,reference,SOURCE,False)


if __name__=='__main__': unittest.main()
