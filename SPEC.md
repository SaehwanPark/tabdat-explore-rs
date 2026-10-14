# TabDat Explore Rust: current product and implementation contract

**Status:** current supported behavior is summarized below; source/tests are
authoritative. Verification details are in the relevant PR threads and
[script evidence](docs/migration/phase3-scripts.md). Update this summary when
implementation changes.

## Product intent

TabDat is a terminal-first, Rust-native, Stata-inspired statistical/EDA tool.
Its value is responsive exploration, dependable statistical analysis, simple
installation, and automation with consistent typed results. It is **not** a full
Stata, Python, R, or earlier TabDat clone. Prefer usable end-to-end workflows to
complete syntax catalogs.

## Implemented and deferred capabilities

The distinction below is intentional. Individual options, errors, edge cases,
and known limitations are established by current source and tests, not inferred
from a parser enum. Historical per-command implementation evidence is preserved
in [the preceding SPEC revision](https://github.com/SaehwanPark/tabdat-explore-rs/blob/333b97dd55524dbf5ce55fac510375df99e4649e/SPEC.md).

| Area | Current state | Important limit |
| --- | --- | --- |
| Binary and CLI | Batch `-c`, scripts `-f`/positional, help, discovery, bounded JSON; noninteractive write/flush failures exit 1 and stop later commands | No-argument invocation starts a persistent line session; batch/script output unchanged |
| Interactive shell | Persistent session, `help`/`?`, readable core EDA/linear results; capable-terminal editing, bounded session-local history and input Ctrl-C recovery; exit/EOF | No completion, persistent history, multiline continuation or in-flight cancellation guarantee; plain piped/dumb input; other result details deferred |
| Scripts | Line commands, comments, multiline SQL, macros, seed, `if`/`else`/`end`, nested `run` with source-classified CLI exits (execution 1, syntax 2, file 3); interactive runs stream readable leaf results | Bounded 91-call compatibility evidence with five explicit seed deviations; seed records context only, not RNG state |
| Data loading and storage | Bounded local eager CSV/Parquet, literal single/double-quoted `use` paths with spaces/commas, local save/export paths | Broad remote/lazy and DTA runtime not installed |
| Data exploration | Bounded inspection, per-variable finite summaries including no-option read-only predicates, frequency tables, key-integrity diagnostics/guards, bounded assertions, data fingerprints and previews; shell/CLI-v1 SQL-NULL missingness and codebook profiles with types, ordering, counts and bounded examples | Not all accepted syntax executes; summarize uses per-variable finite samples, with native finite-arithmetic overflow limits |
| Data transformation | Bounded projection, conditional keep/drop TRUE-row selection, expressions, sort, recode/labels, join/append/reshape, grouped collapse and named-table SQL | Exact support is command/option-specific; do not claim general parity |
| Statistics | Pure Rust linear model/inference utilities, bounded `regress`, `ttest`, `predict`, `test`, `lincom`, VIF; readable shell results for `regress`/`predict`/`lincom`/`test` and all four bounded `ttest` forms | Other estimator families largely parser-only; reference validation remains feature-specific |
| Structured interfaces | CLI-v1 JSON core EDA including read-only describe metadata and missingness/codebook, read-only grouped counts, count-only collapse dataset outcomes, duplicates/isid key guards, bounded assert/datasignature records and no-option conditional summaries, bounded regress/predict/lincom/test, retained-design `estat vif` tables, generate/replace/rename/recode/encode/decode and session-local label dictionaries, SQL tables/named-cohort metadata, named-table activation, lookup joins, compatible dataset stacking and bounded wide/long reshape metadata, column-list keep/drop/select and bounded conditional keep/drop, sort/gsort metadata, save/export outcomes and four-form ttest results, ordered script streams, discovery, and typed transport-independent MCP boundary | No full-results JSON surface or MCP transport/server |
| Visualization | Several command forms parse | Rendering/plot execution not installed |
| Native capabilities | Phase 1 DuckDB/ReadStat/libgretl/plotting feasibility decisions and prototypes | ReadStat, libgretl and plotting adapters are not production integrations |

**Never equate:** command catalog entry = parsed command = implemented execution =
validated statistical behavior = release-quality feature.

## What must be correct

For any **supported** operation:

1. **Observability:** A command must be reachable through its advertised user
   surface, and its output/errors must tell the user what happened.
2. **Data semantics:** Input types, missingness, selection, ordering, labels,
   joins, and post-operation state must be deliberate and tested where relevant.
   Avoid partial mutation on recoverable failure except documented contract cases.
3. **Statistical semantics:** Sample membership and design-matrix conventions,
   estimate parameterization, covariance/inference calculations and model state
   must be specified. Verify changed statistics against an independent trusted
   reference; record meaningful tolerances and disagreements.
4. **Reproducibility:** Scripts run in documented order; supported seed values
   have explicit semantics; saved/JSON results remain stable where promised.
5. **Safety and deployability:** Production application code remains safe Rust by
   default; native resource ownership is local; core runtime needs no Python/R.
   Dependency/license/platform claims require their own evidence.

Correctness is measured at the **supported behavior boundary**, not by achieving
Python equivalence on every incidental lexical input or error string.

## Compatibility policy

| Class | Examples | Default treatment |
| --- | --- | --- |
| Essential | Statistical values/inference, estimation sample, data integrity, script-visible state | Strongly preserve or explicitly fix with high-quality evidence |
| Stable interface | Supported command options, JSON schema/exit behavior, documented `.td` constructs | Preserve or version/document intentional changes |
| Convenience | Error wording, whitespace variants, unsupported parser-only syntax | Best effort; document material differences rather than blocking delivery |
| Historical/future | Unimplemented estimators, prototype adapters, broad Python behavior | Not a prerequisite for a usable Rust milestone |

If an existing supported behavior is deliberately changed, add a focused
regression test, briefly justify its impact, and update user-facing docs. Use a
short note in [migration decisions](docs/migration/decisions.md) for migration
changes; reserve a new ADR for consequential architecture, backend, or API choices.

## Acceptance for next usable release candidate

The next meaningful milestone is **an interactive, dependable core EDA workflow**, not
completion of an exhaustive language phase. Target:

- Run a session interactively without scaffold output; load local CSV/Parquet
  (minimal line session implemented; advanced editing/cancellation remain deferred).
- Inspect, summarize, tabulate, and transform representative small datasets.
- Fit and inspect a supported linear model; predict/test without silent state loss.
- Produce readable terminal results and stable structured output for the advertised subset.
- Run canonical scripts with meaningful path/line errors and explicit seed policy.
- Meet representative statistical reference and data-integrity tests.
- Measure startup/command responsiveness without premature performance promises.

These are **targets**, not statements of current support. See the
[prioritized roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md).

## Where to look for detail

- Source/tests: authoritative for actual implementation and regressions.
- [Architecture](ARCHITECTURE.md): current module/crate boundaries and deferred integration.
- [Migration authority](docs/migration/README.md): pinned Python oracle, compatibility and deviations.
- [Phase 3 language evidence](docs/migration/phase3-language.md): 672-case corpus and parser/fuzz status.
- [Script evidence](docs/migration/phase3-scripts.md): pinned API replay, bounded seed contract and canonical/nested workflow tests.
- [Phase 2 evidence](docs/migration/phase2-core-contracts.md), [Phase 1 report](docs/feasibility/phase1-final-report.md): completed historical decisions.
- [Previous detailed implementation ledger](https://github.com/SaehwanPark/tabdat-explore-rs/blob/333b97dd55524dbf5ce55fac510375df99e4649e/SPEC.md): immutable historical snapshot.

Keep this document concise. Do not append one section per PR or duplicate test
logs here. If detailed evidence is important, place it alongside the relevant
subsystem/test or in the migration archive and link to it.
