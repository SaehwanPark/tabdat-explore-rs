# Patents native conditional-Poisson FE whole-firm bootstrap

The [native source fit](native_conditional_fe.md) now supports a complete native
whole-positive-firm B500 bootstrap. The former256-positive-group limit rejected
the original324 retained firms; only this conditional-FE limit is raised to512.
The [SQL bootstrap](conditional_fe_bootstrap.md) stays valid and separate, using
a different stream construction. Neither route claims historical Stata draws,
duplicate-ID treatment, SEs or intervals. This is not whole-Table23.1 coverage.

## Sample, model and inference

Cameron–Trivedi(2005) §23.3 patents companion program, `patr7079.asc`,
SHA256 `87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
The [source script](native_conditional_fe_bootstrap.td) preserves the existing
canonical numeric/key/count/finite-metadata guards:1730 records/346 firms,
five unique1975–79 years each, integer PAT0..1000 and finite |LOGR|<=20.
Nullable ARDSSIC is nonmodel metadata; SUMPAT is not a fit filter.

Conditional Poisson FE has sole LOGR slope and no common intercept, weights,
offsets or year effects. Joint-finite selection then excludes22 all-zero
firms/110 rows, leaving1620 records/324 positive-total firms. Resample these324
firms with replacement500 times, seed10001, copying all five records for each
occurrence. Repeated selections get distinct FE identities; each replicate
refits the slope. A failed refit aborts, never omits or redraws a replicate.

Bootstrap output is read-only: point, bootstrap mean/bias/SE, full scalar B−1
covariance and type7 percentile95 limits. These are whole-firm pairs summaries,
not the model information inverse, a new normal/t test or analytic cluster
covariance. Finite B has Monte Carlo error; interpretation requires appropriate
independent-firm sampling/regularity assumptions, not survey/causal guarantees.

Group order is first seen among positive-total complete cases. This source
script canonically orders id/year before fitting. One local continuous
`xoshiro256++/splitmix64/u64-rejection-v1` stream supplies every draw; source-
permuted input therefore retains the same stream and coefficient gates.

The actual original FE model remains installed. Later means/residuals use its
retained positive-group normalization, not a replicate, a new current-subset
total or a population/new-firm forecast. All1730 original records remain in the
export:110 zero-total exclusions have NULL means/residuals, while complete-X
slope-only xb is available. Data/labels/named reports remain unchanged by the
bootstrap; no restorable bootstrap model is created. The script is ordered,
not whole-transactional; an export failure does not roll back earlier fits.

## Supported limits

Native conditional FE bootstrap: B2..500,1..16 slopes,2<=G<=512 with G>K,
G*largest_group_size<=40000 and G*largest_group_size*B*K²<=2500000000.
At most G+1024 RNG words are allowed per replication. Original fit/key/sample/
rank/numerical/convergence guards remain in force. Only the positive-group cap
and its error text changed; no formula, RNG, API/schema or dependency changed.
Pooled-Poisson and gamma-RE bootstrap still cap G at256. This source replay is
narrower than the generic bootstrap: at most346 canonical firms/1730 records
and five years per firm. Raising the generic cap does not remove these guards.

## Reproduce original data

Build using pinned Rust1.99.1 and rustup shims (`export PATH="$HOME/.cargo/bin:$PATH"`):

```sh
cargo build --locked -p tabdat-explore-rs
repo=$PWD
work=$(mktemp -d)
curl -fL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
sh examples/mma23p1pannonlin/prepare.sh "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma23p1pannonlin/native_conditional_fe_bootstrap.td" > result.jsonl)
python3 examples/mma23p1pannonlin/check_native_conditional_fe_bootstrap.py oracle \
  "$work" "$work/patr7079.asc" "$work/draws.csv"
Rscript examples/mma23p1pannonlin/conditional_bootstrap_reference.R \
  "$work/patr7079.asc" "$work/draws.csv" "$work/reference"
python3 examples/mma23p1pannonlin/check_native_conditional_fe_bootstrap.py check \
  "$work" "$work/patr7079.asc" "$work/reference"
