# Patents native shared-gamma RE workflow: preimplementation contract

**In progress, not fully verified/delivered yet.** Medium risk: new source workflow
and artifact/state contracts on an existing estimator; no numerical kernel,
public Rust/CLI schema, dependency or existing composition change expected.

## Decision and exact outcome

Following PR433, prioritize a practical graduate panel-count workflow rather
than requiring the remaining descriptive entries to precede estimation work.
The §23.3/Table23.1 [SQL gamma-RE composition](gamma_re_poisson.md) already has
an independent oracle but leaves the *pooled Poisson* model retained. Connect
the existing native `xtpoisson pat logr, re id(id)` to this exact illustration:
load/audit → source-key-ordered preparation → native joint fit/diagnostic LR →
owned marginal prediction/interpretation → text/JSON report/CSV reproduce.
After success the retained model must be **shared-gamma RE**, so subsequent
native prediction uses RE rather than the earlier pooled starting fit.

This is not a new estimator family or whole-table parity. The old composition
remains unchanged as independent method/diagnostic evidence, with its state
limitation clearly distinguished from this native workflow.

## Source, design and inference

Use existing `prepare.sh` and `patents.csv` projection of `patr7079.asc`, SHA256
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Nine fields, `id year cusip ardssic scisect logk sumpat logr pat`; original id is
source-record ordinal, not CUSIP. Retain all346 firms/1730 observations in
1975–79, including22 zero-total firms and missing nonmodel ARDSSIC. `sumpat`
is preserved metadata, not silently replaced by the selected five-year total.

The bounded script admits2..346 firms, each with the five unique years,
positive exact integer id<=346, pat finite nonnegative integer<=1000 and
finite |logr|<=20. Preserve original numeric source values/missingness; canonical
CSV pat may be imported DOUBLE because prepare.sh writes decimal integer
values. Validate integer-valued counts without silently rounding. No missing
model rows are silently removed by this workflow. Source fields remain intact,
new selected-firm-total/RE prediction columns have distinct names; sort id/year.
No user data or existing export owner is overwritten.

Native mean exp(intercept+beta*logr), shared mean-one Gamma firm effect with
variance alpha, alpha independent of regressors; jointly estimate intercept,
logr and lnalpha. Include factorial constants in integrated likelihood. Use
inverse **full joint observed information**, correction1, asymptotic normal95,
alpha delta SE and exponentiated log-Wald interval. No G−1 t, cluster sandwich,
row-NB substitution, weights, offset, year effects, posterior firm effects or
causal claim. lnalpha=0 tests alpha=1, **not** zero random-effect variance.
The native refitted same-sample pooled likelihood supplies the alpha=0 equal
chi-square0/1 boundary-mixture LR. Distinguish numerically underflowed p=0.
Existing bounded native convergence/failure semantics stay unchanged.

Predictions: `n` is population marginal mean, `xb` log mean, residuals pat−mean.
Native inference/predictions and every selected source key must match independent
R, not the old pooled predictions. Script is ordered, not whole-transactional:
loading/preparation clears a previous fit; later export failure retains successful
fit/data/prediction changes. A failed fit/prediction itself must preserve prior
owned model/data/labels and no half-created destination column.

## Acceptance (five observable cases)

1. Original pinned source: all1730 source/prepared/prediction records and actual
   key membership, all346 firms/22 zero-total firms, joint full covariance and
   **37 existing independent R quantities**, plus xb reconstruction. Preserve
   fixed `1e-9+1e-10*abs(reference)` comparisons and relative1e-9 nonzero p gates.
   R4.6.2 base-R integrated likelihood/score/Hessian plus NB-total/multinomial
   factorization is the oracle; reuse existing generator unchanged.
2. Existing inspectable12-firm/60-row source/control and R reference under
   `tests/fixtures/mma23_{conditional_fe,gamma_re}`: all source/model/prediction
   fields, zero firms, both text/JSON and byte-identical exports; permutation
   must restore canonical id/year ordering without changing values/model.
3. Native retained model is GammaPoisson, not Poisson; subsequent prediction
   after workflow success still uses owned RE coefficients. Read-only summary,
   labels and selected active source state are preserved; collisions refuse
   without overwriting existing columns/files or silently refitting.
4. Invalid/missing/nonfinite/fractional/out-of-range data, duplicate keys,
   missing year/incomplete firm and singular design fail before prediction/
   export. A valid count sample with a noninterior dispersion optimum fails
   without replacing the prior pooled fit/data. Do not mistake an output-file
   collision for successful input validation.
5. Real-script independent auditor controls and targeted source/key/sample,
   covariance/parameterization/inference, shared wrong native/R statistic and
   model-ownership/prediction forgeries are rejected. Fixed historical selected
   printed gates remain separate; do not claim printed lnalpha agreement.

## Implementation order and verification

1. Add `native_gamma_re.td`, using existing load/isid/SQL/source guards,
   `xtpoisson`, `predict`, `summarize` and `export`. No parser/runtime changes.
2. Add `check_native_gamma_re.py` and real-script corruption tests; compare
   native structured output to unchanged R, original source and row exports,
   including full matrix/order/convergence and mean-prediction identities.
3. Add `tests/mma23_native_gamma_re_workflow.rs` against existing tiny frozen
   inputs/references; run it explicitly and new Python tests first. Then check
   affected root scope/reverse consumers under the no-repeated-baseline policy,
   scoped Clippy/fmt/diff; broaden if any existing code/helper/schema changes.
   Reproduce original JSON/text modes with exact-source audit, update this doc,
   README/SPEC/roadmap/ledger, review, push/merge and clean both branches.

Stop instead of improvising if native/reference sample, parameterization,
likelihood, covariance or convergence disagree. Diagnose; do not widen tolerances
or turn this bridge into a new-kernel refactor without revising the bounded plan.
Implement exactly this plan. Do not broaden scope. Report files, tests, deviations
and unresolved risks before merge.

Historical source lnalpha .5454692 versus converged float64 .5454693087 remains
unresolved literal parity; the prior float32 probe is not an alternative oracle.
Remaining illustrated normal-RE/GEE/bootstrap outputs and chapter/book inventory
work, I-DETAIL/I-FREQ/I-GRAPH, household ML SE and URBAN/MEDEXP/LNHHEXP
reconciliation all remain required. No missing book source or unavailable Stata build is manufactured.

## Implementation checkpoint (not merge-ready)

Native script and independent auditor now pass the original1730-row/346-firm
source and existing60-row/12-firm control, each with all37 unchanged R fields,
full joint covariance, every source/prediction record and key/order. Additional
Python likelihood/score/observed-information-inverse identities, normal-tail/
alpha/LR relationships and mean/xb/residual identities also pass. No native
kernel or reference formula/tolerance change. R4.6.2 original max score3.27e-12,
independent factorized-likelihood difference1.00e-11; native original converges
in6 iterations, scaled score8.54e-15 and scaled Newton step2.37e-14.

The first manual audit caught repeated guard/scope rows; the source statements
now aggregate to one row (guard reports actual firm count), avoiding1730 repeated
messages. Corrected runs live under `/tmp/tabdat-ch23-native-gamma/` as
`original-json-v2`, `control-json` and `reference`.

Automated script/state/failure/export/permutation tests and forged-artifact
rejections, original text/export equivalence, historical selected gates, final
impact-scoped checks/review and public-status reconciliation remain. Do not
merge or claim the whole slice validated from these two positive runs alone.
