# Migration decision log

This log tracks authority changes and intentional deviations, not feature completion.

| ID | Status | Scope | Decision / evidence |
| --- | --- | --- | --- |
| MIG-0001 | Accepted | Repository and baseline | [ADR 0001](../adr/0001-isolated-rust-port-and-migration-authority.md); Python commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` is the initial external oracle. |
| MIG-0002 | Unresolved | `status` tokenizer diagnostics beyond the bounded sign forms | At the pinned Python revision, `status -1` and `status +1` return `unsupported token in command: -` and `unsupported token in command: +`; the bounded PR #30 correction now matches those forms, including attached signs and a bare `status if`. The broader status lexical matrix (other punctuation, malformed numbers, and malformed condition expressions) remains unresolved until a future tokenizer slice. See the detailed reproduction, paths, impact, and resolution criteria below. Do not claim whole-parser/status parity from the bounded correction. |
| MIG-0003 | Accepted for bounded evaluation | Eager local-Parquet `use` path normalization | Python expands `Path(path).expanduser()` before suffix/existence checks; the bounded Rust runtime accepts an already-resolved local path and deliberately leaves `~` expansion to its caller. This is the documented scope deviation in [ADR 0007](../adr/0007-eager-parquet-duckdb-runtime-boundary.md), `_workspace/use-eager-parquet/01-contract.md`, and `_workspace/use-eager-parquet/02-evidence-data.md`, reviewed in PR #22. |

| MIG-0004 | Accepted | Syntax resource budgets | [ADR 0023](../adr/0023-syntax-resource-budgets-and-fuzz-tooling.md), PR #194: reject expressions above 256 tokens and command parsing at depth 32 to prevent reproduced stack exhaustion. |

| MIG-0005 | Accepted | Script seed syntax/range | Signed 64-bit ASCII decimal seeds; preserve pinned oracle outputs and assert the five explicit Rust rejections. PR #195; see details below. |

| MIG-0006 | Accepted | CLI script exit categories | Classify failure sources instead of diagnostic words; explicit corrections and compatibility impact in PR #205 and details below. |

| MIG-0007 | Accepted | Summary finite sample | Per-variable finite non-NULL sample for all summary aggregates; correct non-finite failures without masking poisoned output. PR #209; details below. |

| MIG-0008 | Accepted | Noninteractive CLI I/O failures | Deliberate exit 1, fallible writes/flush boundaries and script stop guard instead of print-macro panic/exit 101. PR #214; details below. |

| MIG-0009 | Accepted | Bounded conditional keep/drop | PR #230 installs TRUE-row selection with complementary FALSE/UNKNOWN handling and existing predicate rules, not implicit finite filtering or Python/Stata missing-order parity; details below. |

| MIG-0010 | Accepted | Literal quoted `run` paths | PR #243 intentionally changes `run "analysis.td"` from targeting a filename containing quotes to targeting `analysis.td`, and admits spaces/commas inside single/double-quoted paths. Nonempty literal paths only; no escaping, tilde expansion, run options or JSON batch-`run` expansion. Existing unquoted paths and nested-relative resolution remain unchanged. Evidence: parser tests in `crates/tabdat-language/src/lib.rs`/`tests/parser_contract.rs`, plus `tests/quoted_run_paths_contract.rs` for nested text/JSON-script results, located failures, recursion and data/model retention. This supersedes the former Rust quote-preservation expectation; no new Python parity claim is made. |

| MIG-0011 | Accepted | Single-predictor/no-intercept VIF | PR #253 corrects the prior Rust diagnostic failure after a valid `regress y x, noconstant`. The zero-column auxiliary projection predicts zero: SSR equals uncentered TSS, so R² = 0 and VIF/mean-VIF = 1 for a nonzero finite predictor. All-zero input has undefined R²; invalid names/dimensions/nonfinite inputs and general zero-parameter/rank validation remain rejected. Subsequent batch/script commands now proceed, retaining normal output-failure/non-atomic semantics. Independent NumPy zero-column least squares and the statsmodels no-constant R² definition validate this base case; the standalone statsmodels 0.14.6 VIF helper itself raises an empty-design error, not a numerical reference for it. `tools/capture_vif_reporting_reference.py` records five new parent-sample cases with unchanged pins/tolerances and all seven prior cases exactly preserved. Evidence: library/runtime VIF contracts and `tests/cli_json_vif_contract.rs` for samples, state, nested JSON/terminal prediction/export equivalence. This is an intentional mathematical correction, not new Python TabDat parity or a general zero-parameter fitting API. |

| MIG-0012 | Accepted | Prediction discovery option spelling | PR #254 corrects `CommandSchemaResult` for `predict` from the parser-rejected `probabilities` descriptor to accepted reference spelling `pr`, with alternative-option syntax. Consumers of those literal descriptor/syntax strings must update; envelope version/shape, argument requiredness, command effects, parser acceptance and execution remain unchanged. `pr` still does not execute. Packaged help now teaches only installed xb/residual prediction and labels future forms reference/parser-only, rather than promising uninstalled estimator workflows. Evidence: `tests/cli_discovery_contract.rs` checks exact discovery/help, accepted syntax-only previews and rejected alias/conflicting options; `tests/interactive_contract.rs` executes every topic example through export against the existing independent linear reference and checks missing-outcome/target recovery; CLI linear errors reject the deferred forms without writing posterior draws. No new Python parity or prediction calculation claim. |

| MIG-0013 | Accepted | Joint AR extension and `estat` diagnostics | PR #266 adds executable `estat ar, null(b1 b2 ...)` homoskedastic weak-IV-robust joint inference, absent from the pinned Python parser. Every endogenous coefficient must be specified in fit order; exogenous coefficients remain unrestricted nuisance parameters. Eight pinned malformed `estat` calls now enumerate `ar` in their otherwise unchanged diagnostic lists. The original oracle fixture remains unchanged; `tests/python_oracle.rs` explicitly checks only these eight new list spellings, while `tests/parser_contract.rs` and `tests/estat_ar.rs` test current syntax/errors. Runtime/CLI/statistical tests and the independent R auxiliary-regression reference establish the bounded behavior; no confidence-set, robust/cluster AR or whole-estimator parity claim. |
| MIG-0014 | Accepted | Bounded native logit workflow, not whole-command parity | PR #270 installs model-based unweighted logit, explicit `predict ..., pr`/xb and continuous retained-X `estat margins`. Bare predict deliberately retains the existing parser xb default; probability requests must be explicit. Retained numeric y must be exact 0/1 with both classes, not Stata's implicit nonzero recoding. Covariance is inverse Fisher without N-K scaling, normal95/delta inference; robust/cluster/discrete-change forms remain unsupported. CLI-v1 adds typed results and nullable `StatusResult.estimation_family` without changing existing `linear_model`/prediction fields; admission/help lists grow accordingly. Author §14.2 rounded outputs and independent R float64 samples/estimates/covariance/predictions/effects validate the bounded behavior; original parser-oracle fixtures are unchanged. See `examples/mma14p1binary/README.md`, runtime/CLI logit contracts and `docs/reference/command-results.md`. |
| MIG-0015 | Accepted | Native SVG scatter/line artifact contract | PR #276 installs only explicit `.svg` saving with `noopen`, optional up-to-three `lines()`, deterministic outcome-unit y-display `jitter()` and `title()`. Discovery now advertises required saving/noopen and the bounded native options rather than bare deferred syntax; consumers of literal syntax/descriptor strings must update. Older forms still parse but fail with actionable configuration errors. Additive command fields omit defaults and deserialize old payloads; CLI-v1 adds `PlotArtifactResult` without changing old results. Per-mark finite float64 selection, internal stable line sorting/gaps, raw coordinate metadata and no-clobber publication are explicit, with unchanged data/labels/fit/order. This is not Stata RNG/percentage jitter, pixel parity, arbitrary SVG, PNG, anonymization or whole-command parity. Original oracle fixtures stay unchanged; parser, runtime, publication and CLI/geometry contracts plus the Figure14.1 R reference validate this subset. |
| MIG-0016 | Accepted | Native immutable named-fit reports and comparison | PR #278 adds `estimates store/table/drop` for unweighted OLS/logit/probit reporting only, absent from the pinned Python catalog. No restore/refit/persistence/options parity is implied. Explicit case-sensitive ASCII names, collision recovery through deliberate drop, bounded aligned comparison and atomic multi-drop preserve active data/labels/order/estimation. CLI-v1 adds store/table/drop result types with selected coefficient covariance/sample provenance and explicit binary normal versus OLS t (clustered G−1) inference; applicable likelihood/R² conventions stay distinct. Discovery gains one command/effects/help entry and JSON admission/error lists grow; literal catalog/count/string consumers must update. Domain snapshots add default-empty omitted-empty `stored_estimates` and optional linear `estimator` provenance; old serialized states remain readable, but unknown provenance is not guessed as OLS. Original Python oracle fixtures remain unchanged: the workspace boundary test explicitly checks exactly the one new estimates read/control entry, then compares every pinned entry unchanged. Runtime/CLI/serde and exact Table14.2 R evidence validate this report-only subset, not current-data identities/staleness, arbitrary stored post-estimation or whole-family completion. |
| MIG-0017 | Accepted | Bounded native Poisson workflow and explicit mean prediction | PR #280 installs unweighted Poisson ML/independent-row N/(N−1) robust QMLE, normal95/guarded Wald, descriptive deviance/Pearson and retained-X continuous delta effects. Native `poisson y` enables intercept-only fits; exactly this old rejection plus the bare/conditional syntax messages are explicitly reconciled in `tests/python_oracle.rs`, leaving all pinned fixtures unchanged. `predict ..., n` is a new mutually exclusive mean-count kind (`mean_count` on domain wire); bare predict stays xb, and pr/n reject incompatible families. Discovery syntax/options/help and JSON admission/error-list strings grow; literal consumers must update. CLI-v1 adds Poisson fit/effect results without changing old result shapes. Executable owned state uses new `poisson_fit`, preserving the old `poisson` placeholder for reads. No weights/cluster/offset/NB, count probabilities, general count test/lincom or named count reports are claimed. Exact author ASCII Table20.5 Poisson values and independent R full covariance/normal inference/effects/diagnostics/predictions validate this subset, with source/DTA precision differences and repeated-person dependence explicit; independent-row robust is not panel inference. See `examples/mma20p1count/README.md` and command-result contracts. |
| MIG-0018 | Accepted | Bounded jointly estimated NB2 workflow | PR #281 installs unweighted NB2 mean exp(X beta), variance mu+alpha*mu², joint beta/lnalpha observed-information inverse or independent-row N/(N-1) joint-score sandwich, fixed normal95 beta inference and transformed dispersion intervals. No conditional/Fisher substitution or normal alpha=0 test. Counts0..100000 and lnalpha strictly(-12,12), N>K+1 and final undamped positive curvature/convergence are explicit limits, not clipped-alpha fallback. `nbreg y` becomes executable intercept-only syntax; exactly its old rejection and bare/conditional diagnostic messages are reconciled in the oracle test, without editing pinned fixtures. New owned `negative_binomial_fit` preserves legacy `negative_binomial` placeholder reads; CLI-v1 adds NbregResult/NbregMarginalEffectsResult. Shared exact-count ingestion/atomic exp prediction preserves Poisson behavior; n now supports either installed count fit, retaining the prior error variant identity but extending its wording. Discovery/admission/help strings grow. Native Table20.5 NB2 source/R/full-joint covariance/dispersion/effects/diagnostic/prediction evidence is linked in examples/mma20p1count/README.md; robust is not panel-robust. No offsets/cluster/weights/NB1/count-probability UI/formal alpha tests/general restrictions/named count reports are claimed. |
| MIG-0019 | Accepted | One-way original-person Poisson covariance | PR #282 makes existing cluster(var) syntax executable for Poisson only, with exact joint model/ID missingness and original ID preservation, G>=2, group-score G/(G-1) sandwich using converged information, normal95/delta inference, LR suppression and G-1 slope-Wald rank guards. No HC1, G-1 t, pseudoinverse or few-cluster guarantee. CLI-v1 adds cluster metadata only on clustered Poisson fit/effect results and a distinct covariance label; noncluster shapes stay unchanged. Owned records default/omit None for old reads; no restored caches. Discovery syntax advertises cluster; Poisson terminal iteration wording clarifies outer iterations without changing the counter. Exact binary key conversion and compensated summation are reused without changing binary/NB2 semantics. Development-only original-DTA identity enrichment audits every ASCII source cell/order, preserves no-replace targets, and supports a native RAND person-cluster extension (not a printed Table20.5 column):467 synthetic/551 empirical R quantities/all20,186 source/ID/design/prediction records. No author data/program redistribution or production R/Python/native DTA integration. NB2 clustering, household/site/multiway/survey, offsets, formal tests, general restrictions and named count reports remain outside this slice. |
| MIG-0020 | Accepted | One-way full-joint NB2 score covariance | PR #283 executes existing nbreg cluster(var) syntax using full beta/lnalpha observed-information bread and GROUPED full joint scores, G/(G-1), normal95/transformed dispersion CIs and beta-only Wald G-1 rank guards. No conditional MASS/Fisher substitution, ignored dispersion, HC1/Pearson scaling, G-1 t, pseudoinverse or alpha=0 test. Shared exact joint count/ID exclusions preserve wide IDs and state; starting Poisson detaches groups only for the same-row/design initialization. CLI-v1 adds a distinct cluster_joint_score_sandwich_g_over_g_minus_1 label/four-field cluster metadata only on clustered NB2 results; default/omitted-None owned fields preserve old reads and unclustered shapes. Poisson metadata/note helpers are reused with unchanged Poisson output. Discovery/help advertise one-way NB2; parser/oracle fixtures unchanged. Independent development R joint/group score and curvature checks validate527 synthetic/597 empirical quantities/all20,186 original/prepared/person/year/source/design/prediction records, plus singleton equivalence. Native RAND extension is not a printed Table20.5 column, full design/panel/causal analysis, native DTA or production R/Python dependency. Remaining multiway/survey/few-cluster, offsets/weights/probabilities/formal tests/general restrictions/named reports and richer count/book work stay required. |
| MIG-0021 | Accepted | Retained observed/fitted count-frequency reports | PR #284 installs `estat countprob [, max(10)]`, integer1..100, exact0..max-1 plus >=max, after owned Poisson/NB2 fits. Reports original retained y/means/alpha/sample, observed fractions/counts and means of individual plug-in conditional probabilities/expected counts; never probability at average mean, inference or a formal goodness-of-fit test. Robust/cluster covariance does not establish the full law or change plug-in probabilities on the same sample. New typed runtime errors/owned result and CLI-v1 CountProbabilityResult; existing fit/result shapes unchanged. Shared option parsing recognizes numeric max; bare/unknown estat lists add countprob via explicit oracle delta, pinned corpus unchanged. Log PMFs/direct small gamma/beta tails, well-conditioned complements, compensated integer log-factorials and bounded CF convergence avoid underflow/cancellation without clipping/normalization or changing legacy beta/F values. Independent R2090 scalar/95 aggregate quantities plus every20186 source/design/mean/index record and444092 native-helper individual probabilities validate the native/November2006 author-program design. Printed Table20.6 NB2 row materially differs and remains unresolved; not whole-table/book completion. Current-row PMF/CDF predict, formal tests/uncertainty/named reports and other count/core requirements remain open. R/Python development-only; no author data/program/book redistribution. |
| MIG-0022 | Accepted | Gompertz streg diagnostic/discovery extension | PR #336 adds executable `dist(gompertz)` without changing existing Weibull/exponential parsing. The pinned `dist(loglogistic)` rejection now lists `weibull, gompertz or exponential`; `tests/python_oracle.rs` explicitly checks this single clearer diagnostic, leaving the672-case pinned corpus and gap fixture untouched. `--describe-command streg` corrects variables.required to true to reflect its pre-existing mandatory-predictor grammar, not an intercept-only syntax expansion. Literal diagnostic/schema consumers must update; other result shapes, envelope version and command count stay unchanged. New typed Gompertz result/state are additive; native exponential/cluster, full-family and whole-command parity are not implied. See the bounded source workflow/status in examples/mma17p4duration/gompertz.md. |
| MIG-0023 | Accepted | Native centred-rank LOWESS, not Python smoother parity | PR #339 installs the previously parser-only Rust command using the documented rank-centred/shrinking tricube local-linear method for the MMA PSID source-data workflow. This deliberately differs numerically from pinned Python executor.py:6108-6150, which delegates to statsmodels lowess defaults; Python/R nearest-neighbour/robust defaults are not the oracle, and this is not a claim that they are statistically wrong. Preserve existing AST/grammar/default2/3 and joint-finite original-position NULL semantics; source spans.8/.4/.1 are explicit. Stable numeric-X/input-order ties, exact-zero-radius local means, k0 identity, window-local response scaling and N/span/work/representability bounds are explicit native conventions. No owned smoother model is installed or old fit cleared. Additive CLI-v1 LowessResult/admission/diagnostic strings and bandwidth discovery reflect executable support without envelope-version/catalog-count changes; old deferred-runtime errors change accordingly. Independent base-R per-window weighted QR, exact sample/typed-publication/state tests and scale-relative locality regressions validate the defined native method, not historical tied-curve or whole-command parity. Pinned parser/oracle fixtures remain unchanged. Source-script/SVG and closing validation status remain in examples/mma09p1np/lowess.md; do not infer whole-Figure9.3/chapter completion. |
| MIG-0024 | Accepted | Bounded explicit-formula NLS, not SciPy whole-command parity | PR #340 installs `nl` with numeric +,-,*,/, unary minus and exp, explicit ordered parameters/starts and no automatic intercept (`noconstant` leaves the formula unchanged). Joint-finite original-order samples, QR Gauss–Newton with score AND full undamped-step convergence, final rank/observed-loss curvature and fail-closed representability replace Python SciPy/pseudoinverse permissiveness. Classical RSS/(N-P) and HC1 N/(N-P) GN covariance assume the correct conditional mean; approximate Student-t(N-P)95 differs explicitly from pinned Python normal-reference inference. Zero residual variance fails; legitimate zero HC1 SE retains unavailable t/p/CI, not certain inference. Default/xb/mean is the nonlinear formula mean on current X, not a linear index; residuals additionally require current y. Additive owned `nonlinear_fit` preserves legacy `nonlinear` ModelState/tag, with wire roundtrips; new CLI-v1 NlsResult, admission/text/help/discovery and meaningful errors replace deferred-runtime failures without changing envelope version/catalog count/parser grammar. Existing result shapes are unchanged. Independent R analytic controls/profile-amplitude root and original-data pure API validate coefficients/full covariance/inference/projections; actual original-source CLI replay adds120055 numerical/two stricter tail gates and source/forgery/IO/JSON-text evidence; closing checks/review remain WIP in examples/mma05p2nls/README.md. No weighted/FGNLS/ML/known-variance/marginal-effect or whole-Table5.7 claim, optimizer framework or production R/Python dependency. |

MIG-0003 is limited to the
bounded runtime evaluation; the recorded syntax ports establish Rust command
behavior only for their recorded forms, not whole-language parity.

For each conflict/deviation, append an entry with a stable ID and details:

- Status: unresolved, accepted, rejected, or superseded.
- Affected contract and Python/Rust revisions; source/doc/test paths.
- Minimal reproduction, expected/observed results, and environment.
- Alternatives, user impact, and rationale (link an ADR for accepted deviations).
- Exact tests/reference evidence and resolution criteria; blocked dependent slices.
- Reviewing PR and superseding entry, if any.

Never delete unresolved evidence or silently rewrite tolerances/fixtures. Baseline
updates also get an entry identifying old/new pins and affected contract checks.

### MIG-0002 details

- **Affected paths and revisions:** Python `src/tabdat/parser.py` at the pinned
  oracle revision; Rust `crates/tabdat-language/src/lib.rs` and its
  `rejects_arguments_for_read_only_and_exit_commands` regression assertions at
  PR #13's follow-up revision `622e100`, merged as `f6525b3`, with the bounded
  sign/empty-condition correction in PR #30. The surrounding migration evidence
  is `_workspace/parser-describe-syntax/02-evidence-migration.md` and the
  review disposition is `_workspace/parser-describe-syntax/03-review.md`.
- **Minimal reproduction:** in the clean Python oracle, run
  `PYTHONDONTWRITEBYTECODE=1 uv run --no-sync python` and call
  `parse_command("status -1")` and `parse_command("status +1")`; the expected
  errors are `unsupported token in command: -` and `unsupported token in command:
  +`. PR #30 adds exact Rust regressions for those forms, `status-1`,
  `status+1`, signed forms before a comma, and bare `status if`.
- **Impact and rationale:** only malformed punctuation/empty-condition syntax
  after the syntax-only `status` command is affected; no data, session, or
  runtime behavior changes. The correction is intentionally limited to the
  observed contract and leaves the general tokenizer boundary explicit.
- **Resolution criteria and blocked work:** a future tokenizer/parity slice must
  compare the remaining status error matrix, choose Python parity or an accepted
  Rust deviation for other punctuation, malformed numbers, and malformed
  condition expressions, add exact regression tests, and supersede this entry.
  No current bounded syntax slice is blocked; claims of whole-parser/status
  parity remain blocked until that decision is resolved.
- **Review tracking:** discovered during the PR #13 contract review, rechecked
  before PR #14 implementation, and partially addressed by PR #30. Keep this
  entry unresolved until the broader matrix is decided.

### MIG-0003 details

- **Affected paths and revisions:** Python `src/tabdat/backend.py:3945-3966` at
  oracle commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`; Rust path handling in
  `crates/tabdat-runtime/src/lib.rs` at PR #22. The contract and review are
  `_workspace/use-eager-parquet/01-contract.md` and
  `_workspace/use-eager-parquet/03-review.md`.
