# Phase 1 preliminary feasibility assessment

**Disposition: partial / in progress, not complete.** Updated 2026-10-12 after
bounded PRs #162, #163, #165, #166, the measurement slice #168 and
native static-rendering #169, Poisson/plugin #170, NB2/Linux #171, IV #172 and
balanced panel FE/explicit-intercept RE #173, observed-information Tobit #174 and
quantile LP/canonical iid #175, PH survival/observed covariance #176 and
proper two-step selection #177. The
[roadmap](../TABDAT_RUST_PORT_ROADMAP.md) remains the acceptance tracker.
This assessment does not replace a final accepted backend decision or authorize
shipping native artifacts. Proposal intent is not implemented support.

## Evidence and decisions at the current boundary

| Capability | Verified evidence | Current disposition | Missing acceptance evidence |
| --- | --- | --- | --- |
| DuckDB | Local CSV/Parquet/Arrow and bounded runtime; owned connection; HTTP ranges/values/NULL/Arrow and unsigned S3 path-style loopback reads | Continue canonical-engine bounded evaluation under ADRs 0003/0007 | TLS/authenticated S3, extension/offline/native distribution and comprehensive upstream ownership/unsafe review |
| ReadStat | Existing pinned macOS/Linux native builds; Rust DTA facade, labels and system/tagged missing fixtures | Continue isolated DTA candidate, **do not accept product ingestion yet** | Recorded source-quality findings, callback/native review, broader format/oracle references, string-key/empty-string policies, redistribution |
| libgretl | Existing cross-platform build; owned serialized/checked boundary; NIST/Python OLS, Python/R HC1+cluster, Python/R logit+probit and classical integer Poisson; opt-in shared-core plugin build/load; measured first/repeated use | Continue isolated statistical candidate under ADRs 0005/0006/0010; **primary backend not yet accepted** | Remaining estimator fixtures, plugin lifecycle/packaging, general sample/model/inference normalization, updated platform/unsafe/native-license review |
| Visualization | #169 native SVG/resvg scatter, explicit-bin histogram and prepared bar semantics, PNG/raster tests, policy-clean graph and release measurements; rejected #164 preserves exact Python scatter | Accept bounded static-rendering spike under ADR 0011; **no Vega-Lite or product plotting parity** | Automatic bin/category policy, full layout/font UX, Linux/platform and transitive safety/redistribution review |
| Linear algebra | Current safe workspace QR/OLS boundary and libgretl BLAS/LAPACK build evidence | No general linear-algebra substrate is newly selected by these spikes | Bounded faer versus BLAS/LAPACK benchmark, ownership/build/thread/license evidence and accepted ADR |

The original proposal's primary-libgretl and ReadStat preferences are candidates,
not automatic acceptance. The current native OLS/statistical workspace code does
not establish a broad numerical substrate decision or replace the libgretl
estimator feasibility requirements.

## Remaining acceptance work / recorded findings

