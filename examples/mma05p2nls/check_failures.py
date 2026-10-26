#!/usr/bin/env python3
"""Replay actual-source audit forgeries/publication failures; external assets stay external."""
import argparse
import contextlib
import copy
import csv
import io
import json
from pathlib import Path
import tempfile
from unittest.mock import patch
import check_reference
import prepare


def check_failures(native_json, native_csv, reference, source):
  arguments = [native_json, native_csv, reference, source]
  with tempfile.TemporaryDirectory(prefix='tabdat-nls-audit-regressions-') as directory:
    directory = Path(directory)
    with contextlib.redirect_stdout(io.StringIO()):
      check_reference.audit(*arguments, directory/'baseline.json')
    leaves = [json.loads(line) for line in Path(native_json).read_text().splitlines()]
    indices = [i for i, r in enumerate(leaves) if r.get('result_type') == 'NlsResult']
    mutations = {
      'coefficient': lambda a: a[indices[0]]['data']['coefficients'][0].__setitem__('value', 2.0),
      'covariance': lambda a: a[indices[1]]['data']['covariance_matrix'][0].__setitem__(1, 0.1),
      'tiny_tail_erased': lambda a: a[indices[1]]['data']['coefficients'][0].__setitem__('p_value', 0),
      'projection': lambda a: a[indices[0]]['data']['fitted_values'].__setitem__(777, 2.0),
      'binding': lambda a: a[indices[0]]['data']['model'].__setitem__('expression', {'kind':'parameter', 'arguments':0}),
      'sample_order': lambda a: a[indices[0]]['data']['sample']['retained_indices'].__setitem__(9999, 0),
      'diagnostic': lambda a: [r for r in a if r.get('result_type') == 'TableResult'][-1]['data']['rows'][0].__setitem__(3, 1.0),
    }
    cases = []
    for name, mutate in mutations.items():
      data = copy.deepcopy(leaves); mutate(data)
      path = directory/(name+'.jsonl')
      path.write_text(''.join(json.dumps(r)+'\n' for r in data))
      cases.append((name, path, native_csv))
    with Path(native_csv).open(newline='') as stream:
      reader = csv.DictReader(stream); fields = reader.fieldnames; original = list(reader)
    for name, field, value in [('source_identity','y','123.4'), ('source_ordinal','source_row','2'), ('csv_projection','mean_hc1','9.9')]:
      data = copy.deepcopy(original); data[0][field] = value
      path = directory/(name+'.csv')
      with path.open('w', newline='') as stream:
        writer = csv.DictWriter(stream, fieldnames=fields); writer.writeheader(); writer.writerows(data)
      cases.append((name, native_json, path))
    for name, j, c in cases:
      receipt = directory/(name+'-receipt.json')
      try:
        check_reference.audit(j, c, reference, source, receipt)
      except AssertionError:
        pass
      else:
        raise AssertionError(f'{name}: forged artifact accepted')
      assert not receipt.exists(), name
    class FailingWriter:
      def __init__(self, stream): self.stream = stream
      def writerow(self, row): self.stream.write('partially written header\n')
      def writerows(self, rows): raise OSError('injected prepare write failure')
    target = directory/'failed-prepare.csv'
    with patch.object(prepare.csv, 'writer', FailingWriter):
      try: prepare.convert(Path(source)/'mma05data.asc', target)
      except OSError as error: assert str(error) == 'injected prepare write failure'
      else: raise AssertionError('prepare write should fail')
    assert not target.exists() and not list(directory.glob('.nls-prepare-*'))
    prepare.convert(Path(source)/'mma05data.asc', target)
    before = target.read_bytes()
    try: prepare.convert(Path(source)/'mma05data.asc', target)
    except FileExistsError: pass
    else: raise AssertionError('prepare must not clobber')
    assert target.read_bytes() == before and not list(directory.glob('.nls-prepare-*'))
    receipt = directory/'failed-receipt.json'
    with patch.object(check_reference.json, 'dump', side_effect=OSError('injected receipt write failure')):
      try: check_reference.audit(*arguments, receipt)
      except OSError as error: assert str(error) == 'injected receipt write failure'
      else: raise AssertionError('audit write should fail')
    assert not receipt.exists() and not list(directory.glob('.nls-audit-*'))
    existing = directory/'baseline.json'; before = existing.read_bytes()
    try: check_reference.audit(*arguments, existing)
    except FileExistsError: pass
    else: raise AssertionError('audit must not clobber')
    assert existing.read_bytes() == before and not list(directory.glob('.nls-audit-*'))
  print('PASS:10 actual-source forgeries rejected/no receipts; prepare/audit injected write failures and no-clobber checks leave no target corruption/staging')


if __name__ == '__main__':
  parser = argparse.ArgumentParser(description=__doc__)
  for argument in ['native_json', 'native_csv', 'reference', 'source']:
    parser.add_argument(argument, type=Path)
  check_failures(**vars(parser.parse_args()))
