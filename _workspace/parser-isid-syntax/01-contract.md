# Contract: syntax-only `isid`

Status: accepted; PR #23 squash-merged as
`a2f4bbb6d4a7cd11bbdbfd2864099a8c98ceac07`. Execution and full tokenizer
parity remain deferred.

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `main` at `da8ab73` (the merged eager local-Parquet runtime handoff,
with green post-merge CI). This slice is intentionally independent of that
runtime boundary except for preserving exhaustive command-name handling.

## Scope

Add one backend-independent parser form:

```text
isid varlist [, missok]
```

The parser returns an owned typed command containing the ordered key-variable
names and whether the exact lowercase `missok` flag was supplied. It performs
no active-dataset lookup, schema validation, duplicate scan, missingness
calculation, session mutation, backend initialization, or result rendering.

## Python contract

The pinned clean oracle is `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`), Python 3.13.3. Authority paths
are:

- `src/tabdat/models.py:193-199` (`IsidCommand` with `variables` and `missok`);
- `src/tabdat/parser.py:3005-3015` (direct command construction);
- `src/tabdat/parser.py:3036-3123,3159-3200` (argument, option, condition, and
  assignment parsing/diagnostics);
- `tests/test_isid.py:83-98` (focused syntax contract);
- `docs/commands/isid.md:1-68` (public syntax and deferred execution
  semantics); and
- `src/tabdat/help/topics/isid.md:1-25` (interactive syntax/help text).

Accepted direct forms are case-insensitive for the command name and preserve
ordered, unwrapped arguments, including quoted or backtick-quoted identifiers:

```text
isid patient_id visit
ISID `patient_id` visit, missok
isid `a,b`, missok
```

At least one key variable is required. The only accepted option is the exact
lowercase flag `missok`; duplicate `missok` flags still produce `missok=True`.
Options are not generally case-normalized by the pinned parser. Conditions and
assignments are outside this direct form.

The frozen diagnostics for this bounded parser include:

| Input | Diagnostic |
| --- | --- |
| `isid` or `isid, missok` | `isid expects at least one key variable` |
| `isid,` | `comma must be followed by at least one option` |
| `isid patient_id if visit > 0` | `isid only accepts a variable list and missok option` |
| `isid patient_id = other` | `isid only accepts a variable list and missok option` |
| `isid patient_id =` | `isid assignment requires an expression after =` |
| `isid patient_id, report` | `isid unsupported option: report` |
| `isid patient_id, foo bar` | `isid unsupported option: bar, foo` |
| `isid patient_id, missok(true)` | `isid option missok does not accept a value` |
| `isid patient_id, missok 1` | `option missok value must use option=value syntax` |
| `isid patient_id if` | `missing expression after if` |
| `isid patient_id==x` | `unsupported token in command: ==` |

Malformed quote and unsupported-symbol diagnostics continue to come from the
existing bounded parser. Full `if` expressions, options beyond `missok`,
varlist expansion, prefixes, and tokenizer parity are not implied.

## Rust contract

Expose one owned syntax-only variant:

```rust
pub enum Command {
    Isid {
        variables: Vec<String>,
        missok: bool,
    },
    // existing variants unchanged
}
```

The parser keeps the existing pure `parse_command` boundary and uses the
bounded argument/option tokenizers already present in the language crate. The
runtime must continue to return a typed unsupported-command error for `Isid`;
it must not add `IsidResult` or data access in this slice. Ordinary crates keep
`#![forbid(unsafe_code)]`.

## Test contract

Unit and public integration tests will cover:

- bare and mixed-case command names;
- ordered multiple keys, quoted/backtick names, doubled backticks, commas in
  quoted names, control whitespace, and duplicate keys;
- `missok`, duplicate flags, and case-sensitive option handling;
- no-key, condition, assignment, trailing-comma, unknown-option, option-value,
  malformed-option, and unsupported-punctuation diagnostics; and
- unchanged existing parser behavior plus deferred runtime rejection.

The focused oracle command is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_isid.py -k 'test_parse_isid_forms'
```

The broader pinned parser/script regression remains:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
```

Both commands must run against the recorded revision without modifying the
oracle checkout.

## Implementation mapping and deferrals

- **Native Rust:** one typed command variant, direct parser dispatch, bounded
  top-level comma handling, exact option/argument diagnostics, and tests.
- **Runtime boundary:** exhaustive command-name mapping only; `Session::execute`
  continues to reject `Isid` as unsupported.
- **Deferred:** active relation and schema lookup, wildcard/range expansion,
  unknown-variable errors, null-key and duplicate-key semantics, `IsidResult`,
  eager/lazy scans, materialization, labels/panel metadata, `by:` wrappers,
  scripts, CLI/JSON/MCP, full tokenizer/varlist/option parity, and backend
  initialization.

Acceptance requires the focused/full oracle checks, local locked Cargo checks,
policy scans, `git diff --check`, independent parser/contract/workspace review,
and all hosted CI checks. The slice is complete only for this syntax contract;
the roadmap's Phase 4 `isid` execution item remains unchecked.
