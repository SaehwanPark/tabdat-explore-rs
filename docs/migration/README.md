# Python-to-Rust migration: practical compatibility policy

Python TabDat is a **reference for intended behavior**, not a mandate to reproduce
its implementation details or every incidental Python lexical rule. The Rust
project aims for dependable supported workflows, documented intentional changes,
and scientifically valid statistical results.

## Reproducible source of historical behavior

The pinned Python baseline is recorded in
[python-baseline.toml](python-baseline.toml): commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (v0.25.0), tree
`601b236788872323af9277d2276a236154a0f129`. That fixture is still useful
for behavior comparisons and reproducing prior evidence. For large migration
changes, use the checked-out pinned revision and record relevant versions;
do not assume whichever Python environment is on PATH matches it.

Recover an isolated checkout when needed:

```sh
git clone https://github.com/SaehwanPark/tabdat-explore.git tabdat-python-oracle
cd tabdat-python-oracle
git checkout --detach 16b45d9b66b0d80f32d4d220e84d81bc5180bdbe
git status --porcelain
```

Existing per-slice evidence, accepted decisions and fixtures remain in this
directory. A pin establishes provenance; a passing Python suite alone is not
statistical truth.

## Match the behavior that matters

| Tier | Examples | Evidence and default |
| --- | --- | --- |
| A: correctness | Estimation sample, coefficients/covariance/inference, missingness, transforms and reproducibility | Preserve or deliberately correct. Statistical changes also need independent reference validation. |
| B: supported interface | Documented `.td` commands, supported options, session effects, public JSON/exit behavior | Aim for compatibility; test and document intentional divergence. |
| C: incidental behavior | Exact error strings, rare Unicode/whitespace quirks, unbounded Python integers | Best effort only; a justified Rust-specific limit or diagnostic is acceptable. |
| D: unsupported/future | Parser-only estimator variants, future adapters, broad language ambitions | No parity obligation until selected as a real executable feature. |

Use this order when sources disagree: **trusted statistical/data correctness**,
then explicitly supported Rust contracts, then documented Python behavior,
then implementation convenience. Do not silently revise a known public contract.

## Handling a difference

A migration difference does **not** automatically block unrelated work.

1. Establish whether users can encounter it in a **supported** workflow.
2. Determine the consequence: wrong data/statistics, broken public interface,
   confusing but harmless diagnostic, or entirely unsupported behavior.
3. Fix high-impact issues. For lower-impact cases, choose a sensible Rust
   policy, write one focused regression and note the deviation.
4. Log an accepted nontrivial migration difference in
   [decisions.md](decisions.md). Reserve a full ADR for architectural choices;
   ordinary syntax limits do not require a new committee-style process.
5. Report the actual tested support boundary; do not change oracle expectations
   or broaden numeric tolerances merely to make a comparison pass.

**Example: draft PR #195.** Python accepts Unicode-decimal and arbitrarily large
`seed` integers; the Rust script context currently uses `i64`. Decide whether
user-facing reproducibility needs unbounded seeds. If not, document a stable
range and format and keep the corresponding test as an intentional deviation,
without treating it as a blocker for ordinary script execution. Seed parsing
and statistical RNG sequence reproducibility are separate questions.

## Tests and evidence

Favor a few canonical end-to-end `.td` workflows, small inspectable data, and
well-chosen invalid inputs. Differential fixtures are valuable when they cover
supported behavior, not necessarily every Python edge case. Exact text goldens
are appropriate for explicitly stable interfaces, not for all error messages.

Keep evidence proportional to risk. Include the command, expected/actual result,
relevant versions, and a short reason for normalization/deviation when needed.
Never claim independent statistical validation from agreement between the Python
and Rust implementations alone.

The older comprehensive provenance policy, historical acceptance details and
recovery instructions remain in the
[original migration README](https://github.com/SaehwanPark/tabdat-explore-rs/blob/333b97dd55524dbf5ce55fac510375df99e4649e/docs/migration/README.md).
