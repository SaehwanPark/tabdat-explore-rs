# Fishing-mode binary illustration: Cameron–Trivedi (2005), §14.2

This bounded example validates the **classical and unclustered robust numerical
logit, probit and OLS comparison in Table 14.2** (p. 465), under the explicit
converged-information conventions below, and a **native SVG rendering of the
outcomes/fitted values in Figure 14.1** (p. 466), under the explicit native
layout/jitter conventions below. It adds a separately validated continuous
average partial derivative with conditional-design delta-method uncertainty for
each binary family. That effect is an extension, **not** a claimed published table result.

`analysis.td` now composes the **published Tables14.1–14.2/Figure14.1 and all
source/cohort/fitted-value summary blocks** into one validated native workflow.
The output audit below defines that coverage; it is not a literal Stata log or
historical storage/format/RNG replication. Later binary-family methods remain
required work. The bounded native named-report comparison does not restore fit
caches or provide general estimates/persistence/formatting parity.
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

## Complete published-analysis workflow

Run `analysis.td` in a **fresh directory and session**. Its nested
`stored_comparison.td` resolves relative to the example script, while `Nldata.csv`
and all output paths resolve relative to the working directory. No R/Python is
needed to execute the native analysis. The optional development checks below use
base R4.6.1 and standard-library Python, with the existing unchanged tolerances.

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
  -f "$repo/examples/mma14p1binary/analysis.td" > native.jsonl)

# Optional independent development validation, not a production requirement.
Rscript examples/mma14p1binary/reference_comparison.R "$work/Nldata.csv" \
  "$work/R_predictions.csv" > "$work/R_comparison.csv"
Rscript examples/mma14p1binary/reference_analysis.R "$work/Nldata.csv" \
  "$work/R_predictions.csv" "$work/R_source_sample.csv" \
  "$work/R_prepared_sample.csv" > "$work/R_analysis_summary.csv"
python3 examples/mma14p1binary/check_analysis.py "$work/R_analysis_summary.csv" \
  "$work/R_source_sample.csv" "$work/R_prepared_sample.csv" "$work/native.jsonl" \
  "$work/fishing_source_sample.csv" "$work/fishing_prepared_sample.csv"
python3 examples/mma14p1binary/check_stored_comparison.py "$work/R_comparison.csv" \
  "$work/R_predictions.csv" "$work/native.jsonl" "$work/fishing_comparison_predictions.csv"
python3 examples/mma14p1binary/check_figure.py "$work/R_predictions.csv" \
  "$work/fishing_binary_comparison.svg" "$work/native.jsonl"
