# Migration authority and evidence

## Pinned Python source

[python-baseline.toml](python-baseline.toml) identifies the oracle by upstream URL,
full commit, tree, package version, and lockfile digest. The initial baseline is
[Python v0.25.0 release merge](https://github.com/SaehwanPark/tabdat-explore/commit/16b45d9b66b0d80f32d4d220e84d81bc5180bdbe).
A tag, branch tip, installed package, or sibling directory name alone is not a pin.

Recovery evidence (2026-09-08): the clean sibling checkout `../tabdat-explore`
reported commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` and tree
`601b236788872323af9277d2276a236154a0f129`; GitHub's commit API returned the same
values. Its remote points to the repository in the manifest. Every manifest path
was checked against that commit, and `uv.lock` matched its SHA-256 digest. The
local path is a discovery hint, not a required installation location.

To recover an isolated checkout for future validation (choose a new destination):

```sh
git clone https://github.com/SaehwanPark/tabdat-explore.git tabdat-python-oracle
cd tabdat-python-oracle
git checkout --detach 16b45d9b66b0d80f32d4d220e84d81bc5180bdbe
git rev-parse HEAD 'HEAD^{tree}'
git status --porcelain
```

Do not reset or overwrite an existing checkout. Verify the pin, tree, lockfile,
and clean worktree before recording oracle output. Record Python, dependency,
platform, reference-tool, environment, and seed details relevant to each slice.
Python/R and reference environments are external validation tools, not Rust runtime
dependencies. A recorded lockfile does not prove an existing environment matches it.

## Authority and conflicts

Use paths in the manifest at the pinned revision, not mutable documentation-site
pages. `docs/language-semantics.md` owns cross-command semantics; `docs/commands/`
and `src/tabdat/help/topics/` describe command syntax/options/errors; the scripting,
configuration, and MCP documents cover their respective interfaces. README is
orientation, while Python SPEC/changelog/roadmap describe state/history/intent,
not independent proof of behavior.

For each port, recover a contract jointly from those documents, relevant source,
and tests. Exact fixture inputs and expected errors/state matter: inventory entries
are discovery starting points, not assertions that every command is tested. Parser
and script fixtures include inline strings in their test modules; reference tests
and their matrix require further estimator-specific inspection.

Precedence is explicit:

1. An accepted Rust migration decision for a bounded contract records any intentional
  departure and its evidence. It cannot silently remove safety/statistical gates.
2. Otherwise preserve the pinned Python public contract, using docs, source, tests,
  and observed execution together. A disagreement is an unresolved contract, not
  permission to choose whichever result makes the Rust test green.
3. Rust SPEC/architecture define current Rust support and design; the Rust proposal
  and roadmap guide future work but cannot override observed Python behavior by
  implication. Python module layouts do not constrain Rust architecture.
4. For statistical claims, require an independent trusted reference in addition to
  Python. Stop on unexplained disagreement; never widen tolerances silently.

Register conflicts in [decisions.md](decisions.md) before freezing fixtures or
claiming parity. Include reproduction, both behaviors, affected users, and resolution
criteria. An unresolved entry blocks only dependent slices. A chosen deviation
needs a reviewed ADR and explicit tests/docs; correctness fixes may require a
separate upstream Python PR. This repository does not freeze Python development
or authorize changing that repository merely to make a comparison pass.

## Bounded validation already run

From the clean Python checkout using its existing environment:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py tests/test_script.py
```

Observed: Python 3.13.3; **516 passed in 0.82s**. No sync/install was performed and
the worktree remained clean. This is evidence that the bounded parser/script oracle
suite runs in the discovered environment, not a fresh-lockfile reproducibility
claim. CLI, backend, numerical reference, integrated E2E, and full Python suites
were not run. No Rust/Python differential comparison has been implemented.

## Baseline updates

Update the pin only in a dedicated reviewed PR: record old/new commit and reason,
verify upstream identity/clean tree and lock digest, inspect behavior/test/doc
changes affecting migrated slices, rerun their differential/reference checks, and
update fixtures only with explained changes. Keep historical evidence tied to its
original revision; never relabel old outputs with a new baseline.

## Contract handoff for future slices

- **Python contract:** pin, input syntax/defaults, output/error/state behavior,
  source/tests/docs, and known conflicts.
- **Rust contract:** typed inputs/results/errors, state transitions, ownership,
  and lazy capability requirements.
- **Test contract:** test-first cases, exact comparisons/normalizations, oracle and
  trusted-reference commands, environment details, and evidence paths.
- **Implementation mapping:** native Rust, DuckDB, approved FFI, optional capability,
  or explicitly deferred. No backend implementation is implied by this inventory.