- **Minimal reproduction:** Python resolves `use ~/patients.parquet` through
  `Path(...).expanduser()` before local-file validation. Rust receives the
  caller's `PathBuf` unchanged; an unresolved `~` therefore does not name the
  same file unless the caller expands it first.
- **Impact and rationale:** only shell-style home expansion differs. Keeping
  environment-dependent expansion outside the library avoids hidden process
  environment reads at this first runtime boundary; ordinary absolute and
  relative caller-resolved local paths retain the bounded eager-load contract.
- **Resolution criteria and blocked work:** a future path-normalization slice
  must choose caller-only normalization or explicit Rust expansion, add parity
  fixtures for `~` and platform home semantics, and supersede this entry. Broad
  `use` parity remains blocked on that decision and on the deferred formats,
  URI, lazy, and option paths.
- **Review tracking:** accepted for the bounded evaluation in PR #22 and linked
  from ADR 0007; supersede this entry if the runtime API later owns expansion.

### MIG-0004 details

- **Affected contract:** pinned Python parser accepts/rejects large syntax according
  to interpreter recursion behavior; Rust PR #194 adds stable resource diagnostics
  before recursive parsing/AST construction. Existing 672 captured inputs agree.
- **Reproduction:** `generate x = ` plus 20,000 opening parentheses, `1`, and
  20,000 closing parentheses previously triggered sanitizer stack overflow. Rust
  now returns `expression exceeds limit of 256 tokens`. Deep `by`/`bayes:` input
  returns `command nesting exceeds limit of 32`. See ADR 0023 for crash identity.
