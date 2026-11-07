#!/usr/bin/env python3
"""Offline checker regression: real native output plus targeted forged artifacts."""
import csv
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from check_household_bp_lm import audit

REPO = Path(__file__).resolve().parents[2]
FIXTURE = REPO/'tests/fixtures/mma24_household_bp_lm'


def change_csv(path, key, value):
  with path.open(newline='') as f:
    reader = csv.DictReader(f)
    fields, data = reader.fieldnames, list(reader)
  data[0][key] = value
  with path.open('w', newline='') as f:
    writer = csv.DictWriter(f, fieldnames=fields)
    writer.writeheader()
    writer.writerows(data)


class CheckerTest(unittest.TestCase):
  def test_baseline_and_forged_artifacts(self):
    with tempfile.TemporaryDirectory() as tmp:
      root = Path(tmp)
      native, reference = root/'native', root/'reference'
      native.mkdir()
      reference.mkdir()
      raw = FIXTURE/'unbalanced_input.csv'
      shutil.copyfile(raw, native/'vietnam.csv')
      for kind in ('diagnostic','fit','predictions'):
        shutil.copyfile(FIXTURE/f'unbalanced_{kind}.csv', reference/f'{kind}.csv')
      shutil.copyfile(raw, reference/'source.csv')
      with (native/'result.jsonl').open('w') as output:
        subprocess.run([str(REPO/'target/debug/tabdat-explore-rs'),'--json','-f',
          str(REPO/'examples/mma24p1olscluster/household_bp_lm.td')],cwd=native,
          stdout=output,check=True,capture_output=False)
      self.assertEqual(audit(native,reference,raw,'--csv'),(150,16))
      mutations = [('diagnostic',key,'0') for key in ('observations','communes',
        'sum_group_size_squared','group_residual_sum_squares','residual_sum_of_squares',
        'signed_score','lm_statistic','p_value')]
      mutations += [('diagnostic','reference_distribution','one_sided_boundary'),
        ('predictions','bp_xb','0'),('predictions','source_row','999'),('predictions','bp_residual','0')]
      for index,(kind,key,value) in enumerate(mutations):
        with self.subTest(kind=kind,key=key):
          copy = root/f'forged-{index}'
          shutil.copytree(native,copy)
          change_csv(copy/f'household_bp_{kind}.csv',key,value)
          with self.assertRaises(AssertionError):
            audit(copy,reference,raw,'--csv')
      copy = root/'forged-fit'
      shutil.copytree(native,copy)
      path = copy/'result.jsonl'
      events = [json.loads(line) for line in path.read_text().splitlines()]
      events[5]['data']['coefficients'][0]['value'] += .01
      path.write_text('\n'.join(json.dumps(e) for e in events)+'\n')
      with self.assertRaises(AssertionError):
        audit(copy,reference,raw,'--csv')
      # Even a shared native/reference sign flip must fail the residual identity.
      copy, refcopy = root/'shared-sign', root/'shared-reference'
      shutil.copytree(native,copy)
      shutil.copytree(reference,refcopy)
      for path in (copy/'household_bp_diagnostic.csv',refcopy/'diagnostic.csv'):
        change_csv(path,'signed_score','-25.158227413613858')
      with self.assertRaises(AssertionError):
        audit(copy,refcopy,raw,'--csv')


if __name__ == '__main__':
  unittest.main()
