#!/usr/bin/env python3
"""Offline actual-output/source auditor and targeted-forgery regressions."""
import csv
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from check_household_hausman import audit

REPO = Path(__file__).resolve().parents[2]
FIXTURE = REPO/'crates/tabdat-stats/tests/fixtures/panel_joint_equal_group_hausman'


def csv_change(path,key,value):
  with path.open(newline='') as f:
    reader = csv.DictReader(f)
    fields,data = reader.fieldnames,list(reader)
  data[0][key] = value
  with path.open('w',newline='') as f:
    writer = csv.DictWriter(f,fieldnames=fields)
    writer.writeheader()
    writer.writerows(data)


class AuditorTest(unittest.TestCase):
  def test_both_controls_and_seventeen_forgeries(self):
    with tempfile.TemporaryDirectory() as tmp:
      root = Path(tmp)
      for case,n in [('balanced',192),('unbalanced',199)]:
        native,ref = root/case,root/(case+'-reference')
        native.mkdir()
        ref.mkdir()
        raw = FIXTURE/f'{case}_input.csv'
        shutil.copyfile(raw,native/'vietnam.csv')
        shutil.copyfile(raw,ref/'source.csv')
        for kind in ('reference','predictions'):
          shutil.copyfile(FIXTURE/f'{case}_{kind}.csv',ref/f'{kind}.csv')
        with (native/'result.jsonl').open('w') as out:
          subprocess.run([str(REPO/'target/debug/tabdat-explore-rs'),'--json','-f',
            str(REPO/'examples/mma24p1olscluster/household_hausman.td')],cwd=native,stdout=out,check=True)
        self.assertEqual(audit(native,ref,raw,'--csv'),(n,32,0))
      native,ref,raw = root/'unbalanced',root/'unbalanced-reference',FIXTURE/'unbalanced_input.csv'
      changes = [(key,value) for key,value in [('statistic',0),('p_value',0),('df',5),
        ('candidate_observation_count',200),('fe_residual_scale',0),('re_residual_scale',0),
        ('minimum_scaled_cholesky_pivot',0),('re_component_method','swamy_arora'),
        ('covariance_scale_convention','common_fe_scale'),('reference_distribution','normal')]]
      for index,(key,value) in enumerate(changes):
        copy = root/f'field-{index}'
        shutil.copytree(native,copy)
        path = copy/'result.jsonl'
        events = [json.loads(x) for x in path.read_text().splitlines()]
        events[5]['data'][key] = value
        path.write_text('\n'.join(json.dumps(e) for e in events)+'\n')
        with self.subTest(key=key),self.assertRaises(AssertionError):
          audit(copy,ref,raw,'--csv')
      for index,key in enumerate(('fe_coefficients','re_coefficients','covariance_difference','fe_covariance')):
        copy = root/f'array-{index}'
        shutil.copytree(native,copy)
        path = copy/'result.jsonl'
        events = [json.loads(x) for x in path.read_text().splitlines()]
        v = events[5]['data'][key]
        if isinstance(v[0],list):
          v[0][0] += .01
        else:
          v[0] += .01
        path.write_text('\n'.join(json.dumps(e) for e in events)+'\n')
        with self.subTest(key=key),self.assertRaises(AssertionError):
          audit(copy,ref,raw,'--csv')
      for key in ('re_xb','source_row'):
        copy = root/('prediction-'+key)
        shutil.copytree(native,copy)
        csv_change(copy/'household_hausman_predictions.csv',key,'999')
        with self.subTest(key=key),self.assertRaises(AssertionError):
          audit(copy,ref,raw,'--csv')
      copy,refcopy = root/'shared-stat',root/'shared-reference'
      shutil.copytree(native,copy)
      shutil.copytree(ref,refcopy)
      path = copy/'result.jsonl'
      events = [json.loads(x) for x in path.read_text().splitlines()]
      events[5]['data']['statistic'] = 0
      path.write_text('\n'.join(json.dumps(e) for e in events)+'\n')
      path = refcopy/'reference.csv'
      path.write_text('\n'.join('/statistic,0' if line.startswith('/statistic,') else line
        for line in path.read_text().splitlines())+'\n')
      with self.assertRaises(AssertionError):
        audit(copy,refcopy,raw,'--csv')


if __name__ == '__main__':
  unittest.main()
