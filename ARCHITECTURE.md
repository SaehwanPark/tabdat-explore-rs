# TabDat Rust architecture: current boundaries

This is the production workspace's boundary overview, not a delivery ledger.
The product is not feature complete. A compiled command type is not evidence of runtime support. Consult
[SPEC.md](SPEC.md) for the current capability envelope.

## Physical workspace today

| Component | Ownership |
| --- | --- |
| `tabdat-domain` | Typed commands, results/errors, metadata, session/model state and capabilities |
| `tabdat-language` | Command, expression, tokenizer and script parsing |
| `tabdat-stats` | Pure Rust linear/within-FE/balanced-Swamy–Arora-RE/binary-logit/probit/bounded-Poisson/NB2/quantile estimation, covariance/inference, seeded IID/whole-cluster pairs OLS and joint IID pairs quantile bootstrap, and hypothesis utilities |
| `tabdat-io` | Local path/format and script I/O validation; complete staged SVG publication without replacing destinations |
| `tabdat-data` | DuckDB, relations, SQL generation, owned row/cell conversion and one registered scalar UDF (`tabdat_gauss_phi`, delegating to `tabdat-stats`' validated normal CDF); it depends on `tabdat-stats` only for that validated function, with no reverse edge |
| `tabdat-execution` | Session coordination, data/statistical dispatch, script execution and generated SVG artifact coordination |
| `tabdat-reporting` | Help/catalog, bounded structured reporting, core terminal result rendering and pure owned-data scatter/line SVG generation |
| `tabdat-shell` | Batch CLI, persistent session I/O; terminal editing through Rustyline's safe facade, plain piped input |
| `tabdat-mcp` | Transport-independent typed tools; **no MCP server** |
| `tabdat-runtime` | Compatibility re-exports and integration entry points |
| Root binary | Delegates to shell; no arguments enter the line session |

All eleven first-party packages currently forbid unsafe Rust. The Rust workspace
is not an instruction to create more crates. Prefer coherent internal modules;
consider merging crates only when actual change patterns justify the cost.

Quantile regression uses Clarabel 0.11.1's Rust sparse dual LP with only its
`serde` feature (upstream requires it even without serialization use); Python,
BLAS and SDP backends are disabled. TabDat constructs owned CSC arrays with
validated dimensions, sorted in-range row indices and matching column pointers,
keeps the solver local, and returns only owned statistical values. Clarabel's
internal QDLDL uses unchecked indexing relying on symbolic-factorization and
matrix-dimension invariants; this is third-party unsafe Rust, not a claim of an
unsafe-free dependency graph. No native solver handle or FFI is introduced.

## Invariants to protect

1. **One meaning per command.** Human output, batch/JSON and future interactive/MCP
   interfaces should share the same execution semantics and typed results.
2. **Backend isolation.** Domain/statistical API types do not expose DuckDB
   connections, rows, pointers or backend-specific lifetimes. DuckDB remains the
   one canonical initial tabular engine.
3. **Deliberate state.** Parse and validate before mutations when possible;
   document and test whether failure leaves session state unchanged. Preserve
   existing explicit exceptions (such as new-estimation-attempt model clearing)
   unless changing them intentionally.
4. **Statistical correctness before implementation reuse.** TabDat owns its
   sample, missingness, covariance, post-estimation and reporting semantics;
   backends supply numerical computation consistent with those contracts.
5. **Safety without premature infrastructure.** Ordinary Rust uses safe APIs.
   Approved FFI wrappers own foreign handles and resource cleanup; do not
   expose raw native state to domain code. Initialize heavy capabilities lazily.

## Data and execution flow

```text
Batch CLI / line session / future MCP transport
                   |
              shared parser
                   |
       typed command + Session
                   |
         execution coordinator
          /                \
   DuckDB data/I/O      pure statistics
          \                /
             typed results
                   |
         terminal / supported JSON
```

For SVG scatter artifacts, execution additionally calls reporting's pure,
backend-independent SVG builder, then publishes the completed string through I/O.
This narrow execution → reporting edge does not call terminal/JSON rendering,
introduce backend dependencies into reporting, or mutate data/model state. The
shared execution path owns publication, so batch, scripts and typed callers do
not acquire competing artifact semantics. There is no renderer framework or
raster/font dependency.

Future MCP transport, PNG/other visualization families, ReadStat, libgretl and
other native integrations may reuse the same boundaries but are **not** installed
production capabilities. Phase 1 prototypes demonstrate feasibility, not
platform/release certification.

## Architecture decisions: keep them proportional

- Avoid a second data engine, framework/plugin layer, or generic dispatcher unless
  measured user needs make it necessary.
- Prefer a simple module/private helper over a new crate or abstraction when
  the latter introduces more contracts than it removes.
- Use ADRs for *consequential* backend, FFI, serialization, packaging, or
  dependency choices, not ordinary implementation details.
- When a supported numerical, data or state contract changes, test the observable
  behavior; exact Python module organization and incidental diagnostics are
  not architectural invariants.

## References

- [Accepted Phase 2 topology and state decision](docs/adr/0022-domain-state-and-workspace-boundaries.md)
- [Phase 1 backend feasibility decision](docs/adr/0021-phase1-backend-and-linear-algebra-decisions.md)
- [Script/parser budget and fuzzing decision](docs/adr/0023-syntax-resource-budgets-and-fuzz-tooling.md)
- [Previous detailed architecture](https://github.com/SaehwanPark/tabdat-explore-rs/blob/333b97dd55524dbf5ce55fac510375df99e4649e/ARCHITECTURE.md)

Follow source/tests rather than stale architecture prose if a later implementation
has changed. Update the relevant row above when a boundary actually moves.
