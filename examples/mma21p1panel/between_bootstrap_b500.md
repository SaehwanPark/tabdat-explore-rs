# First-program B500 Between pairs bootstrap

Target C21-P1-BE-BOOT, mma21p1panfeandre.do lines183–185: original MOM5,320
rows/532 complete ten-year people, lnhr on intercept/lnwg Between regression.
[Script](between_bootstrap_b500.td), [R reference](between_bootstrap_reference.R),
[auditor](check_between_bootstrap_reference.py), [source hashes/inventory](inventory.md).

## Estimand and compatibility

Compute one mean record per person in ascending numeric ID order, then resample
those records with replacement, equal copy weights, B500/seed10001. On complete
panels this is whole-person pairs resampling and re-estimating the Between fit:
copying all ten observations preserves that person's means. Copies retain their
multiplicity in the OLS mean-record sample, not merged duplicate-ID clusters.
Both intercept and slope refit on every draw. Use B-1 empirical covariance/SE,
bootstrap mean/bias and R-type7 percentile95 intervals; no bootstrap t/p or causal
interpretation is added. Independently sampled people are the sampling units.

The native IID bootstrap uses its continuing xoshiro256++/SplitMix64/u64-rejection
stream. This differs from the native per-rep whole-cluster stream and Stata RNG.
Historical repeated-ID conventions/SE/draw/all-interval parity are not established.
The API reports summaries, not serialized native draws/refit coefficients; the
R oracle recomputes500 full QR fits, but we do not claim direct per-draw comparison.

The original B200/manual-program `between_bootstrap.td` stays byte-for-byte
unchanged. Its default R/auditor calls still validate B200; explicit R argument
`500` and auditor `--b500` select this separate target. The new export is
`panel_between_bootstrap_b500_predictions.csv`, so both workflows coexist in one
directory. Bootstrap is read-only with respect to the retained original OLS fit;
subsequent predictions remain its point predictions, not the final bootstrap draw.
No Rust/parser/kernel/schema/dependency change or production R/Python dependency.

## Scope, failures and evidence

The source script requires3..532 complete T10 people, exact bounded id/year/
source_row keys, finite abs(lnhr/lnwg)<=20 and existing native identification/work
limits. No model-case deletion is hidden: missing/incomplete source panels fail
before fit. This bounded example is not a new general native Between estimator.

Independent R4.6.2 raw means/full QR with the separate Python draw oracle validates
all18 point/mean/bias/covariance/SE/interval/size quantities at fixed abs1e-9.
Original bootstrap intercept/slope SEs are0.06402134297624726/0.02366971303096997.
All532 ordered mean/design/prediction rows (4,256 cells) agree; all5,320 loaded
rows/42,560 raw fields are pinned to the original SHA256/input values. No tolerance
was widened. A16-person/interleaved control adds18 R fields/128 prediction cells;
its checked-in fixtures regenerate with the heterogeneous generator used by the
[classical-intercept control](classical_intercept.md).

Two new actual-script tests cover independent control values, original point
predictions, JSON/text byte parity, refused overwrite, unchanged source files,
duplicate/missing/incomplete input and an unidentified draw. For the three-person
failure control the independent oracle's **first** draw is[0,0,0], rank1 instead
of2; native bootstrap aborts without deleting/redrawing it and produces no export.
Export collision preserves its owner; the script is not a session-wide transaction.
Original-source B500/B200 R/auditor runs pass, both text exports match JSON bytes,
and seven forged B/seed/SE/covariance/interval/prediction/source records are rejected.

Targeted Clippy, fmt, syntax and diff checks pass. The native primitive and existing
Rust consumers are unchanged from the current base's combined1,820-test validation;
that passing baseline was not rerun. New workflow tests were executed explicitly.
No configured remote CI, other-platform or release certification is claimed.

## Reproduce

Prepare MOM.csv from MOM.dat using [prepare.sh](prepare.sh) in a fresh directory,
with the original source URL/hash in the inventory. Build the native binary first.

```sh
root=/absolute/path/to/tabdat-explore-rs
# Working directory contains original MOM.dat and prepared MOM.csv.
"$root/target/debug/tabdat-explore-rs" --json -f \
  "$root/examples/mma21p1panel/between_bootstrap_b500.td" > between_bootstrap_b500.jsonl
Rscript "$root/examples/mma21p1panel/between_bootstrap_reference.R" \
  MOM.dat reference_b500 500
python3 "$root/examples/mma21p1panel/check_between_bootstrap_reference.py" \
  . reference_b500 MOM.dat --b500
```

Use `--text` in a separate fresh directory. For a labelled control, regenerate
`fe_level_bootstrap_synthetic.py heterogeneous fresh_directory`, pass its MOM.dat
to the R reference with500, and add `--synthetic --b500` to the auditor. Only the
original-file hash gate is disabled; source-field/sample/reference checks remain.

This closes the native B500 replication-count gap, not full first-program/table/
chapter/book coverage or historical Stata parity. Next priority: companion-program
inventory/source alignment and remaining complete comparison/reporting needs,
selected against the ledger before further implementation.
