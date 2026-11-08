#!/usr/bin/env python3
"""Native gamma bootstrap controls and matching-artifact corruption defenses."""
import csv
import json
import math
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from check_native_gamma_re_bootstrap import audit, empirical_checks, R_NAMES

ROOT=Path(__file__).resolve().parents[2]
RAW=ROOT/'tests/fixtures/mma23_gamma_re_bootstrap/source.asc'
REFERENCE=ROOT/'tests/fixtures/mma23_native_gamma_re_bootstrap'
SCRIPT=ROOT/'examples/mma23p1pannonlin/native_gamma_re_bootstrap.td'
EXPORT='native_gamma_re_bootstrap_predictions.csv'


def rows(path):
  with path.open(newline='') as stream: return list(csv.DictReader(stream))


def write(path,records):
  with path.open('w',newline='') as stream:
    writer=csv.DictWriter(stream,fieldnames=list(records[0]));writer.writeheader();writer.writerows(records)


def baseline(native):
  native.mkdir();subprocess.run(['sh',str(ROOT/'examples/mma23p1pannonlin/prepare.sh'),str(RAW),str(native/'patents.csv')],capture_output=True,check=True)
  output=subprocess.run([str(ROOT/'target/debug/tabdat-explore-rs'),'--json','-f',str(SCRIPT)],cwd=native,capture_output=True,text=True)
  assert output.returncode==0,(output.stdout,output.stderr);(native/'result.jsonl').write_text(output.stdout)


class NativeGammaBootstrapAudit(unittest.TestCase):
  def test_constant_joint_vectors_have_exact_zero_variance_and_cross_covariance(self):
    v=float.fromhex('0x1.fe1581ecd07ebp-1');point=[v,-v,0.]
    report={'replicate_coefficients':[list(point) for _ in range(500)],'covariance_matrix':[[0.]*3 for _ in range(3)],
      'coefficients':[{'estimate':value,'bootstrap_mean':value,'bias':0.,'standard_error':0.,'ci_lower':value,'ci_upper':value} for value in point]}
    empirical_checks(report)
    report['covariance_matrix'][1][0]=1e-33
    with self.assertRaises(AssertionError): empirical_checks(report)

  def test_actual_control_all_three_parameter_r_refits_and_draws(self):
    with tempfile.TemporaryDirectory() as tmp:
      native=Path(tmp)/'native';baseline(native)
      self.assertEqual(audit(native,REFERENCE,RAW,False),(20000,31,500))

  def test_thirty_corruptions_including_shared_native_r_nuisance_and_covariance(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp);base=root/'base';baseline(base);self.assertEqual(audit(base,REFERENCE,RAW,False),(20000,31,500))
      for corruption in range(30):
        with self.subTest(corruption=corruption):
          native=root/f'native-{corruption}';reference=root/f'reference-{corruption}';shutil.copytree(base,native);shutil.copytree(REFERENCE,reference)
          events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()];d=events[6]['data']
          records=rows(reference/'reference.csv');lookup={r['field']:r for r in records}
          def coupled(index,key,value):
            d['coefficients'][index][key]=value
            prefix={'bootstrap_mean':'mean','bias':'bias','standard_error':'se','ci_lower':'lower','ci_upper':'upper','estimate':'estimate'}[key]
            lookup[prefix+'_'+R_NAMES[index]]['value']=str(value)
          if corruption==0: d['seed']='10002'
          elif corruption==1: d['rng']='stata'
          elif corruption==2: d['cluster_order']='positive_total_only'
          elif corruption==3: d['cluster_labels'].reverse()
          elif corruption==4: d['cluster_sizes'][0]+=1
          elif corruption==5: d['zero_total_group_count']=0
          elif corruption==6: d['sample']['dropped_observations']=40
          elif corruption==7: d['cluster_count']-=8
          elif corruption==8: d['replications']=499
          elif corruption==9: d['replicate_cluster_indices'][0][0]=(d['replicate_cluster_indices'][0][0]+1)%40
          elif corruption==10: d['replicate_observation_counts'][0]-=5
          elif corruption==11: d['replicate_cluster_indices'][0].pop()
          elif corruption==12: d['replicate_coefficients'][0].pop()
          elif corruption==13:
            for i in range(3):
              for j in range(3): d['covariance_matrix'][i][j]*=1.25
              coupled(i,'standard_error',d['coefficients'][i]['standard_error']*math.sqrt(1.25))
            for name,row in lookup.items():
              if name.startswith('covariance_'): row['value']=str(float(row['value'])*1.25)
          elif corruption in (14,15,16):
            component=corruption-14
            for vector in d['replicate_coefficients']: vector[component]+=.1
            fitted=rows(reference/'fits.csv')
            for row in fitted: row[R_NAMES[component]]=str(float(row[R_NAMES[component]])+.1)
            write(reference/'fits.csv',fitted)
            for key in ['bootstrap_mean','bias','ci_lower','ci_upper']: coupled(component,key,d['coefficients'][component][key]+.1)
          elif corruption==17:
            c=d['coefficients'][2];half=1.959963984540054*c['standard_error'];coupled(2,'ci_lower',c['estimate']-half);coupled(2,'ci_upper',c['estimate']+half)
          elif corruption==18: d['confidence_level']=.9
          elif corruption==19: d['interval_method']='normal_95'
          elif corruption==20: d['covariance_convention']='observed_information'
          elif corruption==21: d['covariance_parameter_names']=['_cons','logr','alpha']
          elif corruption==22: d['coefficients'][2]['name']='alpha'
          elif corruption==23: d['covariance_matrix'][1][0]*=1.5
          elif corruption==24: records[-1]=dict(records[0])
          elif corruption==25:
            fitted=rows(reference/'fits.csv');fitted[-1]['replicate']='499';write(reference/'fits.csv',fitted)
          elif corruption==26:
            exported=rows(native/EXPORT);exported[0]['native_re_mean']=str(float(exported[0]['native_re_mean'])+.1);write(native/EXPORT,exported)
          elif corruption==27:
            source=rows(native/'patents.csv');source[-1]=dict(source[0]);write(native/'patents.csv',source)
          elif corruption==28: coupled(2,'estimate',d['coefficients'][2]['estimate']+.1)
          elif corruption==29:
            exported=rows(native/EXPORT);exported[-1]['included_in_native_re']='false';write(native/EXPORT,exported)
          write(reference/'reference.csv',records);(native/'result.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
          if corruption==13:
            with patch('check_native_gamma_re_bootstrap.empirical_checks'):
              self.assertEqual(audit(native,reference,RAW,False),(20000,31,500))
            with self.assertRaisesRegex(AssertionError,'independent empirical covariance'): audit(native,reference,RAW,False)
          elif corruption in (14,15,16):
            with patch('check_native_gamma_re_bootstrap.replicate_score'):
              self.assertEqual(audit(native,reference,RAW,False),(20000,31,500))
            with self.assertRaisesRegex(AssertionError,'independent replicate score'): audit(native,reference,RAW,False)
          else:
            with self.assertRaises((AssertionError,ValueError,KeyError,IndexError)): audit(native,reference,RAW,False)


if __name__=='__main__': unittest.main()
