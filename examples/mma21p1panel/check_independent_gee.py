#!/usr/bin/env python3
"""Audit actual CLI CSVs against R and, for MOM, the pinned source/log."""
import argparse
import csv
import hashlib
import math
from pathlib import Path
import struct
import subprocess
import tempfile

FILES = ('fit', 'coefficients', 'predictions', 'unit_scores')
SOURCE = ('lnhr', 'lnwg', 'kids', 'ageh', 'agesq', 'disab', 'id', 'year')
SHA = '2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'


def rows(path):
  with path.open(newline='') as handle:
    return list(csv.DictReader(handle))


def compare(actual, expected, name):
  if len(actual) != len(expected):
    raise AssertionError(f'{name}: row count {len(actual)} != {len(expected)}')
  max_error = 0.0
  count = 0
  for index, (a, b) in enumerate(zip(actual, expected)):
    for key, value in b.items():
      if key not in a:
        raise AssertionError(f'{name}[{index}] missing {key}')
      if key in ('covariance', 'parameter'):
        if a[key] != value:
          raise AssertionError((name, index, key, a[key], value))
        continue
      if value == '':
        if a[key] != '':
          raise AssertionError((name, index, key, a[key], 'expected NULL'))
        continue
      x, y = float(a[key]), float(value)
      if not (math.isfinite(x) and math.isfinite(y)):
        raise AssertionError((name, index, key, x, y))
      if key == 'p_value':
        ok = x == 0 if y == 0 else abs(x-y) <= abs(y)*1e-8
      else:
        ok = abs(x-y) <= 1e-9
      if not ok:
        raise AssertionError((name, index, key, x, y, abs(x-y)))
      max_error = max(max_error, abs(x-y))
      count += 1
  print(f'{name}: {len(actual)} rows, {count} numerical comparisons, max abs {max_error:.3g}')


def check_log(input_rows):
  # Stata 8 infile plausibly stored its input in float32. This is a separate
  # historical hypothesis, never a replacement for the raw-double R oracle.
  with tempfile.TemporaryDirectory(prefix='independent-gee-float32-') as temporary:
    base = Path(temporary)
    cast = [{k: (repr(struct.unpack('f', struct.pack('f', float(v)))[0])
                  if k in ('lnhr', 'lnwg') else v) for k, v in row.items()}
            for row in input_rows]
    with (base/'input.csv').open('w', newline='') as handle:
      writer = csv.DictWriter(handle, fieldnames=input_rows[0].keys())
      writer.writeheader()
      writer.writerows(cast)
    subprocess.run(['Rscript', str(Path(__file__).with_name('independent_gee_reference.R')),
                    str(base/'input.csv'), str(base/'reference')], check=True)
    fit = rows(base/'reference'/'fit.csv')[0]
    coefs = {(r['covariance'], r['parameter']): r for r in rows(base/'reference'/'coefficients.csv')}
    # (method, parameter, field, author printed value, half last digit).
    facts = [
      ('working_model','lnwg','estimate',.0827436,.00000005),
      ('working_model','lnwg','standard_error',.0091234,.00000005),
      ('working_model','lnwg','z',9.07,.005),
      ('working_model','lnwg','normal95_lower',.064862,.0000005),
      ('working_model','lnwg','normal95_upper',.1006251,.00000005),
      ('working_model','lnwg','slope_wald_chi2_1',82.25,.005),
      ('working_model','_cons','estimate',7.441516,.0000005),
      ('working_model','_cons','standard_error',.0241219,.00000005),
      ('working_model','_cons','z',308.50,.005),
      ('working_model','_cons','normal95_lower',7.394238,.0000005),
      ('working_model','_cons','normal95_upper',7.488795,.0000005),
      ('unit_sandwich_g_adjusted','lnwg','estimate',.0827436,.00000005),
      ('unit_sandwich_g_adjusted','lnwg','standard_error',.0292684,.00000005),
      ('unit_sandwich_g_adjusted','lnwg','z',2.83,.005),
      ('unit_sandwich_g_adjusted','lnwg','p_value',.005,.0005),
      ('unit_sandwich_g_adjusted','lnwg','normal95_lower',.0253785,.00000005),
      ('unit_sandwich_g_adjusted','lnwg','normal95_upper',.1401086,.00000005),
      ('unit_sandwich_g_adjusted','lnwg','slope_wald_chi2_1',7.99,.005),
      ('unit_sandwich_g_adjusted','_cons','estimate',7.441516,.0000005),
      ('unit_sandwich_g_adjusted','_cons','standard_error',.0795795,.00000005),
      ('unit_sandwich_g_adjusted','_cons','z',93.51,.005),
      ('unit_sandwich_g_adjusted','_cons','normal95_lower',7.285543,.0000005),
      ('unit_sandwich_g_adjusted','_cons','normal95_upper',7.597489,.0000005),
    ]
    for method, parameter, key, target, half in facts:
      actual = float(coefs[method,parameter][key])
      if abs(actual-target) > half:
        raise AssertionError(('author printed fact', method, parameter, key, actual, target, half))
    assert abs(float(fit['scale'])-.0803055) <= .00000005
    print(f'float32-input hypothesis: {len(facts)+1} author-log facts within printed precision')


def main():
  parser = argparse.ArgumentParser()
  parser.add_argument('native', type=Path)
  parser.add_argument('reference', type=Path)
  parser.add_argument('source', type=Path, nargs='?')
  args = parser.parse_args()
  for name in FILES:
    actual = rows(args.native/f'panel_independent_gee_{name}.csv')
    expected = rows(args.reference/f'{name}.csv')
    if name == 'fit':
      assert len(actual) == 1
      assert actual[0]['convention'] == 'rss_over_n_independent'
      assert actual[0]['unit_covariance'] == 'g_over_g_minus_1_unit_sandwich'
      assert actual[0]['inference'] == 'normal_z_chi2_1'
      assert actual[0]['gee_join_key'] == '1'
    compare(actual, expected, name)
  if args.source:
    data = args.source.read_bytes()
    if hashlib.sha256(data).hexdigest() != SHA:
      raise AssertionError('MOM.dat hash mismatch')
    tokens = data.split()
    if len(tokens) != 5320*8:
      raise AssertionError('MOM.dat source shape mismatch')
    actual = rows(args.native/'panel_independent_gee_predictions.csv')
    if len(actual) != 5320:
      raise AssertionError('source row count mismatch')
    for index, row in enumerate(actual):
      if float(row['source_row']) != index+1:
        raise AssertionError(('source_row', index, row['source_row']))
      for column, token in zip(SOURCE, tokens[index*8:index*8+8]):
        if float(row[column]) != float(token):
          raise AssertionError(('source mismatch', index, column, row[column], token))
    print('original source: 5320 rows and 42560 fields exactly preserved')
    check_log([{key: row[key] for key in ('source_row', *SOURCE)} for row in actual])


if __name__ == '__main__':
  main()
