#!/usr/bin/env python3
"""Replay source-audit forgeries and receipt publication failures."""
import argparse
import contextlib
import copy
import csv
import io
import json
from pathlib import Path
import tempfile
from unittest.mock import patch
import check_weighted


def check_failures(native_json, native_csv, reference, source):
  args = [native_json, native_csv, reference, source]
  with tempfile.TemporaryDirectory(prefix='tabdat-weighted-audit-') as directory:
    directory = Path(directory)
    with contextlib.redirect_stdout(io.StringIO()):
      check_weighted.audit(*args, directory/'baseline.json')
    leaves = [json.loads(line) for line in Path(native_json).read_text().splitlines()]
    indices = [i for i, value in enumerate(leaves) if value.get('result_type') == 'NlsResult']
    edits = {
      'coefficient': lambda data: data[indices[1]]['data']['coefficients'][0].__setitem__('value', 2),
      'covariance': lambda data: data[indices[2]]['data']['covariance_matrix'][0].__setitem__(1, .1),
      'initial_binding': lambda data: data[indices[0]]['data']['model'].__setitem__('expression', {'kind': 'parameter', 'arguments': 0}),
      'scale_binding': lambda data: data[indices[3]]['data']['model']['predictor_names'].__setitem__(1, 'wnls_scale'),
      'sample_order': lambda data: data[indices[4]]['data']['sample']['retained_indices'].__setitem__(9999, 0),
      'projection': lambda data: data[indices[3]]['data']['fitted_values'].__setitem__(777, 2),
      'diagnostic': lambda data: [v for v in data if v.get('result_type') == 'TableResult'][-1]['data']['rows'][0].__setitem__(4, 1),
    }
    cases = []
    for name, mutate in edits.items():
      data = copy.deepcopy(leaves); mutate(data)
      path = directory/(name+'.jsonl')
      path.write_text(''.join(json.dumps(v)+'\n' for v in data))
      cases.append((name, path, native_csv))
    with Path(native_csv).open(newline='') as stream:
      reader = csv.DictReader(stream); fields = reader.fieldnames; original = list(reader)
    for name, field, value in [('source','y','123.4'), ('ordinal','source_row','2'),
        ('refreshed_weight','fgnls_weight','1'), ('swapped_scale','wnls_scale','1'),
        ('response','wnls_y','1'), ('level_projection','fgnls_level_residual','1')]:
      data = copy.deepcopy(original); data[0][field] = value
      path = directory/(name+'.csv')
      with path.open('w', newline='') as stream:
        writer = csv.DictWriter(stream, fieldnames=fields); writer.writeheader(); writer.writerows(data)
      cases.append((name, native_json, path))
    for name, j, c in cases:
      receipt = directory/(name+'-receipt.json')
      try: check_weighted.audit(j, c, reference, source, receipt)
      except AssertionError: pass
      else: raise AssertionError(f'{name}: forged artifact accepted')
      assert not receipt.exists(), name
    receipt = directory/'failed-receipt.json'
    with patch.object(check_weighted.json, 'dump', side_effect=OSError('injected receipt write failure')):
      try: check_weighted.audit(*args, receipt)
      except OSError as error: assert str(error) == 'injected receipt write failure'
      else: raise AssertionError('audit write must fail')
    assert not receipt.exists() and not list(directory.glob('.weighted-audit-*'))
    existing = directory/'baseline.json'; before = existing.read_bytes()
    try: check_weighted.audit(*args, existing)
    except FileExistsError: pass
    else: raise AssertionError('audit must not clobber')
    assert existing.read_bytes() == before and not list(directory.glob('.weighted-audit-*'))
  print(f'PASS: {len(cases)} forgeries rejected; receipt write failure and no-clobber preserve output and leave no staging')


if __name__ == '__main__':
  parser = argparse.ArgumentParser(description=__doc__)
  for argument in ['native_json', 'native_csv', 'reference', 'source']:
    parser.add_argument(argument, type=Path)
  check_failures(**vars(parser.parse_args()))
