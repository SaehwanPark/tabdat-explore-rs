#!/usr/bin/env python3
"""Actual linear-panel reproduction and independent normalized-FE audit controls."""
import csv
import json
import math
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from check_linear_panel import audit, rows, student_two_tail

ROOT=Path(__file__).resolve().parents[2]
REFERENCE=ROOT/'tests/fixtures/mma23_linear_panel'
RAW=ROOT/'tests/fixtures/mma23_gamma_re_bootstrap/source.asc'
SCRIPT=ROOT/'examples/mma23p1pannonlin/linear_panel.td'
BIN=ROOT/'target/debug/tabdat-explore-rs'


def write(path,records):
  with path.open('w',newline='') as stream:
    writer=csv.DictWriter(stream,fieldnames=list(records[0]));writer.writeheader();writer.writerows(records)


def prepare(native):
  native.mkdir();subprocess.run(['sh',str(ROOT/'examples/mma23p1pannonlin/prepare.sh'),str(RAW),str(native/'patents.csv')],check=True,capture_output=True)


def run(native):
  out=subprocess.run([str(BIN),'--json','-f',str(SCRIPT)],cwd=native,capture_output=True,text=True)
  assert out.returncode==0,(out.stdout,out.stderr)
  (native/'result.jsonl').write_text(out.stdout)


def set_path(value,path,replacement):
  keys=path.strip('/').split('/')
  for key in keys[:-1]: value=value[int(key)] if isinstance(value,list) else value[key]
  if isinstance(value,list): value[int(keys[-1])]=replacement
  else: value[keys[-1]]=replacement


class LinearPanelAudit(unittest.TestCase):
  def test_actual_control_permutation_and_nonmodel_metadata(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp);base=root/'base';prepare(base);run(base)
      self.assertEqual(audit(base,REFERENCE,base/'result.jsonl'),(200,106,0))
      native=root/'permuted';prepare(native);write(native/'patents.csv',list(reversed(rows(native/'patents.csv'))));run(native)
      self.assertEqual(audit(native,REFERENCE,native/'result.jsonl'),(200,106,0))
      # audit checks canonical keys and fixed numerical gates; reversing QR
      # input can change final bits, so byte equality is not a promised invariant.
      native=root/'missing';prepare(native);source=rows(native/'patents.csv')
      for row in source: row['ardssic']=''
      write(native/'patents.csv',source);reference=root/'missing-reference';shutil.copytree(REFERENCE,reference)
      predictions=rows(reference/'predictions.csv')
      for row in predictions: row['ardssic']=''
      write(reference/'predictions.csv',predictions);run(native)
      self.assertEqual(audit(native,reference,native/'result.jsonl'),(200,106,0))

  def test_thirty_frozen_independent_r_student_t_tails(self):
    oracle=rows(REFERENCE/'student_t_tails.csv');self.assertEqual(len(oracle),30)
    for row in oracle:
      expected=float(row['p_value']);actual=student_two_tail(float(row['statistic']),int(row['df']))
      self.assertLessEqual(abs(actual-expected),1e-9*expected,row)
      self.assertEqual(actual,student_two_tail(-float(row['statistic']),int(row['df'])))

  def test_twenty_six_forged_artifacts(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp);base=root/'base';prepare(base);run(base);self.assertEqual(audit(base,REFERENCE,base/'result.jsonl'),(200,106,0))
      for corruption in range(26):
        with self.subTest(corruption=corruption):
          native=root/f'native-{corruption}';reference=root/f'reference-{corruption}';shutil.copytree(base,native);shutil.copytree(REFERENCE,reference)
          events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
          fe=next(e['data'] for e in events if e['result_type']=='FixedEffectsResult');common=fe['common_intercept_statistics'];level=fe['normalized_level']
          oracle=rows(reference/'reference.csv');lookup={(r['model'],r['path']):r for r in oracle}
          def coupled(path,value):
            set_path(fe,path,value);lookup[('fe',path)]['value']=str(value)
          if corruption==0: fe['common_intercept_inference']='normalization_point_only'
          elif corruption==1: common['name']='baseline_firm_intercept'
          elif corruption==2: common['degrees_of_freedom']=level['inference_df']=39
          elif corruption==3: common['confidence_level']=90
          elif corruption==4: level['parameter_names']=['logr','_cons']
          elif corruption==5: level['reference_distribution']='normal'
          elif corruption==6: coupled('/normalized_level/parameters/0',level['parameters'][0]+1)
          elif corruption==7: coupled('/normalized_level/parameters/1',level['parameters'][1]+.1)
          elif corruption==8: coupled('/normalized_level/covariance_matrix/0/1',level['covariance_matrix'][0][1]+.01)
          elif corruption==9:
            # The whole new block is R-consistent with a deliberately wrong
            # variance, including tiny Student-t p/CI, in both matching artifacts.
            for row in rows(REFERENCE/'forged_normalization.csv'): coupled(row['path'],float(row['value']))
          elif corruption==10: coupled('/common_intercept_statistics/p_value',common['p_value']*2)
          elif corruption==11:
            coupled('/common_intercept_statistics/ci_lower',common['ci_lower']-.01);coupled('/common_intercept_statistics/ci_upper',common['ci_upper']+.01)
          elif corruption==12: coupled('/common_intercept_statistics/standard_error',common['standard_error']*2)
          elif corruption==13: coupled('/common_intercept',fe['common_intercept']+1)
          elif corruption==14: level['covariance_matrix'].pop()
          elif corruption==15:
            source=rows(native/'patents.csv');source[-1]=dict(source[0]);write(native/'patents.csv',source)
          elif corruption==16:
            predictions=rows(reference/'predictions.csv');predictions[-1]=dict(predictions[0]);write(reference/'predictions.csv',predictions)
          elif corruption==17: oracle[-1]=dict(oracle[0])
          elif corruption==18: oracle=oracle[:-15]
          elif corruption==19:
            exported=rows(native/'linear_panel_predictions.csv');exported[-1]=dict(exported[0]);write(native/'linear_panel_predictions.csv',exported)
          elif corruption==20:
            source=rows(native/'patents.csv');source[0]['logr']=str(float(source[0]['logr'])+.1);write(native/'patents.csv',source)
          elif corruption==21: coupled('/normalized_level/covariance_matrix/0/0',-level['covariance_matrix'][0][0])
          elif corruption==22: fe['unit_effect_normalization']='unweighted_baseline_unit'
          elif corruption==23: coupled('/common_intercept_statistics/p_value',0.)
          elif corruption==24: coupled('/common_intercept_statistics/p_value',math.erfc(abs(common['statistic'])/math.sqrt(2)))
          elif corruption==25:
            half=1.959963984540054*common['standard_error'];coupled('/common_intercept_statistics/ci_lower',common['value']-half);coupled('/common_intercept_statistics/ci_upper',common['value']+half)
          write(reference/'reference.csv',oracle);(native/'result.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
          if corruption==9:
            with patch('check_linear_panel.normalized_checks'):
              self.assertEqual(audit(native,reference,native/'result.jsonl'),(200,106,0))
            with self.assertRaisesRegex(AssertionError,'independent normalized covariance inverse'): audit(native,reference,native/'result.jsonl')
          else:
            with self.assertRaises((AssertionError,ValueError,KeyError,IndexError)): audit(native,reference,native/'result.jsonl')


if __name__=='__main__': unittest.main()
