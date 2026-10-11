# ADR 0009: Retire GitHub Actions automation

- Status: Accepted
- Date: 2026-10-11
- Scope: Repository automation and local validation process

## Context

The repository had six GitHub Actions workflows for workspace CI, the runtime
boundary, and native feasibility spikes. The project owner requested removing
GitHub Actions because the available Actions usage budget was running low. The
existing checks can still be run locally using the pinned toolchain and policy
commands in `CONTRIBUTING.md`.

## Decision

- Disable GitHub Actions at the repository level so no workflow runs on pushes,
  pull requests, or manual dispatch.
- Remove all tracked definitions under `.github/workflows/`:
  `ci.yml`, `runtime.yml`, `duckdb-spike.yml`, `readstat-spike.yml`,
  `gretl-spike.yml`, and `libgretl-spike.yml`.
- Keep the local Rust and dependency/unsafe-code policy commands in
  `CONTRIBUTING.md`. Do not treat historical Actions results as current checks.
- Require a separate reviewed decision to re-enable repository Actions or add
  replacement hosted automation.

This supersedes ADR 0002's separate Linux CI execution choice. The pinned policy
tool versions and configuration remain available for local review.

## Consequences

- GitHub no longer runs formatting, workspace check/test/Clippy, dependency
  policy, unsafe inventory, or native feasibility workflows for this repository.
- Contributors must run the documented local commands before merge. Those
  results do not provide clean-runner or hosted-platform evidence.
- Existing workflow links in specifications, roadmap entries, and feasibility
  reports remain historical records of runs completed before Actions was
  disabled. They do not imply that automation is active.
- No branch protection or required status checks remain configured on `main`.

## Verification

The repository Actions setting reports `enabled: false`; all six workflow files
are removed from the repository. The local development baseline remains in
`CONTRIBUTING.md`.