- **Impact/alternatives:** large formulas must be split; unrestricted recursion or
  simply moving it to a larger stack does not bound recursive AST lifecycle work.
  Limits do not cap raw SQL, script bytes, tokenization memory or backend resources.
- **Evidence/review:** exact-limit and adversarial property regressions, unchanged
  pinned corpus replay, two successful bounded sanitizer campaigns; three local
  owner review passes in PR #194. Future budget changes require measured stack/AST
  lifecycle evidence. Unrestricted input acceptance is explicitly not claimed.

### MIG-0006: source-classified script exit codes

- **Status/scope:** accepted bounded Rust correction in PR #205, replacing the
  CLI message heuristic present at `4da9d96`. No Python oracle change or claim
  of whole-script parity; library diagnostic text and CLI-v1 fields stay intact.
- **Contract:** script success exits 0; command execution/preflight and recursion
  failures exit 1; script syntax/directive/macro/control failures exit 2;
  script-loading failures and missing data files exit 3. Nested failures retain
  their originating category, path and line. Batch `-c` behavior is unchanged.
- **Reproductions/impact:** after a valid `regress`, `test absent` previously
  exited 3 because its diagnostic said “not found”; it now exits 1. A valid
  `use` of a directory said “expected” and incorrectly exited 2; it now exits 1.
  Unknown commands, invalid seeds, undefined macros and missing `end` now exit 2
  rather than the old 1. Missing data files and directory/non-UTF-8 scripts now
  exit 3 rather than the old 1. Automation depending on these misclassifications
  must use the documented categories rather than diagnostic substrings.
