# `gsort` syntax slice review

Status: accepted after PR #28 (`fd94133`) squash merge; independent review,
hosted checks, readiness, merge, branch cleanup, and post-merge verification are
complete.

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
- The independent parser review found three additional P2/P3 tokenizer-edge
  differences: Python Unicode alphanumeric classification around U+0345,
  attached `if` condition forms, and quote diagnostics after early assignment/
  option delimiters. They are explicitly recorded as out-of-contract deferred
  behavior in `01-contract.md` and `02-evidence-migration.md`; no parser or
  execution scope was widened to absorb them.
- The independent workspace-scope review found no actionable implementation,
  ownership, backend, unsafe-code, dependency, or unrelated-file findings.
- Local full baseline and policy gates passed at `44ac3ca`; all seven hosted
  PR-head jobs passed on `5a837437d86cfa30e7dbc81dcea6f646eee75ec8`.

## Disposition

Independent review is complete with the tokenizer limitations explicitly
dispositioned as deferrals. PR #28 was accepted and merged as `fd94133`; the
temporary branch was deleted locally and remotely. The documentation closeout
commit `1ed5343` is on `main`, whose post-merge CI and native checks passed.
