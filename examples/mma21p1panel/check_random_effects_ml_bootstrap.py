#!/usr/bin/env python3
"""Audit every native RE-ML refit/draw, exact source field and bootstrap summary."""
import csv
import hashlib
import importlib.util
import math
from pathlib import Path
import sys

FIELDS = ['lnhr', 'lnwg', 'kids', 'ageh', 'agesq', 'disab', 'id', 'year']
RNG = 'xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1'


def rows(path):
  with open(path, newline='') as f:
    reader = csv.DictReader(f)
    assert reader.fieldnames and len(set(reader.fieldnames)) == len(reader.fieldnames), path
    result = list(reader)
    assert result and all(None not in r and None not in r.values() for r in result), path
    return result


def integer(value):
  number = float(value)
  assert math.isfinite(number) and number.is_integer(), value
  return int(number)


def close(actual, expected, context, tolerance=1e-9):
  a, e = float(actual), float(expected)
  assert math.isfinite(a) and math.isfinite(e), (context, a, e)
  assert abs(a-e) <= tolerance, (context, a, e, tolerance)


def audit(native, reference, source, synthetic=False):
  raw = source.read_bytes()
  if not synthetic:
    assert hashlib.sha256(raw).hexdigest() == '2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
  tokens = raw.split()
  assert tokens and len(tokens) % 8 == 0
  original = [dict(zip(FIELDS, map(float, tokens[i:i+8]))) for i in range(0, len(tokens), 8)]
  ids = sorted({r['id'] for r in original})
  assert 3 <= len(ids) <= 532 and len(original) == 10*len(ids)
  assert len({(r['id'], r['year']) for r in original}) == len(original)
  assert all(math.isfinite(v) for r in original for v in r.values())
  assert all(r['id'] == int(r['id']) and 1 <= r['id'] <= 532 and r['year'] == int(r['year']) and 1979 <= r['year'] <= 1988 for r in original)
  assert all(sum(r['id'] == id for r in original) == 10 for id in ids)
  rfits = rows(reference/'refits.csv')
  assert [int(r['replicate']) for r in rfits] == list(range(501))
  rfits = {int(r['replicate']): r for r in rfits}
  oracle_path = Path(__file__).resolve().parents[2]/'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py'
  spec = importlib.util.spec_from_file_location('xoshiro_oracle', oracle_path)
  oracle = importlib.util.module_from_spec(spec)
  spec.loader.exec_module(oracle)
  draws = rows(native/'panel_re_ml_bootstrap_draws.csv')
  expected_fields = {'replicate','copy_id','step','person_id','intercept','slope','sigma_e_squared','sigma_u_squared','eta','log_likelihood','profile_score','profile_information','estimate_intercept','estimate_slope','people_per_draw','base_seed'}
  assert set(draws[0]) == expected_fields
  assert len(draws) == 500*len(ids)
  refdraws = rows(reference/'draws.csv')
  assert len(refdraws) == len(draws)
  fits = {}
  offset = 0
  for rep in range(1, 501):
    rng = oracle.stream(10001+rep-1)
    step = 0
    zone = oracle.MASK-oracle.MASK % len(ids)
    for position in range(1, len(ids)+1):
      while True:
        step += 1
        assert step <= len(ids)+1024
        value = next(rng)
        if value < zone:
          break
      index = value % len(ids)
      r, rd = draws[offset], refdraws[offset]
      assert [integer(r[k]) for k in ['replicate','copy_id','step','person_id','people_per_draw','base_seed']] == [rep, position, step, int(ids[index]), len(ids), 10001]
      assert [int(rd[k]) for k in ['replicate','position','step','index0']] == [rep, position, step, index]
      parameters = tuple(r[k] for k in sorted(expected_fields-{'replicate','copy_id','step','person_id'}))
      if rep in fits:
        assert fits[rep][0] == parameters, (rep, 'inconsistent repeated fit metadata')
      else:
        fits[rep] = (parameters, r)
        for k in ['intercept','slope','sigma_e_squared','sigma_u_squared','eta','log_likelihood','profile_score']:
          close(r[k], rfits[rep][k], (rep, k))
        close(r['profile_information'], rfits[rep]['profile_information'], (rep,'numerical profile information'), 2e-6)
        e, u, eta, score, info = map(float, (r[k] for k in ['sigma_e_squared','sigma_u_squared','eta','profile_score','profile_information']))
        assert e > 1e-12 and u >= 0 and 0 <= eta < 24
        close(u, e*math.expm1(eta)/10, (rep,'component transform'))
        assert score <= 1e-8 if eta == 0 else abs(score) <= 1e-8 and info > 1e-8
        for k, refkey in [('estimate_intercept','intercept'),('estimate_slope','slope')]:
          close(r[k], rfits[0][refkey], (rep,k))
      offset += 1
  summary = rows(native/'panel_re_ml_bootstrap_summary.csv')
  expected_summary = rows(reference/'summary.csv')
  assert len(summary) == len(expected_summary) == 1
  s, expected = summary[0], expected_summary[0]
  assert set(s) == set(expected) | {'base_seed','rng','method','interval_method'}
  for k, value in expected.items():
    close(s[k], value, ('summary', k))
  assert s['rng'] == RNG and s['base_seed'] == '10001'
  assert s['method'] == 'whole_person_pairs_refit_balanced_gaussian_ml'
  assert s['interval_method'] == 'type7_percentile_95'
  predictions = rows(native/'panel_re_ml_bootstrap_predictions.csv')
  reference_predictions = rows(reference/'predictions.csv')
  assert len(predictions) == len(reference_predictions) == len(original)
  assert set(predictions[0]) == set(FIELDS) | {'source_row','reml_xb','reml_residual'}
  for i, (p, rp, rawrow) in enumerate(zip(predictions, reference_predictions, original), 1):
    assert set(rp) == set(p)
    assert int(p['source_row']) == int(rp['source_row']) == i
    for k in FIELDS:
      assert float(p[k]) == float(rp[k]) == rawrow[k], (i,k,'source mismatch')
    for k in ['reml_xb','reml_residual']:
      close(p[k], rp[k], (i,k))
    close(p['reml_xb'], float(s['estimate_intercept'])+float(s['estimate_slope'])*rawrow['lnwg'], (i,'retained original xb'))
  print(f'PASS: {len(draws)} exact draws, 500 R refits/score/information/component gates, {len(expected)} summary fields, {len(original)} original prediction rows and {8*len(original)} exact raw fields')


if __name__ == '__main__':
  if len(sys.argv) not in (4,5) or len(sys.argv)==5 and sys.argv[4] != '--synthetic':
    raise SystemExit('usage: check_random_effects_ml_bootstrap.py native_directory reference_directory MOM.dat [--synthetic]')
  audit(*map(Path,sys.argv[1:4]), synthetic=len(sys.argv)==5)
