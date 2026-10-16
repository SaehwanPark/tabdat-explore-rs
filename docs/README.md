# Documentation

## Use and develop TabDat

- [Project overview and quickstart](../README.md)
- [Current capabilities and guarantees](../SPEC.md)
- [CLI, scripts and interactive editing](guides/cli-and-editor.md)
- [Command semantics and CLI-v1 result contracts](reference/command-results.md)
- [Current outcome roadmap](TABDAT_RUST_PORT_ROADMAP.md)
- [Contributor workflow and local checks](../CONTRIBUTING.md)
- [Architecture and module ownership](../ARCHITECTURE.md)
- [Agent instructions](../AGENTS.md)
- [Responsiveness measurements and PTY tooling](performance/core-workflow.md)

In-session `help <command>` is the packaged syntax/workflow reference. Discovery
also includes parser-only forms; consult SPEC before inferring runtime support.

## Evidence and design references

These records explain particular decisions and validated fixtures. They are not
an ordered implementation backlog or a requirement to repeat every historical gate.

| Location | Purpose |
| --- | --- |
| [Migration](migration/README.md) | Python authority, compatibility policy, decisions and bounded language/script evidence |
| [ADRs](adr/) | Consequential backend, state, safety and interface decisions |
| [Feasibility](feasibility/) | Historical native/backend investigations; start with the [Phase 1 report](feasibility/phase1-final-report.md) |
| [Archived proposal](archive/rust-port-proposal.md) | Original vision; superseded by current instructions and capability map |
| [Archived delivery ledger](archive/delivered-workflows.md) | Completed slices and linked PR/reference evidence from the former long roadmap |
| [Task evidence](../_workspace/README.md) | Historical per-slice contracts, implementation notes and reviews, retained at their original paths |
| [Native prototypes](../spikes/README.md) | Isolated feasibility work, not production runtime capabilities |
| [Third-party notices](third-party-notices/) | Notices for referenced or bundled components |

## Optional agent tooling

[Team/skill routing](harness/tabdat/team-spec.md),
[subagent policy](subagents_policy.md) and [AI usage guidance](ai-usage-policy.md)
are contributor aids, not product acceptance gates. Single-agent work is the default.

Keep entry points brief. Put detailed usage in `guides/`, public result/state
contracts in `reference/`, current priorities in the roadmap and historical
proposals/delivery ledgers in `archive/`. Preserve evidence and stable source/fixture
paths unless a specific migration justifies moving them.