- **Decision/evidence:** retain category in owned `ScriptError` at language,
  IO and execution producers instead of matching more words or storing arbitrary
  CLI exit codes in domain errors. `cli_script_exit_contract` covers plain/JSON,
  nested locations, misleading words, prior results/data, and later-file-effect
  prevention. The linear JSON regression test now expects the corrected exit 1.
  Existing script/oracle and workspace tests remain required; no numerical
  calculation, sample, covariance, or model-state policy changes.

### MIG-0005 details

- **Contract/reproduction:** pinned Python `src/tabdat/script.py` accepts
  `seed 1_000`, `seed ٠١٢`, and integers outside signed 64-bit range. The 91-call
  captured oracle includes those two forms and three overflow examples. Rust
  `parse_script_directive` rejects them with `seed expects an integer` and retains
  the script path/line. Python fixture results remain unchanged.
- **Decision/impact:** support optional sign plus ASCII decimal digits in `i64`
  range. Ordinary decimal scripts retain their behavior; scripts using Python's
  expanded integer grammar must use a supported seed. No new arbitrary-precision
  representation or Unicode numeric dependency is warranted for a value currently
  recorded only in ephemeral script context, not consumed by an execution RNG.
  Supporting underscore/Unicode forms alone would not resolve the range issue.
- **Evidence:** `script_seed_contract_is_signed_ascii_decimal_i64` tests signs,
  leading zeros, endpoints and invalid inputs without context mutation.
  `python_script_oracle` checks all 91 captured calls with exactly five explicit
  exceptions and checks their exact Rust errors. Canonical/nested CLI tests verify
  sequential execution and source diagnostics; see [script evidence](phase3-scripts.md).
