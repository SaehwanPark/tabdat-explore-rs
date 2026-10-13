# Optional skill routing and handoffs

Repo-local skills are reference checklists, **not** mandatory workers or phase
gates. Most tasks need no skill or only one. Start with the requested user
outcome and relevant source/tests; expand only when a boundary is genuinely
involved. See [AGENTS.md](../../../AGENTS.md).

| Task | Reference |
| --- | --- |
| Command/script/CLI behavior or Python migration | [Migration skill](../../../.agents/skills/tabdat-migration/SKILL.md) |
| Data, DuckDB, missingness, state, file effects | [Data skill](../../../.agents/skills/tabdat-data-semantics/SKILL.md) |
| Estimators, covariance, inference or predictions | [Statistical skill](../../../.agents/skills/tabdat-statistical-validation/SKILL.md) |
| FFI, native adapters, licenses, packaging | [Native skill](../../../.agents/skills/tabdat-native-backends/SKILL.md) |

A normal change uses a small acceptance test and ordinary checks. When a complex
change needs an asynchronous handoff, write a short note in the existing PR or a
single `_workspace/<task>/handoff.md` with:

- Current branch/commit and objective
- What changed and what was tested (including failures)
- Any consequential decision or unverified assumption
- Smallest remaining step

Do not require separate contract, evidence, review and summary documents. Keep
historical records and ADRs as background evidence; they do not expand scope or
require exhaustive parity. Subagents are optional and governed by
[subagent guidance](../../subagents_policy.md).
