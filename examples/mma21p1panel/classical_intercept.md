# Classical FE normalized common-intercept inference

Target: C21-P1-FE, author first-program lines201–205/Table21.2. Original5,320
MOM observations/532 people/1979–1988; unweighted lnhr on lnwg and person effects.
[Script](classical_intercept.td), [independent R QR](classical_intercept_reference.R),
[auditor](check_classical_intercept.py). The source hashes/inventory and raw-data
license/access cautions are in [inventory.md](inventory.md) and [README.md](README.md).

## Statistical contract

Native one-way FE retains the joint-finite estimation sample, counts n_g, grand
means and normalization sum(n_g*u_g)=0. The common intercept is
`a=mean_y-mean_x' beta`; total unit intercepts are a+u_g. Under iid homoskedastic
errors conditional on retained units/design, DF=N-G-p and sigma²=within_RSS/DF:

- Existing p-by-p slope covariance Vb is unchanged.
- Cov(a,beta)=-mean_x' Vb.
- Var(a)=sigma²/N+mean_x' Vb mean_x.

The grand-mean error is orthogonal to demeaned slope scores, including unbalanced
panels and singletons. Full covariance/point order is (_cons, explicit slopes).
Student-t/95% inference and joint F restrictions use N-G-p; exact references
additionally require normal errors. This is fixed-cohort conditional uncertainty,
**not** variability from resampling people or an assumption of random unit effects.
The [pairs bootstrap](fe_level_bootstrap.md) remains a different target.

Existing `within`/CLI `coefficients`, covariance matrix and parameter names remain
slope-only. Optional kernel `normalized_level` reuses `PostEstimationModel`;
classical native `lincom _cons`, intercept/slope combinations and `test _cons lnwg`
now use its full covariance. Owned moments survive active-data changes; they are
not recomputed from the current frame. `_cons` is withheld as a *common* parameter
if a real slope already owns that name; existing slope naming remains intact.

Unit-cluster/`robust` fits retain slope t/F(G-1) and **withhold common-intercept
inference** with a reason. No classical variance is substituted into a clustered
fit. Nonfinite/negative or ill-conditioned new covariance likewise withholds only
the new inference, not the point fit. Its grand-mean contrast must recover sigma²/N
within sqrt(f64 epsilon) relative error (half of double precision). Review reproduced
a10^12 predictor shift producing a false zero contrast SE without this guard;
the regression now retains identical slope covariance/fits while explicitly
withholding the unrepresentable normalized model. Zero residual variance retains zero SE and point intervals;
t/p are undefined/null, and zero-covariance joint restrictions are not invertible.
No individual-effect intervals, robust common SE, weights, named FE snapshots,
new likelihood or general bootstrap API are added. Float64 coordinate-conditioning
limits still apply to intercept–slope contrasts; this is not arbitrary-precision
inference or evidence that a common-component contrast is causal.

## Workflow and independent evidence

`classical_intercept.td` requires3..532 complete ten-year people, finite bounded
model fields/exact keys. These are source-example guards, **not** new kernel
restrictions: native unweighted FE continues to support existing balanced,
unbalanced, multiple-predictor and singleton designs. It fits, activates an
ordered frame, reports common/mean-at-lnwg2.5 contrasts and a joint zero test,
then exports all four native prediction modes. Interpret xb as the normalized
common component, not an individual conditional prediction; xbu includes its
retained unit intercept. A failed joint restriction on degenerate zero variance
stops the script; point fitting/lincom remain usable separately.

Independent base R4.6.2 `lm.fit` uses a full dummy design with weighted-zero-sum
contrasts: column j is I(g=j)-n_j/n_last*I(g=last). QR covariance uses the full
design rank, not copied within moments/formulas. On original MOM,39 native
coefficient/covariance/SE/t/p/CI/DF/contrast/F fields and5,320 prediction rows
match; all42,560 original raw fields remain exact. Original a/b are
7.219891979652281/0.167675488629312, common SE0.049343396739207, t146.3193143717,
95%CI[7.12315624021309,7.31662771909147], DF4787. These agree with the author's
printed common-intercept row at its displayed precision. All original JSON/text
prediction CSV bytes are identical.

Numerical tolerances were fixed before comparison: coefficients/covariances/SE/CI
and core controls abs1e-9; p-values relative1e-8 with exact matching underflowed
zero. Original joint F is about2.878million in correlated intercept/slope
coordinates, so it uses abs1e-9+relative1e-10. Actual F discrepancy1.64e-7 is only
5.71e-14 relative; the larger absolute difference is expected from the independent
QR/inversion paths, not an estimator/sample difference. No tolerances were widened. Eight source-bound corruptions (SE/cross-covariance,
contrast/F, raw/prediction fields and mode/name metadata) are rejected.

Three balanced/unbalanced/interleaved two-predictor controls (including singleton)
add96 independent QR fields. A separate160-row complete-panel control adds39
fields/160 predictions; fixtures and regeneration R are checked in. Tests cover
response/predictor shift covariance, legacy serde defaults, name collision,
zero RSS, injected nonrepresentable covariance and actual large-offset guards,
exact missing-case keys,
classical joint routing, active-data mutation without refitting, clustered
withholding/state preservation, source preflight and refused overwrite.

Focused tests passed before expanded shared-API validation. Reverse dependency
scope initially covered1811 passing tests. After the review-driven conditioning
guard,1810 reverse-consumer cases pass across328 binaries, plus eight focused
stats cases and two unaffected earlier stats cases:1820 passing/two pre-existing
ignored in the complete combined impact scope. Embedded-help regression and
loadability checks pass; affected doctest targets contain no executable cases.
Affected-crate/changed-target Clippy, formatting and Python syntax pass. An
attempted root all-target Clippy hit the unchanged `excessive_precision` literal
in `tests/mma05_table57_comparison_workflow.rs:54`; it was not altered to hide the
baseline issue. No other-platform or release certification is claimed.

## Reproduce

Use a fresh working directory and the source URL/hash in the inventory:

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/tmp/mom-classical-fe-fresh
mkdir "$work"
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/MOM.dat -o "$work/MOM.dat"
sh "$repo/examples/mma21p1panel/prepare.sh" "$work/MOM.dat" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f \
  "$repo/examples/mma21p1panel/classical_intercept.td" > output.jsonl)
Rscript "$repo/examples/mma21p1panel/classical_intercept_reference.R" \
  "$work/MOM.dat" "$work/reference"
python3 "$repo/examples/mma21p1panel/check_classical_intercept.py" \
  "$work" "$work/reference" "$work/MOM.dat"
```

Use `--text` in a separate fresh directory for readable output. Export collision
preserves its owner and stops later commands; the script is not a session-wide
transaction. The R/Python audit tools are development-only, not production
runtime dependencies. Regenerate the control with
`python3 examples/mma21p1panel/fe_level_bootstrap_synthetic.py heterogeneous fresh_dir`
and pass its MOM.dat to the R reference; the core fixture generator is
`crates/tabdat-stats/tests/fixtures/fixed_effects_common_reference.R`.
`--synthetic` on the auditor labels controls and disables
only the original-file hash gate, not raw-field/sample/reference checks.

C21-P1-FE remains partial: remaining historical panel statistics and complete
comparison reporting are not supplied by this covariance increment. Individual
FE uncertainty, complete companion-program/book inventory and other required
roadmap methods remain open. The separate [B500 Between workflow](between_bootstrap_b500.md) now covers its
native replication-count target. Next reassess companion-program source inventory
and complete comparison/reporting needs.