- **Scope/review:** bounded decision in PR #195 under the refreshed AGENTS.md
  policy; no architecture or public API expansion, so no new ADR. Full Python
  seed parity is not claimed. Specify and validate deterministic RNG semantics
  separately before adding a command that consumes this context value.

### MIG-0007: finite summary samples

- **Scope/reproduction:** Rust `71c24e0` runs native count/avg/stddev/min/max over
  non-NULL values. A loaded DOUBLE column `[1.5, NULL, NaN, +inf, -inf, 3, 4]`
  fails `summarize` during native variance aggregation. Merely turning non-finite
  output into null does not prevent this error or define a coherent sample.
- **Decision/impact:** PR #209 filters each variable to finite, non-NULL numeric
  values before every aggregate: count 3, mean 8.5/3, sample SD with denominator
  2, minimum 1.5, maximum 4. All-invalid/empty samples have count zero and absent
  statistics; singleton SD is absent. NaN/infinity no longer contribute to
  summary counts or extrema. `missing` remains SQL-NULL-only; whole relation
  count, finite-data behavior, typed extrema, ordering and state are unchanged.
  This is an intentional Rust contract correction, not a Python parity claim.
- **Implementation/limits:** native `isfinite` predicate with uncast aggregate
  input, rather than eager row extraction, duplicate calculations, or poisoning
  some statistics while retaining inconsistent counts/extrema. No new dependency
  or unsafe ownership change. Finite native arithmetic can still overflow and
  means/SD retain the engine's floating precision; no broad stability guarantee.
