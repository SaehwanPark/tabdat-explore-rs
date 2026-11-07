#!/usr/bin/env python3
"""Actual mixed-script source audits and targeted artifact-forgery regressions."""
import csv
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from check_household_mixed_report import audit

REPO = Path(__file__).resolve().parents[2]
FIXTURES = REPO/'tests/fixtures/mma24_household_mixed_report'


def csv_change(path, key, value):
  with path.open(newline='') as f:
    reader = csv.DictReader(f)
    fields, data = reader.fieldnames, list(reader)
  data[0][key] = value
  with path.open('w',newline='') as f:
    writer = csv.DictWriter(f,fieldnames=fields)
    writer.writeheader();writer.writerows(data)


def write_events(path, events):
  path.write_text('\n'.join(json.dumps(e) for e in events)+'\n')


class AuditorTest(unittest.TestCase):
  def test_three_controls_and_twenty_four_forgeries(self):
    with tempfile.TemporaryDirectory() as tmp:
      root = Path(tmp)
      for case,n,g in [('balanced',192,32),('unbalanced',199,32),('interior',179,16)]:
        native = root/case;native.mkdir()
        raw = FIXTURES/case/'input.csv'
        shutil.copyfile(raw,native/'vietnam.csv')
        with (native/'result.jsonl').open('w') as out:
          subprocess.run([str(REPO/'target/debug/tabdat-explore-rs'),'--json','-f',
            str(REPO/'examples/mma24p1olscluster/household_mixed_report.td')],cwd=native,stdout=out,check=True)
        self.assertEqual(audit(native,FIXTURES/case,raw,'--csv'),(n,g,327))
      native,ref,raw = root/'interior',FIXTURES/'interior',FIXTURES/'interior/input.csv'
      changes = [
        (0,('covariance',),'unit_sandwich_uncorrected'),
        (0,('reference_distribution',),'student_t'),
        (0,('inference_degrees_of_freedom',),178),
        (1,('covariance',),'nonrobust'),
        (1,('df_residual',),172),
        (1,('panel','intercept_convention'),'population_mean_intercept'),
        (2,('estimator',),'swamy_arora_random_effects_gls'),
        (2,('log_likelihood',),1),
        (3,('log_likelihood',),0),
        (3,('log_likelihood_convention',),'gaussian_working_model_ml_variance_rss_over_n'),
        (3,('r_squared',),.5),
        (0,('panel','time_variable'),'source_row'),
        (0,('sample','total_observations'),180),
        (1,('sample','retained_indices',0),3),
        (3,('cluster_count',),16),
        (1,('coefficients',0,'name'),'_cons'),
        (2,('covariance_matrix',0,0),.5),
        (0,('coefficients',0,'standard_error'),1),
      ]
      rejected = 0
      for i,(model,keys,value) in enumerate(changes):
        copy = root/f'field-{i}';shutil.copytree(native,copy)
        path = copy/'result.jsonl';events = list(map(json.loads,path.read_text().splitlines()))
        target = events[13]['data']['models'][model]
        for key in keys[:-1]:target = target[key]
        target[keys[-1]] = value;write_events(path,events)
        with self.subTest(keys=keys),self.assertRaises(AssertionError):audit(copy,ref,raw,'--csv')
        rejected += 1
      for key in ('ml_xb','source_row'):
        copy = root/('prediction-'+key);shutil.copytree(native,copy)
        csv_change(copy/'household_mixed_report_predictions.csv',key,'999')
        with self.subTest(key=key),self.assertRaises(AssertionError):audit(copy,ref,raw,'--csv')
        rejected += 1
      copy = root/'source';shutil.copytree(native,copy)
      csv_change(copy/'vietnam.csv','educ','999')
      with self.assertRaises(AssertionError):audit(copy,ref,raw,'--csv')
      rejected += 1
      for name,mutation in [('union',lambda e:e[13]['data']['parameter_names'].reverse()),
                            ('missing-store',lambda e:e.pop(6))]:
        copy = root/name;shutil.copytree(native,copy)
        path = copy/'result.jsonl';events = list(map(json.loads,path.read_text().splitlines()))
        mutation(events);write_events(path,events)
        with self.subTest(name=name),self.assertRaises(AssertionError):audit(copy,ref,raw,'--csv')
        rejected += 1
      copy,refcopy = root/'shared-stat',root/'shared-reference'
      shutil.copytree(native,copy);shutil.copytree(ref,refcopy)
      path = copy/'result.jsonl';events = list(map(json.loads,path.read_text().splitlines()))
      events[13]['data']['models'][3]['coefficients'][0]['statistic'] = 0
      events[11]['data']['coefficients'][0]['statistic'] = 0
      write_events(path,events)
      path = refcopy/'re_ml_reference.csv'
      path.write_text('\n'.join('/coefficients/0/statistic,0' if l.startswith('/coefficients/0/statistic,') else l
        for l in path.read_text().splitlines())+'\n')
      with self.assertRaises(AssertionError):audit(copy,refcopy,raw,'--csv')
      rejected += 1
      self.assertEqual(rejected,24)


if __name__=='__main__':
  unittest.main()
