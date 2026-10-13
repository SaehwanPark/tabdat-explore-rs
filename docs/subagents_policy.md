# Optional subagent guidance

**Default: work in one agent/context.** Delegate when the task has an independent
question whose specialist investigation or parallel execution genuinely saves
context or improves quality. Delegation is not required for test execution,
file navigation, formatting, or routine changes.

A bounded assignment should specify the question, relevant paths, permitted
read/write actions, and expected output. Ask for concise findings and test
references, not full intermediate transcripts. Avoid parallel edits to the same
files and resolve conflicts in the primary context. The primary agent owns final
integration, verification, architectural decisions and user communication.

Independent statistical, native safety or specialized review may be useful for
risky changes. A fixed number of self-review passes is not independent evidence.
No required model routing, quota monitor, worker hierarchy or automatic fan-out.
If delegation is unavailable, work serially or report what requires outside
expertise. No child should silently change scope or grant itself permissions.

See [AGENTS.md](../AGENTS.md) for the few project-wide rules.
