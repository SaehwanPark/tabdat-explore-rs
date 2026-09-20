# bounded `ttest` syntax contract

## Status and scope

Status: `contract checkpoint`.

This slice ports the pinned Python parser's direct `ttest` command boundary as
an owned Rust language-layer command. It covers the three syntax forms:

- `ttest <var> == <numeric-value>` (including a signed value);
- `ttest <var> == <other-var>`; and
- `ttest <var>, by(<group-var>)` with the flag-only `welch` and `unequal`
  options.

It does not fit a model, calculate a statistic or p-value, inspect a relation,
validate column types, mutate session state, initialize a backend, or add any
runtime inference behavior.

## Owned Rust contract

Add `Command::Ttest { command: TtestCommand }` with:

```rust
pub struct TtestCommand {
    pub varname1: String,
    pub varname2: Option<String>,
    pub value: Option<String>,
    pub by_variable: Option<String>,
    pub welch: bool,
}
```

The result is owned and preserves decoded quoted/backtick variable names. A
value comparison sets `value` or `varname2`; a `by()` form sets
`by_variable`; these alternatives are mutually exclusive. `unequal` is an
alias for `welch` in the returned typed command. Repeated `welch`/`unequal`
flags are accepted as in the oracle; `by()` may appear only once.

The Rust syntax AST stores the recovered numeric spelling as an owned string
(`"-1.5"`, `".5"`, and so on) rather than eagerly storing `f64`. This follows
the existing expression-number contract, keeps the command AST total and
`Eq`-compatible, and defers numeric conversion to a future statistical runtime
boundary. Numeric acceptance and signed-value diagnostics still follow the
oracle.

## Recovered diagnostics

The parser must preserve these bounded diagnostics:

| Input shape | Diagnostic |
| --- | --- |
| `ttest` | `ttest command expects a variable comparison or a variable with by() option` |
| `ttest wage` | `ttest command expects comparison (e.g. ttest var == value) or by() option` |
| more than one `=`/`==` in a comparison | `ttest command: multiple comparisons` |
| a non-identifier LHS | `ttest command: LHS must be a single variable name` |
| text or another unsupported RHS kind | `ttest command: RHS must be a variable name or a numeric value` |
| malformed signed/multi-token RHS | `ttest command: RHS must be a single variable name or a numeric value` |
| more than one comma | `ttest command: duplicate comma` |
| missing/invalid `by()` | `ttest command requires option by(<variable>)` |
| more than one `by()` | `ttest option by may only be supplied once` |
| unsupported option | `ttest unsupported option: <sorted names>` |

Option values must use the existing language option grammar. `welch` and
`unequal` are flag-only; `by(<variable>)` must contain exactly one identifier.

## Evidence target

The focused oracle evidence will use the pinned Python checkout recorded in the
migration report. Rust coverage will include value, signed value, paired
variable, `by()`, alias flags, quote decoding, duplicate/malformed forms, and
the exact diagnostics above. The runtime contract remains an explicit
`UnsupportedCommand { name: "ttest" }` result.

Broader statistical command parsing, expression constraints (`test`/`lincom`),
estimation, inference, post-estimation state, reporting, CLI/JSON/MCP surfaces,
and Python statistical parity remain deferred.
