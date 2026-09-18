# `gsort` syntax slice review

Status: implementation complete at `85b1a92`; independent review, hosted
checks, readiness, merge, branch cleanup, and post-merge verification are
pending.

Producer: task owner. Consumers: maintainers of the bounded parser contract.

## Scope to review

- `SortKey` and `Command::Gsort { keys }` plus the pure parser in
  `crates/tabdat-language/src/lib.rs`;
- public parser cases in `crates/tabdat-language/tests/parser_contract.rs`;
- runtime deferral in `crates/tabdat-runtime/src/lib.rs` and
  `crates/tabdat-runtime/tests/use_contract.rs`;
- the pinned contract/evidence artifacts in this workspace slice;
- current-state README/SPEC/ARCHITECTURE/roadmap wording;
- pinned Python authority, focused/full oracle results, local locked checks,
  hosted workflows, merge, and branch cleanup.

## Review questions

1. Does signed-key parsing match the pinned Python model, including quoted
   signs, attached symbols, repeated prefixes, control whitespace, and exact
   diagnostics, without broadening execution scope?
2. Does Rust own key values and stay backend-independent, with an exhaustive
   runtime mapping and explicit unsupported-command result?
3. Are the contract citations, raw probe output, local results, known
   tokenizer deviation, workflow links, current-state docs, and deferred
   semantics reproducible and non-stale?
4. Are required checks green at the exact head under review, with no unrelated
   changes or hidden state mutation?

## Preliminary evidence

- The pinned contract probe and focused/full oracle suites passed; results and
  raw output are recorded in `02-evidence-migration.md`.
- Focused Rust checks passed with 40 language unit + 29 integration tests and
  2 runtime unit + 11 integration tests.
- An independent contract probe review verified the owned `SortKey` shape,
  direction handling, quoted-sign distinction, attached forms, and exact
  diagnostics. It also identified malformed numeric-key handling as an
  intentional tokenizer-parity deferral, now recorded in the evidence ledger.
- Hosted acceptance and workspace-scope review remain pending.

## Disposition

Pending independent workspace review and the complete hosted check set. The
bounded implementation is intended for acceptance only after those checks pass;
the temporary branch must then be deleted and post-merge `main` checks recorded.
