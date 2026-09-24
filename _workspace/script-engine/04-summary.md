# Bounded script engine slice summary

## Outcome

Accepted bounded language-layer script engine slice. PR
[#133](https://github.com/SaehwanPark/tabdat-explore-rs/pull/133) was merged to
`main` as
[`7ee3055`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/7ee30554b5dfa917242ba437f191d5eb85b1bf1e).

The language layer now exposes typed script parsing, directive evaluation, macro
expansion, and control flow structures in `tabdat_language::script`:
- `ScriptCommand { line: usize, text: String }` representing individual executable script commands with 1-based start line tracking.
- `ScriptContext` maintaining session macros (`BTreeMap<String, String>`) and optional random seed state (`seed: Option<u64>`).
- Directives: `ScriptDirective::Seed(SeedDirective)`, `ScriptDirective::Let(LetDirective)`.
- Control flow directives: `ControlFlowDirective::If(IfDirective)`, `ControlFlowDirective::Else(ElseDirective)`, `ControlFlowDirective::End(EndDirective)`.
- Block execution state: `ScriptBlockState` tracking conditional branch activation (`active`), matched status (`has_matched`), and else-clause presence (`saw_else`).
- `parse_script(source, path)` and `read_script(path)` for transforming script text or files into streams of executable `ScriptCommand`s.
- `expand_script_macros(text, context, path, line)` implementing Stata-style `$macro` substitution.
- `evaluate_script_condition(condition, context, path, line)` supporting truthiness, relational operators (`==`, `!=`, `<`, `<=`, `>`, `>=`), and macro substitution.

The implementation enforces exact Python-compatible behavior and diagnostics:
- Comments beginning with `#` are stripped; blank or comment-only lines are ignored.
- Multiline SQL commands delimited by triple quotes (`sql """ ... """`) are grouped into a single command with internal newlines preserved and the opening line number recorded.
- Unterminated triple-quoted SQL statements yield exact syntax error diagnostics (`<path>:<line>: unterminated triple-quoted sql command`).
- Directive validation: missing seed argument (`<path>:<line>: seed requires an integer value`), invalid seed integer (`<path>:<line>: invalid seed: ...`), let syntax (`<path>:<line>: let expects syntax: let <macro> = <value>`), invalid macro names (`<path>:<line>: invalid macro name: ...`).
- Macro expansion: undefined macros yield diagnostic (`<path>:<line>: undefined macro: <name>`), while literal `$` characters and non-identifier patterns are preserved.
- Control flow validation: invalid if condition (`<path>:<line>: if requires a condition expression`), unmatched else (`<path>:<line>: else without matching if`), unmatched end (`<path>:<line>: end without matching if`), duplicate else (`<path>:<line>: else already defined for if block`), and unclosed if blocks at EOF (`<path>:<line>: unclosed if block`).

Runtime execution of `.td` scripts (`run <script-path>`), recursive call stack limits, and DuckDB session integration remain explicitly deferred.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only script engine boundary is closed; runtime script execution, recursion rejection, and CLI execution
remain unchecked in the roadmap.
