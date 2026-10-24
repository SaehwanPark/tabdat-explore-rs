# Classic lower-censored Gaussian Tobit — WIP

Exact source: Cameron–Trivedi (2005) §16.2.1 pp.530–531, author March2005
`mma16p1tobit.do` §2A. Its estimation extension is explicitly not a published
coefficient table. Figure16.1's theoretical curves and the program's truncation,
selection and CLAD analyses are separate required outcomes, not this fit.

**Native command execution is not implemented yet.** A pure Rust numerical
kernel now passes five synthetic R/survival/base-R references and the original
source's48 quantities/600 predictions. Owned session state, command execution,
reporting/help and actual-script contracts remain WIP; this is not a delivered CLI
workflow. The files below establish reproducible source/preparation/references.
The intended command/sample/inference/failure contract is in
[the coverage ledger](../../docs/reference/microeconometrics-coverage.md#mma-tobit-bounded-classic-lower-censoring-contract-wip).

| Original author asset | SHA256 |
| --- | --- |
| `mma16p1tobit.asc` | `8ec7ba9cf3da6854217f873441e47fd5492e777904057c8d9ca8f8ee00244ba6` |
| `mma16p1tobit.do` | `8ebd642a8834d8a45ee48f1d6d432aa3ac22a9319aedf1df0e9776fb0959fbc4` |
| `mma16p1tobit.txt` | `8b76e56738fb3f43e752749b6ba9dcc4b98ab55cd3a8d93d3e2facf75fadb638` |

Use the author's exported200-row data, not a replacement Stata/R/Python random
stream. The converter preserves its six tokens/order and adds `source_row`; only
the truncated response's `.` becomes an empty CSV cell. It validates the entire
fixed-format input before emitting CSV. No original data/program/log is committed.

From the repository root, in a fresh directory (native preparation needs awk,
not R/Python):

```sh
repo="$PWD"
work="$(mktemp -d)"
(
  set -e
  cd "$work"
  for asset in mma16p1tobit.asc mma16p1tobit.do mma16p1tobit.txt; do
    curl -fsSL "https://cameron.econ.ucdavis.edu/mmabook/$asset" -o "$asset"
  done
  printf '%s\n' '8ec7ba9cf3da6854217f873441e47fd5492e777904057c8d9ca8f8ee00244ba6  mma16p1tobit.asc' | shasum -a256 -c
  awk -f "$repo/examples/mma16p1tobit/tobit_to_csv.awk" mma16p1tobit.asc > mma16p1tobit.csv
  # Optional development reference: R4.6.1 / survival3.8.6.
  Rscript "$repo/examples/mma16p1tobit/reference_tobit.R" \
    mma16p1tobit.asc R_tobit_predictions.csv > R_tobit_quantities.csv
)
```

The source has70 censored/130 uncensored responses. The reference independently
checks/refines Gaussian `survreg` with base-R `deriv3` likelihood/full score/full
observed information, including a separately re-fitted slope-null. It emits48
quantities (nine joint beta/lnsigma covariance entries) and600 point predictions,
plus every original cell/ordinal. Current values: intercept−2244.5674836,
lnwage956.4877300, sigma896.6810974, LL−1118.3856951, LR65.64311764 on1df,
continuous observed-mean AME625.5806292. Uncertainty is asymptotic normal95,
conditional on X, with positive log-Wald sigma intervals; do not equate these
intervals with historical Stata8's t intervals or an AME with the latent slope.
This reference does not certify a native implementation or whole Chapter16.
