# Classic lower-censored Gaussian Tobit

Exact source: Cameron–Trivedi (2005) §16.2.1 pp.530–531, author March2005
`mma16p1tobit.do` §2A. Its estimation extension is explicitly not a published
coefficient table. Figure16.1's theoretical curves and the program's truncation,
selection and CLAD analyses are separate outcomes, not this fit. The distinct
[corrected Heckman two-step comparison](heckman.md) now has its own native script,
source/reference audit and inference/identification limits; truncation/CLAD and
remaining Chapter16 work stay required.

The native script/checker validates the exact
source sample, full joint covariance, all600 point predictions and continuous
observed-mean inference against independent R. Owned Gaussian state, current
xb/pr/mean predictions, retained-X effects and terminal/CLI-v1/help are installed.
Six focused runtime cases include no-intercept/nonzero-limit/below-limit
recordings, retained effects, joint missingness, numerical/schema/option failures
and failure atomicity. Two actual-script tests use a separate closed-form
Gaussian control (not replacement textbook data), verify full covariance/delta
inference, byte-identical text/JSON exports and input/output-collision failures.
The command/sample/inference/failure contract and evidence are in
[the coverage ledger](../../docs/reference/microeconometrics-coverage.md#mma-tobit-bounded-classic-lower-censoring-contract-and-evidence).
Independent read-only source/evidence review found no remaining blocker after a
CLI predictor-requirement documentation/schema correction. Local macOS validation
reconciled1486 passed/two pre-existing ignored tests across256 current workspace
artifacts with recorded binary hashes; earlier timed-out runner commands remain
incomplete, not passes. This is not CI or other-platform certification.

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
  cp "$repo/examples/mma16p1tobit/tobit.td" .
  # First build at the repository root: cargo build --locked
  "$repo/target/debug/tabdat-explore-rs" --json -f tobit.td > tobit.jsonl
  # Optional independent development validation: R4.6.1 / survival3.8.6.
  Rscript "$repo/examples/mma16p1tobit/reference_tobit.R" \
    mma16p1tobit.asc R_tobit_predictions.csv > R_tobit_quantities.csv
  python3 "$repo/examples/mma16p1tobit/check_tobit.py" "$PWD"
  PYTHONDONTWRITEBYTECODE=1 python3 "$repo/examples/mma16p1tobit/check_tobit_failures.py" "$PWD"
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
The checker validates48 reference gates:46 numerical comparisons plus two
independent R score/step certificates. It **does not equate** R beta/lnsigma
solver diagnostics with Rust's RMS-X, concave delta/scaled inverse-sigma
coordinates. Rust's own reported gates and an independently differentiated
source-likelihood natural-coordinate score are checked separately. The checker
also binds the joint inverse to independently derived observed information,
normal tails/intervals, positive log-Wald scale bounds, full-joint AME variance,
all600 native/R/source-formula predictions and1400 original cells/ordinals.
Scalars use1e-9 absolute+1e-8 relative; positive tails use1e-8 relative (never an
absolute zero-tail escape); probabilities use1e-10 absolute. Joint inverse
identity residuals must be below1e-8, score certificates below1e-10.

Nine corruption regressions reject wrong source hashes/order/cells, latent means
substituted for censored means, incomplete references, shared zero tails, shared
wrong joint covariance/retained gradients and a shared false stationary control.
They mutate only temporary copies and preserve all six input fingerprints.

Native execution/preparation needs no Python/R; the optional development audit
uses Python's standard library and R/survival only for reference generation.
Use a fresh work directory: existing exports are never replaced. Bare `predict`
remains the **latent** mean; observed censored means require `, mean`, not count n
or a conditional truncated-mean interpretation. `estat margins` uses the retained
estimation design, not current edited X. In CLI-v1 the small retained mean
uncensoring probability and its full beta/lnsigma gradient support this audit;
full internal fitted vectors are not dumped into the fit envelope.

The native author-program extension is not a published coefficient-table replay
or whole-illustration/Chapter16 completion. The theoretical Figure16.1,
truncation/selection/two-part/CLAD workflows remain required.