- **Evidence:** `summary-finite-reference.json` and its regeneration tool record
  Python/NumPy/SciPy versions, exact candidate indices and tolerances. Shell,
  batch/nested JSON, loaded-value sample indices, all-invalid/NaN-only/singleton/
  empty cases, metadata/model/row invariants and existing finite regressions are
  tested. Runtime typed fixtures check BIGINT/UBIGINT and DECIMAL(38,2) extrema
  without narrowing, quoted identifiers, and REAL NaN/infinity filtering.

### MIG-0009: conditional row selection

- **Scope:** PR #230 replaces the syntax-only Rust RowFilter rejection at
  `4f77e92` with bounded execution in terminal/scripts/CLI-v1. Column projection
  and its envelopes remain unchanged. This defines the new Rust subset, not a
  changed Python pin or a claim of Python/Stata conditional/missing-order parity.
- **Decision:** keep retains TRUE predicate rows; drop removes TRUE only, keeping
  FALSE/UNKNOWN. Using raw `NOT predicate` for drop would lose UNKNOWN rows and
  break the complementary partition. Only Boolean/NULL expression domains are
  admitted; functions, numeric truthiness, logical-expression and option breadth
  remain outside execution. The existing replace/assert AST/domain/compiler is
  reused, preserving identifier/literal quoting and unsigned arithmetic limits.
