# Lower-truncated Gaussian regression — validated native workflow

## Exact source and distinction

The March2005 `mma16p1tobit.do` §2A comments on an intended **truncated Gaussian
MLE**, but executes no correct truncated MLE. It explicitly warns that censoring
Tobit on `ytrunc` reproduces truncated OLS instead. This required-method slice
fills that documented gap on the author's original exported data. It is a
validated extension, not historical coefficient/stopping parity, a published
table replay, or completion of Chapter16. Source assets/checksums/preparation are
in [README](README.md); do not substitute a new random stream.

Use `ytrunc ~ lnwage` plus intercept, fixed lower limit0. Exactly130 original
positive observations contribute conditional densities;70 missing truncated
outcomes contribute **nothing**, not censoring/Bernoulli masses. Retain original
row identities and raw cells. The [ledger contract](../../docs/reference/microeconometrics-coverage.md#mma-trunc-bounded-lower-truncated-gaussian-contract-and-evidence)
defines support/missingness, inference, effects, prediction, failure and resource
semantics. Native kernel, owned terminal/script execution, full inference,
retained effects and atomic current-X predictions are implemented and pass
focused/source/script and complete local workspace gates. Independent kernel
and integration source/math reviews found no issues; reviewer execution was
not part of either review. Final acceptance and retained failures are below.

## Independent development reference

- R4.6.1, installed `truncreg`0.2-5 and `maxLik`1.5-2.2.
- CRAN archive `truncreg_0.2-5.tar.gz`, SHA256
  `3e8e6a09eb0a34f2af8b40eb368c2e47bbd6697e0874b595940cad300b948be9`,
  MD5 `6ddc73494a349da69b23f859ffe89a54`.
- CRAN Git mirror commit `8fcad24e3106b94baa093aa9928ca5ceef755916`.
  The generator checks the archive and compares the installed main/fit/likelihood
  function bodies to that pinned source, not just a package-version label.
- GPL>=2 code is a development reference only, not vendored or linked into
  production. maxLik was already available; truncreg is installed in a temporary
  development library, not a runtime/application dependency.

From a prepared working directory, with `repo` pointing at this checkout:

```sh
mkdir -p R-lib R-source
Rscript --vanilla -e 'install.packages("truncreg", repos="https://cloud.r-project.org", lib="R-lib", destdir="R-source", dependencies=FALSE, type="source")'
R_LIBS_USER="$PWD/R-lib" Rscript --vanilla \
  "$repo/examples/mma16p1tobit/reference_truncreg.R" \
  mma16p1tobit.asc R_truncreg_predictions.csv \
  R-source/truncreg_0.2-5.tar.gz > R_truncreg_quantities.csv
```

`maxLik`1.5-2.2 must be available in that development R environment. Future
package releases/source drift fail explicitly rather than silently change the
reference. Existing prediction output is refused. No original data is committed.

The package fit explicitly uses `scaled=FALSE`, natural beta/sigma,
Newton–Raphson and the package's analytic observed information.
The response is divided by1000 solely for numerical conditioning; coefficients,
covariance, log-scale and likelihood Jacobian are all transformed back. Independent
base-R `deriv3` differentiates the actual conditional log density and refines
both full/null observed-Newton solutions. Positive information, score and
unconstrained-step certificates are checked; no OPG or conditional-on-sigma
covariance substitution. Three finite-difference steps separately check the
full-joint conditional-mean AME gradient. A four-start base-R likelihood probe
also reaches the same finite regular source solution, not an existence/global
proof by multistart alone.

Current certified original-source values:

| Quantity | Value |
| --- | ---: |
| Intercept | -1682.93983040031 |
| lnwage slope | 789.454004335952 |
| sigma | 874.882624503239 |
| Conditional log likelihood (130 observations) | -1012.605957391 |
| Same-sample intercept+scale null LL | -1021.50696516938 |
| LR / df / p | 17.8020155567762 / 1 / 0.0000245123441161169 |
| Conditional-truncated-mean AME | 451.714502048142 |
| Full-joint delta AME SE | 104.559756594971 |

This conditional130-row likelihood cannot be compared to the200-row Tobit or
selection likelihood as a joint LR test. The AME is not the latent slope, Tobit
censored-mean derivative, Heckman mean or causal effect.

## Independent reference and kernel validation

The reference emits49 exact-inventory quantities (45 numeric values plus four
separately bounded full/null certificates), all9 beta/lnsigma covariance entries,
600 current-X projections on200 rows,1400 original cells/ordinals and exact130
membership. Parent independent standard-library checks reconstruct source score,
observed-information inverse, normal inference, AME/full-joint variance and
predictions. Six malformed-input/output controls reject existing output,
modified archive, truncated source, nonbinary source selection, modified selected
outcome and an in-memory modified installed likelihood body. They produce empty
stdout/no new prediction file and preserve all four input fingerprints.

The native kernel uses the **concave canonical precision** form:
eta=beta/sigma², v=1/sigma²>0, density proportional to
`exp(y*X*eta-v*y²/2)` on known y>lower. Its negative Hessian is the conditional
covariance of the sufficient statistics; this avoids assuming beta/lnsigma
Newton information is positive away from the optimum. Full beta/lnsigma observed
inference is evaluated/transformed at the certified solution. Scale response and
RMS designs using effective selected data, not a huge statistically inactive
lower bound. Finite interior existence still must be checked, not clipped or
inferred from parser acceptance.

Three independent240-row controls cover intercept/nonzero limit, no intercept/
positive limit and the ordinary Gaussian inactive-bound limit. Their153 exact
reference quantities include27 covariance entries and2160 point projections;
normal/tail/scale/null/LR and full-joint effects compare numerically. Six kernel
tests additionally cover units down to1e-9 (normalized back before comparison),
intercept shift1e6, permutation, IID replication/covariance1/r, strict support,
rank/name/resource/weight/identity failures, a positive exponential precision0
closure and current-mode-specific tail behavior. The canonical sufficient-statistic
score/information passes a private finite-difference test. A native original-source
probe passes45 numerical quantities/nine covariance entries/600 projections and
two separately bounded native certificates; this probe alone does not establish
terminal support.

The existing guarded Gaussian Newton loop is shared with Tobit without changing
its initialization, positive-information check, tolerances,60 backtracks or
convergence/failure messages. All six Tobit integration tests and its existing
private derivative test pass. A new malformed-sample test exposed that generic
problem validation permits repeated identities; this estimator now explicitly
rejects repeated/out-of-range or inconsistent candidate/drop identities under
its declared independent-original-row contract. The initial duplicate-ID test
failure and test-literal compile error remain recorded; no tolerance widening.

Frozen118a9d3 independent read-only kernel/math review found no issues; it did
not execute tests or approve the then-pending frontend. Affected stats validation
reconciles199 passed/one pre-existing ignored across46 all-fresh hash-checked
artifacts. Stats doctests and eight reverse-consumer runtime/actual Tobit script
tests pass. The original nextest discovery timeout remains incomplete; a bounded
direct fallback retained a sampled dyld loader stall, unstarted work and a helper
termination/natural-exit race. Only diagnosed incomplete/unstarted targets were
resumed; accepted records preserve prior diagnostics and all current hashes.

## Native workflow and focused integration evidence

From a fresh prepared directory (export refuses an existing target):

```sh
cp "$repo/examples/mma16p1tobit/truncreg.td" .
"$repo/target/debug/tabdat-explore-rs" --json -f truncreg.td > truncreg.jsonl
python3 "$repo/examples/mma16p1tobit/check_truncreg.py" "$PWD"
python3 "$repo/examples/mma16p1tobit/check_truncreg_failures.py" "$PWD"
```

The shipped script loads original data, fits, reports retained-X conditional-mean
AMEs, scores xb/population retention/conditional mean, and exports. Real original
JSON/text runs exit0 with byte-identical exports. The standard-library audit
passes45 numeric comparisons+four bounded R certificates, nine covariance entries,
600 predictions,1400 original cells/ordinals and exact130 membership. In addition
to both native canonical certificate bounds, it reconstructs full raw-coordinate
score/positive observed information/inverse/step, normal/log-Wald inference,
full-joint AME and an INDEPENDENT same-sample null via conditional mean/variance
moment-ratio bisection, then checks null stationarity/LR. Eleven temporary-copy
corruptions reject, including paired native/R covariance/prediction corruption,
shared zero tail and forged native+R stationarity caught at the independent score
gate; six input fingerprints stay unchanged. A later paired native/R SE/statistic/
p/CI corruption exposed a missing coefficient-SE-to-independent-covariance link:
the guard first failed because the poisoned audit passed. The audit now anchors
each coefficient SE to the independently rebuilt observed inverse; all12 guards
pass, preserving six fingerprints. This was an audit-strength fix, not changed
native estimates/covariance or widened tolerances.

Four runtime tests cover both independent intercept conventions/full covariance,
exact asymmetric missingness/support membership, current-X-only projections,
retained effects after edits/drop, labels/order/owned state and fit-vs-prediction
failure atomicity. Extremely negative current indices legitimately score xb/pr0
while conditional-mean tail failure publishes nothing. Parser2/wire2 pass; new
serialized requests reject unknown inference fields rather than silently drop
robust/cluster/weight requests and report IID covariance. Owned
state has a distinct lossless tag and the existing generic mean option retains
its legacy wire tag while resolving by fitted family.

Two actual shipped-script tests use a separate240-retained/242-row affine control,
NOT substitute author data. The reference translates certified R case1 by+0.6,
then verifies a fresh pinned-package fit and full observed symbolic full/null
refinement. Its49 gates/nine covariance entries/726 projections, original cells,
JSON/text equivalence and existing-target/missing-source/export-sentinel cases
pass. The package default endpoint differed by5.884176e-8 in a parameter: the
initial strict reference check failed and was preserved. Actual-density symbolic
Newton refinement resolved it without widening any gate. A u64/usize row-cap
compile error was corrected before the four runtime tests passed.

Discovery/help/schema/explain are implemented; their focused contract passes.
Earlier changed-library/focused Clippy checks pass. The first broader attempt
timed out after checking the root package and remains an incomplete attempt;
final workspace all-target Clippy subsequently passed with warnings denied.

## Final slice acceptance (PR329)

Independent frozen a2a5708 integration/source-math review found no issues. It
explicitly did not execute commands/tests and was conditional on the parent's
remaining regression gate. An earlier same-profile reviewer request failed on
the provider quota limit and gave no approval; the successful retry occurred
after an authoritative session reset, with no route substitution.

Parent local macOS validation now reconciles **1515 passed,0 failed,two
pre-existing ignored** across **264 current artifacts**. All were inventoried
all-fresh and SHA256-checked; the current main executable hash was stable.
Forty-five unchanged non-root artifacts were reused from observed exit0 runs
with identical current hashes and no root-binary dependency. Subsequent passing
records were retained only with exact current artifact hashes; root consumers
also bind the unchanged current main hash. The final two targeted discovery/
compatibility binaries passed31 tests after test-only assertion fixes; all
remaining diagnosed loader-incomplete or unstarted work then passed. Workspace
all-target Clippy with `-D warnings`, workspace doctests, formatting and diff
checks pass. A fresh current-binary original-source JSON/text replay exits0 with
byte-identical exports; the49 reference gates/nine covariance entries/600
predictions/1400 cells/exact130 membership and all12 corruption guards pass.

Initial bounded workspace builds/discovery/runner timeouts remain incomplete,
not passes. A parent runner preflight initially required obsolete hashes for
the two deliberately rebuilt test targets; it stopped before executing tests,
was corrected to compare those targets against the current inventory, and its
failure record remains separate. Actual regression failures exposed stale
command/help counts and the Rust-only extension list; assertions now explicitly
pin `truncreg` help/read effects while preserving the immutable Python oracle.
These test/audit corrections did not change native estimates, covariance or
numerical tolerances. Sampled dyld preparation/fixup stalls, unstarted work and
all earlier diagnostics remain recorded; only diagnosed incomplete/unstarted
work was resumed, not silently reclassified.

This is not whole-Chapter16/book/platform/release certification. Scope is
fixed-left-truncated IID homoskedastic
Gaussian ML; no upper/variable truncation, robust/cluster/weights, factors, named
storage, generic tests or fabricated sigma0 test. No numerical tolerances have
been broadened to conceal a discrepancy.
