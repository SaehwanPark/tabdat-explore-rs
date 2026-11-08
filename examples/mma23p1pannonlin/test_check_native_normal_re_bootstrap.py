"""Hostile bootstrap artifacts, including coherent wrong native/R outputs."""
import copy
import json
import math
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import check_native_normal_re_bootstrap as checker
from check_native_normal_re import posterior
from test_check_native_gamma_re import write
from test_check_native_normal_re import prepare, ROOT, SOURCE, REFERENCE, BIN

BOOT_REFERENCE=ROOT/'tests/fixtures/mma23_native_normal_re_bootstrap'
SCRIPT=ROOT/'examples/mma23p1pannonlin/native_normal_re_bootstrap.td'


def recalculate(report):
  values=report['replicate_coefficients'];means=[math.fsum(row[j] for row in values)/500 for j in range(3)]
  covariance=[[math.fsum((row[i]-means[i])*(row[j]-means[j]) for row in values)/499 for j in range(3)] for i in range(3)]
  report['covariance_matrix']=covariance
  for j,c in enumerate(report['coefficients']):
    c['bootstrap_mean']=means[j];c['bias']=means[j]-c['estimate'];c['standard_error']=math.sqrt(covariance[j][j]);ordered=sorted(row[j] for row in values)
    for key,p in [('ci_lower',.025),('ci_upper',.975)]:
      h=499*p;lo=math.floor(h);c[key]=ordered[lo]+(h-lo)*(ordered[lo+1]-ordered[lo])


def update_r(report,reference):
  records=checker.rows(reference/'reference.csv');lookup={r['field']:r for r in records}
  for c,name in zip(report['coefficients'],checker.R_NAMES):
    for key,prefix in [('estimate','estimate'),('bootstrap_mean','mean'),('bias','bias'),('standard_error','se'),('ci_lower','lower'),('ci_upper','upper')]:lookup[prefix+'_'+name]['value']=repr(c[key])
  for key,i,j in [('aa',0,0),('ab',0,1),('at',0,2),('bb',1,1),('bt',1,2),('tt',2,2)]:lookup['covariance_'+key]['value']=repr(report['covariance_matrix'][i][j])
  write(reference/'reference.csv',records)


class BootstrapAudit(unittest.TestCase):
  @classmethod
  def setUpClass(cls):
    cls.temp=tempfile.TemporaryDirectory();cls.root=Path(cls.temp.name);cls.native=cls.root/'baseline';prepare(cls.native)
    out=subprocess.run([str(BIN),'--json','-f',str(SCRIPT)],cwd=cls.native,capture_output=True,text=True);assert out.returncode==0,out.stderr
    (cls.native/'result.jsonl').write_text(out.stdout)
    cls.events=[json.loads(line) for line in out.stdout.splitlines()]

  @classmethod
  def tearDownClass(cls):cls.temp.cleanup()

  def test_actual500_and_first_moments_preserve_independent_controls(self):
    self.assertEqual(checker.audit(self.native,SOURCE,BOOT_REFERENCE,REFERENCE,False),(200,40))
    for row in checker.rows(ROOT/'crates/tabdat-stats/tests/fixtures/normal_poisson/posterior.csv'):
      s,m,v=float(row['total']),float(row['mean_sum']),math.exp(float(row['log_variance']))
      self.assertEqual(posterior(s,m,v,covariance=False),posterior(s,m,v)[:3])

  def test_twenty_artifact_forgeries_and_coherent_mocked_guard_demonstrations(self):
    for corruption in range(20):
      with self.subTest(corruption=corruption),tempfile.TemporaryDirectory() as tmp:
        root=Path(tmp);native=root/'native';reference=root/'reference';shutil.copytree(self.native,native);shutil.copytree(BOOT_REFERENCE,reference)
        events=copy.deepcopy(self.events);report=events[6]['data']
        if corruption==0:report['estimator']='pooled_poisson'
        elif corruption==1:report['refit_quadrature_nodes']=[128,160]
        elif corruption==2:report['original_quadrature_nodes']=[176,208]
        elif corruption==3:report['refit_initializer']='retry_failed_fit'
        elif corruption==4:report['replicate_cluster_indices'][0][0]=(report['replicate_cluster_indices'][0][0]+1)%40
        elif corruption==5:report['replicate_observation_counts'][0]-=1
        elif corruption==6:report['cluster_order']='sorted'
        elif corruption==7:report['zero_total_group_count']=0
        elif corruption==8:report['singleton_group_count']=1
        elif corruption==9:report['covariance_parameter_names'][-1]='alpha'
        elif corruption==10:report['covariance_matrix'][0][1]+=.01
        elif corruption==11:report['coefficients'][2]['ci_upper']+=.1
        elif corruption==12:report['sample']['dropped_observations']=1
        elif corruption==13:report['random_effect_distribution']='mean_one_gamma'
        elif corruption==14:report['coefficients'][2]['estimate']+=.1
        elif corruption==15:report['seed']=10001
        elif corruption==16:report['replicate_coefficients'][0][2]+=.1
        elif corruption==17:report['replicate_coefficients'].pop()
        elif corruption==18:
          # Coherent covariance/SE corruption agrees with R but violates the
          # actual empirical B-1 covariance of the unchanged replicate vectors.
          report['covariance_matrix']=[[v*1.25 for v in row] for row in report['covariance_matrix']]
          for c in report['coefficients']:c['standard_error']*=math.sqrt(1.25)
          update_r(report,reference)
        elif corruption==19:
          # Coherent wrong replicate/R fit/all uncertainty satisfies agreement
          # and empirical identities, but not raw complete-data score equations.
          report['replicate_coefficients'][0][2]+=.1;recalculate(report);update_r(report,reference)
          fits=checker.rows(reference/'fits.csv');fits[0]['estimate_lnvariance']=repr(report['replicate_coefficients'][0][2]);write(reference/'fits.csv',fits)
        (native/'result.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
        if corruption in (18,19):
          with patch.object(checker,'verify_scores',return_value=None):
            if corruption==18:
              with patch.object(checker,'empirical_checks',return_value=None):self.assertEqual(checker.audit(native,SOURCE,reference,REFERENCE,False),(200,40))
            else:self.assertEqual(checker.audit(native,SOURCE,reference,REFERENCE,False),(200,40))
        with self.assertRaises((AssertionError,ValueError,KeyError,IndexError)):
          checker.audit(native,SOURCE,reference,REFERENCE,False)


if __name__=='__main__':unittest.main()