- **Missingness:** explicit equality with NULL tests SQL NULL; stored empty,
  NaN/infinity are not implicitly missing. Direct DOUBLE `x > 0` uses native
  comparison (NaN/+inf can match). Existing safe arithmetic `try(...)` and finite
  checking turn invalid/nonfinite arithmetic into NULL: `(x + 0) > 0` is UNKNOWN
  for nonfinite x, while `(x + 0) == null` matches it. This is intentionally not
  the finite estimation/summary sample contract; users select it explicitly.
- **State/impact:** preserve schema/types/source and relative row order, labels
  and definitions; synchronize active named tables consistently with in-place
  transforms. Retain cached original fit/sample/inference without remapping its
  row indices; prediction uses selected current rows. Explicit refit uses the
  selected finite estimation sample. No affected/overflow-row diagnostics or
  general atomic native publication/named-synchronization guarantee is added.
- **Evidence:** `conditional_row_filter_contract.rs` covers complements, NULL/
  stored-empty/nonfinite/arithmetic, quoted/wide/decimal/Boolean/unsigned/empty
  cases, exact row identity, labels/named/cache/failure invariants, native query
  failure, closed-form selected OLS (slope1.3/intercept2.2 at fixed 1e-10), and
  statsmodels 0.14.6/NumPy 2.4.4/SciPy 1.17.1 coefficient/SE/fit/prediction
  agreement. Nested streams, persistence and terminal recovery are checked. Shell writer tests stop later
  filters/files at every batch/nested write/flush failure, retaining completed
  effects. Existing projection/replace/parser regressions remain required.

