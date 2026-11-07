#!/usr/bin/env python3
"""Actual native bootstrap and shared-artifact corruption regressions."""
import csv
import json
import math
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from check_native_conditional_fe_bootstrap import audit, empirical_checks

ROOT=Path(__file__).resolve().parents[2]
RAW=ROOT/'tests/fixtures/mma23_conditional_fe/source.asc'
REFERENCE=ROOT/'tests/fixtures/mma23_native_conditional_fe_bootstrap'
SCRIPT=ROOT/'examples/mma23p1pannonlin/native_conditional_fe_bootstrap.td'
EXPORT='native_conditional_fe_bootstrap_predictions.csv'


def write(path,records):
  with path.open('w',newline='') as stream:
    writer=csv.DictWriter(stream,fieldnames=list(records[0]));writer.writeheader();writer.writerows(records)


def rows(path):
  with path.open(newline='') as stream: return list(csv.DictReader(stream))


def baseline(native):
  native.mkdir();subprocess.run(['sh',str(ROOT/'examples/mma23p1pannonlin/prepare.sh'),str(RAW),str(native/'patents.csv')],capture_output=True,check=True)
  output=subprocess.run([str(ROOT/'target/debug/tabdat-explore-rs'),'--json','-f',str(SCRIPT)],cwd=native,capture_output=True,text=True)
  assert output.returncode==0,(output.stdout,output.stderr)
  (native/'result.jsonl').write_text(output.stdout)


class NativeBootstrapAudit(unittest.TestCase):
  def test_exact_constant_replicates_retain_zero_empirical_variance(self):
    for value in [float.fromhex('0x1.fe1581ecd07ebp-1'),-float.fromhex('0x1.fe1581ecd07ebp-1'),0.]:
      report={'replicate_coefficients':[[value] for _ in range(500)],'covariance_matrix':[[0.]],
        'coefficients':[{'estimate':value,'bootstrap_mean':value,'bias':0.,'standard_error':0.,'ci_lower':value,'ci_upper':value}]}
      empirical_checks(report)
      report['covariance_matrix'][0][0]=1e-33
      with self.assertRaises(AssertionError): empirical_checks(report)

  def test_actual_small_source_all_r_refits_draws_predictions(self):
    with tempfile.TemporaryDirectory() as tmp:
      native=Path(tmp)/'native';baseline(native)
      self.assertEqual(audit(native,REFERENCE,RAW,False),(5000,11,500))

  def test_twenty_six_corruptions_including_shared_native_r_fields(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp);base=root/'base';baseline(base)
      self.assertEqual(audit(base,REFERENCE,RAW,False),(5000,11,500))
      for corruption in range(26):
        with self.subTest(corruption=corruption):
          native=root/f'native-{corruption}';reference=root/f'reference-{corruption}';shutil.copytree(base,native);shutil.copytree(REFERENCE,reference)
          events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
          d=events[6]['data'];c=d['coefficients'][0];records=rows(reference/'reference.csv');lookup={r['field']:r for r in records}
          def coupled(field,value):
            c[field]=value;lookup['bootstrap_se' if field=='standard_error' else field]['value']=str(value)
          if corruption==0: d['seed']='10002'
          elif corruption==1: d['rng']='stata'
          elif corruption==2: d['cluster_order']='sorted_after_drawing'
          elif corruption==3: d['cluster_labels'].reverse()
          elif corruption==4: d['cluster_sizes'][0]+=1
          elif corruption==5: d['zero_total_group_count']=0
          elif corruption==6: d['zero_total_observation_count']=0
          elif corruption==7: d['candidate_group_count']-=1
          elif corruption==8: d['replications']=499
          elif corruption==9: d['replicate_cluster_indices'][0][0]=(d['replicate_cluster_indices'][0][0]+1)%10
          elif corruption==10: d['replicate_observation_counts'][0]-=5
          elif corruption==11: d['replicate_cluster_indices'][0].pop()
          elif corruption==12: d['replicate_coefficients'].pop()
          elif corruption==13:
            # Matching but wrong native/R SE/covariance cannot be detected by
            # artifact equality alone; recompute from the actual500 refits.
            d['covariance_matrix'][0][0]*=1.25;coupled('standard_error',c['standard_error']*math.sqrt(1.25))
          elif corruption==14:
            for row in d['replicate_coefficients']: row[0]+=.1
            replicated=rows(reference/'replicates.csv')
            for row in replicated: row['beta']=str(float(row['beta'])+.1)
            write(reference/'replicates.csv',replicated)
            for field in ['bootstrap_mean','bias','ci_lower','ci_upper']: coupled(field,c[field]+.1)
          elif corruption==15:
            half=1.959963984540054*c['standard_error'];coupled('ci_lower',c['estimate']-half);coupled('ci_upper',c['estimate']+half)
          elif corruption==16: d['confidence_level']=.9
          elif corruption==17: d['interval_method']='normal_95'
          elif corruption==18: d['covariance_convention']='conditional_information'
          elif corruption==19: d['covariance_parameter_names']=['_cons']
          elif corruption==20: d['sample']['dropped_observations']=0
          elif corruption==21: records[-1]=dict(records[0])
          elif corruption==22:
            replicated=rows(reference/'replicates.csv');replicated[-1]['replicate']='499';write(reference/'replicates.csv',replicated)
          elif corruption==23:
            exported=rows(native/EXPORT);exported[0]['native_fe_mean']=str(float(exported[0]['native_fe_mean'])+.1);write(native/EXPORT,exported)
          elif corruption==24:
            source=rows(native/'patents.csv');source[-1]=dict(source[0]);write(native/'patents.csv',source)
          elif corruption==25: coupled('estimate',c['estimate']+.1)
          write(reference/'reference.csv',records);(native/'result.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
          if corruption==13:
            with patch('check_native_conditional_fe_bootstrap.empirical_checks'):
              self.assertEqual(audit(native,reference,RAW,False),(5000,11,500))
            with self.assertRaisesRegex(AssertionError,'independent empirical covariance'): audit(native,reference,RAW,False)
          elif corruption==14:
            with self.assertRaisesRegex(AssertionError,'independent replicate score'): audit(native,reference,RAW,False)
          else:
            with self.assertRaises((AssertionError,ValueError,KeyError,IndexError)): audit(native,reference,RAW,False)


if __name__=='__main__': unittest.main()