printf 'Outputs: %s\n' "$work"
)
```

Replace `--json` with `--text` for native terminal reports. A piped terminal can
run `run "/absolute/path/to/analysis.td"` followed by `exit`; JSON/text/piped runs
produce byte-identical three CSVs and SVG. Fresh conversion/R references and
prediction/SVG bytes also match the separately validated leaf workflows.

### Author-output audit and native conventions

| Author program/output block | Native execution and independently checked evidence |
| --- | --- |
| 1,182-person source summary after `income/1000` | 17 ordered variables; all 85 N/mean/sample n−1 SD/min/max quantities; `fishing_source_sample.csv` checks all 1,182 IDs and 18 fields including ID |
| Mode2/4 preparation, full overall/pier/charter summaries | `charter=1(mode==4)`, `pratio=100*ln(pcharter/ppier)`, `lnrelp=ln(pcharter/ppier)`; 20 variables × 3 cohorts × 5 = 300 quantities; `fishing_prepared_sample.csv` checks all 630 IDs and 21 fields |
| Cross-tool prepared-data output | Native headered CSV preserves all prepared fields, including charter/lnrelp and source identity; not the author's headerless two-column ASCII or directly runnable LIMDEP input (MMA-MAXSCORE remains separate) |
| Table14.1 | Explicit overall/pier/charter four-variable summary blocks; all 60 quantities |
| Six fits and Table14.2 | Existing native ordered classical/robust logit/probit/OLS fits and immutable six-column report; all 94 R quantities and covariance/reference/sample metadata, plus all 630 prediction records |
| Four-variable fitted-value summary | Charter/logit probability/probit probability/OLS fitted value; all 20 quantities; OLS min/max remain −0.202734122 / 1.307384388, not clipped |
| Figure14.1 | Same native SVG bytes as `figure.td`; all 630 outcomes, 1,890 R prediction vertices and actual geometry checked |
| Run control, labels, intermediate sorts, quiet refits, Stata log/graph formatting | Native structured/text reports and explicit numeric mode cohorts replace the log/value-label display; source order is preserved, line sorting is private, previously computed classical predictions are reused, SVG replaces WMF, deterministic y-only jitter replaces Stata RNG jitter |

The new reference checks **465 quantities in eight ordered summary blocks
(93 rows)**, all source/prepared records and all generated transformations at
float64 precision. Summary tolerance is absolute `1e-10`, prepared values
`1e-12`; the existing selected-covariance/tail/prediction and `1e-8` pixel
geometry checks are unchanged. Constants have SD0, not missing, and SD uses n−1.
The source's income maximum yields float64 `ydiv1000=12.499998`, rather than
forcing the rounded author display `12.5`. Corruption probes reject population
SD, swapped cohorts, clipped OLS summaries, incorrect income/pratio scaling,
permuted source IDs and changed source prices.

This validates the defined published-analysis outcomes and all descriptive
blocks, **not every auxiliary field/layout in a Stata fit log**. Individual OLS
ANOVA/F/interval displays are not added to the comparison table; `test`/`lincom`
remain separate native commands. Binary normal z versus OLS t(628), robust ML
N/(N−1) versus OLS HC1, Gaussian RSS/N working likelihood and probit's converged
information differences remain as documented below. No restored fit is implied.
The final active fit is robust OLS (the last stored fit), not the author's quiet
classical refit; all three plotted predictions are classical.

The script reloads the same source inside its existing six-fit leaf workflow;
its final active data are the prediction dataset. The full prepared fields are
available in `fishing_prepared_sample.csv`. Outputs refuse replacement, and
stored names refuse overwrite. A failure stops subsequent commands and reports
the **originating script/line**, including `stored_comparison.td` for a nested
failure. Earlier completed exports remain: **the script is not a transaction**.
Recover in another fresh directory/session or deliberately manage only your own
outputs/stored reports. Never delete the source or overwrite existing files as
an automatic recovery step. Actual-example CLI regressions cover the four
publication collision stages, schema/price/key failures and unchanged source
bytes, as well as all three execution surfaces.

## Individual binary-fit and AME recipes

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

## Unclustered robust covariance and reproducible reference

`robust_logit.td` and `robust_probit.td` repeat the exact preparation above with
`, robust`. They preserve the same coefficients, sample and predictions. The
covariance is `N/(N-1) * B * sum(s_i s_i') * B`, where `s_i` is the individual
score and `B` is inverse observed information at the converged fit (equal to
inverse Fisher for logit). The [Stata ML correction](https://www.stata.com/manuals/u20.pdf)
(§20.22.2) is **not** OLS HC1's N/(N-K). Independent observations are assumed;
this is not cluster/survey inference, a causal estimand, or protection against a
wrong conditional-mean interpretation.

Coefficient and retained-X delta-AME inference remain asymptotic normal. JSON
labels both fit and margins `robust_sandwich_n_over_n_minus_1`. The LR statistic,
restriction count and pseudo-R² remain descriptive; **LR p-value and reference
are null**. The fit-level omnibus Wald statistic tests the log-price slope
against zero using selected covariance, W = b_s' V_ss^-1 b_s, with asymptotic
chi-square(1) inference. This is not robust LR or a conditional-mean
specification test. Without an intercept it tests every declared coefficient;
multiple-column designs use the full slope covariance, not a sum of z-squares.
Rank-limited/singular restriction covariance gives explicit unavailable test
metadata without discarding the fit, never a pseudoinverse/reduced-q test.

With `work` set to a directory containing the checksum-verified `Nldata.csv`
acquired above, run from the repository root (fresh robust output filenames):

```sh
repo="$PWD"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma14p1binary/robust_logit.td" > robust_logit.jsonl)
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma14p1binary/robust_probit.td" > robust_probit.jsonl)
Rscript crates/tabdat-stats/tests/fixtures/binary_robust_reference.R \
  "$work/Nldata.csv" > "$work/robust_reference.csv"
python3 examples/mma14p1binary/check_robust_reference.py \
  "$work/robust_reference.csv" "$work/robust_logit.jsonl" "$work/robust_probit.jsonl"
Rscript examples/mma14p1binary/reference_logit.R "$work/Nldata.csv" \
  "$work/R_robust_logit.csv" "$work/fishing_robust_logit_predictions.csv"
Rscript examples/mma14p1binary/reference_probit.R "$work/Nldata.csv" \
  "$work/R_robust_probit.csv" "$work/fishing_robust_probit_predictions.csv"
```

The R 4.6.1 script independently computes the scores, converged information,
sandwich and full delta gradients; it also generates the committed synthetic
CSV reference when run without a data argument. The standard-library Python
checker is development-only and compares all **50** coefficient/covariance/AME
and omnibus-Wald quantities; R's existing prediction checkers verify all 630 source-row IDs,
outcomes, designs, probabilities and indices. Neither tool is needed at runtime.
Tolerances remain 1e-10 absolute for moderate outputs and 1e-10 relative plus
`f64::MIN_POSITIVE` for test probabilities. Native synthetic tests additionally
cover both families with/without intercept, one/two predictors, intercept-only
models, predictor units, invalid outcomes and unsupported cluster/weight designs.

| Robust quantity | Independent R fixed-point reference |
| --- | ---: |
| Logit intercept / slope SE | 0.147347754734682 / 0.130755606121941 |
| Logit continuous AME SE | 0.00570499760609184 |
| Probit intercept / slope SE | 0.0794501824989468 / 0.0692948032742171 |
| Probit continuous AME SE | 0.00568139267322038 |
| Logit omnibus Wald chi-square(1) / p | 194.280378518603 / 3.69887865746942e-44 |
| Probit omnibus Wald chi-square(1) / p | 232.020896467226 / 2.16086165111978e-52 |

The author output reports robust Wald statistics 194.28 (logit) and 232.07
(probit). Logit's rounded statistic agrees. Probit's converged-information
232.020896467226 intentionally does not equal the old Stata value; the same
information/covariance difference described below propagates to W and its tail.
Independent R validates that convention rather than widening tolerances or
claiming exact historical Wald replication.

Rounded Table 14.2 coefficient/z entries agree. This is **not** an exact replay
of Stata 8's stored information: its reported probit robust SEs are
0.0794429/0.0692881, versus the converged-information values above. As for the
classical probit reference, the target is the independently validated fixed
point, not a lagged Hessian; sandwich bread enters twice. Float64 preparation
also does not emulate the author's float32 storage. These convention differences
are explicit; no reference tolerance was widened to hide them. Robust support
advances this subset, not all author output or the whole binary family.

## Six-model numerical comparison and OLS fitted values

`comparison.td` prepares the cohort once, fits classical logit/probit/OLS then
robust logit/probit/OLS, and exports all six fits' predictions in one source-ID
ordered CSV. The production app reports six native JSON/text fit results;
neither R nor Python is required to fit, interpret those results or export.
There is no hidden refit or dependency on a named-fit table command.

From the repository root with `work` containing the checksum-verified
`Nldata.csv` prepared above (fresh comparison export filename):

```sh
repo="$PWD"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma14p1binary/comparison.td" > comparison.jsonl)
Rscript examples/mma14p1binary/reference_comparison.R "$work/Nldata.csv" \
  "$work/R_comparison_predictions.csv" > "$work/R_comparison.csv"
python3 examples/mma14p1binary/check_comparison.py \
  "$work/R_comparison.csv" "$work/R_comparison_predictions.csv" \
  "$work/comparison.jsonl" "$work/fishing_comparison_predictions.csv"
```

The optional development-only checker validates **94** six-model quantities
(coefficients, full selected covariance, SEs, z/t statistics and tails,
likelihoods and appropriate goodness-of-fit fields) and all **630** original
source/outcome/design/prediction records. It then prints a joined native-value
comparison as a validation convenience, **not a production estimate-table
feature**. Base R4.6.1 `lm`/`logLik`/`vcov` and explicit HC1 matrix algebra
independently validate OLS; binary references retain the analytic fixed-point
convention above. Moderate values and predictions use 1e-10 absolute, prepared
design 1e-12, and test tails 1e-10 relative plus `f64::MIN_POSITIVE`.

| OLS quantity | Independent R reference |
| --- | ---: |
| Intercept / slope | 0.784154230742988 / -0.242913654440961 |
| Classical SE, intercept / slope | 0.0134701118551607 / 0.0104328468939993 |
| Robust HC1 SE, intercept / slope | 0.0119566488459203 / 0.00862919092660423 |
| R² / Gaussian working-model log likelihood | 0.463304487997616 / -195.167320651517 |
| Min / max linear fitted value | -0.202734122 / 1.307384388 |

OLS is a linear probability model here, not Bernoulli ML: its `xb` fitted
values are intentionally **not clipped to [0,1]** or relabeled probabilities.
Classical covariance uses RSS/(N-K); robust HC1 uses N/(N-K), unlike binary ML's
N/(N-1). OLS coefficient tests use t(628), not binary normal z. Its reported
Gaussian working-density log likelihood uses ML variance RSS/N and does not
supply a cross-family or robust LR test. OLS R² and binary McFadden pseudo-R²
remain distinct measures. Classical/robust predictions agree exactly within
each family; the six native results reproduce the numerical comparison under
the documented float64/converged-information conventions, not exact historical
storage or reusable report formatting.

## Native named-report Table14.2 comparison

`stored_comparison.td` runs the same six fits and prediction exports, adding
`estimates store` after each fit and a native `estimates table` in the authors'
column order: blogit, bprobit, bOLS, bloghet, bprobhet, bOLShet. This is an
application-generated comparison, unlike the optional development checker's
joined display above. Report capture and comparison do not refit or change the
active model: the last robust OLS fit and all prediction values remain intact.

With `work/Nldata.csv` checksum-verified as above, use a fresh output directory:

```sh
repo="$PWD"
stored_work=$(mktemp -d)
cp "$work/Nldata.csv" "$stored_work/Nldata.csv"
(cd "$stored_work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma14p1binary/stored_comparison.td" > stored.jsonl)
Rscript examples/mma14p1binary/reference_comparison.R "$stored_work/Nldata.csv" \
  "$stored_work/R_predictions.csv" > "$stored_work/R_comparison.csv"
python3 examples/mma14p1binary/check_stored_comparison.py \
  "$stored_work/R_comparison.csv" "$stored_work/R_predictions.csv" \
  "$stored_work/stored.jsonl" "$stored_work/fishing_comparison_predictions.csv"
printf 'Native comparison and reference outputs: %s\n' "$stored_work"
```

Run with `--text` instead for the native coefficient/statistic table, or use a
no-argument terminal `run ".../stored_comparison.td"`, again from a fresh output
directory. Native text and piped table bytes match; all JSON/text/piped prediction
exports match byte-for-byte, including the previous non-stored workflow. The
actual native six-column text table was inspected: requested order, labelled
normal z versus OLS t(628), selected covariance, N630, descriptive likelihoods
and OLS R² versus binary McFadden pseudo-R². Values display six decimals; this is
not the author's exact three-decimal layout or Stata8 storage convention.

The independent base-R4.6.1 reference/checker validates **all 94 quantities in the
stored table**, including selected covariance, SE/statistic/tails, root MSE,
residual df and likelihood, as well as all **630 source/design/prediction
records** from the six fits. Exact sample/reference/covariance metadata and
requested order are also checked. Fresh checksum-verified conversion and R
references reproduce previous evidence byte-for-byte; statistical/design/tail
tolerances are unchanged. Corruption probes reject fabricated tiny-tail zero,
binary Student-t metadata, wrong sample positions/covariance/column order and
mislabelled OLS likelihood.

These are immutable session-local **reporting snapshots**, not restorable model
caches. Stored reports survive data reloads/edits, refits and failed fits; active
prediction/margins still use the ordinary current model. JSON retains selected
covariance and fit-time sample positions/groups, not a current-data identity or
staleness guarantee. Names are case-sensitive ASCII identifiers of 1–32 bytes;
at most 32 reports, 1–8 explicit unique models per table, and 64 coefficients per
report/union. Name collisions require an explicit drop; a failed multi-drop is
atomic. WLS/GLS/IV, restore, disk persistence, implicit all/latest and formatting
options are uninstalled. Missing terms are NA/null, not zero. No cross-model or
robust LR inference is supplied. See `help estimates` and the
[structured contract](../../docs/reference/command-results.md#named-fit-reports-and-comparison).
No R/Python is required by the application, and no author data are committed.

## Native Table14.1 summaries

`summary.td` prepares the same 630-person cohort and reports `charter`, `pcharter`,
`ppier` and `lnrelp` in three native summary blocks: **overall**, **pier (mode2,
N178)**, then **charter (mode4, N452)**. Each reports N, mean, sample SD using
**n−1**, min and max. The conditional summaries do not filter or reorder the
prepared relation. `fishing_summary_sample.csv` exports all six preparation
columns in original source-ID order so actual membership/design can be checked,
not merely N. No estimator is fitted or reporting engine changed.

From the repository root, with `work/Nldata.csv` checksum-verified as above:

```sh
repo="$PWD"
summary_work=$(mktemp -d)
cp "$work/Nldata.csv" "$summary_work/Nldata.csv"
(cd "$summary_work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma14p1binary/summary.td" > summary.jsonl)
Rscript examples/mma14p1binary/reference_summary.R "$summary_work/Nldata.csv" \
  "$summary_work/R_summary_sample.csv" > "$summary_work/R_summary.csv"
python3 examples/mma14p1binary/check_summary.py \
  "$summary_work/R_summary.csv" "$summary_work/R_summary_sample.csv" \
  "$summary_work/summary.jsonl" "$summary_work/fishing_summary_sample.csv"
printf 'Summary and verification outputs: %s\n' "$summary_work"
```

Use a fresh output directory: the sample export is never replaced. `--text` and
no-argument terminal `run ".../summary.td"` also work. Their sample exports were
checked byte-identical to JSON execution. Native terminal output was inspected:
three four-variable blocks, including constant-outcome SD0 in the two groups.
The script order supplies the cohort meanings; this is not a joined table or
new grouped-summary formatting command.

Independent base R4.6.1 `mean`, `sd`, `min` and `max` match **all 60 summary
quantities** and **630 source/outcome/design records**. Moderate-value tolerances
remain absolute 1e-10 and prepared-record tolerances 1e-12. Source data were freshly
checksum-verified and conversion compared byte-for-byte; no source data are
committed. These are float64 calculations, not exact Stata8 float32 storage or
printed-format parity. The checker rejects population-SD substitution, swapped
cohort blocks, wrong counts and permuted source identities. Native CLI tests
exercise actual-script success, surfaces and export order, plus missing columns,
invalid negative prices, duplicate IDs and export collision. The reference and
checker enforce the exact textbook cohort; the native script's `isid` is a key
guard, not a substitute for those full sample/reference checks.

Remaining work includes other author exploratory/fitted-value summaries and
reporting beyond the bounded named-report comparison above. Table14.1 numerical validation does
not complete §14.2 or the binary milestone.

## Native Figure14.1 SVG

`figure.td` runs `comparison.td`, labels its existing prediction columns and saves
`fishing_binary_comparison.svg`: all 630 outcomes and classical logit/probit/OLS
curves, sorted internally by x with original-row ties. The application requires
no R/Python, external assets, bundled fonts or plotting runtime. The command
preserves the prepared data, labels, row order and last robust OLS fit.

Use a **fresh output directory**, even if you already ran `comparison.td`:
`figure.td` also exports its prediction CSV and neither file is replaced. With
`work` containing the checksum-verified `Nldata.csv` above:

```sh
repo="$PWD"
figure_work=$(mktemp -d)
cp "$work/Nldata.csv" "$figure_work/Nldata.csv"
(cd "$figure_work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma14p1binary/figure.td" > figure.jsonl)
Rscript examples/mma14p1binary/reference_comparison.R "$figure_work/Nldata.csv" \
  "$figure_work/R_predictions.csv" > "$figure_work/R_comparison.csv"
python3 examples/mma14p1binary/check_comparison.py \
  "$figure_work/R_comparison.csv" "$figure_work/R_predictions.csv" \
  "$figure_work/figure.jsonl" "$figure_work/fishing_comparison_predictions.csv"
python3 examples/mma14p1binary/check_figure.py \
  "$figure_work/R_predictions.csv" "$figure_work/fishing_binary_comparison.svg" \
  "$figure_work/figure.jsonl"
printf 'SVG and verification outputs: %s\n' "$figure_work"
```

Open the SVG yourself in an SVG-capable browser/viewer. `--text` or a no-argument
terminal `run ".../figure.td"` works from a fresh output directory too. JSON and
piped-terminal replays produce byte-identical SVG/prediction exports. The
optional independent checker validates all 630 outcomes and **1,890 R prediction
vertices**, source order/stable x ties, metadata and actual SVG coordinates at
**1e-8 pixel absolute tolerance**. The existing 94 fit-quantity and full-record
checks remain unchanged. Deliberately clipped geometry, permuted row identities
and fabricated un-jittered metadata are rejected. The actual full SVG was
visually inspected in Chrome on macOS27.0.1 arm64: readable axes/legend, two
outcome bands, distinguishable color/dash curves and both unbounded OLS ends.
This is named-host evidence, not cross-platform/font or pixel-equality approval.

The authors' code sorts the dataset and uses Stata `jitter(3)`, WMF export and
historical float32 preparation. Native plotting instead sorts line vertices
internally without changing data order, uses float64 coordinates, fixed layout,
viewer-resolved sans-serif fonts, rounded ticks and distinct colors/dashes.
`jitter(0.03)` applies a deterministic y-only ordinal display pattern in outcome
units, **not Stata's percentage/RNG convention or seed behavior**. Raw 0/1
outcomes and predicted values are unchanged, and the OLS line's range
[-0.202734122, 1.307384388] is not clipped or mislabeled a Bernoulli probability.
The title/axis wording explicitly distinguishes probabilities from OLS fits.
This reproduces the illustration's data/curves, not its exact layout or pixels.

SVG embeds raw float64 coordinates and candidate-row indices: display jitter is
**not anonymization**, so review artifacts before sharing them. The command is
bounded to 10,000 candidate rows/80-character labels, explicit `.svg` saving and
`noopen`, and requires a local hard-link-capable filesystem for complete
no-clobber publication. PNG, automatic opening/default paths, replacement and
other plot families remain uninstalled; see `help scatter`. No generated
source-data graphic is redistributed here.

## Interpretation, numerical limits and remaining work

Both default binary fits are unweighted independent-Bernoulli maximum likelihood,
normal coefficient/AME inference and asymptotic chi-square LR inference,
**without N-K covariance scaling**. The robust variants follow the sandwich
and suppressed-LR-inference contract above. The default logit reports inverse-Fisher
covariance; the default probit reports inverse per-observation observed information at the
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

One-way clustered fits have separate [external method evidence](../contraception_cluster/README.md),
not a new fishing-mode illustration claim.

Weighted/survey forms, categorical/factor/interactions and
discrete changes with uncertainty, prediction intervals, general `lincom`/`test`
after either binary family and graphics beyond the bounded SVG scatter/line
workflow are not delivered by this slice. They
remain required where specified by the roadmap/illustrations; neither this
example nor passing synthetic tests establish binary-family or full-book
completion.