### MIG-0008: noninteractive CLI I/O failures

- **Scope/reproduction:** Rust `b4125c6`, Darwin arm64: close the reader of
  stdout before `--json -c "use data.csv" -c count` emits its first result.
  `println!` panics with Broken pipe and exits 101. Noninteractive discovery,
  diagnostics and argument-error printing share the same panic-prone boundary.
- **Decision/impact:** PR #214 uses fallible output/diagnostic writers and flushes
  each emitted result/diagnostic before subsequent commands. I/O failure returns
  1, even if an ordinary syntax/file error would otherwise return 2/3, with one
  best-effort CLI I/O diagnostic on stderr. Failed stderr or argument-error
  reporting also returns 1 without another panic. Successful bytes and normal
  parse/runtime/file categories remain unchanged; interactive editing is separate.
- **Rationale/limits:** Suppressing BrokenPipe into success would hide failed
  output; keeping panics gives an accidental exit contract. Script observer
  callbacks are infallible, so the original I/O error is retained and the existing
  next-command preflight guard stops execution. The synthetic stop is not reported
  as a ScriptError. Failure on the final result is still fatal. Completed effects,
  including the command whose output failed, are not rolled back; partial output
  can remain and no JSON error envelope is guaranteed on broken streams. Silent
  plain batch/script success and unused streams are not proactively probed. No
  new file atomicity, native ownership or Python-parity claim is made.
- **Evidence:** `cli_output_tests.rs` injects write/flush failures in discovery,
  diagnostics, first/middle/final batch and nested-script results, with a working
  nested prediction baseline and no later observers. Unix binary tests use peers
  closed before process launch (no timing races) for stdout/stderr, including
  argument errors. Existing CLI/script/envelope regressions preserve normal bytes
  and categories. This is local Unix evidence, not platform certification.
