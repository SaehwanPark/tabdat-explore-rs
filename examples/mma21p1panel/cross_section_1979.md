# WIP: Chapter21 1979 cross-sectional regression

## Contract before implementation

Reproduce author residual program line114: intercept OLS of 1979 lnhr on lnwg
across532 people after the complete balanced MOM reshape. Compose this from
existing load/SQL/OLS/lincom/test/predict/export; no new native API. Input is3..532
complete unique ten-year people with exact finite bounded keys and lnhr/lnwg;
select 1979 before fitting, preserving source keys/order. Report source sample,
coefficients/full classical RSS/(N−2) covariance, t and95% slope CI, joint slope
zero F(1,N−2), and fitted/residual rows. Exercise owned OLS state after SQL
materialization. This is a descriptive cross-section, not panel FE or a causal
claim.

Acceptance: (1) original N532, independent R QR full25 coefficient/covariance/
inference/lincom fields, test F/p and all532 source/prediction rows, fixed abs1e-9
and p-relative1e-8/exact underflow0; (2) independent author printed facts at
stated precision, with separate float32 reconstruction if raw double crosses a
printed boundary and no widened R tolerance; (3) interleaved dyadic control and
orthogonal exact-zero slope control; (4) text/JSON and retained state. Important
failures: duplicate/incomplete/missing/nonfinite/invalid key, rank deficiency and
export collision must fail without overwriting source or existing owner.
Preserve explicit logged sample and inference; no claim of complete residual or
chapter inventory. Independent R/Python are development-only.
