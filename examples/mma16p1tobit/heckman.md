# Corrected Heckman two-step selection workflow

This bounded native terminal/script workflow is independently validated. It is
not whole-Chapter16/release certification or a published coefficient-table replay.
Exact author analysis:
`mma16p1tobit.do` (March2005) §2A, associated with MMA §16.2.1 pp530–531.
The same original export has200 selection observations and130 observed outcomes:
selection `dy ~ lnwage`, outcome `ytrunc ~ lnwage`, intercept in both.
Use the existing [source assets/converter](README.md); no replacement RNG/data.
The separate later RAND selection illustration remains required.

The author explicitly warns that manual second-stage OLS SEs are incorrect.
This reference uses probit observed ML covariance and the conventional model-based
Heckman/Greene correction for generated Mills regressors, not ordinary/HC OLS.
Let t=Z gamma, lambda=phi(t)/Phi(t), delta=lambda*(lambda+t),
W=[X,lambda], alpha=the Mills coefficient and A=W'W on selected rows. Then:

- sigma²=selected RSS/N_selected + alpha²*mean_selected(delta), rho=alpha/sigma;
- F=W'diag(delta)Z;
- V_outcome=A^-1[W'diag(sigma²-alpha²*delta)W
  +alpha²*F V_probit F']A^-1.

There is no residual-DF multiplier. Normal95 inference is reported separately for
selection coefficients and augmented-outcome coefficients (including alpha).
Sigma/rho are points only; no cross-equation restrictions, joint ML likelihood,
scale/correlation SEs or prediction uncertainty are invented. Native planning
requires regular interior abs(rho)<1, with an explicit failure rather than
Stata-style ad-hoc clipping/covariance adjustments.

## Independent development reference

R4.6.1 `glm`/`lm`, base-R `deriv3` observed-Hessian/Newton refinement, and the
**unmodified external** sampleSelection1.2-16 `heckitVcov` function provide the
reference. This is **not** a claim that the full sampleSelection package is
installed. Its unrelated imports are unnecessary for this standalone covariance
function. It is GPL>=2 development-only source, not copied into the repository,
linked into native production, or a new Rust/runtime dependency.

Pinned CRAN-mirror commit: `5115e01a03d09a46ac1b92813c785c5799b4d654`.
`heckitVcov.R` SHA256:
`7506c7a0d7151f909ecc41d4597bfb11f21dc21222f561729fea79fc7decc769`.
The script additionally checks its exact MD5 via base-R tools before sourcing.

After preparing the original source in a fresh directory:

```sh
repo=/absolute/path/to/tabdat-explore-rs
curl -fsSL https://raw.githubusercontent.com/cran/sampleSelection/5115e01a03d09a46ac1b92813c785c5799b4d654/R/heckitVcov.R -o heckitVcov.R
printf '%s\n' '7506c7a0d7151f909ecc41d4597bfb11f21dc21222f561729fea79fc7decc769  heckitVcov.R' | shasum -a256 -c
Rscript --vanilla "$repo/examples/mma16p1tobit/reference_heckman_two_step.R" \
  mma16p1tobit.asc R_heckman_predictions.csv heckitVcov.R > R_heckman_quantities.csv
```

Execute the native workflow in that same prepared directory (no R/Python is
needed by the native commands):

```sh
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
cp "$repo/examples/mma16p1tobit/heckman.td" .
"$repo/target/debug/tabdat-explore-rs" --json -f heckman.td > heckman.jsonl
python3 "$repo/examples/mma16p1tobit/check_heckman.py" .
python3 "$repo/examples/mma16p1tobit/check_heckman_failures.py" .
```

The two Python commands are optional development audits. Native export refuses
an existing target; use a fresh prepared directory for another run (including
`--text`), rather than overwriting results. The checker reuses standard-library
normal inference helpers from `check_tobit.py`; no production Python dependency.

The reference emits58 exact-inventory quantities (13 covariance entries),1200
point predictions and1400 original cells/ordinals. First-stage log likelihood
is−105.296713010228; outcome intercept745.306997010131, slope176.646780648963,
alpha−498.995959534882; corrected SEs1617.58343858516,425.002572278714,
760.500897353647; sigma740.143348254785 and rho−.674188264626690.
Native validation will target these declared float64/reference conventions, not
claim exact historical Stata storage/stopping/digit parity. Scalar gates planned
at1e-9 absolute+1e-8 relative, positive tails1e-8 relative and probabilities1e-10
absolute; investigate differences rather than widening them.

Current evidence: reference execution and exact schema/inventory checks pass;
independent algebra reconstructs the covariance correction; source cells and all
1200 prediction formulas check independently. Existing-output, modified external
reference, truncated-input, nonbinary-selection and selected-missing-outcome
cases reject with empty stdout and no new prediction artifact. Four existing
input/output/reference fingerprints remain unchanged. Initial top-level chol
printing and named-scalar field suffixes were caught and fixed; their failed
CSV/inventory diagnostics are retained, not counted as passing runs.

## Kernel and terminal workflow validation

`crates/tabdat-stats/src/heckman.rs` exports `fit_heckman_two_step` and
`heckman_predict_row`. Inputs are explicit eligible-selection and selected-outcome
problems; supplied original samples must match the exact ordered subset. Weights/
clusters, deficient rank, invalid flags, nonregular sigma/rho and unsupported
represented Mills values fail rather than silently substituting inference.
Bounds: at most10000 eligible rows,32 parameters per stage (including Mills),
N*q²+N_selected*p²<=20million. Intercept fits center at min(selected outcome),
so the scale is the observed selected range; otherwise max(abs(y)). Relative
sigma<=1e-12 is numerically degenerate. No ridge, pseudoinverse or rho clipping.

Two independently differentiated R glm/lm/pinned-covariance controls validate98
quantities (26 covariance entries) and3600 helper predictions, with/without
intercepts and an excluded selection predictor. Seven kernel tests also cover
exact samples, rank/separation/bounds/iteration/scale/rho failures, affine outcome
units (including1e-9), large intercept shifts, row permutation, IID replication
and covariance1/replication scaling, and equation-specific/tail predictions.
On the original source,55 numerical quantities plus3 separately bounded,
coordinate-dependent solver certificates pass, including13 covariance entries,
1000 core-helper predictions and200 fitted-selection linear indices;1400
original cells/ordinals remain exact. This is a kernel probe, **not DuckDB/CLI
execution or a completed analysis workflow**.

A frozen-source independent read-only statistical review of `ad664c5` found no
issues; it did not execute tests. Parent current-code checks passed: seven
focused tests,192 passing stats-crate tests across45 hash-checked current artifacts
plus one pre-existing ignored external-source test, all-target stats Clippy,
workspace format/diff checks and zero stats doctests. The original broad nextest
attempt compiled then timed out in discovery with no test outcomes; it remains
incomplete. The bounded direct-artifact fallback verified all45 inventory hashes
before/after, retained the identical passing focused artifact and ran the other44
artifacts to exit0. This is local macOS evidence, not CI/platform certification.
Final frontend/reverse-consumer evidence is below; this earlier kernel-only
milestone does not itself establish terminal support.

## Identification, integration and supported limits

There is **no excluded selection predictor** in the author design. Identification
uses the normal functional form; corr(lnwage,Mills)=−.97446818460552 makes the
small-sample outcome estimates imprecise. An exclusion variable is not created or
implied. Negative fitted rho is an estimate, not evidence of a causal mechanism.

The [ledger contract](../../docs/reference/microeconometrics-coverage.md#mma-select-bounded-heckman-two-step-contract-and-evidence)
sets the exact two-stage eligibility/missingness policy, explicit twostep mode,
current latent/selection/Mills/conditional-selected/observed-contribution point
predictions and state/failure requirements. Parser/owned state, typed atomic prediction/reporting/help and `heckman.td` are
now implemented. Two parser tests, two legacy/owned wire tests and six runtime
contracts pass; runtime cases include original membership/asymmetric missingness,
full independent-R covariance, all five projections, changed/dropped current
inputs, exact DECIMAL selection before rounding, failed-fit stale-state clearing
and numerical/schema/target/typed-auxiliary prediction atomicity. Bare legacy
ML remains unsupported without changing the prior fit; only explicit two-step
attempts clear stale state. Initial test setup failures (uppercase option names,
integer d assumed DOUBLE, parser-invalid auxiliary combinations) were diagnosed
and corrected rather than widening language/numerical behavior. The initial
u64/usize compile mismatch is fixed; original failure logs are retained.

The actual source script now passes `check_heckman.py`:55 numerical comparisons
plus3 separately bounded certificates,13 covariance entries,1000 exported native
projections/200 first-stage indices and1400 original cells/ordinals. It independently
certifies the source likelihood/score/information inverse, selected normal
equations, sigma moment and generated-regressor covariance; shared wrong reference
and native results do not suffice. `check_heckman_failures.py` rejects10 owned-copy
corruptions, including shared covariance, shared zero tail and shared false
stationary control; all6 original artifact fingerprints stay unchanged.
Two actual-script root tests add a separate closed-form24-row control with
selection proportions1/4,1/2,3/4, exact probit gamma=(0,qnorm(.75)), outcome
beta=(3,2),alpha=.5,sigma=1,rho=.5. Base R and the pinned external covariance
function supply49 quantities/all13 covariance entries and120 predictions; text/
JSON exports are byte-identical. Export-sentinel, missing-input and existing
prediction-target failures preserve files. Fixtures and their development-only
R generator are in `tests/fixtures/heckman_script_*`; this is not replacement
book data. The original200-row script also passes the current text/JSON export
comparison and both independent audit drivers.

Independent read-only kernel and frozen integration reviews found no issues;
neither reviewer executed tests. Parent current workspace verification reconciles
1497 passed/two pre-existing ignored across259 exact fresh artifacts, with all
binary/main hashes checked. Earlier nextest/build and bounded loader timeouts
remain incomplete; dyld samples, bounded resumptions and complete diagnostics
are retained in PR evidence. A discovery snapshot initially omitted the newly
advertised three kinds; its explicit expectations/help/explain cases were updated
and all25 discovery tests pass. No numerical tolerance/parser expansion hid a
failure. Workspace all-target Clippy/format/diff and doctests pass; the earlier
timed-out Clippy/doctest commands are not relabeled successful. This is local
macOS evidence, not Actions/other-platform certification.

Do not present the existing parser-only bare heckman form as ML or
silently reinterpret it as two-step. Conditional-selected means differ from
Tobit truncation, and the zero-filled observed-contribution mean is not an
imputation of unobserved outcomes or a causal effect.
