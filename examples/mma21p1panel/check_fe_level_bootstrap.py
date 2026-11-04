#!/usr/bin/env python3
"""Audit every FE level-bootstrap draw/refit, normalization and retained source prediction."""
import hashlib
import importlib.util
from pathlib import Path
import sys
from check_random_effects_gls_refit_bootstrap import rows, integer, close, FIELDS, RNG


def audit(native, reference, source, synthetic=False):
  raw=source.read_bytes()
  if not synthetic:
    assert hashlib.sha256(raw).hexdigest()=='2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
  tokens=raw.split();assert tokens and len(tokens)%8==0
  original=[dict(zip(FIELDS,map(float,tokens[i:i+8]))) for i in range(0,len(tokens),8)]
  ids=sorted({r['id'] for r in original});g=len(ids)
  assert 3<=g<=532 and len(original)==10*g and len({(r['id'],r['year']) for r in original})==10*g
  for id in ids:assert sorted(r['year'] for r in original if r['id']==id)==list(range(1979,1989))
  rfits=rows(reference/'refits.csv');assert [integer(r['replicate']) for r in rfits]==list(range(501))
  rfits={integer(r['replicate']):r for r in rfits}
  path=Path(__file__).resolve().parents[2]/'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py'
  spec=importlib.util.spec_from_file_location('xoshiro_oracle',path);oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
  draws=rows(native/'panel_fe_level_bootstrap_draws.csv');rdraws=rows(reference/'draws.csv')
  assert len(draws)==len(rdraws)==500*g
  fitfields={'intercept','slope','sample_mean_lnwg','sample_mean_lnhr','residual_df','within_rss','normalized_effect_mean'}
  keys={'replicate','copy_id','step','person_id'}
  expected=keys|fitfields|{'estimate_intercept','estimate_slope','people_per_draw','base_seed'}
  fits={};offset=0
  for rep in range(1,501):
    rng=oracle.stream(10001+rep-1);step=0;zone=oracle.MASK-oracle.MASK%g
    for position in range(1,g+1):
      while True:
        step+=1;assert step<=g+1024
        value=next(rng)
        if value<zone:break
      index=value%g;r=draws[offset];rd=rdraws[offset]
      assert set(r)==expected
      assert [integer(r[k]) for k in ['replicate','copy_id','step','person_id','people_per_draw','base_seed']]==[rep,position,step,int(ids[index]),g,10001]
      assert [integer(rd[k]) for k in ['replicate','position','step','index0']]==[rep,position,step,index]
      payload=tuple(r[k] for k in sorted(expected-keys))
      if rep in fits:assert fits[rep]==payload,(rep,'inconsistent refit fields')
      else:
        fits[rep]=payload
        for k in fitfields:close(r[k],rfits[rep][k],(rep,k))
        assert integer(r['residual_df'])==9*g-1 and float(r['within_rss'])>=0
        assert abs(float(r['normalized_effect_mean']))<=1e-10*(1+abs(float(r['intercept'])))
        close(r['intercept'],float(r['sample_mean_lnhr'])-float(r['slope'])*float(r['sample_mean_lnwg']),(rep,'level normalization'),1e-12)
        close(r['estimate_intercept'],rfits[0]['intercept'],(rep,'original intercept'))
        close(r['estimate_slope'],rfits[0]['slope'],(rep,'original slope'))
      offset+=1
  s=rows(native/'panel_fe_level_bootstrap_summary.csv');rs=rows(reference/'summary.csv')
  assert len(s)==len(rs)==1;s,rs=s[0],rs[0]
  assert set(s)==set(rs)|{'base_seed','rng','method','interval_method'}
  assert s['base_seed']=='10001' and s['rng']==RNG
  assert s['method']=='whole_person_pairs_refit_normalized_level_fe' and s['interval_method']=='type7_percentile_95'
  for k,v in rs.items():close(s[k],v,('summary',k))
  p=rows(native/'panel_fe_level_bootstrap_predictions.csv');rp=rows(reference/'predictions.csv')
  assert len(p)==len(rp)==len(original)
  for i,(a,b,src) in enumerate(zip(p,rp,original),1):
    assert set(a)==set(b)==set(FIELDS)|{'source_row','fe_xb','fe_xbu','fe_u','fe_residual'}
    assert integer(a['source_row'])==integer(b['source_row'])==i
    for k in FIELDS:assert float(a[k])==float(b[k])==src[k],(i,k,'raw source')
    for k in ['fe_xb','fe_xbu','fe_u','fe_residual']:close(a[k],b[k],(i,k))
    close(a['fe_xb'],float(s['estimate_intercept'])+float(s['estimate_slope'])*src['lnwg'],(i,'retained xb'))
    close(a['fe_xbu'],float(a['fe_xb'])+float(a['fe_u']),(i,'conditional mean'))
  print(f'PASS: {500*g} exact draws, 500 copied-raw R QR refits/seven fields, {len(rs)} summary fields, {len(p)} retained FE prediction rows/{8*len(p)} exact raw fields')


if __name__=='__main__':
  if len(sys.argv) not in (4,5) or len(sys.argv)==5 and sys.argv[4]!='--synthetic':
    raise SystemExit('usage: check_fe_level_bootstrap.py native_directory reference_directory MOM.dat [--synthetic]')
  audit(*map(Path,sys.argv[1:4]),synthetic=len(sys.argv)==5)