```

Preparation, draw and reference destinations must be fresh/no-clobber. `--text`
in another fresh work directory gives a readable report and byte-identical
`native_conditional_fe_bootstrap_predictions.csv`. R/Python and publisher data
are development-only, not production dependencies. See the [small frozen control
and regeneration steps](../../tests/fixtures/mma23_native_conditional_fe_bootstrap/README.md).

Original native slope is−.0377642353572117; B500 mean−.03949474486351266,
bias−.0017305095063009646, SE.10611640537009748,
variance.011260691488670854, percentile95
[−.25644527423070357,.1568733681907946]. Do not confuse this resampling SE with
the original model-based SE.03255176135039196 or interpret the interval as a
bootstrap p-value. No historical author-uncertainty match is claimed.

## Independent validation and failure evidence

Unchanged R4.6.2 `conditional_bootstrap_reference.R` solves raw conditional
score roots for all500 replicates and cross-checks three with weighted dummy-
Poisson GLMs. It receives only raw data and independently checked draw indices,
not native coefficients. Source-specific [auditor](check_native_conditional_fe_bootstrap.py)
checks all162000 draws against the independent Python RNG, exact labels/counts/
zero-total exclusions,11 R fields/full scalar covariance and every R refit.
Maximum original slope difference1.20e-11 passes fixed absolute1e-9; no tolerance
widening. The R reference's [−10,10] score-root bracket is a checked original/
control oracle domain, not a new generic native-estimator restriction.

Raw per-replicate conditional scores, empirical B−1 variance/SE/mean/bias and
sorted type7 quantiles independently protect against shared wrong native/R
artifacts. A dimensionless covariance/empirical-variance ratio uses1e-9; zero
empirical variance must remain exactly zero, not be dropped. Original source/
mean normalization, likelihood/information identities and xb/residuals are
checked separately. All1730 post-bootstrap records are byte-identical to the
independently audited native source-fit export, including110 NULL exclusions.

Verified scope:

- Modified/new kernel module4/4:257/324/512 positive groups accepted,513 rejected,
  closed-form refits/zero-total exclusions, copied-row/work budgets, singular-
  replicate aborts and existing invalid-contract cases. Fresh affected stats-
  crate308 pass/one pre-existing skip (58 binaries,12.440s).
- Runtime bootstrap contracts3/3: original model/data/labels/named reports,
  exact wide keys, selected samples and failure preservation. No later kernel
  change invalidates these runs.
- New [actual-script Rust tests](../../tests/mma23_native_conditional_fe_bootstrap_workflow.rs)
  3/3: all11 frozen R fields/500 refits, original-owned predictions, JSON/text,
  file collision, reversed/all-null metadata and a257-positive-firm B500 replay.
- [Python tests](test_check_native_conditional_fe_bootstrap.py)3/3: actual control,
  exact constant-refit/zero-variance checks (also matched by base-R SD),
  and26 rejected forgeries. Matching native/R wrong covariance/SE passes artifact
  equality only when empirical checks are deliberately disabled; the real audit
  rejects it. Jointly shifted native/R refits fail raw score equations. Wrong
  draws/labels/counts/exclusions, normal intervals, source/prediction corruption
  and duplicate/truncated reference rows fail.
- Original and60-row/12-firm control each pass500 independent R refits and three
  weighted dummy-GLM checks. Control retains50 rows/ten positive firms with5000
  independently checked draws. Frozen data consist only of this existing
  synthetic control's11 fields/500 coefficients, not publisher records.

The old executable's original324-group rejection was reproduced. A temporary
extraction probe initially assumed integer-formatted count tokens; corrected
validation accepts exact integer-valued decimals such as32.00000. No production
count semantics or tolerance changed. Final author review reproduced an auditor-
only false rejection for exactly constant refits: unanchored sum/500 rounded the
mean by one ulp and created phantom variance. Anchoring the mean preserves exact
zero; a dedicated positive/negative/zero control rejects nonzero forged variance,
base-R SD confirms zero, and all three Python cases/original audit pass again.
No production inference or tolerance changed. Scoped stats-all-target/new-root-
target Clippy and fmt/diff checks pass. Verification logs/references, including original
readable/JSON output, are `/tmp/tabdat-ch23-native-fe-bootstrap/`. No new full-
workspace/doctest/CI/platform/release or independent-code-review pass is claimed;
no Rust doc examples changed. Existing unrelated all-target root Clippy precision
warning remains outside scope. Fresh affected reporting58/58 and reverse-
consumer CLI discovery29/29 pass after the help-cap update; run IDs/logs are
recorded in the PR.

## Remaining required work

This closes the bounded native original-data bootstrap barrier, not the entire
chapter/graduate core/book. Native gamma/pooled source-sized bootstrap limits,
Gaussian FE/RE automatic resampling, native normal-Poisson RE/bootstrap, GEE
bootstrap, general analytic clustered/weighted/offset conditional FE, named
source tables, historical draws/SE/interval/repeated-ID reconciliation and the
complete illustration inventory remain required where applicable. The
[37-block inventory](inventory.md) keeps those distinctions explicit.
