# Fishing-mode binary illustration: Cameron–Trivedi (2005), §14.2

This bounded example reproduces the **classical logit and probit columns of
Table 14.2** (p. 465) and the logit probability data underlying Figure 14.1
(p. 466). It adds a separately validated continuous average partial derivative
with conditional-design delta-method uncertainty for each family. That effect
is an extension, **not** a claimed published table result.

It does not complete the entire illustration: robust logit/probit columns,
the cross-model comparison and the actual figure remain required work.
No author data or code is redistributed here.

## Source, sample and design

The authors' [program](https://cameron.econ.ucdavis.edu/mmabook/mma14p1binary.do)
and [Stata 8 output](https://cameron.econ.ucdavis.edu/mmabook/mma14p1binary.txt)
identify §14.2, pp. 464–466, Tables 14.1–14.2 and Figure 14.1. The
[index](https://cameron.econ.ucdavis.edu/mmabook/mmaprograms.html) abbreviates the
page range as 464–465. Data originate in Herriges and Kling (1999),
*Review of Economics and Statistics* 81:62–72.

SHA-256 provenance:

| File | SHA-256 |
| --- | --- |
| `Nldata.asc` | `1fb6f7d5abaf790d4df2c6a13a499ca42c0db4f950d941ba92565c30b4ab6c22` |
| `mma14p1binary.do` | `a443a2988eb737ed8f58dd0537c015a1a639f8de2a8c7bcef41c8a5f49f9cfa5` |
| `mma14p1binary.txt` | `dbf9bb8f00a126269b099090e7acd569725a2cf218275c384901ac13c5ab6d03` |

The ASCII stream has **16 numeric fields per person**, 1,182 people, CRLF line
endings and no final newline. `nldata_to_csv.awk` groups tokens, not physical
lines, preserves decimal tokens and adds an original **1-based `source_row`**.
It rejects malformed/incomplete/empty records; the recipe only publishes a CSV
after successful conversion. The converter does not impute or filter people.

- Keep mode **2 (pier)** or **4 (charter)**: 630 people, 178 pier and 452 charter.
  One source comment reverses these codes; the executable code and output resolve
  them as stated here.
- Outcome `charter = 1(mode == 4)`, predictor `lnrelp = ln(pcharter/ppier)`, intercept.
  Source `pchart` is Stata's abbreviation for `pcharter`. Both the logit and the
  probit script use this identical preparation.
- Preserve source-row order. No additional missing/nonfinite exclusions occur.
  The fit's retained indices `0..629` refer to the prepared 630-row cohort;
  `source_row` identifies each original person in the 1,182-record input.
- Read source decimals and prepare the design in float64. Do not silently emulate
  Stata's float32 import/generated-variable storage. Author **reported rounded**
  results agree; independent R validates the float64 calculation at higher precision.

## Run and independently check

From the repository root, with `curl`, POSIX `awk` and the built Rust binary:

```sh
(
set -eu
cargo build --locked --bin tabdat-explore-rs
repo="$PWD"
work=$(mktemp -d)
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/Nldata.asc -o "$work/Nldata.asc"
printf '%s  %s\n' \
  1fb6f7d5abaf790d4df2c6a13a499ca42c0db4f950d941ba92565c30b4ab6c22 \
  "$work/Nldata.asc" | shasum -a 256 -c -
awk -f examples/mma14p1binary/nldata_to_csv.awk "$work/Nldata.asc" \
  > "$work/Nldata.csv.tmp" && mv "$work/Nldata.csv.tmp" "$work/Nldata.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma14p1binary/logit.td")
Rscript examples/mma14p1binary/reference_logit.R "$work/Nldata.csv" \
  "$work/R_predictions.csv" "$work/fishing_logit_predictions.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma14p1binary/probit.td")
Rscript examples/mma14p1binary/reference_probit.R "$work/Nldata.csv" \
  "$work/R_probit_predictions.csv" "$work/fishing_probit_predictions.csv"
printf 'Outputs: %s\n' "$work"
)
```

Stop if the checksum/conversion fails. Each script writes its own
`fishing_*_predictions.csv`; each intentionally refuses to overwrite an existing
file, so a replay needs a fresh output directory (copy `Nldata.csv` there). For
a text replay, replace `--json` with `--text`. No arguments enter the persistent
terminal session; the same commands work there. **Bare `predict` retains the
existing xb parser default**; request `, pr` explicitly for probabilities.

R 4.6.1 `stats::glm` is an optional **development-only** reference, no R/Python
is needed by TabDat. Both reference scripts reevaluate the information
matrices at the returned coefficients, independent of any last-iteration
working-weight lag in glm's cached QR covariance; both verify **every
source-row ID**, outcome, design value, probability and linear index, not just
N. The convergence conventions differ by family: for the **logit** the plain
glm stop (deviance change) already sits at the score fixed point; for the
**probit** the deviance is flat near the optimum, so `reference_probit.R`
polishes glm's solution with Newton steps on the analytic observed information
until the gradient norm is < 1e-13 and prints the glm-stop and fixed-point
score norms (on this design: 2.94579e-07 → 2.57012e-14, one polishing step).

| Logit quantity | Independent R reference |
| --- | ---: |
| Intercept / slope | 2.0531249423051365 / -1.8225304317903031 |
| Model-based SE, intercept / slope | 0.16893067646375609 / 0.14456807108042660 |
| Log likelihood / null log likelihood | -206.82697122830305 / -375.06166897675286 |
| LR chi-square(1) / p | 336.46939549689961 / 3.7478749624517373e-75 |
| McFadden pseudo-R² | 0.44855209599911783 |
| Mean / min / max probability | 0.7174603174603175 / 0.004719638213908227 / 0.9974745605784411 |
| Continuous AME / delta SE | -0.18392143550221252 / 0.0052985086445694511 |
| AME normal 95% interval | [-0.19430632161734276, -0.17353654938708227] |
| AME normal two-sided p | 5.2063951837379612e-264 |

Native replay matches coefficients, covariance, SEs, intervals and effects at
`1e-10` absolute; log likelihoods/LR at `1e-8` absolute; all-row probabilities and
indices at `1e-10` absolute; test probabilities at `1e-10` **relative** plus
`f64::MIN_POSITIVE` (so a fabricated zero cannot pass for the tiny AME tail).
Observed largest logit probability/index errors were approximately `2.0e-15`/`4.2e-14`.

| Probit quantity | Independent R reference |
| --- | ---: |
| Intercept / slope | 1.1943604331481330 / -1.0555145307066702 |
| Model-based SE, intercept / slope | 0.089508361628277577 / 0.076115584870155517 |
| Log likelihood / null log likelihood | -204.4108645034095 / -375.06166897675286 |
| LR chi-square(1) / p | 341.30160894668671 / 3.3220403402590648e-76 |
| McFadden pseudo-R² | 0.45499398789248358 |
| Mean / min / max probability | 0.72018995284383147126 / 0.00098774792299152303 / 0.99973774175233098838 |
| Continuous AME / delta SE | -0.18675505365724743 / 0.0054545071552692895 |
| AME normal 95% interval | [-0.19744569123499126, -0.1760644160795036] |
| AME normal two-sided p | 6.4315448068460033e-257 |

The probit replay satisfies the same tolerance policy, with observed largest
probability/index errors of approximately `5.0e-14`/`3.0e-13`. The committed
synthetic references in `crates/tabdat-stats` and `crates/tabdat-runtime` use
`1e-10` absolute for moderate outputs.
`sh examples/mma14p1binary/test_converter.sh` additionally checks CRLF/final-record
preservation and malformed/incomplete/empty-input failure without publishing over
an existing destination.

## Interpretation, numerical limits and remaining work

Both families are unweighted independent-Bernoulli maximum likelihood, normal
coefficient/AME inference and asymptotic chi-square LR inference, **without
N-K covariance scaling**. The logit reports inverse-Fisher covariance; the
probit reports the inverse of the per-observation observed information at the
converged fit, computed from the converged QR. With an intercept the LR null
estimates a constant probability; without it the zero-coefficient null is
probability 0.5.

Stata 8 computes its probit SEs from the penultimate-iterate Hessian, so its
classical-column SEs can differ from these converged fixed-point values in
trailing digits. This example does not replicate that lagged-iterate
convention; the reference standard is the R fixed point described above.

`estat margins` averages the family-specific continuous derivative over the
**retained training design**: logit `beta_j*p*(1-p)`, probit
`beta_j*phi(beta_0 + beta'x)`. Its full delta gradient includes
intercept/slopes' covariance, conditional on observed X. Current-data edits do
not change this estimand. The effect is a
predictor-column derivative, not an automatic discrete change, interaction chain
rule, or causal effect. Here the negative AME is probability units per unit of
**log relative price**, not per price unit or an exact finite-change prediction.

Fitting requires both outcome classes with exact 0/1 values on the complete
finite sample. Native Decimal outcomes are validated on their exact integer
payload/scale before conversion: nearby values must not round into a binary one.
Domain rejection follows complete-case selection, so an otherwise excluded row
still does not enter the model. Scaled Newton/QR uses stable likelihood/curvature, backtracking,
100 information evaluations and absolute `1e-10` score **and full undamped step**
bounds in unit-column-norm coordinates. A flat likelihood/small score alone must
not publish a separated fit. Combined outlier/separating-contrast regressions
cover a false-convergence trap found in independent review. This is not an
analytic separation classifier; numerical singularity/nonconvergence is an
explicit error, including extreme finite designs without adequate absolute
numerical resolution. Identified models with saturated outlier probabilities
are not rejected merely because probabilities round to zero/one.

Robust/cluster/weighted/survey forms, categorical/factor/interactions and
discrete changes with uncertainty, prediction intervals, general `lincom`/`test`
after either family and rendered graphics are not delivered by this slice. They
remain required where specified by the roadmap/illustrations; neither this
example nor passing synthetic tests establish binary-family or full-book
completion.