1. **Rejected plotting candidate:** reproducible `vl-convert-rs` 1.9.0 renders
   correctly but has 19 vulnerabilities / 32 denied warnings. Fresh rc7 application
   resolution still has 4 vulnerabilities / 6 denied warnings (the earlier
   published-lock-only audit had 4/7). Upstream pins prevent targeted upgrades.
   [Draft #164](https://github.com/SaehwanPark/tabdat-explore-rs/pull/164) remains
   unmerged. Native SVG + resvg 0.48.1 **resolves the bounded static-artifact
   advisory gate** with 0 vulnerabilities / 0 warnings and no waiver, under
   [ADR 0011](../adr/0011-static-svg-renderer-feasibility.md). Its
   [report](native-svg.md) records exact fixture semantics but differing layout,
   explicit-bin/category limitations, fonts and outstanding platform/safety work.
2. **Native plugin scope:** #170 resolves the bounded classical integer Poisson
   prerequisite: opt-in shared core plus count module, matching linkage, private
   bounded path and serialized process lifetime. Default static core is unchanged.
   [ADR 0012](../adr/0012-count-plugin-shared-core-spike.md) and
   [report](poisson-plugin.md) record gettext/host-detection prerequisites, loader
   error recovery, nonrelocatable paths and native otool diagnostic. Other module
   lifecycles, count options and full platform/packaging acceptance remain open.
3. **Remaining numerical fixtures:** dynamic panel fixture remains.
   Heckman two-step now passes corrected outcome/observed probit equation blocks,
   exact distinct samples and sigma/rho ([report](heckman.md), ADR 0017), with
   original ordinary-OLS SE discrepancy and zero-placeholder cross block explicit.
   Native early-failure dataset mutation and packed-size arithmetic are guarded;
   MLE/robust/missing-outcome and broader selection contracts remain open.
   Exponential/Weibull PH survival now passes full observed references with explicit
   AFT/censor/time-density and approximate-SE normalization ([report](survival.md),
   ADR 0016). Quantile LP/canonical iid now passes HiGHS/R references,
   with original IRLS and native sparsity deviations explicit ([report](quantile.md),
   ADR 0015). Classical Tobit now passes observed-information
   references, explicitly not fallback BFGS SE parity ([report](tobit.md), ADR 0014). Balanced FE/explicit-intercept RE now pass
   Python/R plm references ([report](panel.md)); other panel contracts remain open. NB2 (ADR 0013,
   explicit optimizer/name normalization) and IV/2SLS now pass trusted references,
   including fresh Linux x86_64 emulated runs. Passed linear/binary fixtures cannot substitute for these families.
4. **Safety/distribution gates:** new libgretl safe-API defects were corrected and
   reviewed, but no comprehensive native/transitive unsafe review, sanitizer/leak
   proof or license/NOTICE/corresponding-source distribution approval ran. The
   ReadStat report retains warnings about native non-DTA error paths; do not
   interpret build-warning demotions as source-quality approval.
5. **Verification risk:** an unchanged runtime reshape test failed once on a
   generated temporary Parquet read; focused and full reruns passed, cause
   unproven. Tracked separately as
   [issue #167](https://github.com/SaehwanPark/tabdat-explore-rs/issues/167).

The bounded static-rendering graph now passes mandatory dependency policy.
The "no unresolved P0 feasibility blocker" gate remains unchecked pending a full
feasibility review; native plugins, unvalidated estimators and safety/distribution
gates independently prevent a whole-phase completion claim.

## Next bounded loops

1. Preserve draft #164 as rejected-candidate evidence. Native SVG/resvg is the
   accepted bounded static-artifact candidate, not production plotting. Follow up
   with automatic bins/category normalization, layout/font and Linux/platform
   evidence before integrating it into command execution.
2. Continue from the validated count/NB2, IV and balanced panel fixtures with
   another bounded plugin family. Preserve predeclared tolerances,
   Python/R reference provenance and exact samples. The integer/fraction contract
   and native initialization diagnostic need explicit product decisions before
   integration; passing Poisson does not accept all count models or plugins.
3. Port/validate the remaining estimator families in small dependency-ordered
   PRs, preserving sample identity, covariance/optimizer conventions and failures.
4. Review ReadStat and DuckDB unsafe/lifecycle boundaries and distribution matrix;
   close native source-quality/semantics issues or explicitly reject candidates.
5. Benchmark/select the initial general linear-algebra route and record accepted
   backend/license/packaging decisions in ADRs. Review every Phase 1 exit gate.

## Workflow / verification summary

Every implemented target started with an early draft PR and progress commits.
Completed, locally verified slices were merged autonomously per user authority;
the failed-policy renderer remains draft. Actions stay disabled under ADR 0009.
Root locked format/check/test/Clippy, advisory/deny and first-party unsafe policy
checks are separate from statistical/FFI acceptance. Python/R were external
validation tools only; no Python/R runtime dependency was added.

Reviews were three separate **owner** passes per completed slice (with follow-up
on corrected VCV behavior), not independent hosted reviewers. No independent
reviewer runtime was available. The source/results, remaining limitations and
review disposition are recorded in individual reports and PRs. Do not represent
this preliminary matrix or any PR merge as completing the requested Phase 1.
